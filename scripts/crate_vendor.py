"""A closed crates.io directory source for the frozen Cargo planner."""
from dataclasses import dataclass
import json
from pathlib import Path
import tomllib

from package_snapshot import MAX_BYTES, REGISTRY, Snapshot


@dataclass(frozen=True)
class Vendor:
    root: Path
    packages: tuple[Snapshot, ...]

    def verify(self, lockfile):
        if not self.root.is_absolute() or self.root.resolve() != self.root or not self.root.is_dir():
            raise ValueError('vendor root must be a real absolute directory')
        by_id = {package.cargo_id: package for package in self.packages}
        roots = {package.root for package in self.packages}
        if (not self.packages or len(by_id) != len(self.packages) or len(roots) != len(self.packages)
                or any(root.parent != self.root for root in roots)
                or set(self.root.iterdir()) != roots):
            raise ValueError('vendor inventory must contain exactly its unique package snapshots')
        if lockfile.stat().st_size > MAX_BYTES:
            raise ValueError('lockfile exceeds byte budget')
        lock = tomllib.loads(lockfile.read_text())
        if lock.get('version') != 4 or not isinstance(lock.get('package'), list):
            raise ValueError('unsupported frozen Cargo lockfile')
        locked = set()
        for row in lock['package']:
            source = row.get('source')
            if source is None:
                if 'checksum' in row:
                    raise ValueError('path package cannot claim a registry checksum')
                continue
            key = f"{source}#{row.get('name')}@{row.get('version')}"
            package = by_id.get(key)
            if (source != REGISTRY or package is None or key in locked
                    or row.get('checksum') != package.archive_sha256):
                raise ValueError('lockfile package differs from the frozen vendor closure')
            locked.add(key)
        if locked != by_id.keys():
            raise ValueError('vendor closure differs from the complete registry lockfile set')
        for package in self.packages:
            package.verify()

    def arguments(self):
        return ['--config', 'source.crates-io.replace-with="rustgm-frozen"',
                '--config', 'source.rustgm-frozen.directory=' + json.dumps(str(self.root))]

    def bind(self, plan, inputs):
        """One package index and one unit walk; never rehash a package per unit."""
        by_source = {str(package.root / name): package for package in self.packages
                     for name, _, _ in package.files}
        result = []
        for unit in plan.units:
            package = by_source.get(unit.target.src_path)
            if package is None:
                if not unit.pkg_id.startswith('path+') or Path(unit.target.src_path) not in inputs:
                    raise ValueError('planned unit has no declared package source')
                result.append(None)
            else:
                if unit.pkg_id != package.cargo_id:
                    raise ValueError('planned source belongs to a different registry package')
                result.append(package.identity)
        return result
