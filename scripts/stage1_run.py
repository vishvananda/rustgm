#!/usr/bin/env python3
"""Link a joint-graph LowIR program through the pinned object bridge and run it.

Bring-up only: the stage-1 driver for a whole program that one frontend
session emitted as a single LowIR unit, with the native-links manifest that
the same session wrote (``--native-links-output``). It emits one object with
the pinned CPPGM at -O0, as the bring-up profile does, and links it with the
system C driver, which supplies the C startup objects; the manifest names the
libraries that a std-using program needs. Each run starts in a fresh
directory with the given arguments, standard input and environment (nothing
inherited) under a time and an address-space limit.

``OUT/record.json`` names every input, tool and output by hash and records
each command's status, wall and user time and peak memory, and each run's
exit status or signal, output and whether a bring-up stub was reached. With
``--backtrace``, a run that a signal ends is repeated once under gdb and the
innermost frames are recorded: today a reached stub or trapping intrinsic
aborts without writing anything.

Run under the resource and storage guards. ``OUT`` must not exist.
"""
import argparse
import json
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import sys
import time

import backend
from host_process import digest
import native_links

# A reached bring-up stub writes this prefix and its function's path to
# standard error before it aborts.
STUB_MARKER = b"bring-up stub reached: "
SUPPORT = ("backend.py", "host_process.py", "native_links.py")


def artifact(path):
    path = Path(path)
    return {"path": str(path), "bytes": path.stat().st_size, "sha256": digest(path)}


def resident_mib():
    """This driver's resident size. A child's peak counts the pages it shared
    with the driver before it executed its program, so a peak at or below this
    size is only an upper bound on the program's own."""
    with open("/proc/self/statm") as stream:
        return int(stream.read().split()[1]) * resource.getpagesize() / (1024 * 1024)


def measure(argv, *, cwd, stdout, stderr, limit, memory=None, stdin=None, environment=None):
    """Run one command to completion or to its limit and return its measured row."""
    argv = [str(part) for part in argv]
    floor = resident_mib()

    def limits():
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        if memory is not None:
            resource.setrlimit(resource.RLIMIT_AS, (memory, memory))

    started = time.monotonic()
    with open(stdout, "wb") as out, open(stderr, "wb") as err, \
            (open(stdin, "rb") if stdin is not None else open(os.devnull, "rb")) as source:
        child = subprocess.Popen(argv, cwd=cwd, env=environment, stdin=source, stdout=out,
                                 stderr=err, preexec_fn=limits, start_new_session=True)
        deadline = started + limit
        timed_out = False
        while True:
            pid, code, usage = os.wait4(child.pid, os.WNOHANG)
            if pid:
                break
            if time.monotonic() > deadline:
                timed_out = True
                os.killpg(child.pid, signal.SIGKILL)
                _, code, usage = os.wait4(child.pid, 0)
                break
            time.sleep(0.02)
        child.returncode = os.waitstatus_to_exitcode(code)
    status = child.returncode
    return {"argv": argv, "cwd": str(cwd), "limit_seconds": limit,
            "status": status if status >= 0 else None,
            "signal": signal.Signals(-status).name if status < 0 else None,
            "timed_out": timed_out, "seconds": round(time.monotonic() - started, 3),
            "user_seconds": round(usage.ru_utime, 3),
            "peak_mib": round(usage.ru_maxrss / 1024, 1),
            "peak_exact": usage.ru_maxrss / 1024 > floor + 1}


def frames(text, count=12):
    """The innermost function names of a gdb backtrace."""
    names = []
    for line in text.splitlines():
        if line.startswith("#") and " (" in line:
            head = line.split(" (", 1)[0].split()
            names.append(head[-1])
            if len(names) == count:
                break
    return names


def backtrace(executable, out, index, options, stdin, environment):
    """Repeat a run under gdb in a fresh directory and record where it stopped."""
    debugger = shutil.which("gdb")
    if debugger is None:
        return {"error": "gdb is not installed"}
    folder = out / f"run-{index}-gdb"
    folder.mkdir()
    log = out / f"run-{index}.backtrace.txt"
    command = [debugger, "-nx", "-q", "-batch", "-ex", "set startup-with-shell off", "-ex", "run",
               "-ex", "bt", "--args", executable, *options.arg]
    row = measure(command, cwd=folder, stdout=log, stderr=out / f"run-{index}.backtrace.stderr",
                  limit=options.time_limit, stdin=stdin, environment=environment)
    return {"path": str(log), "status": row["status"], "timed_out": row["timed_out"],
            "frames": frames(log.read_text(errors="replace"))}


def stub(stderr):
    """The function a reached bring-up stub named, if any."""
    data = Path(stderr).read_bytes()
    at = data.rfind(STUB_MARKER)
    if at < 0:
        return None
    return data[at + len(STUB_MARKER):].split(b"\n", 1)[0].decode(errors="replace")


