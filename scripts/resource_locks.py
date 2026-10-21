"""Cross-worktree leases: two heavy jobs or one exclusive validation batch."""
from contextlib import contextmanager, ExitStack
import fcntl
import hashlib
import json
import os
from pathlib import Path
import time
import uuid

from resource_policy import atomic_json


def try_lock(handle, mode):
    try:
        fcntl.flock(handle, mode | fcntl.LOCK_NB)
        return True
    except BlockingIOError:
        return False


def acquire(handle, mode, deadline):
    announced = 0
    while not try_lock(handle, mode):
        now = time.monotonic()
        if now >= deadline:
            raise ValueError("timed out waiting for a shared resource lease")
        if now - announced >= 15:
            print("resources: waiting for another guarded job", flush=True)
            announced = now
        time.sleep(0.2)


def active_records(directory, slots):
    result = []
    for index in range(slots):
        with (directory / f"slot-{index}.lock").open("a+") as handle:
            if try_lock(handle, fcntl.LOCK_EX):
                continue
            path = directory / f"slot-{index}.json"
            # An owner may be between acquiring the kernel lock and publishing.
            try:
                result.append(json.loads(path.read_text()))
            except (FileNotFoundError, json.JSONDecodeError):
                result.append({"slot": index, "state": "acquiring"})
    return result


@contextmanager
def lease(directory, root, policy, kind, wait=3600):
    directory.mkdir(parents=True, exist_ok=True)
    deadline = time.monotonic() + wait
    with ExitStack() as stack:
        def opened(name):
            return stack.enter_context((directory / name).open("a+"))

        # An exclusive waiter holds the turnstile, so new ordinary jobs cannot
        # starve it. Worktree ownership also covers source-mutating Cargo tools.
        identity = hashlib.sha256(str(root).encode()).hexdigest()[:20]
        worker = opened(f"worktree-{identity}.lock")
        acquire(worker, fcntl.LOCK_EX, deadline)
        turn = opened("turnstile.lock")
        acquire(turn, fcntl.LOCK_EX if kind == "exclusive" else fcntl.LOCK_SH, deadline)
        gate = opened("gate.lock")
        acquire(gate, fcntl.LOCK_EX if kind == "exclusive" else fcntl.LOCK_SH, deadline)
        if kind != "exclusive":
            fcntl.flock(turn, fcntl.LOCK_UN)
        slots = [opened(f"slot-{index}.lock") for index in range(policy["jobs"])]
        index = None
        while index is None:
            for ordinal, handle in enumerate(slots):
                if try_lock(handle, fcntl.LOCK_EX):
                    index = ordinal
                    break
            if index is None:
                if time.monotonic() >= deadline:
                    raise ValueError("timed out waiting for a heavy-job slot")
                time.sleep(0.2)
        record = {"pid": os.getpid(), "token": uuid.uuid4().hex, "root": str(root),
                  "kind": kind, "slot": index, "started": time.time(),
                  "budget_bytes": policy["worker_gib"] * 1024 ** 3}
        path = directory / f"slot-{index}.json"
        atomic_json(path, record)
        # Keep leases held by the launched process if its supervisor is killed.
        held = [worker, gate, slots[index]] + ([turn] if kind == "exclusive" else [])
        # Kernel locks, rather than stale JSON or PID reuse, own availability.
        yield record, path, tuple(handle.fileno() for handle in held)


def inherited(directory, root, environment, kind):
    path = environment.get("RUSTGM_RESOURCE_LEASE")
    token = environment.get("RUSTGM_RESOURCE_TOKEN")
    if not path and not token:
        return False
    path = Path(path or "")
    if path.parent != directory or path.name not in {"slot-0.json", "slot-1.json"}:
        raise ValueError("invalid inherited resource lease")
    record = json.loads(path.read_text())
    if record.get("token") != token or record.get("root") != str(root):
        raise ValueError("foreign or expired resource lease")
    pid = os.getppid()
    for _ in range(256):
        if pid == record.get("pid"):
            if kind == "exclusive" and record["kind"] != "exclusive":
                raise ValueError("cannot upgrade an ordinary lease; request exclusive at the outer command")
            return True
        if pid <= 1:
            break
        try:
            text = Path(f"/proc/{pid}/stat").read_text()
        except FileNotFoundError:
            break
        pid = int(text[text.rfind(")") + 2:].split()[1])
    raise ValueError("resource lease does not belong to a live ancestor")
