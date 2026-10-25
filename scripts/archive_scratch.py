#!/usr/bin/env python3
"""Losslessly compact inactive PID-named test inputs without losing evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tarfile
import time

from storage import active_artifacts, check_space, exclusive, minimum_reserve

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def snapshot(folders):
    return {path: (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
            for folder in folders for path in [folder, *folder.rglob("*")]
            for info in [path.lstat()]}


def archive(root, hours):
    with exclusive(root):
        check_space(root, minimum_reserve())
        if active_artifacts(root):
            raise ValueError("stop compiler/test processes before archiving scratch")
        if (root / "obj/.keep-artifacts").exists():
            raise ValueError("obj is pinned; scratch archival refused")
        cutoff = time.time() - hours * 3600
        parents = [root / "obj", root / "obj/parallel"]
        folders = sorted(path for parent in parents if parent.is_dir()
                         and not parent.is_symlink() and not (parent / ".keep-artifacts").exists()
                         for path in parent.iterdir()
                         if path.is_dir() and not path.is_symlink()
                         and re.search(r"-\d{3,}(?:-|$)", path.name)
                         and path.stat().st_mtime < cutoff)
        folders = [path for path in folders if not any(path.rglob(".keep-artifacts"))]
        if not folders:
            print("No inactive PID-named scratch directories to archive")
            return
        destination = root / "obj/archives"
        if destination.is_symlink():
            raise ValueError("archive directory cannot be a symlink")
        destination.mkdir(exist_ok=True)
        path = destination / f"scratch-{time.time_ns()}.tar.gz"
        manifest = {}
        originals = snapshot(folders)
        # Tar stores symlinks themselves; no external target is read or removed.
        with tarfile.open(path, "w:gz", compresslevel=1, dereference=False) as output:
            for folder in folders:
                for item in sorted([folder, *folder.rglob("*")]):
                    relative = item.relative_to(root).as_posix()
                    if item.is_file() and not item.is_symlink():
                        manifest[relative] = digest(item)
                    output.add(item, arcname=relative, recursive=False)
        with tarfile.open(path, "r:gz") as saved:
            for member in saved:
                if member.isfile() or member.islnk():
                    with saved.extractfile(member) as content:
                        restored_digest = hashlib.file_digest(content, "sha256").hexdigest()
                    if restored_digest != manifest.pop(member.name):
                        raise ValueError(f"archive verification failed: {member.name}")
        if manifest:
            raise ValueError("archive is missing original files; originals retained")
        if snapshot(folders) != originals:
            raise ValueError("scratch changed while archiving; originals retained")
        checksum = digest(path)
        # Verification finished before removal. Keep a searchable top-level index.
        receipt = {"archive": str(path.relative_to(root)), "sha256": checksum,
                   "directories": [str(folder.relative_to(root)) for folder in folders],
                   "restore": f"tar -xzf {path.relative_to(root)} -C .",
                   "state": "verified"}
        index = path.with_suffix(".json")
        index.write_text(json.dumps(receipt, indent=2) + "\n")
        for folder in folders:
            shutil.rmtree(folder)
        receipt["state"] = "archived"
        index.write_text(json.dumps(receipt, indent=2) + "\n")
        print(f"Archived {len(folders)} scratch directories: {path}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--older-than-hours", type=float, default=24)
    args = parser.parse_args()
    if args.older_than_hours < 0:
        parser.error("age must be nonnegative")
    archive(ROOT, args.older_than_hours)
