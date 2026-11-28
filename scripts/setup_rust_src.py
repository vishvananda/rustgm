#!/usr/bin/env python3
"""Verify and prepare pinned Rust 1.98.0 library sources and locked PA35/PA37 crates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REVISION = "88d9e12ae178fab0fb5cc050a94da85685d449ea"
CORE_MANIFEST = Path(__file__).with_name("original_core_manifest.json")

# Official rust-src archives omit these four non-Rust repository dotfiles.
OPTIONAL_DOTFILES = frozenset({
    "library/portable-simd/.gitignore",
    "library/stdarch/.gitignore",
    "library/stdarch/.gitmodules",
    "library/stdarch/crates/stdarch-verify/.gitattributes",
})

REQUIRED_LIBRARY_ROOTS = (
    "core/src/lib.rs",
    "alloc/src/lib.rs",
    "std/src/lib.rs",
    "unwind/src/lib.rs",
    "std_detect/src/lib.rs",
    "panic_abort/src/lib.rs",
    "vendor/cfg-if-1.0.4/src/lib.rs",
    "vendor/libc-0.2.185/src/lib.rs",
    "vendor/hashbrown-0.17.1/src/lib.rs",
    "vendor/rustc-demangle-0.1.27/src/lib.rs",
)

PA35_CRATES = (
    ("unicode-ident", "1.0.24", "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75"),
    ("tinyvec_macros", "0.1.1", "1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20"),
    ("tinyvec", "1.10.0", "bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa"),
    ("unicode-normalization", "0.1.25", "5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8"),
)


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def normalize_library_root(candidate):
    path = Path(candidate).expanduser().resolve()
    if (path / "core/src/lib.rs").is_file():
        return path
    if (path / "library/core/src/lib.rs").is_file():
        return path / "library"
    if (path / "lib/rustlib/src/rust/library/core/src/lib.rs").is_file():
        return path / "lib/rustlib/src/rust/library"
    if (path / "rust-src/lib/rustlib/src/rust/library/core/src/lib.rs").is_file():
        return path / "rust-src/lib/rustlib/src/rust/library"
    return None


def verify_library(library, *, full_core_check=True):
    library = Path(library).resolve()
    for rel in REQUIRED_LIBRARY_ROOTS:
        if not (library / rel).is_file():
            raise ValueError(f"library tree {library} is missing {rel}")
    if not full_core_check or not CORE_MANIFEST.is_file():
        return library
    pinned = json.loads(CORE_MANIFEST.read_text())
    rust_root = library.parent
    for row in pinned["files"]:
        rel = row["path"]
        target = rust_root / rel
        if not target.is_file():
            if rel in OPTIONAL_DOTFILES:
                continue
            raise ValueError(f"library tree {library} is missing pinned file {rel}")
        if sha256_file(target) != row["sha256"]:
            raise ValueError(f"library tree {library} differs from pinned SHA-256 at {rel}")
    return library


def rustup_sysroot(root):
    toolchain_file = root / "rust-toolchain.toml"
    channel = tomllib.loads(toolchain_file.read_text())["toolchain"]["channel"]
    rustc = Path(subprocess.check_output(
        ["rustup", "which", "--toolchain", channel, "rustc"],
        cwd=root, text=True, timeout=30,
    ).strip())
    version = subprocess.check_output([str(rustc), "-vV"], text=True, timeout=30)
    if f"commit-hash: {REVISION}" not in version:
        raise ValueError(f"pinned rustc is not revision {REVISION}: {version.strip()}")
    return rustc.parents[1]


def install_rust_src_from_manifest(root, dest):
    sysroot = rustup_sysroot(root)
    installed = normalize_library_root(sysroot)
    if installed is not None:
        return verify_library(installed)
    manifest_path = sysroot / "lib/rustlib/multirust-channel-manifest.toml"
    if not manifest_path.is_file():
        raise ValueError(
            "pinned rust-src is not installed locally and channel manifest is absent; "
            "run `rustup component add rust-src` or set RUSTGM_RUST_SRC"
        )
    manifest = tomllib.loads(manifest_path.read_text())
    source = manifest["pkg"]["rust-src"]["target"]["*"]
    url, expected_sha = source["xz_url"], source["xz_hash"]
    dest = Path(dest).resolve()
    dest.mkdir(parents=True, exist_ok=True)
    archive = dest / Path(url).name
    if not archive.is_file():
        print(f"Downloading pinned rust-src ({Path(url).name})...", flush=True)
        request = urllib.request.Request(url, headers={"User-Agent": "RustGM-setup-rust-src"})
        tmp = archive.with_suffix(archive.suffix + ".part")
        with urllib.request.urlopen(request, timeout=180) as response, tmp.open("wb") as out:
            shutil.copyfileobj(response, out)
        tmp.rename(archive)
    if sha256_file(archive) != expected_sha:
        raise ValueError(f"downloaded {archive} differs from toolchain manifest hash {expected_sha}")
    extracted = dest / "extracted"
    if not extracted.is_dir():
        with tempfile.TemporaryDirectory(dir=dest, prefix=".extract-") as scratch:
            with tarfile.open(archive) as bundle:
                bundle.extractall(scratch, filter="data")
            Path(scratch).rename(extracted)
    top = next(extracted.iterdir())
    if (top / "git-commit-hash").read_text().strip() != REVISION:
        raise ValueError(f"extracted rust-src is not revision {REVISION}")
    library = top / "rust-src/lib/rustlib/src/rust/library"
    return verify_library(library)


def ensure_rust_library(root=ROOT, *, rust_src_override=None, verify=True):
    root = Path(root).resolve()
    candidates = []
    if rust_src_override is not None:
        candidates.append(Path(rust_src_override))
    if env_root := os.environ.get("RUSTGM_RUST_SRC"):
        candidates.append(Path(env_root))
    candidates.append(root / "obj/rust-src/extracted")
    candidates.append(root / "obj/rust-src")
    # Also check sibling demand-work cache if present on a maintainer machine.
    candidates.append(root / "obj/parallel/demand-work/rust-src/extracted")
    for candidate in candidates:
        if not candidate.exists():
            continue
        if candidate.name == "extracted" and candidate.is_dir():
            for child in sorted(candidate.iterdir()):
                found = normalize_library_root(child)
                if found is not None:
                    return verify_library(found, full_core_check=verify)
        found = normalize_library_root(candidate)
        if found is not None:
            return verify_library(found, full_core_check=verify)
    return install_rust_src_from_manifest(root, root / "obj/rust-src")


def find_cached_crate(name, version, expected_sha, extra_dirs=()):
    filename = f"{name}-{version}.crate"
    search_dirs = [*extra_dirs]
    if env_cache := os.environ.get("RUSTGM_CRATE_CACHE"):
        search_dirs.append(Path(env_cache))
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    cache_root = cargo_home / "registry/cache"
    if cache_root.is_dir():
        search_dirs.extend(sorted(cache_root.iterdir()))
    for folder in search_dirs:
        candidate = Path(folder) / filename
        if candidate.is_file() and sha256_file(candidate) == expected_sha:
            return candidate
    return None


def ensure_vendor_crates(root=ROOT, *, crates=PA35_CRATES, vendor_dir=None, cache_dir=None):
    root = Path(root).resolve()
    vendor = Path(vendor_dir).resolve() if vendor_dir else (root / "obj/vendor-crates").resolve()
    archives = vendor / ".archives"
    archives.mkdir(parents=True, exist_ok=True)
    extra = [Path(cache_dir)] if cache_dir else []
    extra.append(archives)
    for name, version, expected_sha in crates:
        crate_dir = vendor / f"{name}-{version}"
        stamp = crate_dir / ".rustgm-verified-sha256"
        if crate_dir.is_dir() and stamp.is_file() and stamp.read_text().strip() == expected_sha:
            continue
        archive = find_cached_crate(name, version, expected_sha, extra)
        if archive is None:
            url = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
            print(f"Downloading locked crate {name} {version}...", flush=True)
            target = archives / f"{name}-{version}.crate"
            tmp = target.with_suffix(".crate.part")
            request = urllib.request.Request(url, headers={"User-Agent": "RustGM-setup-crates"})
            with urllib.request.urlopen(request, timeout=120) as response, tmp.open("wb") as out:
                shutil.copyfileobj(response, out)
            if sha256_file(tmp) != expected_sha:
                tmp.unlink(missing_ok=True)
                raise ValueError(f"downloaded {name}-{version}.crate differs from locked SHA-256")
            tmp.rename(target)
            archive = target
        if crate_dir.exists():
            shutil.rmtree(crate_dir)
        with tempfile.TemporaryDirectory(dir=vendor, prefix=f".{name}-") as scratch:
            with tarfile.open(archive, "r:gz") as bundle:
                bundle.extractall(scratch, filter="data")
            extracted = Path(scratch) / f"{name}-{version}"
            if not extracted.is_dir():
                raise ValueError(f"{archive.name} did not extract to {name}-{version}/")
            extracted.rename(crate_dir)
        stamp.write_text(expected_sha + "\n")
    return vendor


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust-src", type=Path, help="existing pinned Rust 1.98.0 library directory")
    parser.add_argument("--with-pa35-crates", action="store_true",
                        help="also verify and extract the locked PA35/PA37 crates")
    parser.add_argument("--crate-cache", type=Path, help="directory containing .crate archives")
    parser.add_argument("--print-library", action="store_true", help="print only the verified library path")
    parser.add_argument("--skip-core-hash", action="store_true",
                        help="check only required roots instead of hashing all 718 core files")
    args = parser.parse_args()
    try:
        library = ensure_rust_library(ROOT, rust_src_override=args.rust_src, verify=not args.skip_core_hash)
        if args.with_pa35_crates:
            vendor = ensure_vendor_crates(ROOT, cache_dir=args.crate_cache)
            if not args.print_library:
                print(f"Verified locked PA35 crates: {vendor}")
        if args.print_library:
            print(library)
        else:
            print(f"Verified pinned Rust 1.98.0 library: {library}")
        return 0
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"setup-rust-src error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
