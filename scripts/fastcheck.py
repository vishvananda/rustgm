#!/usr/bin/env python3
"""Run focused development validation within a five-minute budget after admission."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

from resource_policy import discover
from storage_incremental import retire

ROOT = Path(__file__).resolve().parents[1]
# The controller bounds the queue itself; this only covers its admission accounting.
ADMISSION_GRACE = 300


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("suites", nargs="*", metavar="SUITE[::CASE]")
    result.add_argument("--lib-case", action="append", default=[], help="complete private test path")
    result.add_argument("--pa", action="append", default=[], choices=[f"pa{n}" for n in range(1, 45)],
                        help="affected text-oracle assignment; uses the dev-fast CLI")
    result.add_argument("--ignored", action="store_true", help="select ignored/native tests explicitly")
    result.add_argument("--prune-under", action="append", default=[], help="native output root inside obj")
    result.add_argument("--seconds", type=float, default=300,
                        help="budget from admission, at most 300 seconds")
    result.add_argument("--wait-seconds", type=float, default=3600,
                        help="longest queue for a resource slot, at most 3600 seconds; not charged to the budget")
    result.add_argument("--max-output-gib", type=float, help="absolute cap including retained output")
    result.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    result.add_argument("--evidence-root", type=Path, help=argparse.SUPPRESS)
    return result


def admission_marker(folder):
    """The admitted worker creates this file; the launcher's budget starts there."""
    return folder.with_suffix(".admitted")


def terminate(process):
    """Let the resource supervisor stop its cgroup before killing the launcher."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
        process.wait(timeout=20)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=2)
    except ProcessLookupError:
        process.wait()


def watch(process, marker, args, started, clock=time.monotonic):
    """Queueing is not validation time: the budget runs from the worker's admission."""
    admitted = None
    announced = False
    while True:
        try:
            status = process.wait(timeout=0.2)
        except subprocess.TimeoutExpired:
            status = None
        now = clock()
        if admitted is None and marker.exists():
            admitted = now
            if announced:
                print(f"Fastcheck admitted after {now - started:.0f}s; the budget starts now.", flush=True)
        if status is not None:
            return status, admitted
        if admitted is None:
            if not announced and now - started >= 2:
                print(f"Fastcheck awaiting resource admission (slot queue at most {args.wait_seconds:.0f}s); "
                      "the wait is not charged to the budget.", flush=True)
                announced = True
            if now - started >= args.wait_seconds + ADMISSION_GRACE:
                print("Fastcheck FAILED: resource admission did not complete; stopping owned processes.",
                      flush=True)
                terminate(process)
                return 124, admitted
        elif now - admitted >= args.seconds - 22:
            print("Fastcheck FAILED: wall-clock budget exhausted; stopping owned processes.", flush=True)
            terminate(process)
            return 124, admitted


def launch(args, started):
    if not 20 <= args.seconds <= 300:
        raise ValueError("budget must be between 20 and 300 seconds")
    if not 0 <= args.wait_seconds <= 3600:
        raise ValueError("admission wait must be between zero and 3600 seconds")
    retired = retire(ROOT)
    if retired:
        print(f"Retired {len(retired)} obsolete build-cache entries; latest warm caches retained.", flush=True)
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    folder.parent.mkdir(parents=True, exist_ok=True)
    marker = admission_marker(folder)
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", str(args.wait_seconds)]
    if discover(ROOT) is not None:
        # This is a real absolute cap, not a smaller reservation after refusal.
        command += ["--max-output-gib", str(args.max_output_gib or 16)]
    elif args.max_output_gib is not None:
        raise ValueError("explicit output caps require the configured resource controller")
    command += ["--", sys.executable, "scripts/fastcheck.py", *sys.argv[1:],
                "--worker", "--evidence-root", str(folder)]
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    status = 1
    admitted = None
    try:
        status, admitted = watch(process, marker, args, started)
        if status and admitted is None:
            print("Fastcheck FAILED: not admitted by the resource controller; nothing was checked.", flush=True)
        return status
    except BaseException:
        terminate(process)
        raise
    finally:
        marker.unlink(missing_ok=True)
        queued = None if admitted is None else admitted - started
        folder.with_suffix(".launcher.json").write_text(json.dumps({
            "state": "passed" if status == 0 else "failed", "status": status,
            "seconds": time.monotonic() - started, "queued_seconds": queued,
            "command": command}, indent=2) + "\n")


def main():
    started = time.monotonic()
    args = parser().parse_args()
    try:
        if args.worker:
            if args.evidence_root is None:
                raise ValueError("internal worker requires an evidence root")
            admission_marker(args.evidence_root).touch()
            args.deadline = started + args.seconds - 25
            from fastcheck_run import check
            return check(ROOT, args)
        return launch(args, started)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Fastcheck FAILED: {error}", file=sys.stderr)
        return 1
    finally:
        if not args.worker:
            print(f"Fastcheck total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
