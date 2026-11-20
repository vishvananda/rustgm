"""Local shared-worktree policy and bounded artifact accounting."""
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import stat
import subprocess
import time
import zlib

GIB = 1024 ** 3
DEFAULTS = {"schema": 1, "jobs": 2, "reserve_gib": 150, "worker_gib": 50,
            "cpu_percent": 800, "memory_gib": 12, "tasks": 256, "worktrees": {}}
# Reviewed Cargo job and libtest thread counts inside one job's CPU and memory
# scope. Fastcheck compiles one large unit at a time and uses every granted CPU.
# Other guarded builds may compile the library and its unit-test harness
# together, which peaks near MemoryMax even with two jobs; eight reach the cap.
WIDTH = 8
BUILD_JOBS = 2


def common_directory(root):
    result = subprocess.run(["git", "-C", str(root), "rev-parse", "--git-common-dir"],
                            capture_output=True, text=True, check=False)
    if result.returncode:
        return None
    return (root / result.stdout.strip()).resolve()


def discover(root):
    common = common_directory(root)
    if common is None or not (common / "rustgm-resources.json").is_file():
        return None
    policy = json.loads((common / "rustgm-resources.json").read_text())
    validate(policy)
    return common / "rustgm-resources", policy


def validate(policy):
    if policy.get("schema") not in (1, 2) or policy.get("jobs") not in (2, 3):
        raise ValueError("resource policy requires schema 1 or 2 and two or three job slots")
    for key in ("reserve_gib", "worker_gib", "cpu_percent", "memory_gib", "tasks"):
        if not isinstance(policy.get(key), int) or policy[key] <= 0:
            raise ValueError(f"invalid resource policy: {key}")
    if policy["reserve_gib"] < 150 or policy["worker_gib"] > 50:
        raise ValueError("parallel work requires at least 150 GiB reserve and at most 50 GiB per worker")
    if policy["cpu_percent"] > 800 or policy["memory_gib"] > 12 or policy["tasks"] > 256:
        raise ValueError("resource limits exceed the reviewed parallel policy")
    if not isinstance(policy.get("worktrees"), dict):
        raise ValueError("missing registered worktrees")


def width(policy=None, limit=WIDTH):
    """Build jobs and test threads use the CPUs a job is granted, up to a reviewed limit."""
    granted = policy["cpu_percent"] // 100 if policy else os.cpu_count() or 1
    return str(max(1, min(limit, granted)))


def output_budget(policy, maximum=None):
    """An explicit cap is absolute, not an allowance in addition to warm output."""
    configured = policy["worker_gib"] * GIB
    if maximum is None:
        return configured
    if policy["schema"] != 2:
        raise ValueError("--max-output-gib requires controller schema 2; activate at an idle boundary")
    if (isinstance(maximum, bool) or not isinstance(maximum, (int, float))
            or maximum <= 0 or maximum > policy["worker_gib"] or not math.isfinite(maximum)):
        raise ValueError("--max-output-gib must be positive, finite and no greater than worker_gib")
    budget = int(maximum * GIB)
    if budget < 1:
        raise ValueError("--max-output-gib must allow at least one byte")
    return budget


