#!/usr/bin/env python3
"""Build a student preview from the public files of one committed revision."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

from case_inputs import support_files
from course import ROOT, SIDECARS, check_relative_path, load_cases
import upstream

PUBLIC_FILES = {
    ".gitattributes", "LICENSE-APACHE", "LICENSE-MIT", "NOTICE", "rust-toolchain.toml",
    "backend.lock.json", "backend.distribution.json",
    "shared/CPPGM-NOTICE", "shared/README.md", "shared/lowir.md",
    "third_party/rust/COPYRIGHT", "third_party/rust/LICENSE-APACHE", "third_party/rust/LICENSE-MIT",
    "third_party/rust-analyzer/LICENSE-APACHE", "third_party/rust-analyzer/LICENSE-MIT",
    "scripts/backend.py", "scripts/setup_backend.py", "scripts/native_links.py",
    "scripts/course.py", "scripts/case_inputs.py", "scripts/crate_graph.py", "scripts/upstream.py",
    "scripts/course_suites.py", "scripts/finite_contracts.py", "scripts/reference_batches.py",
    "scripts/storage.py", "scripts/storage_archives.py", "scripts/resource_locks.py",
    "scripts/resource_policy.py", "scripts/resource_runner.py",
    "scripts/setup_rust_src.py", "scripts/grade_consumers.py", "scripts/grade_pa34_driver.py",
    "scripts/pa37_self_host.py", "scripts/build_self_hosting.py", "scripts/link_self_hosting.py",
    "scripts/compare_self_hosting.py", "scripts/stage1_run.py", "scripts/host_inputs.py",
    "scripts/host_process.py", "scripts/host_build_script.py", "scripts/host_profile.py",
    "scripts/build_environment.py", "scripts/build_script_output.py", "scripts/build_units.py",
    "scripts/local_package.py", "scripts/manifest_lints.py", "scripts/package_snapshot.py",
    "scripts/target_units.py", "scripts/original_core_manifest.json",
}
STUDENT_FILES = {
    ".gitignore", "Cargo.toml", "Cargo.lock", "Makefile", "README.md", "src/main.rs",
    "assignment.mk", "assignment-consumers.mk", "examples/lowir-library/return42.c",
}


def git(*arguments):
    return subprocess.check_output(["git", "-C", str(ROOT), *arguments])


def git_json(commit, name):
    return json.loads(git("show", f"{commit}:{name}"))


def student_stage(stage):
    return {key: value for key, value in stage.items() if key != "release_evidence"}


def selection(assignments, through):
    through = through or "pa37"
    stages = sorted(assignments["stages"], key=lambda stage: stage["number"])
    for index, stage in enumerate(stages):
        if stage["path"] == through:
            return stages[:index + 1]
    raise ValueError(f"unknown student assignment: {through}")


def fixture_files(stage, manifest):
    paths = {f"{stage}/README.md", f"{stage}/tests/manifest.json"}
    for case in manifest["cases"]:
        primary = case["path"]
        for name in [primary, *support_files(case)]:
            check_relative_path(name, stage)
            if not name.startswith("tests/"):
                raise ValueError(f"{stage}: fixture outside tests: {name}")
            paths.add(f"{stage}/{name}")
        paths.update(f"{stage}/{primary}{suffix}" for suffix in SIDECARS)
    return paths


def consumer_files(commit, stage):
    suite = git_json(commit, f"{stage}/tests/suite.json")
    paths = {f"{stage}/README.md", f"{stage}/tests/suite.json", f"{stage}/{suite['outcomes']}"}
    outcomes = git_json(commit, f"{stage}/{suite['outcomes']}")
    for case in outcomes["cases"]:
        for key in ("path", "stdout"):
            check_relative_path(case[key], stage)
            paths.add(f"{stage}/{case[key]}")
    if lint_rel := suite.get("lint_manifest"):
        check_relative_path(lint_rel, stage)
        paths.add(f"{stage}/{lint_rel}")
        lint_plan = git_json(commit, f"{stage}/{lint_rel}")
        lint_parent = Path(lint_rel).parent
        for case in lint_plan["cases"]:
            rel = (lint_parent / case["path"]).as_posix()
            check_relative_path(rel, stage)
            paths.add(f"{stage}/{rel}")
    return paths


def driver_files(commit, stage):
    suite_rel = f"{stage}/tests/build-driver/suite.json"
    tracked = set(git("ls-tree", "-rz", "--name-only", commit, "--", suite_rel).decode().split("\0")[:-1])
    if suite_rel not in tracked:
        return set()
    suite = git_json(commit, suite_rel)
    paths = {suite_rel}
    for rel in suite["files"]:
        full = f"{stage}/tests/build-driver/{rel}"
        check_relative_path(f"tests/build-driver/{rel}", stage)
        paths.add(full)
    return paths


def self_hosting_files(stage):
    return {f"{stage}/README.md", f"{stage}/Makefile", f"{stage}/tests/suite.json"}


def extract(commit, stages, public, handout_paths, tree):
    overlays = {f"student/{path}": path for path in STUDENT_FILES}
    overlays["student/assignment.mk"] = "scripts/student-assignment.mk"
    overlays["student/assignment-consumers.mk"] = "scripts/student-assignment-consumers.mk"
    wanted = {**{path: path for path in public}, **overlays}
    fixture_roots = {(stage, "tests") for stage in stages}
    command = ["git", "-C", str(ROOT), "archive", commit, "--", *stages, "doc",
               "scripts", "student", "shared", "third_party"]
    command.extend(handout_paths)
    command.extend(sorted(path for path in PUBLIC_FILES if "/" not in path))
    with subprocess.Popen(command, stdout=subprocess.PIPE) as process, process.stdout as stream:
        with tarfile.open(fileobj=stream, mode="r|") as archive:
            for entry in archive:
                if entry.name not in wanted:
                    continue
                output = tree / wanted.pop(entry.name)
                output.parent.mkdir(parents=True, exist_ok=True)
                if entry.isfile():
                    output.write_bytes(archive.extractfile(entry).read())
                    output.chmod(entry.mode & 0o777)
                elif entry.issym() and tuple(entry.name.split("/", 2)[:2]) in fixture_roots:
                    output.symlink_to(entry.linkname)
                else:
                    raise ValueError(f"unsupported exported file: {entry.name}")
        while stream.read(64 * 1024):
            pass
    if process.returncode:
        raise subprocess.CalledProcessError(process.returncode, command)
    if wanted:
        raise ValueError(f"missing committed export files: {', '.join(sorted(wanted))}")


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def validate(tree, active_entries, public):
    import grade_consumers
    for name in public:
        path = tree / name
        if path.is_symlink():
            stage = name.split("/")[0]
            tests = (tree / stage / "tests").resolve(strict=True)
            link = path.readlink()
            if link.is_absolute() or not path.resolve(strict=True).is_relative_to(tests):
                raise ValueError(f"export symlink escapes its assignment fixtures: {name}")
    upstream.verify(tree)
    counts = {}
    for entry in active_entries:
        stage = entry["path"]
        kind = entry.get("suite_kind", "text")
        if kind == "text":
            _, cases = load_cases(tree, stage)
            for case in cases:
                oracle = tree / stage / (case["path"] + ".ref.exit_status")
                if oracle.read_bytes() != f"{case['exit_status']}\n".encode():
                    raise ValueError(f"{oracle.relative_to(tree)} disagrees with its manifest")
            counts[stage] = len(cases)
        elif kind == "consumers":
            _, outcomes, _ = grade_consumers.load_suite(tree, stage)
            counts[stage] = len(outcomes["cases"])
        elif kind == "self-hosting":
            suite = json.loads((tree / stage / "tests/suite.json").read_text())
            if suite.get("schema") != 1 or suite.get("kind") != "self-hosting":
                raise ValueError(f"{stage}: invalid self-hosting suite.json")
            counts[stage] = len(suite["compare_outputs"]) + len(suite["negative_controls"])
        else:
            raise ValueError(f"{stage}: unsupported suite_kind {kind}")
    return counts


def export(revision, through, destination, *, include_pending=False):
    if destination.exists() or destination.is_symlink():
        raise FileExistsError(f"destination already exists: {destination}")
    commit = git("rev-parse", "--verify", "--end-of-options", f"{revision}^{{commit}}").decode().strip()
    assignments = git_json(commit, "assignments.json")
    selected = selection(assignments, through)
    pending = [entry["path"] for entry in selected if entry["status"] == "planned"]
    if pending and not include_pending:
        raise ValueError(f"grading suites are pending for {', '.join(pending)}")
    active_entries = [entry for entry in selected if entry["status"] in ("implemented", "implemented-draft")]
    stages = [entry["path"] for entry in active_entries]
    if len(stages) + len(pending) != len(selected):
        raise ValueError("unknown assignment status in export selection")
    handout_paths = [f"{stage}/README.md" for stage in pending]
    public = PUBLIC_FILES | set(git("ls-tree", "-rz", "--name-only", commit, "--", "doc").decode().split("\0")[:-1])
    public.update(handout_paths)
    driver_stages = set()
    for entry in active_entries:
        stage = entry["path"]
        kind = entry.get("suite_kind", "text")
        if kind == "text":
            public.update(fixture_files(stage, git_json(commit, f"{stage}/tests/manifest.json")))
            dfiles = driver_files(commit, stage)
            if dfiles:
                public.update(dfiles)
                driver_stages.add(stage)
        elif kind == "consumers":
            public.update(consumer_files(commit, stage))
        elif kind == "self-hosting":
            public.update(self_hosting_files(stage))
        else:
            raise ValueError(f"{stage}: unknown suite_kind {kind}")
    sources = git_json(commit, "third_party/sources.json")
    sources["files"] = [entry for entry in sources["files"] if entry["destination"] in public]
    assignments["stages"] = [student_stage(entry) for entry in selected]
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".rustgm-export-", dir=destination.parent) as scratch:
        tree = Path(scratch) / "tree"
        extract(commit, stages, public, handout_paths, tree)
        write_json(tree / "assignments.json", assignments)
        write_json(tree / "third_party/sources.json", sources)
        for entry in active_entries:
            stage = entry["path"]
            kind = entry.get("suite_kind", "text")
            if kind == "text":
                makefile = "include ../scripts/student-assignment.mk\n"
                if stage in driver_stages:
                    makefile += (
                        "\ntest: test-driver\n"
                        ".PHONY: test-driver\n"
                        "ifeq ($(origin RUSTGM_TEST_APP),undefined)\n"
                        "test-driver: all\n"
                        "endif\n"
                        "test-driver:\n"
                        '\t$(PYTHON) "$(ROOT)/scripts/grade_pa34_driver.py" --app "$(abspath $(RUSTGM_TEST_APP))"\n'
                    )
                (tree / stage / "Makefile").write_text(makefile, encoding="utf-8")
            elif kind == "consumers":
                (tree / stage / "Makefile").write_text(
                    "include ../scripts/student-assignment-consumers.mk\n", encoding="utf-8"
                )
        counts = validate(tree, active_entries, public)
        write_json(tree / "export.json", {
            "schema": 1,
            "source_commit": commit,
            "backend_lock_sha256": hashlib.sha256((tree / "backend.lock.json").read_bytes()).hexdigest(),
            "cases": counts,
            "handouts_only": pending,
        })
        tree.rename(destination)
    print(f"Exported {len(selected)} handouts and {sum(counts.values())} cases from {commit[:12]} to {destination}")
    if pending:
        print(f"Handouts with grading suites pending: {', '.join(pending)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--revision", default="HEAD", help="committed source revision (default: HEAD)")
    track = parser.add_mutually_exclusive_group(required=True)
    track.add_argument("--through", help="last student assignment in the export prefix, e.g. pa11")
    track.add_argument("--self-hosting", action="store_true", help="select the student track through self-hosting (PA37)")
    parser.add_argument("--include-pending-handouts", action="store_true",
                        help="preview planned handouts without their grading suites")
    parser.add_argument("destination", type=Path, help="new output directory")
    args = parser.parse_args()
    try:
        export(args.revision, args.through, args.destination.absolute(),
               include_pending=args.include_pending_handouts)
    except (OSError, ValueError, KeyError, RuntimeError, tarfile.TarError, subprocess.CalledProcessError) as error:
        print(f"export error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
