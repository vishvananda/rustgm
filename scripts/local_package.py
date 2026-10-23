"""Freeze original local package trees; Cargo path IDs remain invocation-local."""
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tomllib

from package_snapshot import inventory


def fingerprint(files, directories):
    value = json.dumps((files, directories), separators=(',', ':'), ensure_ascii=False)
    return hashlib.sha256(value.encode()).hexdigest()


@dataclass(frozen=True)
class LocalPackage:
    name: str
    version: str
    origin: str
    root: Path
    files: tuple[tuple[str, str, int], ...]
    directories: tuple[tuple[str, int], ...]
    inventory_sha256: str

    @property
    def cargo_id(self):
        suffix = self.version if self.root.name == self.name else f'{self.name}@{self.version}'
        return f'path+{self.root.as_uri()}#{suffix}'

    @property
    def identity(self):
        return {'source': 'local', 'origin': self.origin, 'name': self.name,
                'version': self.version, 'inventory_sha256': self.inventory_sha256}

    def verify(self):
        validate_identity(self.name, self.version, self.origin)
        if (fingerprint(self.files, self.directories) != self.inventory_sha256
                or inventory(self.root) != (self.files, self.directories)):
            raise ValueError('local package snapshot inventory changed')
        document = tomllib.loads((self.root / 'Cargo.toml').read_text())
        package = document.get('package', {})
        if package.get('name') != self.name or package.get('version') != self.version:
            raise ValueError('local package manifest identity is not explicit or differs')


def validate_identity(name, version, origin):
    if not re.fullmatch(r'[A-Za-z0-9_-]+', name) or not re.fullmatch(
            r'[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.+-]+)?', version):
        raise ValueError('local package requires an explicit name/version')
    parts = origin.split('/')
    if (not origin or PurePosixPath(origin).is_absolute()
            or any(part in ('', '.', '..') for part in parts)
            or any(not re.fullmatch(r'[A-Za-z0-9_.-]+', part) for part in parts)):
        raise ValueError('local origin must be an explicit portable relative identifier')


def capture(root, name, version, origin):
    """Caller declares the local tree; no copy, rewrite, exclusion or registry claim."""
    validate_identity(name, version, origin)
    files, directories = inventory(root)
    package = LocalPackage(name, version, origin, root, files, directories,
                           fingerprint(files, directories))
    package.verify()
    return package


def validate_layout(packages):
    roots = sorted(package.root for package in packages)
    if (len(packages) > 4096 or len(set(roots)) != len(roots)
            or len({p.origin for p in packages}) != len(packages)
            or sum(len(p.files) for p in packages) > 1_000_000):
        raise ValueError('local package inventory is duplicate or exceeds count budget')
    # Lexical order places descendants immediately after the owning root.
    if any(right.is_relative_to(left) for left, right in zip(roots, roots[1:])):
        raise ValueError('local package roots must not overlap')


def validate(packages):
    validate_layout(packages)
    for package in packages:
        package.verify()


def bind(plan, packages):
    """One source index, then one graph walk; never hash a package per unit."""
    by_source = {str(p.root / name): p for p in packages for name, _, _ in p.files}
    result = []
    for unit in plan.units:
        package = by_source.get(unit.target.src_path)
        if unit.pkg_id.startswith('path+'):
            if package is None or package.cargo_id != unit.pkg_id:
                raise ValueError('planned local source lacks its exact package snapshot')
            result.append(package.identity)
        else:
            if package is not None:
                raise ValueError('local source cannot claim a registry package identity')
            result.append(None)
    return result
