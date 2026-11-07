#!/usr/bin/env python3
"""Re-plan the frozen core/alloc/std graphs under RustGM's fixed course profile.

Cargo is only the package planner. Its dev profile is set explicitly to what
RustGM can express: no optimizer, no debug information, debug assertions off,
overflow checks on, panic abort. Upstream bootstrap also ships std without
debug assertions or debuginfo; RustGM fixes overflow checks on, so that one
setting is RustGM's own rather than bootstrap's. The release profile is not
used: the workspace manifest gives its compiler_builtins and rustc-demangle
package overrides (codegen units, opt-level "s") that RustGM cannot express.

The declaration is written before planning. Each new graph must equal its
historical dev-profile graph in every field except the profile. Nothing is
compiled; the observation records tool, input and receipt hashes.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from build_units import Execution
from cargo_plan import Job, execute as plan_cargo
from host_process import digest
import stdlib_inputs
import stdlib_targets

ROOT = Path(__file__).resolve().parents[1]
BASE = stdlib_targets.SETS['dev-profile']
SETTINGS = (('opt-level', '0'), ('debug', '0'), ('debug-assertions', 'false'),
            ('overflow-checks', 'true'), ('panic', 'abort'))
# Declared before planning: every target unit resolves to RustGM's settings.
TARGET_PROFILE = {'name': 'dev', 'opt_level': '0', 'lto': 'false', 'codegen_backend': None,
                  'codegen_units': None, 'debuginfo': 0, 'split_debuginfo': None,
                  'debug_assertions': False, 'overflow_checks': True, 'rpath': False,
                  'incremental': False, 'panic': 'abort'}
IMPLEMENTATION = ('stdlib_plan.py', 'stdlib_inputs.py', 'stdlib_targets.py', 'cargo_plan.py')


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def shape(plan, packages):
    """Every unit field except its profile, with packages as portable identities."""
    def key(index):
        unit = plan.units[index]
        package = packages[unit.pkg_id]
        source = Path(unit.target.src_path).relative_to(package.planned_root).as_posix()
        return (package.identity, unit.target.name, unit.target.kind, unit.target.crate_types, source,
                unit.target.edition, unit.target.doc, unit.target.doctest, unit.target.test,
                unit.platform or '', unit.mode, unit.features)

    return sorted((key(index), sorted((key(edge.index), edge.extern_crate_name, edge.public, edge.noprelude,
                                       edge.nounused) for edge in unit.dependencies),
                   json.dumps(json.loads(unit.profile_json).get('rustflags')))
                  for index, unit in enumerate(plan.units))


def profiles(plan):
    """Distinct resolved profiles by execution role, as observed."""
    result = {}
    for unit in plan.units:
        rows = result.setdefault(unit.execution.value, [])
        profile = json.loads(unit.profile_json)
        if profile not in rows:
            rows.append(profile)
    return result


def check(plan, historical, pins, source_root, vendor_root):
    target = [json.loads(unit.profile_json) for unit in plan.units if unit.execution is Execution.TARGET_COMPILE]
    if any({key: profile[key] for key in TARGET_PROFILE} != TARGET_PROFILE for profile in target):
        raise ValueError('a target unit differs from the declared course profile')
    if any(json.loads(unit.profile_json)['panic'] != 'unwind' for unit in plan.units
           if unit.execution is not Execution.TARGET_COMPILE):
        raise ValueError('a host unit lost its unwind profile')
    new = stdlib_targets.bind(pins, plan, source_root, vendor_root)
    old = stdlib_targets.bind(pins, historical, source_root, vendor_root)
    if shape(plan, new) != shape(historical, old):
        raise ValueError('course-profile graph differs from its historical graph beyond the profile')


def run(args):
    pins = json.loads(BASE.read_text())
    _, historical = stdlib_targets.load('dev-profile')
    out = args.out
    out.mkdir(parents=True, exist_ok=False)
    implementation = {name: digest(Path(__file__).with_name(name)) for name in IMPLEMENTATION}
    declaration = {'state': 'predicted before course-profile planning',
                   'base_outcomes': {'path': str(BASE.relative_to(ROOT)), 'sha256': digest(BASE)},
                   'profile': {'name': 'dev', 'settings': dict(SETTINGS)},
                   'expected_units': {root: len(plan.units) for root, (plan, _) in historical.items()},
                   'expected_target_profile': TARGET_PROFILE, 'expected_host_panic': 'unwind',
                   'expected_structure': 'equal to the historical dev-profile graph except profiles',
                   'target_compilations': 0}
    save(out / 'declaration.json', declaration)
    tools = stdlib_inputs.tools(pins)
    vendor = stdlib_inputs.registry(pins, args.archives, out / 'inputs')
    local = stdlib_inputs.local(pins, args.source_root)
    workspace = stdlib_inputs.workspace(pins, args.source_root)
    (cargo, cargo_sha256), (rustc, rustc_sha256) = tools['cargo'], tools['rustc']
    target = pins['configuration']['target']
    observations = []
    for root, (old, _) in historical.items():
        job = Job(cargo, cargo_sha256, rustc, rustc_sha256, workspace[0][0], workspace[1][0], workspace,
                  root, target, default_features=True, vendor=vendor, local_packages=local, profile=SETTINGS)
        folder = out / f'plan-{root}'
        plan = plan_cargo(job, folder)
        if len(plan.units) != declaration['expected_units'][root]:
            raise ValueError('course-profile graph unit count differs from its declaration')
        check(plan, old, pins, args.source_root, vendor.root)
        observations.append({'root': root, 'units': len(plan.units),
                             'receipt_sha256': digest(folder / 'receipt.json'),
                             'graph_sha256': digest(folder / 'graph.stdout'), 'profiles': profiles(plan)})
    if any(digest(Path(__file__).with_name(name)) != sha for name, sha in implementation.items()):
        raise ValueError('re-planning implementation changed during execution')
    observation = {
        'schema': 1,
        'state': 'observed frozen Cargo unit graphs under the RustGM course profile; '
                 'no RustGM target compilation or library artifact',
        'base_outcomes': declaration['base_outcomes'],
        'configuration': {'target': target, 'profile': 'dev', 'settings': dict(SETTINGS),
                          'mechanism': 'explicit CARGO_PROFILE_DEV_* planner environment',
                          'panic': 'abort', 'default_features': True, 'features': []},
        'rationale': 'bootstrap ships std without debug assertions or debuginfo; RustGM has no '
                     'optimizer and fixes overflow checks on, so opt-level 0 and overflow checks on '
                     'are RustGM settings rather than bootstrap settings',
        'structure': declaration['expected_structure'],
        'tool_sha256': pins['tool_sha256'],
        'declaration_sha256': digest(out / 'declaration.json'),
        'implementation_sha256': implementation,
        'observations': observations,
    }
    save(out / 'observation.json', observation)
    print(json.dumps({row['root']: [row['units'], row['graph_sha256']] for row in observations}, indent=2))
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--archives', type=Path, required=True,
                        help='directory holding the pinned registry .crate archives')
    parser.add_argument('--source-root', type=Path, required=True,
                        help='root of the pinned Rust source tree (rust-1.98.0-<revision>)')
    parser.add_argument('--worker', action='store_true', help='caller already holds the storage guard')
    args = parser.parse_args()
    args.out, args.archives, args.source_root = (path.absolute() for path in
                                                 (args.out, args.archives, args.source_root))
    if args.worker:
        try:
            return run(args)
        except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
            print(f'stdlib re-plan: failure: {error}', file=sys.stderr)
            return 2
    command = [sys.executable, 'scripts/storage.py', 'run', '--prune-under', str(args.out), '--',
               sys.executable, __file__, *sys.argv[1:], '--worker']
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
