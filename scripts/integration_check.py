#!/usr/bin/env python3
"""Integration check: one dev-fast build, then every part, one summary line each.

The parts are the static gates, the whole private unit binary, every harness
suite group and the named course text suites against the CLI just built, at the
granted width. Every part runs even after another fails, and the command fails
if any part failed. The queue is not charged; compiles share their own ceiling.
"""
import argparse
import json
from pathlib import Path
import re
import sys
import threading
import time

from fastcheck import admission_marker, launch
from fastcheck_gates import Digests
from fastcheck_run import Beside, Steps, build, build_cli, fingerprint
from maintainer_suites import discover
from storage import check_space, exclusive, minimum_reserve

ROOT = Path(__file__).resolve().parents[1]
GATES = ("assignment-metadata", "format", "architecture")
RESULT = re.compile(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", re.M)
COURSE_TOTAL = re.compile(r"^(pa\d+): (\d+)/(\d+)$", re.M)
COURSE_FAILURE = re.compile(r"^(pa\d+/\S+): FAIL", re.M)
LIMITS = (60, 14400)
GIB = 1024 ** 3


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("stages", nargs="+", metavar="paN", choices=[f"pa{n}" for n in range(1, 45)],
                        help="course text suites to run against the dev-fast CLI")
    result.add_argument("--seconds", type=float, default=10800,
                        help="ceiling for gates and test execution together, at most 14400 seconds")
    result.add_argument("--compile-seconds", type=float, default=3600,
                        help="hard ceiling for all compile steps together, at most 3600 seconds")
    result.add_argument("--wait-seconds", type=float, default=3600,
                        help="longest queue for a resource slot, at most 3600 seconds")
    result.add_argument("--max-output-gib", type=float, help="absolute cap including retained output")
    result.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    result.add_argument("--evidence-root", type=Path, help=argparse.SUPPRESS)
    return result


class Parts:
    """One row per part in the receipt; the summary is printed from these rows."""

    def __init__(self, steps):
        self.steps = steps
        steps.receipt["parts"] = []

    def add(self, name, passed, detail, failures=()):
        with self.steps.lock:
            self.steps.receipt["parts"].append({"part": name, "passed": passed, "detail": detail,
                                                "failures": list(failures)})
        self.steps.save()

    def missing(self, names, reason):
        for name in names:
            self.add(name, False, f"not run: {reason}")


def attempt(steps, name, command, **options):
    """Run one part's command; its failure is reported by the caller, never raised."""
    try:
        return 0, steps.run(name, command, **options)
    except (TimeoutError, ValueError):
        path = steps.folder / f"{name}.stdout"
        output = path.read_text(errors="replace") if path.is_file() else ""
        with steps.lock:
            # A budget exhausted before the start leaves no row: that is a timeout too.
            status = next((row["status"] for row in reversed(steps.receipt["steps"]) if row["name"] == name), 124)
        return status, output


def libtest_failures(output):
    """Names from libtest's closing failures list."""
    closing = output.rsplit("\nfailures:\n", 1)
    if len(closing) != 2:
        return []
    return re.findall(r"^    (\S+)$", closing[1].split("\ntest result:", 1)[0], re.M)


def unit_part(steps, parts, name, binary):
    status, output = attempt(steps, name, [binary], cwd=steps.root / "dev")
    counts = RESULT.findall(output)
    if len(counts) != 1:
        parts.add(name, False, f"status {status}; no single libtest result", libtest_failures(output))
        return
    verdict, passed, failed, ignored = counts[0][0], *map(int, counts[0][1:])
    if status == 0 and verdict == "ok" and passed > 0:
        parts.add(name, True, f"{passed} passed, {ignored} ignored")
    else:
        detail = f"{failed} of {passed + failed} failed" if passed else "ran no tests"
        parts.add(name, False, f"{detail}; status {status}", libtest_failures(output))


def course_part(steps, parts, stage, cli):
    jobs = steps.environment["RUST_TEST_THREADS"]
    status, output = attempt(steps, stage, [sys.executable, "scripts/course.py", stage, "--app", cli,
                                            "--jobs", jobs])
    totals = [row for row in COURSE_TOTAL.findall(output) if row[0] == stage]
    failures = COURSE_FAILURE.findall(output)
    if status == 0 and len(totals) == 1 and int(totals[0][2]) > 0 and totals[0][1] == totals[0][2]:
        parts.add(stage, True, f"{totals[0][1]}/{totals[0][2]} cases")
    elif totals:
        parts.add(stage, False, f"{totals[0][1]}/{totals[0][2]} cases; status {status}", failures)
    else:
        parts.add(stage, False, f"status {status}; no case summary", failures)


def gate_parts(steps, parts, gates):
    gates.join(report=False)
    rows = steps.receipt["steps"]
    for name in GATES:
        found = [row for row in rows if row.get("gate") == name]
        if not found:
            reason = "; ".join(map(str, gates.errors)) or "no result"
            parts.add(f"gate {name}", False, f"not run: {reason}")
            continue
        last = found[-1]
        if last.get("skipped"):
            parts.add(f"gate {name}", True, f"skipped: fingerprint {last['fingerprint'][:16]} "
                                            f"passed in {last['matched_receipt']}")
        else:
            parts.add(f"gate {name}", last["status"] == 0, f"status {last['status']}, {last['seconds']:.1f}s")


def execute(root, args, steps, parts):
    suites = discover(root)
    targets = sorted(set(suites.values()))
    digests = Digests(root)
    steps.receipt["sources"] = fingerprint(root, digests)
    python = root / "obj/audit-venv/bin/python"
    if not python.is_file():
        raise ValueError("run make setup-audit once before the integration check")
    gates = Beside(root, steps, python, digests)
    groups = {"lib": [], **{target: [] for target in targets}}
    binaries = {}
    try:
        binaries = build(root, args, steps, groups)
        build_cli(root, steps)
        parts.add("build", True, "private harness, every suite group and the CLI")
    except (TimeoutError, ValueError) as error:
        parts.add("build", False, str(error))
    gate_parts(steps, parts, gates)
    if not binaries:
        parts.missing(["lib", *targets, *args.stages], "the build failed")
        return
    for name, binary in binaries.items():
        unit_part(steps, parts, name, binary)
    cli = root / "obj/rust/dev-fast/rustgm"
    if not cli.is_file():
        parts.missing(args.stages, "the CLI was not built")
        return
    for stage in dict.fromkeys(args.stages):
        course_part(steps, parts, stage, cli)
    if fingerprint(root, digests) != steps.receipt["sources"]:
        parts.add("sources", False, "changed during the run; results do not qualify the current source")


def summary(rows):
    lines = []
    for row in rows:
        failures = row["failures"]
        shown = ", ".join(failures[:30]) + (f" and {len(failures) - 30} more" if len(failures) > 30 else "")
        lines.append(f"  {'PASS' if row['passed'] else 'FAIL'} {row['part']}: {row['detail']}"
                     + (f" ({shown})" if shown else ""))
    failed = [row["part"] for row in rows if not row["passed"]]
    lines.append(f"Integration: {len(rows) - len(failed)} of {len(rows)} parts passed"
                 + (f"; failed: {', '.join(failed)}" if failed else ""))
    return lines


def own_cgroup():
    try:
        lines = Path("/proc/self/cgroup").read_text().splitlines()
    except OSError:
        return None
    return next((Path("/sys/fs/cgroup") / line[3:].lstrip("/") for line in lines if line.startswith("0::")), None)


def anonymous_bytes(folder):
    try:
        rows = (folder / "memory.stat").read_text().splitlines()
    except OSError:
        return None
    return next((int(row.split()[1]) for row in rows if row.startswith("anon ")), None)


class MemoryMarks:
    """Each step's anonymous-memory high-water mark in this job's cgroup.

    A sample is charged to every step running at that moment, so a peak names
    the parts that overlapped to produce it.
    """

    def __init__(self, steps, folder, interval=1.0):
        self.steps, self.folder, self.interval, self.peaks = steps, folder, interval, {}
        self.stop = threading.Event()
        self.thread = threading.Thread(target=self.main, daemon=True)
        if folder is not None and anonymous_bytes(folder) is not None:
            self.thread.start()

    def main(self):
        while not self.stop.wait(self.interval):
            value = anonymous_bytes(self.folder)
            with self.steps.lock:
                for name in self.steps.running if value is not None else ():
                    self.peaks[name] = max(self.peaks.get(name, 0), value)

    def close(self):
        self.stop.set()
        if self.thread.is_alive():
            self.thread.join()
        with self.steps.lock:
            self.steps.receipt["memory_anon_peak_gib"] = {
                name: round(value / GIB, 2) for name, value in sorted(self.peaks.items(), key=lambda row: -row[1])}
        return next(iter(self.steps.receipt["memory_anon_peak_gib"].items()), None)


def check(root, args):
    with exclusive(root):
        check_space(root, minimum_reserve())
        steps = Steps(root, argparse.Namespace(**vars(args), prune_under=[]))
        steps.receipt["profile"] = "dev-fast integration"
        parts = Parts(steps)
        marks = MemoryMarks(steps, own_cgroup())
        try:
            execute(root, args, steps, parts)
        except (OSError, ValueError) as error:
            parts.add("integration", False, str(error))
        highest = marks.close()
        if highest:
            print(f"Integration memory: anonymous peak {highest[1]:.2f} GiB while {highest[0]} ran", flush=True)
        rows = steps.receipt["parts"]
        steps.receipt["state"] = "passed" if rows and all(row["passed"] for row in rows) else "failed"
        steps.save()
        print("\n".join(summary(rows)), flush=True)
        print(f"Integration evidence: {steps.folder}", flush=True)
        return 0 if steps.receipt["state"] == "passed" else 1


def resources(folder):
    """The controller receipt's high-water marks, once the supervisor has written them."""
    try:
        receipt = json.loads((folder / "receipt.json").read_text())
        record = json.loads((ROOT / receipt["resource_receipt"]).read_text())
    except (OSError, ValueError, KeyError):
        return None
    marks = {key: record.get(key) for key in ("memory_peak_bytes", "memory_anon_peak_bytes",
                                               "memory_oom_kills", "tasks_peak", "used_bytes")}
    return receipt["resource_receipt"], marks


def main():
    started = time.monotonic()
    args = parser().parse_args()
    try:
        if args.worker:
            if args.evidence_root is None:
                raise ValueError("internal worker requires an evidence root")
            admission_marker(args.evidence_root).touch()
            return check(ROOT, args)
        status, folder = launch(args, started, script="scripts/integration_check.py",
                                evidence="obj/qualification/integration", seconds=LIMITS, label="Integration")
        found = resources(folder)
        if found:
            path, marks = found
            peak = marks["memory_anon_peak_bytes"]
            print(f"Integration resources: anonymous memory peak "
                  f"{'unknown' if peak is None else f'{peak / GIB:.2f} GiB'}, "
                  f"{marks['memory_oom_kills'] or 0} OOM kill(s); {path}", flush=True)
        return status
    except (OSError, ValueError) as error:
        print(f"Integration FAILED: {error}", file=sys.stderr)
        return 1
    finally:
        if not args.worker:
            print(f"Integration total: {time.monotonic() - started:.2f}s", flush=True)


if __name__ == "__main__":
    sys.exit(main())
