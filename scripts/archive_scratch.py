#!/usr/bin/env python3
"""Losslessly compact inactive scratch or explicitly reviewed closed roots."""
import argparse
import os
from pathlib import Path
import re
import time

from scratch_archive import archive_root, walk
from storage import active_artifacts, check_space, cleanup_roots, exclusive, minimum_reserve

ROOT = Path(__file__).resolve().parents[1]
MAX_ROOTS = 64


def checked_root(root, value):
    """Reuse cleanup confinement, then reject archival's protected namespaces."""
    selected = cleanup_roots(root, [value])
    if len(selected) != 1:
        raise ValueError("Archive root or ancestor is pinned")
    path = selected[0]
    reserved = [root / name for name in ("obj/archives", "obj/parallel/resources")]
    if path == root / "obj" or any(
            path.is_relative_to(item) or item.is_relative_to(path) for item in reserved):
        raise ValueError("Archive roots cannot select obj, archives, or resource ledgers/ancestors")
    if not path.is_dir():
        raise ValueError("Archive roots must be existing closed directories")
    return path


def unpinned(folder):
    entries = walk(folder)
    try:
        return not any(path.name in {".keep-artifacts", ".git"} for path, _, _ in entries)
    finally:
        entries.close()


def selected_roots(root, hours, closed_roots):
    cutoff = time.time() - hours * 3600
    if closed_roots:
        if len(closed_roots) > MAX_ROOTS:
            raise ValueError("Select at most 64 closed roots per invocation")
        paths = [checked_root(root, value) for value in closed_roots]
        for path in paths:
            if path.stat().st_mtime >= cutoff or not unpinned(path):
                raise ValueError(f"Archive root is too recent or contains a pin/worktree: {path}")
        yield from cleanup_roots(root, paths)
        return
    # Existing immediate, old PID-named default selection remains unchanged.
    for parent in (root / "obj", root / "obj/parallel"):
        if not parent.is_dir() or parent.is_symlink() or (parent / ".keep-artifacts").exists():
            continue
        with os.scandir(parent) as entries:
            for entry in entries:
                path = Path(entry.path)
                if (entry.is_dir(follow_symlinks=False)
                        and re.search(r"-\d{3,}(?:-|$)", entry.name)
                        and entry.stat(follow_symlinks=False).st_mtime < cutoff and unpinned(path)):
                    yield checked_root(root, path)


def archive(root, hours, closed_roots=(), *, chunk_members=10_000, chunk_bytes=64 * 1024 ** 2):
    if hours < 0 or chunk_members < 1 or chunk_bytes < 1:
        raise ValueError("Age must be nonnegative; chunk bounds must be positive")
    with exclusive(root):
        check_space(root, minimum_reserve())
        if active_artifacts(root):
            raise ValueError("Stop compiler/test processes before archiving scratch")
        if (root / "obj/.keep-artifacts").exists():
            raise ValueError("obj is pinned; scratch archival refused")
        count = 0
        for folder in selected_roots(root, hours, closed_roots):
            archive_root(root, folder, chunk_members, chunk_bytes)
            count += 1
        if not count:
            print("No inactive PID-named scratch directories to archive")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--older-than-hours", type=float, default=24)
    parser.add_argument("--closed-root", action="append", default=[], metavar="DIRECTORY",
                        help="archive only explicitly reviewed closed roots inside obj; repeatable")
    parser.add_argument("--chunk-members", type=int, default=10_000)
    parser.add_argument("--chunk-bytes", type=int, default=64 * 1024 ** 2,
                        help="chunk payload threshold; a single file is never split")
    args = parser.parse_args()
    archive(ROOT, args.older_than_hours, args.closed_root,
            chunk_members=args.chunk_members, chunk_bytes=args.chunk_bytes)
