"""Retire idle codegen scratch and superseded sessions; preserve warm caches."""
import json
import fcntl
from pathlib import Path
import shutil
import time

from storage import active_artifacts, exclusive


def remove_session(root, session):
    unshared = 0
    for path in session.rglob("*"):
        if path.is_file() and not path.is_symlink():
            info = path.stat()
            if info.st_nlink == 1:
                unshared += info.st_blocks * 512
    row = {"path": str(session.relative_to(root)), "unshared_bytes": unshared}
    shutil.rmtree(session)
    return row


def retire_abandoned_session(root, session):
    """A known working session is abandoned only when its Rust lock is free."""
    lock = session.parent / (session.name.removesuffix("-working") + ".lock")
    if (not session.name.endswith("-working")
            or not (session / "work-products.bin").is_file()
            or any(session.rglob(".keep-artifacts")) or not lock.exists()):
        return None
    if lock.is_symlink():
        raise ValueError("Incremental retirement refuses a symlinked session lock")
    with lock.open("rb") as descriptor:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return None
        return remove_session(root, session)


def retire_codegen_objects(root, folder):
    """Loose rcgu objects are linker scratch; incremental copies remain intact."""
    if not folder.exists() or (folder / ".keep-artifacts").exists():
        return []
    paths = sorted(folder.glob("*.rcgu.o"))
    if any(path.is_symlink() for path in paths):
        raise ValueError("Incremental retirement refuses a symlinked codegen object")
    rows = []
    for path in paths:
        if not path.is_file():
            continue
        info = path.stat()
        rows.append({"path": str(path.relative_to(root)),
                     "unshared_bytes": info.st_blocks * 512 if info.st_nlink == 1 else 0})
        path.unlink()
    return rows


def retire(root):
    """Run only at an idle boundary, before resource admission and Cargo launch."""
    root = Path(root).resolve()
    with exclusive(root):
        if active_artifacts(root):
            raise ValueError("Incremental retirement refused: compiler/test is active")
        folder = root / "obj/rust/dev-fast/incremental"
        objects = folder.parent / "deps"
        for ancestor in (root / "obj", root / "obj/rust", folder.parent, folder, objects):
            if ancestor.is_symlink():
                raise ValueError("Incremental retirement refuses a symlinked cache")
        if any((ancestor / ".keep-artifacts").exists()
               for ancestor in (root / "obj", root / "obj/rust", folder.parent, folder)):
            return []
        rows = retire_codegen_objects(root, objects)
        for crate in sorted(folder.iterdir()) if folder.exists() else ():
            if crate.is_symlink():
                raise ValueError("Incremental retirement refuses a symlinked crate cache")
            if not crate.is_dir() or (crate / ".keep-artifacts").exists():
                continue
            sessions = []
            for session in crate.iterdir():
                if session.is_symlink():
                    raise ValueError("Incremental retirement refuses a symlinked session")
                if session.is_dir() and session.name.startswith("s-"):
                    abandoned = retire_abandoned_session(root, session)
                    if abandoned is not None:
                        rows.append(abandoned)
                        continue
                if (session.is_dir() and session.name.startswith("s-")
                        and not session.name.endswith("-working")
                        and (session / "work-products.bin").is_file()
                        and not any(session.rglob(".keep-artifacts"))):
                    sessions.append(session)
            sessions.sort(key=lambda p: (p.stat().st_mtime_ns, p.name))
            for session in sessions[:-1]:
                if any(session.rglob(".keep-artifacts")):
                    continue
                rows.append(remove_session(root, session))
        if rows:
            record = root / "obj/cleanup" / f"incremental-{time.time_ns()}.json"
            record.parent.mkdir(parents=True, exist_ok=True)
            record.write_text(json.dumps({"reason": "idle codegen scratch and superseded dev-fast sessions",
                                          "retired": rows}, indent=2) + "\n")
        return rows
