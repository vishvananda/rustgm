"""Focused checks, exact execution receipts and scoped cleanup for fastcheck."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import threading
import time

from fastcheck_gates import Digests, Gates, edition
from maintainer_batch import artifacts, inventory, package_environment, selection
from maintainer_suites import discover
from resource_policy import discover as controller, width
from storage import check_space, cleanup_roots, exclusive, minimum_reserve, prune


def fingerprint(root, digests=None):
    """Every input whose change invalidates a run; unchanged files reuse their stat-keyed hash."""
    digests = digests or Digests(root)
    paths = [root / path for path in ("dev/Cargo.toml", "dev/Cargo.lock",
              "scripts/fastcheck.py", "scripts/fastcheck_run.py", "scripts/fastcheck_gates.py",
              "scripts/gate_trace.py", "scripts/rust_audit.py",
              "scripts/rust-module-roles.json", "scripts/maintainer_batch.py",
              "scripts/maintainer_suites.py", "scripts/storage.py", "scripts/storage_incremental.py", "scripts/resource_runner.py",
              "scripts/resource_policy.py", "scripts/resource_locks.py")]
    paths += [root / path for path in ("assignments.json", "assignment-plan.json",
              "docs/assignment-requirements.json", "Makefile", "scripts/assignment_metadata.py", "scripts/assignment_plan.py",
              "scripts/course.py", "scripts/course_suites.py", "scripts/reference_batches.py",
              "scripts/check_native.py", "scripts/check_layout.py", "scripts/check_layouts.py", "scripts/check_syntax.py")]
    paths += sorted(root.glob("pa*/tests/manifest.json"))
    paths += sorted(root.glob("pa*/tests/units/*.json"))
    rows = {row[1]: row[2] for row in digests.tree(root / "dev", suffix=".rs") if row[0] == "file"}
    return {**rows, **{str(path.relative_to(root)): digests.file(path) for path in paths}}


class Steps:
    def __init__(self, root, args):
        self.root, self.args = root, args
        self.folder = args.evidence_root
        cleanup_roots(root, [self.folder, *args.prune_under])
        self.folder.mkdir(parents=True, exist_ok=False)
        # Two clocks: gates and tests spend the feedback budget, while compile
        # steps share a ceiling whose only purpose is to catch a hung compiler.
        self.started = time.monotonic()
        self.deadline = self.started + args.seconds
        self.compile_left = args.compile_seconds
        # Gates run in threads beside the compile; receipt updates share this lock.
        self.lock = threading.RLock()
        self.receipt = {"profile": "dev-fast", "state": "running", "steps": [],
                        "budget": {"feedback_seconds": args.seconds,
                                   "compile_ceiling_seconds": args.compile_seconds}}
        jobs = width((controller(root) or (None, None))[1])
        self.receipt["width"] = int(jobs)
        token = os.environ.get("RUSTGM_RESOURCE_TOKEN")
        if token:
            # The controller's receipt holds this run's memory high-water marks.
            self.receipt["resource_receipt"] = f"obj/parallel/resources/{token}.json"
        self.environment = {**os.environ, **package_environment(root), "CARGO_INCREMENTAL": "1",
                            "CARGO_TARGET_DIR": str(root / "obj/rust"),
                            "CARGO_BUILD_JOBS": jobs, "RUST_TEST_THREADS": jobs}

    def save(self):
        with self.lock:
            compiled = self.args.compile_seconds - self.compile_left
            self.receipt["budget"].update(
                compile_used=compiled, feedback_used=time.monotonic() - self.started - compiled)
            (self.folder / "receipt.json").write_text(json.dumps(self.receipt, indent=2) + "\n")

    def skip(self, name, fingerprint, matched):
        """A gate whose inputs match a passed run records that match instead of rerunning."""
        with self.lock:
            self.receipt["steps"].append({"name": name, "gate": name, "status": 0, "seconds": 0.0,
                                          "charged": "feedback", "skipped": True,
                                          "fingerprint": fingerprint, "matched_receipt": matched})
        self.save()
        print(f"SKIP {name}: inputs match fingerprint {fingerprint[:16]} passed in {matched}", flush=True)

    def run(self, name, command, *, cwd=None, overrides=None, compiles=False, extra=None, together=False):
        """Run one step against the clock that owns it.

        A killed rustc discards its incremental session, so a compile is never
        stopped at the feedback budget: it finishes, persists its cache and moves
        the budget's deadline by exactly the time it took. Compiles run
        `together` leave the clocks to `charge`, once for their common time.
        """
        kind = "compile ceiling" if compiles else "development feedback budget"
        with self.lock:
            remaining = self.compile_left if compiles else self.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f"{name}: {kind} exhausted")
        print(f"START {name} ({remaining:.0f}s of the {kind} remaining)", flush=True)
        started = time.monotonic()
        with (self.folder / f"{name}.stdout").open("w") as output, (
                self.folder / f"{name}.stderr").open("w") as errors:
            try:
                result = subprocess.run(list(map(str, command)), cwd=cwd or self.root,
                                        env={**self.environment, **(overrides or {})}, stdout=output, stderr=errors,
                                        timeout=remaining, check=False)
                status = result.returncode
            except subprocess.TimeoutExpired:
                status = 124
        elapsed = time.monotonic() - started
        with self.lock:
            if compiles and not together:
                self.compile_left -= elapsed
                self.deadline += elapsed
            self.receipt["steps"].append({"name": name, "command": list(map(str, command)),
                                          "seconds": elapsed, "status": status,
                                          "charged": "compile" if compiles else "feedback",
                                          "environment": overrides or {}, **(extra or {}),
                                          **({"together": True} if together else {})})
        self.save()
        print(f"END {name}: {elapsed:.2f}s, status {status}", flush=True)
        stdout = self.output(name, status, kind, compiles)
        return stdout

    def charge(self, elapsed):
        """Charge compiles that ran together their common wall time, once."""
        with self.lock:
            self.compile_left -= elapsed
            self.deadline += elapsed
        self.save()

    def note(self, name, **fields):
        with self.lock:
            next(row for row in reversed(self.receipt["steps"]) if row["name"] == name).update(fields)
        self.save()

    def output(self, name, status, kind, compiles):
        stdout = (self.folder / f"{name}.stdout").read_text()
        if status:
            print((stdout + (self.folder / f"{name}.stderr").read_text())[-10000:], flush=True)
            if status == 124:
                raise TimeoutError(f"{name}: {kind} exhausted" + ("; the compiler looks hung" if compiles else ""))
            raise ValueError(f"{name} failed with status {status}")
        return stdout


# A changed-file list longer than this is checked by the complete command.
PARTIAL_FORMAT_LIMIT = 400


def traced_gate(root, steps, gates, name, method):
    """Run a Python gate under its trace; cache the pass only if it read nothing undeclared."""
    command, fingerprint, allowed, listings = getattr(gates, method)()
    matched = gates.matched(name, fingerprint)
    if matched:
        steps.skip(name, fingerprint, matched)
        return
    trace = steps.folder / f"{name}.trace.json"
    python, script, *rest = command
    steps.run(name, [python, "scripts/gate_trace.py", trace, script, *rest],
              extra={"gate": name, "fingerprint": fingerprint})
    problems = gates.outside(trace, allowed, listings)
    if not problems and getattr(gates.fresh(), method)()[1] != fingerprint:
        problems = ["inputs changed while the gate ran"]
    if problems:
        steps.note(name, cached=False, uncached_because=problems[:8])
    else:
        gates.passed(name, fingerprint, steps.folder.relative_to(root).as_posix())


def format_gate(root, steps, gates):
    """Check changed files only; the complete check decides any failure and every other case."""
    command, key = gates.format_key()
    fingerprint = gates.format_fingerprint(key)
    mode, changed = gates.format_plan(key)
    if mode == "skip":
        matched = gates.matched("format", fingerprint)
        if matched:
            steps.skip("format", fingerprint, matched)
            return
        mode = "full"
    language = edition(root)
    extra = {"gate": "format", "fingerprint": fingerprint}
    if mode == "partial" and language is not None and len(changed) <= PARTIAL_FORMAT_LIMIT:
        try:
            steps.run("format", ["rustfmt", "--check", "--edition", language, *changed],
                      extra={**extra, "partial": changed})
        except ValueError:
            steps.run("format-full", command, extra=extra)
    else:
        steps.run("format", command, extra=extra)
    fresh = gates.fresh()
    if fresh.format_fingerprint(fresh.format_key()[1]) != fingerprint:
        steps.note("format", cached=False, uncached_because=["inputs changed while the gate ran"])
        return
    gates.passed("format", fingerprint, steps.folder.relative_to(root).as_posix(), key=key, files=gates.sources)


class Beside:
    """The static gates, in threads beside the compile; joined before any test runs."""

    def __init__(self, root, steps, python, digests=None):
        self.errors = []
        self.thread = threading.Thread(target=self.main, args=(root, steps, python, digests), daemon=True)
        self.thread.start()

    def guard(self, function, *arguments):
        try:
            function(*arguments)
        except Exception as error:  # reported by join(), after the compile has finished
            self.errors.append(error)

    def main(self, root, steps, python, digests):
        def gates_in_threads():
            gates = Gates(root, python, digests)
            jobs = [(traced_gate, root, steps, gates, "assignment-metadata", "metadata"),
                    (format_gate, root, steps, gates),
                    (traced_gate, root, steps, gates, "architecture", "architecture")]
            threads = [threading.Thread(target=self.guard, args=job, daemon=True) for job in jobs]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()
            gates.digests.save()
        self.guard(gates_in_threads)

    def join(self, *, report=True):
        self.thread.join()
        if report and self.errors:
            raise self.errors[0]


def execute(root, args, steps):
    suites = discover(root)
    names = args.suites or ([] if args.lib_case else ["lexer"])
    groups = selection(suites, names, args.lib_case)
    if args.ignored and not args.prune_under:
        raise ValueError("native fast checks require --prune-under for their disposable output roots")
    steps.receipt["groups"] = groups
    # Fingerprint content rather than just HEAD: dirty worker snapshots are normal.
    digests = Digests(root)
    steps.receipt["sources"] = fingerprint(root, digests)
    steps.save()
    python = root / "obj/audit-venv/bin/python"
    if not python.is_file():
        raise ValueError("run make setup-audit once before fastcheck")
    gates = Beside(root, steps, python, digests)
    try:
        binaries = build(root, args, steps, groups)
    except BaseException:
        gates.join(report=False)
        raise
    # No test runs after a failed static gate.
    gates.join()
    run_tests(root, args, steps, groups, binaries, digests)


COMMON = ["--manifest-path", "dev/Cargo.toml", "--locked", "--profile", "dev-fast"]
# Cargo renders the diagnostics; its JSON on stdout names the executables.
MESSAGES = "--message-format=json-render-diagnostics"
# The strict check of the public targets keeps its own cache beside the build's.
PUBLIC_LINT = "obj/rust/public-lint"


def lint_environment():
    """clippy-driver as the workspace compiler with every warning an error."""
    driver = shutil.which("clippy-driver")
    if driver is None:
        raise ValueError("fastcheck requires the installed clippy-driver")
    return {"RUSTC_WORKSPACE_WRAPPER": driver, "CLIPPY_ARGS": "-Dwarnings"}


def together(steps, runs):
    """Run compiles at once. Each finishes, since a killed rustc loses its
    session, and a failure is reported after both have stopped."""
    results, errors = {}, []

    def one(name, command, overrides):
        try:
            results[name] = steps.run(name, command, overrides=overrides, compiles=True, together=True)
        except Exception as error:  # reraised below, once the other compile has finished
            errors.append(error)

    started = time.monotonic()
    threads = [threading.Thread(target=one, args=run) for run in runs]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    steps.charge(time.monotonic() - started)
    if errors:
        raise errors[0]
    return [results[name] for name, *_ in runs]


def linted_build(root, steps, build_step, lint_step):
    """Compile with rustc while clippy-driver checks the same targets beside it.

    Clippy's crate-level passes rerun over the whole library after any edit,
    about a third of a one-module rebuild; inside the build they sat on its
    critical path. As a metadata-only check in its own target directory they
    overlap the build, and the pair costs about the check's time. Both must
    pass, and the two Cargo processes share the granted width.
    """
    lint = lint_environment()
    jobs = int(steps.environment.get("CARGO_BUILD_JOBS", "1"))
    first, second = str(max(1, (jobs + 1) // 2)), str(max(1, jobs // 2))
    (name, command), (lint_name, lint_command) = build_step, lint_step
    return together(steps, [
        (name, command, {"CARGO_BUILD_JOBS": first}),
        (lint_name, lint_command, {"CARGO_TARGET_DIR": str(root / PUBLIC_LINT), "CARGO_BUILD_JOBS": second, **lint}),
    ])[0]


def build_cli(root, steps):
    """The CLI from the public build, checked strictly beside it; both are fresh after a public selection."""
    linted_build(root, steps, ("build-cli", ["cargo", "build", *COMMON, "--bin", "rustgm"]),
                 ("clippy-cli", ["cargo", "check", *COMMON, "--lib", "--bins", MESSAGES]))


def build(root, args, steps, groups):
    common = COMMON
    link = ["--no-run", MESSAGES]
    binaries = {}
    if "lib" in groups:
        # The private harness is linted and linked by one clippy-driver build.
        linked = steps.run("clippy-private", ["cargo", "test", *common, "--lib", *link],
                           overrides={"CARGO_TARGET_DIR": str(root / "obj/rust/private-lint"), **lint_environment()},
                           compiles=True)
        binaries.update(artifacts(linked, {"lib": groups["lib"]}))
        steps.receipt["private_harness_producer"] = "clippy-private"
    remaining = {name: cases for name, cases in groups.items() if name != "lib"}
    if remaining:
        # An integration target builds the library, itself and the CLI it runs.
        build_targets = [flag for name in remaining for flag in ("--test", name)]
        built = linted_build(root, steps, ("build-public", ["cargo", "test", *common, *build_targets, *link]),
                             ("clippy-public", ["cargo", "check", *common, "--lib", "--bins", *build_targets,
                                                MESSAGES]))
        binaries.update(artifacts(built, remaining))
    return binaries


def run_tests(root, args, steps, groups, binaries, digests=None):
    selected = {}
    flags = ["--ignored"] if args.ignored else []
    for group, binary in binaries.items():
        listed = steps.run(group + "-inventory", [binary, *flags, "--list"], cwd=root / "dev")
        if not args.ignored:
            ignored = steps.run(group + "-ignored", [binary, "--ignored", "--list"], cwd=root / "dev")
            ignored_rows = set(ignored.splitlines())
            listed = "\n".join(row for row in listed.splitlines() if row not in ignored_rows)
        selected[group] = inventory(listed, group, groups[group])
    steps.receipt["selected"] = selected
    steps.receipt["binaries"] = {group: {"path": str(binary),
                                         "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
                                  for group, binary in binaries.items()}
    steps.save()
    for group, binary in binaries.items():
        started = time.time()
        output = steps.run(group, [binary, *flags, "--exact", *selected[group]], cwd=root / "dev")
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
        if len(counts) != 1 or tuple(map(int, counts[0])) != (len(selected[group]), 0, 0):
            raise ValueError(f"{group}: execution count differs from exact selection")
        # Never traverse all historical obj output after each focused test.
        if args.prune_under:
            steps.receipt.setdefault("cleanup", []).append({"group": group, "removed": prune(
                root, apply=True, since=started, paths=args.prune_under)})
            steps.save()
    if args.pa:
        # The public build and check: already complete after an integration selection.
        build_cli(root, steps)
        for stage in dict.fromkeys(args.pa):
            steps.run(stage, [sys.executable, "scripts/course.py", stage,
                              "--app", root / "obj/rust/dev-fast/rustgm"])
    after = fingerprint(root, digests)
    if after != steps.receipt["sources"]:
        raise ValueError("source changed during validation; evidence does not qualify the current source")
    print(f"Passed {sum(map(len, selected.values()))} exact tests across {len(groups)} target(s).", flush=True)


def check(root, args):
    with exclusive(root):
        check_space(root, minimum_reserve())
        steps = Steps(root, args)
        try:
            execute(root, args, steps)
            steps.receipt["state"] = "passed"
            return 0
        except Exception as error:
            steps.receipt.update(state="failed", error=str(error))
            raise
        finally:
            steps.save()
            print("Fastcheck clocks: {feedback_used:.0f}s of the {feedback_seconds:.0f}s feedback budget, "
                  "{compile_used:.0f}s compiling".format(**steps.receipt["budget"]), flush=True)
            print(f"Fastcheck evidence: {steps.folder}", flush=True)
