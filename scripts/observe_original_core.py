#!/usr/bin/env python3
"""Bounded CI observation of unchanged official core; never builds a compiler."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import resource
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path(__file__).with_name("original_core_manifest.json")
LIMIT = 64 * 1024 * 1024


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write(out, name, value):
    (out / name).write_text(json.dumps(value, indent=2) + "\n")


class Download:
    """Hash compressed bytes without retaining the archive or its member list."""
    def __init__(self, stream, deadline):
        self.stream, self.deadline = stream, deadline
        self.digest, self.bytes = hashlib.sha256(), 0

    def read(self, size):
        if time.monotonic() >= self.deadline:
            raise TimeoutError("official source download exceeded its deadline")
        block = self.stream.read(size)
        self.digest.update(block)
        self.bytes += len(block)
        return block


def fetch(manifest, source, deadline):
    expected = {row["path"]: row for row in manifest["files"]}
    installed = set()
    request = urllib.request.Request(manifest["official_archive"]["url"],
                                     headers={"User-Agent": "RustGM-original-core-observer"})
    with urllib.request.urlopen(request, timeout=10) as response:
        download = Download(response, deadline)
        with tarfile.open(fileobj=download, mode="r|gz") as archive:
            for member in archive:
                # Python tar readers otherwise retain every historical TarInfo.
                archive.members.clear()
                path = PurePosixPath(member.name)
                if path.is_absolute() or ".." in path.parts:
                    raise ValueError("unsafe official archive member")
                if not path.parts or path.parts[0] != "rust-" + manifest["revision"]:
                    raise ValueError("official archive root does not match pinned commit")
                relative = "/".join(path.parts[1:])
                row = expected.get(relative)
                if row is None:
                    continue
                if not member.isfile() or relative in installed or member.size != row["bytes"]:
                    raise ValueError("duplicate, nonregular or wrong-sized pinned source")
                target = source / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                digest = hashlib.sha256()
                with archive.extractfile(member) as data, target.open("xb") as output:
                    while block := data.read(128 * 1024):
                        if time.monotonic() >= deadline:
                            raise TimeoutError("official source extraction deadline")
                        digest.update(block)
                        output.write(block)
                if digest.hexdigest() != row["sha256"]:
                    raise ValueError("pinned source bytes differ: " + relative)
                target.chmod(0o444)
                installed.add(relative)
        while download.read(128 * 1024):
            pass
    if installed != set(expected):
        raise ValueError("official source archive is incomplete: " + str(sorted(set(expected) - installed)))
    actual = {"url": response.geturl(), "bytes": download.bytes,
              "sha256": download.digest.hexdigest()}
    if any(actual[key] != manifest["official_archive"][key] for key in ("bytes", "sha256")):
        raise ValueError("official compressed source archive changed")
    return actual


def file_limit():
    resource.setrlimit(resource.RLIMIT_FSIZE, (LIMIT, LIMIT))


def observe(command, out, deadline):
    with (out / "stdout").open("xb") as stdout, (out / "stderr").open("xb") as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr,
                                   start_new_session=True, preexec_fn=file_limit)
        try:
            return process.wait(timeout=min(120, max(0.1, deadline - time.monotonic() - 8)))
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            raise TimeoutError("original-core observer deadline") from None


def opened_sources(trace, source, manifest):
    expected = {row["path"]: row for row in manifest["files"]}
    opened, seen = [], set()
    with trace.open() as lines:
        for line in lines:
            match = re.search(r'openat\([^,]+, "([^"\n]+)".*\)\s+=\s+(-?[0-9]+)', line)
            if not match:
                continue
            path = Path(match[1])
            if not path.is_absolute():
                directory = re.search(r'openat\([^,]*<([^>]+)>', line)
                path = (Path(directory[1]) if directory else ROOT) / path
            path = path.resolve()
            if not path.is_relative_to(source) or path.is_dir():
                continue
            relative = str(path.relative_to(source))
            if not any(relative.startswith(prefix) for prefix in manifest["prefixes"]):
                raise ValueError("original source outside fetched manifest requested: " + relative)
            if int(match[2]) < 0:
                if relative in expected:
                    raise ValueError("verified original source could not be opened: " + relative)
                # Ordinary foo.rs/foo/mod.rs fallback probes may fail within a
                # fully pinned prefix. This does not authorize a missing subtree.
                continue
            if not path.is_file() or path in seen:
                continue
            seen.add(path)
            if relative not in expected or sha(path) != expected[relative]["sha256"]:
                raise ValueError("unindexed or changed original source opened: " + relative)
            opened.append({**expected[relative], "first_open_order": len(opened)})
    if not opened or opened[0]["path"] != "library/core/src/lib.rs":
        raise ValueError("original core root was not the first opened pinned source")
    return opened


def worker(args):
    started = time.monotonic()
    deadline = started + 185  # Outer storage/launcher wrapper has a hard190s limit.
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    result = {"infrastructure_failure": True, "classification": "infrastructure/setup failure"}
    try:
        manifest = json.loads(MANIFEST.read_text())
        cli, producer = args.cli.resolve(strict=True), args.producer_binding.resolve(strict=True)
        binding = json.loads(producer.read_text())
        if sha(cli) != manifest["expected_cli_sha256"] or binding["candidate_sha256"] != sha(cli):
            raise ValueError("current compiler or producer binding hash differs")
        if any(binding["compiler_source_sha256"].get(path) != digest
               for path, digest in manifest["config_source_sha256"].items()):
            raise ValueError("compiler target/profile source differs from declared source")
        strace = shutil.which("strace")
        if strace is None:
            raise ValueError("strace is absent; CI setup must install it; no untraced fallback")
        source = ROOT / "obj/parallel" / ("original-core-inputs-" + manifest["revision"])
        source.mkdir(parents=True, exist_ok=False)
        archive = fetch(manifest, source, min(deadline - 12, started + 60))
        if not all(sha(source / row["path"]) == row["sha256"] for row in manifest["files"]):
            raise ValueError("not all722 original source hashes verified before observer")
        command = [strace, "-f", "-qq", "-yy", "-s", "4096", "-e", "trace=openat", "-o",
                   str(out / "whole-core.openat"), str(cli), "--emit-types", "--edition=2024",
                   "--crate-name", "core", "--crate-disambiguator", "core_1_98_88d9",
                   "--source-core", "core@core_1_98_88d9", str(source / "library/core/src/lib.rs")]
        immutable = {str(path): sha(path) for path in (cli, producer, Path(strace), MANIFEST, Path(__file__))}
        write(out, "binding.json", {"command": command, "immutable": immutable, "archive": archive,
              "profile": manifest["profile"], "source_files": 722, "core_files": 309,
              "producer": binding, "path_adaptation": {"source_root": str(source),
              "old_source_root": "/home/vishvananda/rustgm-sources/rust-1.98.0-" + manifest["revision"]},
              "semantic_environment_overrides": [], "observer_deadline_seconds": 120,
              "worker_hard_deadline_seconds": 190})
        status = observe(command, out, deadline)
        opened = opened_sources(out / "whole-core.openat", source, manifest)
        write(out, "opened-sources.json", opened)
        text = (out / "stderr").read_text(errors="replace")
        first = next((line for line in text.splitlines() if line.startswith("ERROR:")), None)
        stable = (all(sha(path) == digest for path, digest in immutable.items())
                  and all(sha(source / row["path"]) == row["sha256"] for row in manifest["files"]))
        bounded = all((out / name).stat().st_size < LIMIT for name in ("stdout", "stderr", "whole-core.openat"))
        infra = (status not in (0, 1) or not stable or not bounded or "panicked at" in text
                 or (status == 1 and first is None))
        result.update(status=status, infrastructure_failure=infra, first_error=first,
                      inputs_unchanged=stable, opened_source_count=len(opened),
                      classification="infrastructure/plan failure" if infra else
                      "observed original-core blocker" if status == 1 else "types view only; no artifact/native claim")
    except (OSError, ValueError, KeyError, tarfile.TarError, TimeoutError) as error:
        result["error"] = str(error)
    result["total_seconds"] = time.monotonic() - started
    result["raw_sha256"] = {name: sha(out / name) for name in
                            ("stdout", "stderr", "whole-core.openat", "opened-sources.json") if (out / name).is_file()}
    write(out, "result.json", result)
    print(json.dumps(result, indent=2))
    return int(result["infrastructure_failure"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--producer-binding", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--worker", action="store_true", help="caller already holds storage guard and hard190s timeout")
    args = parser.parse_args()
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise ValueError("remote CI observer only; local admission is not authorized")
    if args.worker:
        return worker(args)
    command = ["timeout", "--signal=TERM", "--kill-after=5s", "190s", sys.executable,
               "scripts/storage.py", "run", "--prune-under", str(args.out), "--",
               sys.executable, __file__, *sys.argv[1:], "--worker"]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
