"""Cross-worktree leases: bounded heavy jobs or one exclusive validation batch."""
from contextlib import contextmanager, ExitStack
import fcntl
import hashlib
import json
import os
from pathlib import Path
import threading
import time
import uuid

from resource_policy import atomic_json, output_budget, policy_identity


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


def process_identity(pid):
    """PID plus kernel start ticks distinguish an ancestor from a reused PID."""
    text = Path(f"/proc/{pid}/stat").read_text()
    fields = text[text.rfind(")") + 2:].split()
    return int(fields[1]), int(fields[19])


@contextmanager
def admission(directory):
    """Never wait for a slot or another lease while holding this short lock."""
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / "admission.lock").open("a+") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def _active_records(directory, slots):
    result = []
    for index in range(slots):
        with (directory / f"slot-{index}.lock").open("a+") as handle:
            if try_lock(handle, fcntl.LOCK_EX):
                continue
            path = directory / f"slot-{index}.json"
            # Schema 1 owners do not participate in admission locking. Missing,
            # stale or legacy records must never supply a smaller reservation.
            try:
                row = json.loads(path.read_text())
                if not isinstance(row, dict) or row.get("slot") != index:
                    row = {"slot": index, "state": "acquiring"}
                result.append(row)
            except (FileNotFoundError, json.JSONDecodeError):
                result.append({"slot": index, "state": "acquiring"})
    return result


def active_records(directory, slots):
    with admission(directory):
        return _active_records(directory, slots)


def holders(rows):
    """Name the running jobs so a queued command shows what it is waiting for."""
    now = time.time()
    names = []
    for row in rows:
        started = row.get("started")
        age = f" for {max(0, now - started):.0f}s" if isinstance(started, (int, float)) else ""
        names.append(Path(str(row.get("root") or "an acquiring job")).name + age)
    return ", ".join(names) or "a finishing job"


# A waiter refreshes its ticket while it polls. One silent this long (a stopped
# process) stops holding the queue for the waiters behind it.
QUEUE_STALL = 60
HEARTBEAT = 5
# While a waiter queues, it repeats its idle work (accounting) at most this often.
IDLE_INTERVAL = 60


class Ticket:
    """A first-come-first-served place in the heavy-job slot queue.

    Only the oldest live ticket may claim a slot, and it leaves the queue in the
    same admission critical section, so the next waiter is at the head at once.
    A ticket is live while its waiter holds its flock: a killed waiter never
    blocks the queue. Waiters running older scripts take no ticket; they may
    still claim a free slot first, but nothing here can block them.
    """

    def __init__(self, directory, root, kind):
        self.directory = directory
        self.folder = directory / "queue"
        self.folder.mkdir(parents=True, exist_ok=True)
        self.handle = None
        self.beaten = time.monotonic()
        with admission(directory):
            # Creation and locking are one admission section, so no scan sees an
            # unlocked new ticket. Numbers only need to exceed every queued one.
            number = 1 + max((int(path.name.partition("-")[0]) for path in self.folder.glob("*.ticket")
                              if path.name.partition("-")[0].isdigit()), default=0)
            self.path = self.folder / f"{number:012d}-{uuid.uuid4().hex}.ticket"
            self.handle = self.path.open("x+")
            fcntl.flock(self.handle, fcntl.LOCK_EX)
            json.dump({"root": str(root), "kind": kind, "pid": os.getpid(), "queued": time.time()},
                      self.handle)
            self.handle.flush()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        if self.handle is not None:
            with admission(self.directory):
                self.leave()

    def ahead(self):
        """Live, recently refreshed tickets queued earlier. Hold the admission lock."""
        now = time.time()
        rows = []
        for path in sorted(self.folder.glob("*.ticket")):
            if path.name >= self.path.name:
                break
            try:
                with path.open("r+") as handle:
                    if try_lock(handle, fcntl.LOCK_EX):
                        path.unlink(missing_ok=True)  # its waiter exited without leaving
                        continue
                if now - path.stat().st_mtime < QUEUE_STALL:
                    rows.append(path)
            except FileNotFoundError:
                continue
        return rows

    def leave(self):
        """Hold the admission lock: unlink while still locked, so no scan sees a dead ticket."""
        if self.handle is not None:
            self.path.unlink(missing_ok=True)
            self.handle.close()
            self.handle = None

    def beat(self, now):
        if now - self.beaten >= HEARTBEAT:
            self.beaten = now
            try:
                os.utime(self.path)
            except OSError:
                pass


def queued(directory):
    """The live queue in order, for status output."""
    rows = []
    with admission(directory):
        for path in sorted((directory / "queue").glob("*.ticket")):
            try:
                with path.open("r+") as handle:
                    if try_lock(handle, fcntl.LOCK_EX):
                        continue
                    row = json.loads(handle.read() or "{}")
                rows.append({**row, "stale": time.time() - path.stat().st_mtime >= QUEUE_STALL})
            except (FileNotFoundError, json.JSONDecodeError):
                continue
    return rows


def peer_budget(row, policy):
    configured = output_budget(policy)
    budget = row.get("budget_bytes")
    if (policy["schema"] == 2 and row.get("schema") == 2
            and row.get("policy_identity") == policy_identity(policy)
            and row.get("state") in {"acquired", "starting", "running"}
            and type(budget) is int and 0 < budget <= configured
            and type(row.get("pid")) is int and row["pid"] > 1
            and type(row.get("process_start")) is int):
        try:
            if process_identity(row["pid"])[1] == row["process_start"]:
                return budget
        except OSError:
            pass
    return configured


