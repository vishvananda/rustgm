"""Validate versioned frontend native dependency manifests before external linking."""
import json
from pathlib import Path


def unique_fields(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise ValueError("duplicate native manifest field")
        result[name] = value
    return result


def load(path):
    path = Path(path)
    with path.open("rb") as stream:
        data = stream.read(1_000_001)
    if len(data) > 1_000_000:
        raise ValueError("native link manifest exceeds byte budget")
    plan = json.loads(data, object_pairs_hook=unique_fields)
    if (not isinstance(plan, dict) or set(plan) != {"schema", "target", "libraries"}
            or type(plan["schema"]) is not int or plan["schema"] != 1
            or plan["target"] != "x86_64-linux"):
        raise ValueError("unsupported native link manifest")
    libraries = plan["libraries"]
    if not isinstance(libraries, list) or len(libraries) > 1024:
        raise ValueError("native dependency count exceeds budget")
    total = 0
    for library in libraries:
        if not isinstance(library, dict) or set(library) != {"name", "kind"}:
            raise ValueError("invalid native dependency record")
        name = library["name"]
        if (not isinstance(name, str) or not name or "\0" in name
                or len(name.encode("utf-8")) > 4096
                or library["kind"] not in ("dylib", "static")):
            raise ValueError("invalid native library name or kind")
        total += len(name.encode("utf-8"))
        if total > 65_536:
            raise ValueError("native dependency names exceed budget")
    return libraries


def arguments(libraries, search):
    # Rust source cannot supply arbitrary driver switches. Every library stays
    # one argv element after a fixed -l or -l: prefix; subprocess never uses a shell.
    result = [f"-L{Path(directory).resolve()}" for directory in search]
    if libraries:
        result.append("-Wl,--start-group")
        for library in libraries:
            name = library["name"]
            result.append(f"-l:lib{name}.a" if library["kind"] == "static" else f"-l{name}")
        result.append("-Wl,--end-group")
    return result
