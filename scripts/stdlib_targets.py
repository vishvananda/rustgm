#!/usr/bin/env python3
"""Drive the frozen original core/alloc/std unit graphs through the target executor.

The raw Cargo graphs are authenticated against the frozen PA34 observation
hashes. Packages bind to those observations' local inventories and registry
archive pins. Planning launches nothing; execution stops at the first failing
unit. No run of this tool is a library build until every target unit completes.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import tomllib

from build_environment import package_environment
from build_units import parse
from host_inputs import declared_features
import manifest_lints
from target_units import TargetPackage, execute, plan_units

ROOT = Path(__file__).resolve().parents[1]
OUTCOMES = ROOT / 'pa34/tests/stdlib-graph-outcomes.json'
GRAPHS = ROOT / 'pa34/tests/spec/stdlib-unit-graphs'
SOURCE_CORE = 'core'


def load(outcomes_path=OUTCOMES, graphs=GRAPHS):
    """Decode each frozen graph only if its bytes match the observed hash."""
    outcomes = json.loads(outcomes_path.read_text())
    target = outcomes['configuration']['target']
    result = {}
    for observation in outcomes['observations']:
        data = (graphs / f"{observation['root']}.json").read_bytes()
        if hashlib.sha256(data).hexdigest() != observation['graph_sha256']:
            raise ValueError('frozen unit graph differs from its observed hash: ' + observation['root'])
        plan = parse(data, target)
        if len(plan.units) != observation['units']:
            raise ValueError('frozen unit graph count differs from its observation')
        result[observation['root']] = (plan, observation['graph_sha256'])
    return outcomes, result


def manifest(root, name, version):
    """Cargo's package environment, declared features and lint table, if readable."""
    path = root / 'Cargo.toml'
    if not path.is_file():
        return None, (), ()
    document = tomllib.loads(path.read_text())
    environment = package_environment(SimpleNamespace(root=root, name=name, version=version), document)
    return (tuple(sorted(environment.items())), declared_features(document),
            manifest_lints.arguments(document.get('lints', {})))


def bind(outcomes, plan, source_root=None, vendor_root=None):
    """Bind every package ID to its frozen identity and an explicit current root."""
    local = {row['origin'].split('/', 2)[2]: row for row in outcomes['local_packages']}
    registry = {(row['name'], row['version']): row for row in outcomes['registry_lockfile_closure']}
    packages = {}
    for unit in plan.units:
        if unit.pkg_id in packages:
            continue
        locator, _, fragment = unit.pkg_id.partition('#')
        if locator.startswith('path+file://'):
            planned = Path(locator.removeprefix('path+file://'))
            relative = next((path for path in local if planned.as_posix().endswith('/' + path)), None)
            version = fragment.rpartition('@')[2]
            if relative is None or local[relative]['version'] != version:
                raise ValueError('local package is absent from the frozen inventory: ' + unit.pkg_id)
            row = local[relative]
            original = Path(planned.as_posix()[:-len(relative) - 1])
            root = (source_root or original) / relative
            identity = f"local {row['origin']} {row['name']} {version}"
            key, is_registry, name = row['inventory_sha256'], False, row['name']
        elif locator == 'registry+https://github.com/rust-lang/crates.io-index':
            name, _, version = fragment.partition('@')
            row = registry.get((name, version))
            source = Path(unit.target.src_path)
            folder = next((part for part in source.parents if part.name == f'{name}-{version}'), None)
            if row is None or folder is None:
                raise ValueError('registry package is absent from the frozen closure: ' + unit.pkg_id)
            planned, root = folder, (vendor_root or folder.parent) / folder.name
            identity = f'registry crates.io {name} {version}'
            key, is_registry = row['archive_sha256'], True
        else:
            raise ValueError('unsupported frozen package source: ' + unit.pkg_id)
        environment, features, lints = manifest(root, name, version)
        packages[unit.pkg_id] = TargetPackage(identity, key, root, planned, environment, is_registry,
                                              features, lints)
    return packages


def source_core(plan):
    """The sysroot core is the frozen inventory's local `core` package."""
    owners = {unit.pkg_id for unit in plan.units if unit.target.name == SOURCE_CORE
              and unit.pkg_id.startswith('path+file://')}
    if len(owners) != 1:
        raise ValueError('frozen graph lacks exactly one local sysroot core')
    return owners.pop()


def summarize(steps):
    return [{'unit': step.unit, 'crate': step.crate_name, 'gaps': [f'{gap.field}={gap.value}' for gap in step.gaps]}
            for step in steps]


def run(args):
    outcomes, graphs = load()
    out = args.out.absolute()
    out.mkdir(parents=True, exist_ok=False)
    report = {'analysis': 'frozen original stdlib target units; no Cargo or rustc target compile',
              'qualification': False, 'graphs': {}}
    for root, (plan, graph_sha256) in graphs.items():
        packages = bind(outcomes, plan, args.source_root, args.vendor_root)
        folder = out / root
        steps = plan_units(plan, packages, folder, source_core=source_core(plan))
        entry = {'graph_sha256': graph_sha256, 'plan': summarize(steps)}
        if args.rustgm is not None:
            receipt = execute(steps, packages, args.rustgm, args.rustgm_sha256, folder,
                              {'root': root, 'graph_sha256': graph_sha256})
            entry.update(state=receipt['state'], failure=receipt['failure'],
                         blocked=[row['unit'] for row in receipt['blocked']],
                         completed=[row['unit'] for row in receipt['completed']])
        else:
            entry['state'] = 'planned only; not a build'
        report['graphs'][root] = entry
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return 0 if all(entry['state'] == 'complete' for entry in report['graphs'].values()) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--source-root', type=Path, help='current root of the pinned Rust source tree')
    parser.add_argument('--vendor-root', type=Path, help='current root of the authenticated registry snapshots')
    parser.add_argument('--rustgm', type=Path, help='pinned RustGM binary; omit to plan only')
    parser.add_argument('--rustgm-sha256', help='declared RustGM SHA-256, required with --rustgm')
    parser.add_argument('--worker', action='store_true', help='caller already holds the storage guard')
    args = parser.parse_args()
    if (args.rustgm is None) != (args.rustgm_sha256 is None):
        parser.error('--rustgm and --rustgm-sha256 must be given together')
    if args.worker:
        try:
            return run(args)
        except (OSError, ValueError, KeyError) as error:
            print(f'stdlib target units: infrastructure failure: {error}', file=sys.stderr)
            return 2
    command = [sys.executable, 'scripts/storage.py', 'run', '--prune-under', str(args.out), '--',
               sys.executable, __file__, *sys.argv[1:], '--worker']
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