@contextmanager
def lease(directory, root, policy, kind, wait=3600, *, budget=None, idle=None):
    """Queue for a slot and hold it. While waiting, run idle() in a thread at most once per
    IDLE_INTERVAL; it finishes before the lease is granted, so it never overlaps the job."""
    directory.mkdir(parents=True, exist_ok=True)
    configured = output_budget(policy)
    budget = configured if budget is None else budget
    if (type(budget) is not int or not 0 < budget <= configured
            or (budget != configured and policy["schema"] != 2)):
        raise ValueError("invalid output cap for controller protocol")
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
        place = stack.enter_context(Ticket(directory, root, kind))
        index = None
        announced = 0
        helper, next_idle = None, 0.0
        while index is None:
            # Publication is part of claiming the slot: a concurrent snapshot
            # cannot observe a newly held lock with its previous owner's cap.
            with admission(directory):
                ahead = place.ahead()
                for ordinal, handle in enumerate(() if ahead else slots):
                    if try_lock(handle, fcntl.LOCK_EX):
                        index = ordinal
                        break
                if index is not None:
                    place.leave()
                    record = {"pid": os.getpid(), "token": uuid.uuid4().hex, "root": str(root),
                              "kind": kind, "slot": index, "started": time.time(),
                              "schema": policy["schema"], "state": "acquired",
                              "budget_bytes": budget}
                    if policy["schema"] == 2:
                        record.update(policy_identity=policy_identity(policy),
                                      controller=str(directory.resolve()),
                                      process_start=process_identity(os.getpid())[1])
                    path = directory / f"slot-{index}.json"
                    atomic_json(path, record)
            if index is None:
                now = time.monotonic()
                if now >= deadline:
                    raise ValueError("timed out waiting for a heavy-job slot")
                place.beat(now)
                if idle is not None and now >= next_idle and (helper is None or not helper.is_alive()):
                    helper = threading.Thread(target=idle, daemon=True)
                    helper.start()
                    next_idle = now + IDLE_INTERVAL
                if now - announced >= 30:
                    turn = f"{len(ahead)} queued ahead" if ahead else "next in the queue"
                    print("resources: waiting for a heavy-job slot held by "
                          + holders(active_records(directory, policy["jobs"])) + f"; {turn}", flush=True)
                    announced = now
                time.sleep(0.2)
        if helper is not None:
            helper.join()
        # Keep leases held by the launched process if its supervisor is killed.
        held = [worker, gate, slots[index]] + ([turn] if kind == "exclusive" else [])
        # Kernel locks, rather than stale JSON or PID reuse, own availability.
        yield record, path, tuple(handle.fileno() for handle in held)


def inherited(directory, root, environment, kind, slots=2, *, policy=None):
    path = environment.get("RUSTGM_RESOURCE_LEASE")
    token = environment.get("RUSTGM_RESOURCE_TOKEN")
    if not path and not token:
        return False
    path = Path(path or "")
    if path.parent != directory or path.name not in {f"slot-{i}.json" for i in range(slots)}:
        raise ValueError("invalid inherited resource lease")
    record = json.loads(path.read_text())
    if not isinstance(record, dict) or record.get("token") != token or record.get("root") != str(root):
        raise ValueError("foreign or expired resource lease")
    if policy is not None and policy["schema"] == 2:
        if (record.get("schema") != 2 or record.get("policy_identity") != policy_identity(policy)
                or record.get("controller") != str(directory.resolve())
                or path.name != f"slot-{record.get('slot')}.json"
                or type(record.get("pid")) is not int or record["pid"] <= 1
                or type(record.get("process_start")) is not int
                or record.get("kind") not in {"ordinary", "exclusive"}):
            raise ValueError("inherited resource lease belongs to another controller policy")
    pid = os.getppid()
    for _ in range(256):
        if pid == record.get("pid"):
            if policy is not None and policy["schema"] == 2:
                try:
                    actual_start = process_identity(pid)[1]
                except FileNotFoundError as error:
                    raise ValueError("resource lease ancestor exited") from error
                if actual_start != record.get("process_start"):
                    raise ValueError("resource lease ancestor process identity changed")
            if kind == "exclusive" and record["kind"] != "exclusive":
                raise ValueError("cannot upgrade an ordinary lease; request exclusive at the outer command")
            return record
        if pid <= 1:
            break
        try:
            pid = process_identity(pid)[0]
        except FileNotFoundError:
            break
    raise ValueError("resource lease does not belong to a live ancestor")


def inherited_budget(record, policy, requested=None):
    """Only the existing outer supervisor can enforce a nested command's cap."""
    configured = output_budget(policy)
    budget = record.get("budget_bytes", configured)
    if type(budget) is not int or not 0 < budget <= configured:
        raise ValueError("invalid inherited output cap")
    if policy["schema"] == 2:
        if (record.get("schema") != 2 or record.get("state") not in {"starting", "running"}
                or record.get("supervised_budget_bytes") != budget):
            raise ValueError("inherited output cap lacks an active outer supervisor")
    elif budget != configured:
        raise ValueError("schema 1 cannot inherit a variable output cap")
    if requested is not None and requested != budget:
        raise ValueError("nested output cap must equal the outer cap; request it at the outer command")
    return budget
