"""Extract explicitly pinned crates without changing their original file bytes."""
from dataclasses import dataclass
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tarfile
import tomllib

from host_build_script import digest

MAX_FILES = 100_000
MAX_BYTES = 256 * 1024 * 1024
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
CHECKSUM = '.cargo-checksum.json'


def inventory(root):
    """Visit each entry once; extra files and modes are inputs too."""
    if not root.is_absolute() or root.resolve() != root or not root.is_dir():
        raise ValueError('package root must be a real absolute directory')
    files, directories, count, total = [], [], 0, 0
    for directory, dirs, names in os.walk(root, followlinks=False):
        count += len(dirs) + len(names)
        if count > MAX_FILES:
            raise ValueError('package inventory exceeds entry budget')
        parent = Path(directory)
        directories.append((parent.relative_to(root).as_posix(), stat.S_IMODE(parent.stat().st_mode)))
        if any((parent / name).is_symlink() for name in dirs):
            raise ValueError('package directory is a symlink')
        for name in names:
            path = parent / name
            info = path.lstat()
            if not stat.S_ISREG(info.st_mode):
                raise ValueError('package input must be a regular file')
            total += info.st_size
            if total > MAX_BYTES:
                raise ValueError('package inventory exceeds byte budget')
            files.append((path.relative_to(root).as_posix(), digest(path), stat.S_IMODE(info.st_mode)))
    return tuple(sorted(files)), tuple(sorted(directories))


@dataclass(frozen=True)
class Snapshot:
    name: str
    version: str
    archive: Path
    archive_sha256: str
    root: Path
    files: tuple[tuple[str, str, int], ...]
    directories: tuple[tuple[str, int], ...]

    @property
    def cargo_id(self):
        return f'{REGISTRY}#{self.name}@{self.version}'

    @property
    def identity(self):
        # Paths and Cargo row indices never enter the portable package identity.
        return {'source': REGISTRY, 'name': self.name, 'version': self.version,
                'archive_sha256': self.archive_sha256}

    def verify(self):
        if self.archive.is_symlink() or digest(self.archive) != self.archive_sha256:
            raise ValueError('package archive differs from its pin')
        if inventory(self.root) != (self.files, self.directories):
            raise ValueError('package snapshot inventory changed')


def member_path(name, package):
    parts = name.rstrip('/').split('/')
    if (not parts or parts[0] != package or len(parts) > 128
            or any(part in ('', '.', '..') for part in parts)
            or '\\' in name or any(ord(char) < 32 for char in name)):
        raise ValueError('archive path escapes or is not canonical')
    if len(parts) == 2 and parts[1] == CHECKSUM:
        raise ValueError('archive contains reserved Cargo checksum metadata')
    return PurePosixPath(*parts)


def extract(archive, sha256, name, version, vendor):
    """The caller supplies an independently trusted archive checksum; no network.

    A failed extraction remains diagnostic output and is never a Snapshot.
    Checksums generated for Cargo are derived from these authenticated bytes;
    an existing vendor checksum file is never accepted as provenance.
    """
    if not re.fullmatch(r'[A-Za-z0-9_-]+', name) or not re.fullmatch(
            r'[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.+-]+)?', version):
        raise ValueError('invalid pinned package name/version')
    if not re.fullmatch(r'[a-f0-9]{64}', sha256):
        raise ValueError('invalid package checksum')
    if (not archive.is_absolute() or archive.is_symlink() or not archive.is_file()
            or archive.stat().st_size > MAX_BYTES or digest(archive) != sha256):
        raise ValueError('package archive differs from its pin or exceeds budget')
    if not vendor.is_absolute() or vendor.resolve() != vendor or not vendor.is_dir():
        raise ValueError('vendor root must be a real absolute directory')
    package = f'{name}-{version}'
    root = vendor / package
    root.mkdir()
    seen, total = set(), 0
    with tarfile.open(archive, 'r:gz') as source:
        for entry in source:
            relative = member_path(entry.name, package)
            if relative in seen or len(seen) >= MAX_FILES:
                raise ValueError('duplicate archive entry or entry budget exceeded')
            seen.add(relative)
            path = vendor.joinpath(*relative.parts)
            if entry.isdir():
                path.mkdir(parents=True, exist_ok=True)
                continue
            if not entry.isfile() or entry.issparse() or len(relative.parts) == 1:
                raise ValueError('archive entry must be a regular file or directory')
            total += entry.size
            if entry.size < 0 or total > MAX_BYTES:
                raise ValueError('archive exceeds byte budget')
            path.parent.mkdir(parents=True, exist_ok=True)
            with source.extractfile(entry) as content, path.open('xb') as destination:
                for block in iter(lambda: content.read(1024 * 1024), b''):
                    destination.write(block)
            # Retain executable intent, never set-id or permissions from another host.
            path.chmod(0o755 if entry.mode & 0o111 else 0o644)
    manifest = tomllib.loads((root / 'Cargo.toml').read_text())['package']
    if manifest.get('name') != name or manifest.get('version') != version:
        raise ValueError('archive manifest differs from pinned package identity')
    files, directories = inventory(root)
    checksums = {relative: sha for relative, sha, _ in files}
    metadata = root / CHECKSUM
    metadata.write_text(json.dumps({'package': sha256, 'files': checksums}, sort_keys=True) + '\n')
    metadata.chmod(0o644)
    files = tuple(sorted((*files, (CHECKSUM, digest(metadata), 0o644))))
    if digest(archive) != sha256:
        raise ValueError('package archive changed during extraction')
    return Snapshot(name, version, archive, sha256, root, files, directories)
