"""Retire superseded completed dev-fast sessions while preserving the warm cache."""
import json
from pathlib import Path
import shutil
import time

from storage import active_artifacts, exclusive


def retire(root):
    """Run only at an idle boundary, before resource admission and Cargo launch."""
    root = Path(root).resolve()
    with exclusive(root):
        if active_artifacts(root):
            raise ValueError("Incremental retirement refused: compiler/test is active")
        folder = root / "obj/rust/dev-fast/incremental"
        for ancestor in (root / "obj/rust", folder.parent, folder):
            if ancestor.is_symlink():
                raise ValueError("Incremental retirement refuses a symlinked cache")
        if not folder.exists():
            return []
        if any((ancestor / ".keep-artifacts").exists()
               for ancestor in (root / "obj", root / "obj/rust", folder.parent, folder)):
            return []
        rows = []
        for crate in sorted(folder.iterdir()):
            if crate.is_symlink():
                raise ValueError("Incremental retirement refuses a symlinked crate cache")
            if not crate.is_dir() or (crate / ".keep-artifacts").exists():
                continue
            sessions = []
            for session in crate.iterdir():
                if session.is_symlink():
                    raise ValueError("Incremental retirement refuses a symlinked session")
                if (session.is_dir() and session.name.startswith("s-")
                        and not session.name.endswith("-working")
                        and (session / "work-products.bin").is_file()
                        and not any(session.rglob(".keep-artifacts"))):
                    sessions.append(session)
            sessions.sort(key=lambda p: (p.stat().st_mtime_ns, p.name))
            for session in sessions[:-1]:
                if any(session.rglob(".keep-artifacts")):
                    continue
                unshared = 0
                for path in session.rglob("*"):
                    if path.is_file() and not path.is_symlink():
                        info = path.stat()
                        if info.st_nlink == 1:
                            unshared += info.st_blocks * 512
                rows.append({"path": str(session.relative_to(root)),
                             "unshared_bytes": unshared})
                shutil.rmtree(session)
        if rows:
            record = root / "obj/cleanup" / f"incremental-{time.time_ns()}.json"
            record.parent.mkdir(parents=True, exist_ok=True)
            record.write_text(json.dumps({"reason": "superseded completed dev-fast sessions",
                                          "retired": rows}, indent=2) + "\n")
        return rows
