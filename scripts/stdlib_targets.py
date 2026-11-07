#!/usr/bin/env python3
"""Drive the frozen original core/alloc/std unit graphs through the target executor.

The raw Cargo graphs are authenticated against their frozen observation hashes.
The course-profile set, planned under RustGM's fixed profile, is the default;
the dev-profile set remains historical evidence. Packages bind to the frozen
local inventories and registry archive pins. With `--archives`, one combined run
authenticates that closure, runs each graph's host build scripts through
host_graph and plans target units that consume those real outputs. Execution
stops at the first failing unit. No run is a library build until every target
unit completes.
"""
import argparse
from dataclasses import replace
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import tomllib

from build_environment import Context, package_environment
from build_units import Execution, parse
from host_inputs import declared_features, identifier
import host_graph
from host_process import CommandLog, digest
import manifest_lints
import stdlib_inputs
from target_units import TargetPackage, execute, plan_units

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / 'pa34/tests'
GRAPHS = TESTS / 'spec/stdlib-unit-graphs'
SETS = {'course-profile': TESTS / 'stdlib-course-profile-outcomes.json',
        'dev-profile': TESTS / 'stdlib-graph-outcomes.json'}
SOURCE_CORE = 'core'
ENVIRONMENT = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}
# Build scripts see the target as its units are planned: panic abort, debug
# assertions off. The three original scripts never read debug_assertions.
TARGET_CFG_FLAGS = ('-Cpanic=abort', '-Cdebug-assertions=no')
TEMPLATE = TESTS / 'spec/stdlib-host/compiler-builtins/expected.stdout.template'
IMPLEMENTATION = ('stdlib_targets.py', 'stdlib_inputs.py', 'host_graph.py', 'target_units.py')


def load(name='course-profile', graphs=GRAPHS):
    """Decode each graph of a frozen set only if its bytes match the observed hash.

    Returns the observations that pin the package closure and the graphs.
    """
    outcomes = json.loads(SETS[name].read_text())
    pins = outcomes
    if 'base_outcomes' in outcomes:
        base = ROOT / outcomes['base_outcomes']['path']
        if digest(base) != outcomes['base_outcomes']['sha256']:
            raise ValueError('frozen base observations differ from their pin')
        pins = json.loads(base.read_text())
    target = outcomes['configuration']['target']
    result = {}
    for observation in outcomes['observations']:
        data = (graphs / name / f"{observation['root']}.json").read_bytes()
        if hashlib.sha256(data).hexdigest() != observation['graph_sha256']:
            raise ValueError('frozen unit graph differs from its observed hash: ' + observation['root'])
        plan = parse(data, target)
        if len(plan.units) != observation['units']:
            raise ValueError('frozen unit graph count differs from its observation')
        result[observation['root']] = (plan, observation['graph_sha256'])
    return pins, result


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
    local = {stdlib_inputs.relative(row): row for row in outcomes['local_packages']}
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


def relocate(plan, packages):
    """Rewrite each planned package root to its bound current root.

    Path package IDs and every source path move together, so host and target
    executors see one consistent graph. Identities and keys do not change.
    """
    units, moved = [], {}
    for unit in plan.units:
        package = packages[unit.pkg_id]
        source = Path(unit.target.src_path)
        if not source.is_relative_to(package.planned_root):
            raise ValueError('planned source lies outside its bound package')
        pkg_id = unit.pkg_id
        if pkg_id.startswith('path+'):
            pkg_id = f"path+{package.root.as_uri()}#{pkg_id.partition('#')[2]}"
        if moved.setdefault(pkg_id, unit.pkg_id) != unit.pkg_id:
            raise ValueError('relocated packages collide')
        target = replace(unit.target, src_path=str(package.root / source.relative_to(package.planned_root)))
        units.append(replace(unit, pkg_id=pkg_id, target=target))
    return (replace(plan, units=tuple(units)),
            {pkg_id: replace(packages[original], planned_root=packages[original].root)
             for pkg_id, original in moved.items()})


def source_core(plan):
    """The sysroot core is the frozen inventory's local `core` package."""
    owners = {unit.pkg_id for unit in plan.units if unit.target.name == SOURCE_CORE
              and unit.pkg_id.startswith('path+file://')}
    if len(owners) != 1:
        raise ValueError('frozen graph lacks exactly one local sysroot core')
    return owners.pop()


