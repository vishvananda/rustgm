#!/usr/bin/env python3
"""Run focused development validation within a five-minute wall-clock budget."""
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


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("suites", nargs="*", metavar="SUITE[::CASE]")
    result.add_argument("--lib-case", action="append", default=[], help="complete private test path")
    result.add_argument("--pa", action="append", default=[], choices=[f"pa{n}" for n in range(1, 45)],
                        help="affected text-oracle assignment; uses the dev-fast CLI")
    result.add_argument("--ignored", action="store_true", help="select ignored/native tests explicitly")
    result.add_argument("--prune-under", action="append", default=[], help="native output root inside obj")
    result.add_argument("--seconds", type=float, default=300, help="total budget, at most 300 seconds")
    result.add_argument("--max-output-gib", type=float, help="absolute cap including retained output")
    result.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    result.add_argument("--deadline", type=float, help=argparse.SUPPRESS)
    result.add_argument("--evidence-root", type=Path, help=argparse.SUPPRESS)
    return result


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


def launch(args, started):
    if not 20 <= args.seconds <= 300:
        raise ValueError("budget must be between 20 and 300 seconds")
    retired = retire(ROOT)
    if retired:
        print(f"Retired {len(retired)} superseded incremental sessions; latest warm caches retained.", flush=True)
    folder = ROOT / f"obj/qualification/fast/{time.time_ns()}"
    command = [sys.executable, "scripts/resources.py", "run", "--wait-seconds", "0"]
    if discover(ROOT) is not None:
        # This is a real absolute cap, not a smaller reservation after refusal.
        command += ["--max-output-gib", str(args.max_output_gib or 16)]
    elif args.max_output_gib is not None:
        raise ValueError("explicit output caps require the configured resource controller")
    command += ["--", sys.executable, "scripts/fastcheck.py", *sys.argv[1:],
                "--worker", "--deadline", str(started + args.seconds - 25),
                "--evidence-root", str(folder)]
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    status = 1
    try:
        status = process.wait(timeout=max(0.01, started + args.seconds - 22 - time.monotonic()))
        return status
    except subprocess.TimeoutExpired:
        print("Fastcheck FAILED: wall-clock budget exhausted; stopping owned processes.", flush=True)
        terminate(process)
        status = 124
        return 124
    except BaseException:
        terminate(process)
        raise
    finally:
        folder.parent.mkdir(parents=True, exist_ok=True)
        folder.with_suffix(".launcher.json").write_text(json.dumps({
            "state": "passed" if status == 0 else "failed", "status": status,
            "seconds": time.monotonic() - started, "command": command}, indent=2) + "\n")


def main():
    started = time.monotonic()
    args = parser().parse_args()
    try:
        if args.worker:
            if args.deadline is None or args.evidence_root is None:
                raise ValueError("internal worker requires deadline and evidence root")
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
