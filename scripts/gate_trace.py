"""Run a Python gate as `python script.py ARGS` would, recording what it reads.

Usage: gate_trace.py TRACE SCRIPT [ARGS...]. TRACE receives the files opened,
the directories listed and whether any process was started, from Python audit
events. A cached pass is only as sound as its fingerprint, so fastcheck caches a
traced gate only when every recorded read lies inside the gate's declared inputs.
"""
import json
import os
from pathlib import Path
import runpy
import sys

SPAWNS = {"subprocess.Popen", "os.system", "os.posix_spawn", "os.spawn", "os.exec", "os.fork", "os.forkpty"}


def main():
    trace, script, *arguments = sys.argv[1:]
    opened, listed, spawned = set(), set(), []
    writing = []

    def hook(event, payload):
        if writing:
            return
        if event == "open" and payload and isinstance(payload[0], (str, bytes, os.PathLike)):
            opened.add(os.path.abspath(os.fsdecode(payload[0])))
        elif event in ("os.listdir", "os.scandir") and payload:
            folder = payload[0]
            if isinstance(folder, (str, bytes, os.PathLike)) or folder is None:
                listed.add(os.path.abspath(os.fsdecode(folder if folder is not None else ".")))
        elif event in SPAWNS:
            spawned.append(event)

    sys.addaudithook(hook)
    sys.argv = [script, *arguments]
    sys.path.insert(0, str(Path(script).resolve().parent))
    status = 0
    try:
        runpy.run_path(script, run_name="__main__")
    except SystemExit as exit_:
        status = exit_.code if isinstance(exit_.code, int) else (0 if exit_.code is None else 1)
    finally:
        writing.append(True)
        Path(trace).write_text(json.dumps({"opened": sorted(opened), "listed": sorted(listed),
                                           "spawned": sorted(set(spawned))}) + "\n")
    return status


if __name__ == "__main__":
    sys.exit(main())