def package_name(pkg_id):
    locator, _, fragment = pkg_id.partition('#')
    return fragment.partition('@')[0] if '@' in fragment else Path(locator).name


def contexts(compiler, compiler_sha256, target, folder):
    """Explicit, recorded host and target cfg facts for build-script execution."""
    folder.mkdir(parents=True)
    commands = []

    def save():
        (folder / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')

    log = CommandLog(folder, folder, commands, save)
    result = {}
    for platform, name, flags in ((None, 'host-cfg', ()), (target, 'target-cfg', TARGET_CFG_FLAGS)):
        if log.run(name, [compiler, '--print=cfg', '--target', target, *flags], ENVIRONMENT):
            raise ValueError('explicit compiler cfg query failed')
        path = folder / f'{name}.stdout'
        cfg = {}
        for line in path.read_text().splitlines():
            key, equals, value = line.partition('=')
            cfg.setdefault(key, [])
            if equals:
                cfg[key].append(json.loads(value))
        result[platform] = Context(target, target, compiler, compiler_sha256,
                                   tuple((key, tuple(values)) for key, values in cfg.items()),
                                   ((path, digest(path)),))
    return result


def check_scripts(plan, generated, pins, packages):
    """The real script outputs must reproduce the frozen original observations."""
    frozen = pins['original_host_scripts']
    for index, result in generated.items():
        unit = plan.units[index]
        name = package_name(unit.pkg_id)
        expected = frozen.get(name)
        if expected is None or len(result.output.directives) != expected['directives']:
            raise ValueError('host script output differs from its frozen observation: ' + name)
        if name == 'compiler_builtins':
            root = str(packages[unit.pkg_id].root).encode()
            stdout = (result.folder.parent / 'execute.stdout').read_bytes()
            if stdout != TEMPLATE.read_bytes().replace(b'{PACKAGE_ROOT}', root):
                raise ValueError('compiler-builtins script output differs from its frozen template')


def host_scripts(plan, packages, inputs, tools, folder):
    """Run every host unit of a relocated graph once; returns index -> Generated."""
    owners = {unit.pkg_id for unit in plan.units if unit.execution is not Execution.TARGET_COMPILE}
    if not owners:
        return {}, None
    selected = [snapshot for snapshot in inputs if snapshot.cargo_id in owners]
    if {snapshot.cargo_id for snapshot in selected} != owners:
        raise ValueError('host unit lacks its authenticated package snapshot')
    # Cargo without -vv caps every non-local package, host builds included.
    caps = {pkg_id: 'allow' for pkg_id in owners if packages[pkg_id].registry}
    compiler, sha256 = tools['rustc']
    context = contexts(compiler, sha256, next(u.platform for u in plan.units if u.platform), folder / 'context')
    generated = host_graph.execute(plan, selected, context, folder / 'host', lint_caps=caps)
    return generated, digest(folder / 'host/receipt.json')


def summarize(steps):
    return [{'unit': step.unit, 'crate': step.crate_name, 'gaps': [f'{gap.field}={gap.value}' for gap in step.gaps]}
            for step in steps]


def timeouts(values):
    result = {}
    for value in values:
        name, equals, seconds = value.partition('=')
        if not equals or not seconds.isdigit() or name in result:
            raise ValueError('--unit-timeout takes one CRATE=SECONDS per crate')
        result[name] = int(seconds)
    return result


def run(args):
    pins, graphs = load(args.graphs)
    out = args.out.absolute()
    out.mkdir(parents=True, exist_ok=False)
    implementation = {name: digest(Path(__file__).with_name(name)) for name in IMPLEMENTATION}
    report = {'analysis': 'frozen original stdlib target units; no Cargo or rustc target compile',
              'qualification': False, 'graph_set': args.graphs, 'implementation_sha256': implementation,
              'lint_cap_override': args.analysis_cap_lints,
              'graphs': {}}
    selected = args.root or list(graphs)
    limits = timeouts(args.unit_timeout)
    if set(limits) - {identifier(unit.target.name) for root in selected for unit in graphs[root][0].units
                      if unit.execution is Execution.TARGET_COMPILE}:
        raise ValueError('--unit-timeout names no selected target crate')
    inputs = tools = None
    if args.archives is not None:
        tools = stdlib_inputs.tools(pins)
        vendor = stdlib_inputs.registry(pins, args.archives.absolute(), out / 'inputs')
        inputs = (*stdlib_inputs.local(pins, args.source_root), *vendor.packages)
        args.vendor_root = vendor.root
    for root in selected:
        plan, graph_sha256 = graphs[root]
        packages = bind(pins, plan, args.source_root, args.vendor_root)
        folder = out / root
        entry = {'graph_sha256': graph_sha256, 'host': None}
        scripts = {}
        if inputs is not None:
            plan, packages = relocate(plan, packages)
            snapshots = {snapshot.cargo_id: snapshot for snapshot in inputs}
            packages = {pkg_id: replace(package, snapshot=snapshots[pkg_id])
                        for pkg_id, package in packages.items()}
            scripts, receipt = host_scripts(plan, packages, inputs, tools, folder)
            check_scripts(plan, scripts, pins, packages)
            entry['host'] = {'receipt_sha256': receipt, 'scripts': {
                str(index): {'package': package_name(plan.units[index].pkg_id), 'key': result.key,
                             'directives': len(result.output.directives)}
                for index, result in scripts.items()}}
        steps = plan_units(plan, packages, folder / 'target', source_core=source_core(plan), scripts=scripts,
                           lint_cap=args.analysis_cap_lints,
                           timeouts={name: seconds for name, seconds in limits.items()
                                     if any(identifier(unit.target.name) == name for unit in plan.units
                                            if unit.execution is Execution.TARGET_COMPILE)})
        entry['plan'] = summarize(steps)
        if args.rustgm is not None:
            receipt = execute(steps, packages, args.rustgm, args.rustgm_sha256, folder / 'target',
                              {'set': args.graphs, 'root': root, 'graph_sha256': graph_sha256,
                               'lint_cap_override': args.analysis_cap_lints})
            entry.update(state=receipt['state'], failure=receipt['failure'],
                         blocked=[row['unit'] for row in receipt['blocked']],
                         completed=[row['unit'] for row in receipt['completed']])
        else:
            entry['state'] = 'planned only; not a build'
        report['graphs'][root] = entry
    if any(digest(Path(__file__).with_name(name)) != sha for name, sha in implementation.items()):
        raise ValueError('stdlib target implementation changed during execution')
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return 0 if all(entry['state'] == 'complete' for entry in report['graphs'].values()) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--graphs', choices=sorted(SETS), default='course-profile')
    parser.add_argument('--root', action='append', choices=('core', 'alloc', 'std'),
                        help='graph to process; repeat, default all')
    parser.add_argument('--source-root', type=Path, help='current root of the pinned Rust source tree')
    parser.add_argument('--vendor-root', type=Path, help='current root of the authenticated registry snapshots')
    parser.add_argument('--archives', type=Path,
                        help='directory of the pinned registry archives; authenticates the closure and '
                             'runs the host build scripts (requires --source-root)')
    parser.add_argument('--rustgm', type=Path, help='pinned RustGM binary; omit to plan only')
    parser.add_argument('--rustgm-sha256', help='declared RustGM SHA-256, required with --rustgm')
    parser.add_argument('--unit-timeout', action='append', default=[], metavar='CRATE=SECONDS',
                        help='explicit, recorded compile limit for one whole-library unit (default 120)')
    parser.add_argument('--analysis-cap-lints', choices=('allow',),
                        help='analysis only: also cap local packages, unlike Cargo and bootstrap')
    parser.add_argument('--worker', action='store_true', help='caller already holds the storage guard')
    args = parser.parse_args()
    for name in ('out', 'source_root', 'vendor_root', 'archives', 'rustgm'):
        if getattr(args, name) is not None:
            setattr(args, name, getattr(args, name).absolute())
    if (args.rustgm is None) != (args.rustgm_sha256 is None):
        parser.error('--rustgm and --rustgm-sha256 must be given together')
    if args.archives is not None and (args.source_root is None or args.vendor_root is not None
                                      or args.graphs != 'course-profile'):
        parser.error('--archives needs --source-root, no --vendor-root and the course-profile graphs')
    if args.worker:
        try:
            return run(args)
        except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
            print(f'stdlib target units: infrastructure failure: {error}', file=sys.stderr)
            return 2
    command = [sys.executable, 'scripts/storage.py', 'run', '--prune-under', str(args.out), '--',
               sys.executable, __file__, *sys.argv[1:], '--worker']
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