def policy_identity(policy):
    return hashlib.sha256(json.dumps(policy, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


class OutputLimitError(ValueError):
    def __init__(self, used, budget):
        self.used = used
        super().__init__(f"worktree generated artifacts {used} bytes exceed the absolute "
                         f"output cap {budget} bytes ({budget / GIB:g} GiB)")


def check_usage(used, budget):
    if used > budget:
        raise OutputLimitError(used, budget)


def output_roots(root, entry):
    selected = []
    for spelling in entry["outputs"]:
        path = Path(spelling)
        if path.is_absolute() or not path.parts or path.parts[0] != "obj" or ".." in path.parts:
            raise ValueError("resource outputs must be relative directories inside obj")
        current = root
        for part in path.parts:
            current /= part
            if current.is_symlink() or (current.exists() and not current.is_dir()):
                raise ValueError("resource outputs cannot contain symlinks or files")
        selected.append(root / path)
    if not selected:
        raise ValueError("resource accounting requires output roots")
    result = []
    for path in sorted(set(selected), key=lambda p: (len(p.parts), str(p))):
        if not any(path.is_relative_to(parent) for parent in result):
            result.append(path)
    return result


def atomic_json(path, value):
    temporary = path.with_name(path.name + f".{os.getpid()}.tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def free_space(root, policy, reserved=0):
    free = shutil.disk_usage(root).free
    minimum = policy["reserve_gib"] * GIB + reserved
    if free < minimum:
        raise ValueError(f"parallel storage: {free / GIB:.1f} GiB free, "
                         f"{minimum / GIB:.1f} GiB required including reservations")
    return free


# A directory whose own entry and files have all been quiet for SETTLE seconds
# keeps its listing while its inode and mtime are unchanged, for 30 to 60
# minutes (REFRESH, staggered per directory), and is then listed again: by the
# next live scan during a job, and by at most REFRESH_LIMIT directories in each
# admission or final scan, which are on a job's critical path.
SETTLE = 120
REFRESH = 3600
REFRESH_LIMIT = 2000
USAGE_CACHE = "obj/accounting/usage.json"


class UsageCache:
    """Per-directory listings: inode, mtime, non-directory blocks and subdirectories.

    Creating, removing or renaming an entry changes its directory's mtime, so a
    directory with an unchanged inode and mtime still has the same entries. Only
    a file grown in place inside a quiet directory can go uncounted, until that
    directory's listing is refreshed. Free space is always measured exactly.
    """

    def __init__(self, root, rows=None, *, refresh_limit=None):
        self.path = root / USAGE_CACHE
        # Expired listings this scan may list again; None lists every one.
        self.refresh_left = refresh_limit
        if rows is None:
            try:
                document = json.loads(self.path.read_text())
                rows = document["dirs"] if document.get("version") == 1 else {}
            except (OSError, ValueError, KeyError, TypeError, AttributeError):
                rows = {}
        self.rows = rows if isinstance(rows, dict) else {}
        self.seen = {}

    def reusable(self, key, info, now):
        row = self.rows.get(key)
        if not (isinstance(row, list) and len(row) == 6 and row[:2] == [info.st_ino, info.st_mtime_ns]
                and row[4] is True and isinstance(row[5], (int, float))
                and isinstance(row[2], int) and isinstance(row[3], list)):
            return None
        if now >= row[5]:
            if self.refresh_left is None or self.refresh_left > 0:
                if self.refresh_left is not None:
                    self.refresh_left -= 1
                return None
        self.seen[key] = row
        return row[2], row[3]

    def listed(self, key, info, files, folders, newest, now):
        spread = 0.5 + (zlib.crc32(key.encode()) % 1000) / 2000
        self.seen[key] = [info.st_ino, info.st_mtime_ns, files, folders, now - newest > SETTLE,
                          now + REFRESH * spread]

    def save(self, roots):
        """Keep rows outside this scan's roots; replace the rest with what it saw."""
        prefixes = [str(path) for path in roots]
        kept = {key: row for key, row in self.rows.items()
                if not any(key == prefix or key.startswith(prefix + os.sep) for prefix in prefixes)}
        try:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            temporary = self.path.with_name(f"{self.path.name}.{os.getpid()}.tmp")
            temporary.write_text(json.dumps({"version": 1, "dirs": {**kept, **self.seen}},
                                            separators=(",", ":")))
            temporary.replace(self.path)
        except OSError:
            pass


class UsageScan:
    """Incremental scans keep disk monitoring responsive even with many files.

    Count allocated blocks conservatively, including pins and caches. No symlink
    is followed. Duplicate hardlinks may be charged twice rather than undercounted.
    With a cache, a quiet unchanged directory costs one stat per subdirectory
    instead of one per entry.
    """
    def __init__(self, roots, cache=None):
        self.roots = list(roots)
        # Plain strings: pathlib objects cost more than the stat calls on large trees.
        self.pending = [(str(path), None) for path in self.roots]
        self.cache = cache
        self.total = 0
        # Directory listings plus stats: the scan's work, for receipts and bounds.
        self.operations = 0
        self.done = False

    def close(self):
        """A scan holds no directory open between steps."""

    def step(self, budget=4096):
        operations = 0
        while operations < budget:
            if not self.pending:
                if not self.done and self.cache is not None:
                    self.cache.save(self.roots)
                self.done = True
                return self.total
            folder, info = self.pending.pop()
            work = self.visit(folder, info)
            operations += work
            self.operations += work
        return None

    def listing(self, folder, info, now):
        files, folders, newest, operations = 0, [], max(info.st_mtime, info.st_ctime), 1
        with os.scandir(folder) as entries:
            for item in entries:
                if item.is_dir(follow_symlinks=False):
                    folders.append(item.name)
                    continue
                try:
                    found = item.stat(follow_symlinks=False)
                except FileNotFoundError:
                    continue
                operations += 1
                files += found.st_blocks * 512
                newest = max(newest, found.st_mtime, found.st_ctime)
        if self.cache is not None:
            self.cache.listed(folder, info, files, folders, newest, now)
        return files, folders, operations

    def visit(self, folder, info):
        try:
            info = info or os.lstat(folder)
        except FileNotFoundError:
            return 1
        if stat.S_ISLNK(info.st_mode):
            raise ValueError("resource output became a symlink")
        if not stat.S_ISDIR(info.st_mode):
            return 1
        now = time.time()
        cached = self.cache.reusable(folder, info, now) if self.cache is not None else None
        try:
            if cached:
                files, folders, operations = (*cached, 1)
            else:
                # Check again just before listing, so a directory replaced by a link is never followed.
                info = os.lstat(folder)
                if stat.S_ISLNK(info.st_mode):
                    raise ValueError("resource output became a symlink")
                files, folders, operations = self.listing(folder, info, now)
        except FileNotFoundError:
            return 1
        self.total += files
        prefix = folder + os.sep
        for name in folders:
            path = prefix + name
            try:
                found = os.lstat(path)
            except FileNotFoundError:
                continue
            operations += 1
            self.total += found.st_blocks * 512
            if stat.S_ISDIR(found.st_mode):
                self.pending.append((path, found))
        return operations


def usage(root, entry, policy, budget=None, cache=None):
    """A blocking scan before or after a job: it lists a bounded number of expired directories."""
    budget = output_budget(policy) if budget is None else budget
    cache = UsageCache(root, refresh_limit=REFRESH_LIMIT) if cache is None else cache
    scan = UsageScan(output_roots(root, entry), cache)
    try:
        while not scan.done:
            scan.step()
            free_space(root, policy)
            check_usage(scan.total, budget)
        return scan.total
    finally:
        scan.close()
