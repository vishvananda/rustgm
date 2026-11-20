"""Fastcheck's static gates, keyed on a fingerprint of every input they read.

A gate whose fingerprint matches one it passed with is skipped, and its receipt
row names the fingerprint and the receipt that passed. The fingerprint covers
the gate's command, this implementation, the interpreter or tool version and
the content of every declared input. Python gates run under `gate_trace.py`; a
pass is cached only when every file the gate opened or listed lies inside its
declared inputs and it started no process, so a cache cannot hide an input.

Formatting is checked per file: a file's verdict depends only on its content,
the rustfmt version, its configuration and the edition. With those unchanged,
only changed files are checked, and any failure of that partial check is
decided by a full `cargo fmt --check`.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import sysconfig
import time

STATE = Path("obj/fast-gates")
# A file written within this window may change again inside one timestamp tick.
RACY = 2.0
KEPT = 16
IMPLEMENTATION = ("scripts/fastcheck_gates.py", "scripts/gate_trace.py")
METADATA_FILES = ("assignments.json", "assignment-plan.json", "Makefile")
METADATA_TREES = ("docs", "scripts")
# Bytecode caches are derived from fingerprinted sources and validated by Python.
SKIPPED_DIRECTORIES = {"__pycache__"}
AUDIT_FILES = ("scripts/rust_audit.py", "scripts/rust-module-roles.json")
# Rustfmt reads the nearest configuration above each file, else the user's.
CONFIGURATION = ("rustfmt.toml", ".rustfmt.toml")


def digest_text(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


class Digests:
    """Content hashes reused while a file's inode, size, mtime and ctime are unchanged."""

    def __init__(self, root):
        self.root = root
        self.path = root / STATE / "digests.json"
        try:
            self.rows = json.loads(self.path.read_text())
        except (OSError, ValueError):
            self.rows = {}
        if not isinstance(self.rows, dict):
            self.rows = {}
        self.used = {}

    def file(self, path):
        return self.digest(str(path), path.relative_to(self.root).as_posix(), path.lstat())

    def digest(self, path, relative, info):
        key = [info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns]
        row = self.used.get(relative) or self.rows.get(relative)
        if isinstance(row, list) and len(row) == 5 and row[:4] == key:
            digest = row[4]
        else:
            with open(path, "rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if time.time() - max(info.st_mtime, info.st_ctime) > RACY:
            self.used[relative] = key + [digest]
        return digest

    def tree(self, folder, *, suffix="", skipped=frozenset()):
        """Every directory and file below folder in a stable order; links are not followed."""
        prefix = len(str(self.root)) + 1
        rows = []
        pending = [str(folder)]
        while pending:
            current = pending.pop()
            rows.append(["dir", current[prefix:]])
            try:
                entries = sorted(os.scandir(current), key=lambda entry: entry.name)
            except FileNotFoundError:
                continue
            folders = []
            for entry in entries:
                if entry.is_dir(follow_symlinks=False):
                    if entry.name not in skipped:
                        folders.append(entry.path)
                elif entry.name.endswith(suffix):
                    relative = entry.path[prefix:]
                    if entry.is_symlink():
                        rows.append(["link", relative, os.readlink(entry.path)])
                    else:
                        rows.append(["file", relative,
                                     self.digest(entry.path, relative, entry.stat(follow_symlinks=False))])
            pending.extend(reversed(folders))
        return rows

    def files(self, names):
        rows = []
        for name in names:
            path = self.root / name
            rows.append(["file", name, self.file(path)] if path.is_file() else ["absent", name])
        return rows

    def save(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
        temporary = self.path.with_name(f"{self.path.name}.{os.getpid()}.tmp")
        temporary.write_text(json.dumps(self.used, separators=(",", ":")))
        temporary.replace(self.path)


def tool_version(command, root):
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, check=False)
    return [result.returncode, result.stdout.strip(), result.stderr.strip()]


def audit_interpreter(python):
    """The audit venv's identity: its configuration and installed distributions."""
    venv = Path(python).parent.parent
    try:
        configuration = (venv / "pyvenv.cfg").read_text()
    except OSError:
        configuration = None
    installed = sorted(path.name for path in venv.glob("lib/python*/site-packages/*.dist-info"))
    return [str(python), configuration, installed]


def interpreter_paths():
    """The standard library and site directories of the Python running the gates."""
    paths = sysconfig.get_paths()
    return sorted({Path(paths[key]) for key in ("stdlib", "platstdlib", "purelib", "platlib") if key in paths})


def edition(root):
    found = re.findall(r'^\s*edition\s*=\s*"([^"]+)"', (root / "dev/Cargo.toml").read_text(), re.M)
    return found[0] if len(set(found)) == 1 else None


def configuration_files(root):
    """Rustfmt configuration that any dev file could resolve, present or absent."""
    folders = [root / "dev", *(root / "dev").parents]
    folders += [Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config")) / "rustfmt"]
    rows = []
    for folder in folders:
        for name in CONFIGURATION:
            path = folder / name
            rows.append([str(path), hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None])
    for path in sorted((root / "dev").rglob("*rustfmt.toml")):
        rows.append([str(path), hashlib.sha256(path.read_bytes()).hexdigest()])
    return rows


class Gates:
    """Fingerprints, passed-run records and trace checks for the three static gates."""

    def __init__(self, root, audit_python, digests=None):
        self.root = root
        self.audit_python = audit_python
        self.digests = digests or Digests(root)
        self.implementation = self.digests.files(IMPLEMENTATION)
        self.sources = {row[1]: row[2] for row in self.digests.tree(root / "dev", suffix=".rs")
                        if row[0] == "file"}
        (root / STATE).mkdir(parents=True, exist_ok=True)

    def fresh(self):
        """The same fingerprints recomputed now, reusing this run's digests."""
        digests = Digests.__new__(Digests)
        digests.root, digests.path = self.digests.root, self.digests.path
        digests.rows, digests.used = {**self.digests.rows, **self.digests.used}, self.digests.used
        return type(self)(self.root, self.audit_python, digests)

    def metadata(self):
        command = [sys.executable, "scripts/assignment_metadata.py"]
        trees = sorted(path.name for path in self.root.glob("pa*") if path.is_dir()) + list(METADATA_TREES)
        inputs = [self.digests.files(METADATA_FILES)]
        inputs += [self.digests.tree(self.root / name, skipped=SKIPPED_DIRECTORIES) for name in trees]
        identity = [sys.version, sys.executable]
        allowed = [self.root / name for name in (*METADATA_FILES, *trees)] + interpreter_paths()
        return command, digest_text([command, self.implementation, identity, inputs]), allowed

    def architecture(self):
        command = [str(self.audit_python), "scripts/rust_audit.py", "--root", ".", "--cache", "obj/fast-audit.json"]
        try:
            roles = sorted(json.loads((self.root / AUDIT_FILES[1]).read_text()))
        except (OSError, ValueError, TypeError):
            roles = []
        inputs = [self.digests.files(AUDIT_FILES), sorted(self.sources.items()),
                  [[role, (self.root / role).is_file()] for role in roles]]
        identity = audit_interpreter(self.audit_python)
        allowed = [self.root / "dev", *(self.root / name for name in AUDIT_FILES),
                   self.root / "obj/fast-audit.json", self.root / "obj/fast-audit.tmp",
                   Path(self.audit_python).parent.parent, *interpreter_paths()]
        return command, digest_text([command, self.implementation, identity, inputs]), allowed

    def format_key(self):
        command = ["cargo", "fmt", "--manifest-path", "dev/Cargo.toml", "--all", "--", "--check"]
        tools = [tool_version(["rustfmt", "--version"], self.root),
                 tool_version(["cargo", "fmt", "--version"], self.root)]
        return command, digest_text([command, self.implementation, tools, self.digests.files(["dev/Cargo.toml",
                                     "rust-toolchain.toml"]), configuration_files(self.root), edition(self.root)])

    def record(self, name):
        try:
            return json.loads((self.root / STATE / f"{name}.json").read_text())
        except (OSError, ValueError):
            return {}

    def write(self, name, value):
        path = self.root / STATE / f"{name}.json"
        temporary = path.with_name(f"{path.name}.{os.getpid()}.tmp")
        temporary.write_text(json.dumps(value) + "\n")
        temporary.replace(path)

    def matched(self, name, fingerprint):
        """The passed receipt for this fingerprint, if it still records that pass."""
        for row in self.record(name).get("passed", []):
            if row.get("fingerprint") != fingerprint:
                continue
            try:
                receipt = json.loads((self.root / row["receipt"] / "receipt.json").read_text())
            except (OSError, ValueError, KeyError):
                continue
            if any(step.get("gate") == name and step.get("status") == 0 and step.get("fingerprint") == fingerprint
                   and not step.get("skipped") for step in receipt.get("steps", [])):
                return row["receipt"]
        return None

    def passed(self, name, fingerprint, receipt, **extra):
        rows = [row for row in self.record(name).get("passed", []) if row.get("fingerprint") != fingerprint]
        self.write(name, {**self.record(name), **extra,
                          "passed": [{"fingerprint": fingerprint, "receipt": receipt}, *rows][:KEPT]})

    def outside(self, trace, allowed):
        """Reads a traced gate made outside its declared inputs; any one forbids caching."""
        try:
            seen = json.loads(Path(trace).read_text())
        except (OSError, ValueError):
            return ["trace missing"]
        roots = [path.resolve() for path in allowed]
        problems = [f"started a process: {event}" for event in seen.get("spawned", [])]
        for path in seen.get("opened", []) + seen.get("listed", []):
            resolved = Path(path).resolve()
            if resolved == Path(trace).resolve() or str(resolved) == os.devnull:
                continue
            if not any(resolved == root or resolved.is_relative_to(root) for root in roots):
                problems.append(f"read {path}")
        return problems

    def format_plan(self, key):
        """Return (mode, files): skip, partial with changed files, or full."""
        state = self.record("format")
        known = state.get("files") if state.get("key") == key else None
        if not isinstance(known, dict) or set(known) - set(self.sources):
            return "full", []
        changed = sorted(path for path, digest in self.sources.items() if known.get(path) != digest)
        return ("partial", changed) if changed else ("skip", [])

    def format_fingerprint(self, key):
        return digest_text([key, sorted(self.sources.items())])
