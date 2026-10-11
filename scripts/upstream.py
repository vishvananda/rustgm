#!/usr/bin/env python3
"""Verify provenance offline, or explicitly fetch pinned upstream source bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def verify(root=ROOT):
    manifest = json.loads((root / "third_party/sources.json").read_text())
    for item in manifest["files"]:
        path = root / item["destination"]
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
            raise ValueError(f"upstream bytes changed or missing: {item['destination']}")
    print(f"Verified {len(manifest['files'])} pinned upstream fixtures")


def fetch():
    manifest = json.loads((ROOT / "third_party/sources.json").read_text())
    for item in manifest["files"]:
        source = manifest["repositories"][item["repository"]]
        url = f"https://raw.githubusercontent.com/{source['github']}/{source['commit']}/{item['path']}"
        data = urllib.request.urlopen(url, timeout=30).read()
        if hashlib.sha256(data).hexdigest() != item["sha256"]:
            raise ValueError(f"upstream checksum mismatch: {url}")
        path = ROOT / item["destination"]
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    verify()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    try:
        fetch() if args.fetch else verify()
    except (ValueError, OSError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
