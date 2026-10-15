#!/usr/bin/env python3
"""Bound generated artifacts; keep source, diagnostics, receipts and pinned baselines."""
import argparse
from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
GIB = 1024 ** 3
CACHE_DIRS = {"rust", "audit-venv", "cleanup"}
GENERATED_SUFFIXES = {".o", ".rlib", ".rmeta"}


def minimum_reserve():
    default = "8" if os.environ.get("GITHUB_ACTIONS") == "true" else "40"
    return float(os.environ.get("RUSTGM_MIN_FREE_GIB", default))


def check_space(root, minimum):
    free = shutil.disk_usage(root).free
    if free < minimum * GIB:
        raise ValueError(f"Only {free / GIB:.1f} GiB free; {minimum:g} GiB required. "
                         "Run make tidy (or make tidy-build for Cargo output) first.")


@contextmanager
def exclusive(root):
    folder = root / "obj"
    if folder.is_symlink():
        raise ValueError("Refusing a symlinked obj directory")
    folder.mkdir(exist_ok=True)
    with (folder / ".storage.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError("Another protected RustGM command is running; cleanup refused") from error
        yield


def active_artifacts(root):
    """Also protect unwrapped Cargo tests and measured executables on Linux."""
    proc = Path("/proc")
    if not proc.exists():
        return []
    prefix = str(root / "obj") + "/"
    active = []
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            executable = os.readlink(entry / "exe")
            command = (entry / "comm").read_text().strip()
            cwd = os.readlink(entry / "cwd")
            if executable.startswith(prefix) or (
                command in {"cargo", "rustc", "clippy-driver"}
                and (cwd == str(root) or cwd.startswith(str(root) + "/"))
            ):
                active.append(int(entry.name))
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
    return active


def disposable(path, info):
    if not stat.S_ISREG(info.st_mode):
        return False
    if path.suffix in GENERATED_SUFFIXES:
        return True
    if info.st_mode & 0o111 and info.st_size >= 4:
        with path.open("rb") as stream:
            return stream.read(4) == b"\x7fELF"
    return path.name in {"rendered-output.tmp", "full-sample.out"}


def prune(root, *, apply=False, hours=0, build_cache=False, since=0):
    active = active_artifacts(root)
    if active:
        raise ValueError(f"RustGM build/test executables still running: {active}; cleanup refused")
    obj = root / "obj"
    if obj.is_symlink():
        raise ValueError("Refusing a symlinked obj directory")
    cutoff = time.time() - hours * 3600
    records = obj / "cleanup"
    records.mkdir(parents=True, exist_ok=True)
    record = records / f"{time.time_ns()}.jsonl"
    count = freed = 0
    with record.open("w") as log:
        for directory, dirs, files in os.walk(obj, followlinks=False):
            folder = Path(directory)
            dirs[:] = [name for name in dirs if not (folder / name).is_symlink()
                       and name != ".git" and not (folder / name / ".keep-artifacts").exists()
                       and not (folder == obj and name in CACHE_DIRS)]
            for name in files:
                path = folder / name
                try:
                    info = path.lstat()
                    if not since <= info.st_mtime <= cutoff or not disposable(path, info):
                        continue
                    log.write(json.dumps({"path": str(path.relative_to(root)),
                                          "bytes": info.st_size, "links": info.st_nlink,
                                          "removed": apply}) + "\n")
                    if apply:
                        log.flush()
                        path.unlink()
                    count += 1
                    if info.st_nlink == 1:
                        freed += info.st_blocks * 512
                except FileNotFoundError:
                    continue
        cache = obj / "rust"
        if build_cache and cache.exists():
            if cache.is_symlink():
                raise ValueError("Refusing a symlinked Cargo cache")
            log.write(json.dumps({"path": "obj/rust", "removed": apply}) + "\n")
            if apply:
                shutil.rmtree(cache)
    print(f"{'Removed' if apply else 'Would remove'} {count} generated files "
          f"({freed / GIB:.2f} GiB unshared blocks); record: {record}")
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("check", "prune", "run"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--build-cache", action="store_true")
    parser.add_argument("--no-prune", action="store_true", help="guard build/check commands without a post-run scan")
    parser.add_argument("--hours", type=float, default=0)
    parser.add_argument("--minimum-gib", type=float,
                        default=minimum_reserve())
    args, command = parser.parse_known_args()
    if command[:1] == ["--"]:
        command = command[1:]
    if args.hours < 0 or args.minimum_gib < 0 or (command and args.action != "run"):
        parser.error("invalid arguments")
    try:
        if args.action == "check":
            check_space(ROOT, args.minimum_gib)
            return 0
        with exclusive(ROOT):
            if args.action == "prune":
                prune(ROOT, apply=args.apply, hours=args.hours, build_cache=args.build_cache)
                return 0
            if not command:
                parser.error("run requires a command after --")
            check_space(ROOT, args.minimum_gib)
            environment = dict(os.environ)
            environment.setdefault("CARGO_INCREMENTAL", "0")
            started = time.time()
            result = subprocess.run(command, cwd=ROOT, env=environment, check=False)
            # Failed runs retain their executable artifacts for diagnosis. An explicit
            # prune can remove them once the failure has been investigated.
            if result.returncode == 0 and not args.no_prune:
                prune(ROOT, apply=True, since=started)
            return result.returncode if result.returncode >= 0 else 128 - result.returncode
    except (OSError, ValueError) as error:
        print(f"storage: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
