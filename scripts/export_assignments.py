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
}
STUDENT_FILES = {
    ".gitignore", "Cargo.toml", "Cargo.lock", "Makefile", "README.md", "src/main.rs",
    "assignment.mk", "examples/lowir-library/return42.c",
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


def extract(commit, stages, public, handout_paths, tree):
    overlays = {f"student/{path}": path for path in STUDENT_FILES}
    overlays["student/assignment.mk"] = "scripts/student-assignment.mk"
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


def validate(tree, stages, public):
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
    for stage in stages:
        _, cases = load_cases(tree, stage)
        for case in cases:
            oracle = tree / stage / (case["path"] + ".ref.exit_status")
            if oracle.read_bytes() != f"{case['exit_status']}\n".encode():
                raise ValueError(f"{oracle.relative_to(tree)} disagrees with its manifest")
        counts[stage] = len(cases)
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
    stages = [entry["path"] for entry in selected if entry["status"] in ("implemented", "implemented-draft")]
    if len(stages) + len(pending) != len(selected):
        raise ValueError("unknown assignment status in export selection")
    handout_paths = [f"{stage}/README.md" for stage in pending]
    public = PUBLIC_FILES | set(git("ls-tree", "-rz", "--name-only", commit, "--", "doc").decode().split("\0")[:-1])
    public.update(handout_paths)
    for stage in stages:
        public.update(fixture_files(stage, git_json(commit, f"{stage}/tests/manifest.json")))
    sources = git_json(commit, "third_party/sources.json")
    sources["files"] = [entry for entry in sources["files"] if entry["destination"] in public]
    assignments["stages"] = [student_stage(entry) for entry in selected]
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".rustgm-export-", dir=destination.parent) as scratch:
        tree = Path(scratch) / "tree"
        extract(commit, stages, public, handout_paths, tree)
        write_json(tree / "assignments.json", assignments)
        write_json(tree / "third_party/sources.json", sources)
        for stage in stages:
            (tree / stage / "Makefile").write_text("include ../scripts/student-assignment.mk\n", encoding="utf-8")
        counts = validate(tree, stages, public)
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
