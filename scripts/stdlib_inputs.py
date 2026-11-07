"""Authenticate the frozen stdlib package closure into explicit current roots.

Registry archives are copied from a caller-supplied directory only if their
bytes equal the frozen lockfile closure, then extracted into a fresh vendor
directory. Local packages are captured in place and must equal their frozen
inventories. Nothing is downloaded, copied from the source tree or rewritten.
"""
from pathlib import Path
import shutil
import subprocess
import tomllib

from crate_vendor import Vendor
from host_process import digest
from local_package import capture
from package_snapshot import extract

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ('library/Cargo.toml', 'library/Cargo.lock')


def tools(outcomes):
    """The pinned Cargo planner and rustc query/host compiler of the frozen plans."""
    channel = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    result = {}
    for name in ('cargo', 'rustc'):
        query = ['rustup', 'which', '--toolchain', channel, name]
        path = Path(subprocess.check_output(query, cwd=ROOT, text=True, timeout=30).strip())
        sha256 = digest(path)
        if sha256 != outcomes['tool_sha256'][f'{name}_sha256']:
            raise ValueError(f'pinned {name} differs from the frozen planning tool')
        result[name] = (path, sha256)
    return result


def relative(row):
    """`rust/<revision>/library/...` origins name a path below the source root."""
    parts = row['origin'].split('/', 2)
    if len(parts) != 3 or parts[0] != 'rust':
        raise ValueError('frozen local package origin is not a source-tree path')
    return parts[2]


def local(outcomes, source_root):
    """Every frozen local package, captured where it is; its inventory must not differ."""
    packages = []
    for row in outcomes['local_packages']:
        package = capture(source_root / relative(row), row['name'], row['version'], row['origin'])
        if package.inventory_sha256 != row['inventory_sha256']:
            raise ValueError('local package differs from its frozen inventory: ' + row['origin'])
        packages.append(package)
    return tuple(packages)


def workspace(outcomes, source_root):
    """The workspace manifest and lockfile pins of the frozen observations."""
    pins = []
    for name in WORKSPACE:
        path = source_root / name
        expected = outcomes['source_manifests'][name]
        if path.is_symlink() or not path.is_file() or digest(path) != expected:
            raise ValueError('workspace input differs from its frozen pin: ' + name)
        pins.append((path, expected))
    return tuple(pins)


def registry(outcomes, archives, folder):
    """Extract the complete pinned registry closure into a fresh vendor directory."""
    folder = Path(folder).absolute()
    store, vendor = folder / 'archives', folder / 'vendor'
    store.mkdir(parents=True)
    # Upstream packages ship object files that output cleanup would otherwise
    # remove; pinning the input tree keeps every snapshot verifiable.
    (folder / '.keep-artifacts').touch(exist_ok=False)
    vendor.mkdir()
    snapshots = []
    for row in outcomes['registry_lockfile_closure']:
        name = f"{row['name']}-{row['version']}.crate"
        source = Path(archives) / name
        if source.is_symlink() or not source.is_file() or digest(source) != row['archive_sha256']:
            raise ValueError('registry archive differs from the frozen closure: ' + name)
        shutil.copyfile(source, store / name)
        snapshots.append(extract(store / name, row['archive_sha256'], row['name'], row['version'],
                                 vendor.resolve()))
    return Vendor(vendor.resolve(), tuple(snapshots))
