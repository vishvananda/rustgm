"""Local shared-worktree policy and bounded artifact accounting."""
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess

GIB = 1024 ** 3
DEFAULTS = {"schema": 1, "jobs": 2, "reserve_gib": 150, "worker_gib": 50,
            "cpu_percent": 800, "memory_gib": 12, "tasks": 256, "worktrees": {}}


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


class UsageScan:
    """Incremental scans keep disk monitoring responsive even with many files.

    Count allocated blocks conservatively, including pins and caches. No symlink
    is followed. Duplicate hardlinks may be charged twice rather than undercounted.
    """
    def __init__(self, roots):
        self.pending = list(roots)
        self.current = None
        self.total = 0
        self.done = False

    def close(self):
        if self.current is not None:
            self.current.close()
            self.current = None

    def step(self, budget=4096):
        for _ in range(budget):
            if self.current is None:
                if not self.pending:
                    self.done = True
                    return self.total
                folder = self.pending.pop()
                if folder.is_symlink():
                    raise ValueError("resource output became a symlink")
                try:
                    self.current = os.scandir(folder)
                except FileNotFoundError:
                    continue
            item = next(self.current, None)
            if item is None:
                self.close()
                continue
            try:
                info = item.stat(follow_symlinks=False)
                self.total += info.st_blocks * 512
                if item.is_dir(follow_symlinks=False):
                    self.pending.append(Path(item.path))
            except FileNotFoundError:
                continue
        return None


def usage(root, entry, policy, budget=None):
    budget = output_budget(policy) if budget is None else budget
    scan = UsageScan(output_roots(root, entry))
    try:
        while not scan.done:
            scan.step()
            free_space(root, policy)
            check_usage(scan.total, budget)
        return scan.total
    finally:
        scan.close()