def build(lowir, links, out, record, save):
    """Emit and link the program as the bring-up profile's object bridge does."""
    libraries = native_links.load(links) if links is not None else []
    tools = backend.load_tools(("cppgm",))
    record["tools"]["cppgm"] = artifact(tools["cppgm"])
    record["inputs"]["libraries"] = libraries

    def run(command):
        name = "link" if command[0] == "cc" else "object"
        row = measure(command, cwd=out, stdout=out / f"{name}.stdout", stderr=out / f"{name}.stderr",
                      limit=backend.time_limit(command))
        row["name"] = name
        record["build"].append(row)
        save()
        if row["timed_out"]:
            raise ValueError(f"{name} exceeded its runaway limit of {row['limit_seconds']} s")
        if row["status"] != 0:
            raise ValueError(f"{name} failed: {(out / f'{name}.stderr').read_text(errors='replace')[:2000]}")

    executable = out / "program"
    backend.build_objects([lowir], executable, libraries=libraries, run=run)
    record["executable"] = artifact(executable)
    save()
    return executable


def runs(executable, out, options, record, save):
    expected = Path(options.expect_stdout).read_bytes() if options.expect_stdout else None
    environment = dict(item.split("=", 1) for item in options.env)
    stdin = Path(options.stdin).resolve(strict=True) if options.stdin else None
    for index in range(options.runs):
        folder = out / f"run-{index}"
        folder.mkdir()
        row = measure([executable, *options.arg], cwd=folder, stdout=out / f"run-{index}.stdout",
                      stderr=out / f"run-{index}.stderr", limit=options.time_limit,
                      memory=options.memory_limit * 1024 * 1024, stdin=stdin,
                      environment=environment)
        row.update(environment=environment, stdin=artifact(stdin) if stdin else None,
                   stdout=artifact(out / f"run-{index}.stdout"),
                   stderr=artifact(out / f"run-{index}.stderr"),
                   stub=stub(out / f"run-{index}.stderr"),
                   files=sorted(path.name for path in folder.iterdir()))
        problems = []
        if row["timed_out"]:
            problems.append("timed out")
        if options.expect_status is not None and row["status"] != options.expect_status:
            problems.append(f"status {row['status']} {row['signal'] or ''}".strip()
                            + f", expected {options.expect_status}")
        if expected is not None and (out / f"run-{index}.stdout").read_bytes() != expected:
            problems.append("stdout differs from the expected output")
        row["problems"] = problems
        if options.backtrace and row["signal"] and not row["timed_out"] and index == 0:
            row["backtrace"] = backtrace(executable, out, index, options, stdin, environment)
        record["runs"].append(row)
        save()


def drive(options):
    lowir = Path(options.lowir).resolve(strict=True)
    links = Path(options.native_links).resolve(strict=True) if options.native_links else None
    if lowir.suffix != ".lowir":
        raise ValueError("the stage-1 driver accepts one serialized .lowir program")
    out = Path(options.out).resolve()
    if out.exists():
        raise ValueError("stage-1 output directory already exists")
    out.mkdir(parents=True)
    scripts = Path(__file__).resolve().parent
    record = {
        "schema": 1, "state": "building",
        "inputs": {"lowir": artifact(lowir), "native_links": artifact(links) if links else None},
        "tools": {"cc": shutil.which("cc"), "backend_lock_sha256": digest(backend.ROOT / "backend.lock.json"),
                  "implementation_sha256": {name: digest(scripts / name)
                                            for name in (*SUPPORT, Path(__file__).name)}},
        "limits": {"run_seconds": options.time_limit, "run_memory_mib": options.memory_limit},
        "build": [], "runs": [],
    }
    path = out / "record.json"

    def save():
        path.write_text(json.dumps(record, indent=2) + "\n")

    save()
    try:
        executable = build(lowir, links, out, record, save)
        record["state"] = "running"
        runs(executable, out, options, record, save)
        if digest(lowir) != record["inputs"]["lowir"]["sha256"]:
            raise ValueError("the LowIR program changed during the stage-1 run")
        record["state"] = "complete"
    except (OSError, ValueError, KeyError) as error:
        record.update(state="failed", error=str(error))
    finally:
        save()
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("lowir")
    parser.add_argument("--native-links", help="the session's native-links manifest")
    parser.add_argument("--out", required=True)
    parser.add_argument("--runs", type=int, default=1)
    parser.add_argument("--arg", action="append", default=[], help="one program argument (repeatable)")
    parser.add_argument("--stdin", help="a file given as standard input")
    parser.add_argument("--env", action="append", default=[], help="NAME=VALUE (repeatable)")
    parser.add_argument("--time-limit", type=int, default=600, help="seconds per run")
    parser.add_argument("--memory-limit", type=int, default=8192, help="address space per run, MiB")
    parser.add_argument("--expect-status", type=int)
    parser.add_argument("--expect-stdout")
    parser.add_argument("--backtrace", action="store_true",
                        help="repeat the first run that a signal ends under gdb")
    options = parser.parse_args()
    if options.runs < 1 or options.time_limit < 1 or options.memory_limit < 1:
        parser.error("runs and limits must be positive")
    if any("=" not in item for item in options.env):
        parser.error("--env takes NAME=VALUE")
    try:
        record = drive(options)
    except (OSError, ValueError) as error:
        print(f"stage-1 run failed: {error}", file=sys.stderr)
        return 1
    for row in record["runs"]:
        summary = {key: row[key] for key in ("status", "signal", "seconds", "peak_mib", "stub", "problems")}
        if "backtrace" in row:
            summary["frames"] = row["backtrace"].get("frames", [])[:8]
        print(json.dumps(summary))
    if record["state"] != "complete":
        print(f"stage-1 run failed: {record.get('error')}", file=sys.stderr)
        return 1
    return 1 if any(row["problems"] for row in record["runs"]) else 0


if __name__ == "__main__":
    sys.exit(main())
