"""Run one leased process tree with kernel CPU/RAM limits and disk monitoring."""
import os
import signal
import subprocess
import sys
import time

from resource_locks import active_records, inherited, lease
from resource_policy import GIB, UsageScan, atomic_json, discover, free_space, output_roots, usage


def scope_command(command, unit, policy):
    return ["systemd-run", "--user", "--scope", "--quiet", "--collect", f"--unit={unit}",
            "-p", f"CPUQuota={policy['cpu_percent']}%", "-p", f"MemoryMax={policy['memory_gib']}G",
            "-p", "MemorySwapMax=0", "-p", f"TasksMax={policy['tasks']}", *command]


def stop(process, unit):
    # The scope includes descendants that change process groups or orphan themselves.
    try:
        subprocess.run(["systemctl", "--user", "stop", unit], stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL, timeout=10, check=False)
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


def supervise(command, root, environment, policy, entry, record, path, descriptors):
    unit = f"rustgm-job-{record['token']}.scope"
    record.update(command=command, unit=unit, state="starting",
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
                if scanner.total > policy["worker_gib"] * GIB:
                    raise ValueError("worktree generated artifacts exceeded 50 GiB; command stopped")
                if scanner.done:
                    record["used_bytes"] = scanner.total
                    atomic_json(path, record)
                    scanner.close()
                    scanner = None
                    next_scan = now + 10
            time.sleep(0.5)
        result = process.returncode
        # A detached background build must not outlive its lease.
        if scope_active(unit):
            raise ValueError("command left descendants running; scope stopped")
        # Short commands still need a final budget check.
        record["used_bytes"] = usage(root, entry, policy)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        reason = str(error)
        if process is not None:
            stop(process, unit)
        result = 128 + stopped[0] if stopped else 1
    finally:
        if scanner is not None:
            scanner.close()
        for sig, handler in handlers.items():
            signal.signal(sig, handler)
        record.update(state="finished", finished=time.time(), returncode=result, error=reason)
        atomic_json(path, record)
        folder = root / "obj/parallel/resources"
        folder.mkdir(parents=True, exist_ok=True)
        receipt = folder / f"{record['token']}.json"
        # The nonce is an inheritance token, not useful as retained evidence.
        atomic_json(receipt, {key: value for key, value in record.items() if key != "token"})
    if reason:
        print(f"resources: {reason}", file=sys.stderr, flush=True)
    return result if result >= 0 else 128 - result


def run(root, command, environment=None, *, kind="ordinary", wait=3600):
    environment = dict(os.environ if environment is None else environment)
    found = discover(root)
    if found is None:
        if kind == "exclusive":
            raise ValueError("exclusive work requires a configured shared resource controller")
        return subprocess.run(command, cwd=root, env=environment, check=False).returncode
    directory, policy = found
    root = root.resolve()
    entry = policy["worktrees"].get(str(root))
    if entry is None:
        raise ValueError("worktree is not registered with the shared resource controller")
    environment.update(CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2", CARGO_INCREMENTAL="0",
                       RUSTGM_TEST_JOBS="2", CARGO_TARGET_DIR=str(root / "obj/rust"))
    if inherited(directory, root, environment, kind):
        return subprocess.run(command, cwd=root, env=environment, check=False).returncode
    with lease(directory, root, policy, kind, wait) as (record, path, descriptors):
        used = usage(root, entry, policy)
        # Reserve the unused allowance of this job and full allowances of peers.
        # This is conservative when another worker already has a warm cache.
        others = [row for row in active_records(directory, policy["jobs"])
                  if row.get("slot") != record["slot"]]
        reservation = max(0, record["budget_bytes"] - used)
        reservation += sum(row.get("budget_bytes", record["budget_bytes"]) for row in others)
        free_space(root, policy, reservation)
        record["used_bytes"] = used
        return supervise(command, root, environment, policy, entry, record, path, descriptors)
