"""Run one leased process tree with kernel CPU/RAM limits and disk monitoring."""
import os
import signal
import subprocess
import sys
import time

from resource_locks import active_records, inherited, inherited_budget, lease, peer_budget
from resource_policy import (OutputLimitError, UsageScan, atomic_json, check_usage, discover, free_space,
                             output_budget, output_roots, usage)


def scope_command(command, unit, policy):
    return ["systemd-run", "--user", "--scope", "--quiet", "--collect", f"--unit={unit}",
            "-p", f"CPUQuota={policy['cpu_percent']}%", "-p", f"MemoryMax={policy['memory_gib']}G",
            "-p", "MemorySwapMax=0", "-p", f"TasksMax={policy['tasks']}", *command]


def stop(process, unit):
    # The scope includes descendants that change process groups or orphan themselves.
    try:
        stopped = subprocess.run(["systemctl", "--user", "stop", unit], stdout=subprocess.DEVNULL,
                                 stderr=subprocess.DEVNULL, timeout=5, check=False).returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        stopped = False
    if not stopped:
        # Killing the systemctl client does not cancel its pending stop job.
        # A scope's default stop timeout can outlive our launcher and lease.
        try:
            subprocess.run(["systemctl", "--user", "kill", "--kill-whom=all",
                            "--signal=SIGKILL", unit], stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL, timeout=5, check=False)
        except (OSError, subprocess.TimeoutExpired):
            pass
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        except ProcessLookupError:
            process.wait()


def scope_active(unit):
    result = subprocess.run(["systemctl", "--user", "is-active", "--quiet", unit],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                            timeout=10, check=False)
    return result.returncode == 0


def finished(root, record, path, result, reason, *, state="finished"):
    record.update(state=state, finished=time.time(), returncode=result, error=reason)
    atomic_json(path, record)
    folder = root / "obj/parallel/resources"
    folder.mkdir(parents=True, exist_ok=True)
    # The nonce is an inheritance token, not useful as retained evidence.
    receipt = folder / f"{record['token']}.json"
    atomic_json(receipt,
                {key: value for key, value in record.items() if key != "token"})
    return receipt


def observed_failure(record, error):
    # A failed scan still records the observed overshoot, not its last successful
    # scan. All accounting includes retained caches, pins and prior receipts.
    if isinstance(error, OutputLimitError):
        record["used_bytes"] = error.used


def supervise(command, root, environment, policy, entry, record, path, descriptors):
    unit = f"rustgm-job-{record['token']}.scope"
    budget = record.setdefault("budget_bytes", output_budget(policy))
    record.update(command=command, unit=unit, state="starting",
                  supervised_budget_bytes=budget,
                  limits={key: policy[key] for key in
                          ("jobs", "reserve_gib", "worker_gib", "cpu_percent", "memory_gib", "tasks")})
    atomic_json(path, record)
    environment.update(RUSTGM_RESOURCE_LEASE=str(path), RUSTGM_RESOURCE_TOKEN=record["token"])
    process = None
    scanner = None
    stopped = []
    handlers = {}
    result = 1
    reason = None
    accounted = False
    try:
        for sig in (signal.SIGTERM, signal.SIGINT):
            handlers[sig] = signal.signal(sig, lambda number, _frame: stopped.append(number))
        process = subprocess.Popen(scope_command(command, unit, policy), cwd=root,
                                   env=environment, start_new_session=True, pass_fds=descriptors)
        record.update(state="running", launcher=process.pid)
        atomic_json(path, record)
        next_scan = 0
        while process.poll() is None:
            if stopped:
                raise ValueError(f"resource supervisor interrupted by signal {stopped[0]}")
            free_space(root, policy)
            now = time.monotonic()
            if scanner is None and now >= next_scan:
                scanner = UsageScan(output_roots(root, entry))
            if scanner is not None:
                scanner.step()
                record["used_bytes"] = scanner.total
                check_usage(scanner.total, budget)
                if scanner.done:
                    atomic_json(path, record)
                    scanner.close()
                    scanner = None
                    next_scan = now + 10
            time.sleep(0.5)
        result = process.returncode
        # A detached background build must not outlive its lease.
        if scope_active(unit):
            raise ValueError("command left descendants running; scope stopped")
        # Count our own receipt too, including for very short commands. A final
        # usage-number rewrite normally keeps its allocated block count. Permit
        # one rescan if the receipt crosses a block boundary; never silently
        # omit that growth or loop on changing bookkeeping.
        receipt = finished(root, record, path, result, None, state="finalizing")
        for _ in range(2):
            blocks = receipt.stat().st_blocks
            record["used_bytes"] = usage(root, entry, policy, budget)
            if stopped:
                raise ValueError(f"resource supervisor interrupted by signal {stopped[0]}")
            finished(root, record, path, result, None)
            if receipt.stat().st_blocks == blocks:
                accounted = True
                break
        if not accounted:
            raise ValueError("resource receipt allocation changed during final accounting")
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        reason = str(error)
        observed_failure(record, error)
        if process is not None:
            stop(process, unit)
        result = 128 + stopped[0] if stopped else 1
    finally:
        if scanner is not None:
            scanner.close()
        for sig, handler in handlers.items():
            signal.signal(sig, handler)
        if not accounted:
            finished(root, record, path, result, reason)
    if reason:
        print(f"resources: {reason}", file=sys.stderr, flush=True)
    return result if result >= 0 else 128 - result


def run(root, command, environment=None, *, kind="ordinary", wait=3600, max_output_gib=None):
    environment = dict(os.environ if environment is None else environment)
    found = discover(root)
    if found is None:
        if kind == "exclusive" or max_output_gib is not None:
            raise ValueError("exclusive work and output caps require a configured shared resource controller")
        return subprocess.run(command, cwd=root, env=environment, check=False).returncode
    directory, policy = found
    budget = output_budget(policy, max_output_gib)
    root = root.resolve()
    entry = policy["worktrees"].get(str(root))
    if entry is None:
        raise ValueError("worktree is not registered with the shared resource controller")
    environment.update(CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2", CARGO_INCREMENTAL="0",
                       RUSTGM_TEST_JOBS="2", CARGO_TARGET_DIR=str(root / "obj/rust"))
    outer = inherited(directory, root, environment, kind, policy["jobs"], policy=policy)
    if outer:
        inherited_budget(outer, policy, budget if max_output_gib is not None else None)
        return subprocess.run(command, cwd=root, env=environment, check=False).returncode
    with lease(directory, root, policy, kind, wait, budget=budget) as (record, path, descriptors):
        record["command"] = command
        try:
            if discover(root) != (directory, policy):
                raise ValueError("resource policy changed while waiting; retry the command")
            used = usage(root, entry, policy, budget)
            record["used_bytes"] = used
            # Reserve our unused absolute cap and full peer caps. An unknown or
            # legacy peer always reserves configured worker_gib, never our cap.
            others = [row for row in active_records(directory, policy["jobs"])
                      if row.get("slot") != record["slot"]]
            reservation = max(0, budget - used)
            reservation += sum(peer_budget(row, policy) for row in others)
            record["reservation_bytes"] = reservation
            free_space(root, policy, reservation)
        except (OSError, ValueError) as error:
            observed_failure(record, error)
            finished(root, record, path, 1, str(error))
            raise
        return supervise(command, root, environment, policy, entry, record, path, descriptors)
