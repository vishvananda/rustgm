#!/usr/bin/env python3
"""Install the exact locked C++ backend distribution for native qualification."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import urllib.request

import backend

ROOT = Path(__file__).resolve().parents[1]
MAX_ARCHIVE = 32 * 1024 ** 2
MAX_TOOL = 128 * 1024 ** 2


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def install(root, bundle):
    lock = json.loads((root / "backend.lock.json").read_text())
    distribution = json.loads((root / "backend.distribution.json").read_text())
    if distribution["backend_lock_sha256"] != digest(root / "backend.lock.json"):
        raise ValueError("distribution describes a different backend lock")
    if lock["target"] != "x86_64-linux":
        raise ValueError("distribution requires the locked x86-64 Linux backend")
    if bundle.stat().st_size > MAX_ARCHIVE or digest(bundle) != distribution["sha256"]:
        raise ValueError("backend distribution differs from the locked archive")
    expected = {**lock["sha256"], **backend.object_profile(lock)["sha256"]}
    if set(expected) != {"lowir", "lowir2native", "cppgm++"}:
        raise ValueError("unexpected backend tool inventory")
    destination = root / "obj/backend"
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="setup-", dir=destination) as temporary:
        prepared = {}
        seen = set()
        with tarfile.open(bundle) as archive:
            for member in archive:
                if (member.name not in {*expected, "manifest.json", "NOTICE"}
                        or member.name in seen or not member.isfile()
                        or not 0 <= member.size <= MAX_TOOL):
                    raise ValueError("unexpected backend archive entry")
                seen.add(member.name)
                if member.name not in expected:
                    continue
                path = Path(temporary) / member.name
                with archive.extractfile(member) as source, path.open("xb") as output:
                    while block := source.read(1024 * 1024):
                        output.write(block)
                if digest(path) != expected[member.name]:
                    raise ValueError(f"{member.name}: distribution tool differs from backend lock")
                prepared[member.name] = path
        if set(prepared) != set(expected):
            raise ValueError("backend distribution is missing a locked tool")
        # Validate the whole bundle before publishing pins or configuration.
        tools = {name: str(backend.pin_tool(path, name, expected[name], root=root))
                 for name, path in prepared.items()}
        with tempfile.NamedTemporaryFile(mode="w", dir=root, prefix=".backend-local-", delete=False) as output:
            json.dump({"tools": tools}, output, indent=2)
            output.write("\n")
            temporary_config = Path(output.name)
        temporary_config.replace(root / "backend.local.json")
    print("Configured all three exact locked backend tools")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, help="use an already downloaded locked archive")
    args = parser.parse_args()
    if args.bundle is not None:
        install(ROOT, args.bundle)
        return
    distribution = json.loads((ROOT / "backend.distribution.json").read_text())
    destination = ROOT / "obj/backend"
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="download-", dir=destination) as temporary:
        bundle = Path(temporary) / "bundle.tar.gz"
        request = urllib.request.Request(distribution["url"], headers={"User-Agent": "RustGM-backend-setup"})
        with urllib.request.urlopen(request, timeout=60) as source, bundle.open("xb") as output:
            size = 0
            while block := source.read(1024 * 1024):
                size += len(block)
                if size > MAX_ARCHIVE:
                    raise ValueError("backend distribution exceeds its size limit")
                output.write(block)
        install(ROOT, bundle)


if __name__ == "__main__":
    main()
