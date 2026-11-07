#!/usr/bin/env python3
"""Run small authored target graphs through an actual RustGM binary.

Outcomes are declared before execution: a library plus binary consumer builds
completely and reproducibly, including from a Rust 2018 library, a rejected
consumer stops after its library, and an unsupported crate type fails before any
process starts.
"""
import argparse
import json
from pathlib import Path
import sys
import time

from build_units import parse
from host_process import digest
from target_units import TargetPackage, execute, plan_units

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / 'pa29/tests/spec/closed-records'
TRIPLE = 'x86_64-unknown-linux-gnu'
PROFILE = dict(name='release', opt_level='0', lto='false', codegen_backend=None, codegen_units=None,
               debuginfo=0, split_debuginfo=None, debug_assertions=False, overflow_checks=True,
               rpath=False, incremental=False, panic='abort', strip={'deferred': 'None'})
CASES = {
    'library-and-binary': {'client': 'pass-construct.rs', 'edition': '2024',
                           'expected': {'state': 'complete', 'completed': ['records', 'client'],
                                        'failure': None}},
    'rejected-binary': {'client': 'reject-private-field.rs', 'edition': '2024',
                        'expected': {'state': 'failed', 'completed': ['records'],
                                     'failure': {'crate': 'client', 'kind': 'compile'}}},
    'legacy-edition-library': {'client': 'pass-construct.rs', 'edition': '2018',
                               'expected': {'state': 'complete', 'completed': ['records', 'client'],
                                            'failure': None}},
    'rlib-beside-dylib': {'client': 'pass-construct.rs', 'edition': '2024', 'kinds': ['dylib', 'rlib'],
                          'expected': {'state': 'complete', 'completed': ['records', 'client'],
                                       'failure': None, 'dropped': {'records': ['dylib']}}},
    'unsupported-crate-type': {'client': 'pass-construct.rs', 'edition': '2024', 'kinds': ['dylib'],
                               'expected': {'state': 'failed', 'completed': [],
                                            'failure': {'crate': 'records', 'kind': 'unsupported'},
                                            'blocked': ['client']}},
}


def unit(name, kinds, source, edition, dependencies=()):
    return dict(pkg_id=f'path+file://{FIXTURES}#{name}@1.0.0',
                target=dict(kind=kinds, crate_types=kinds, name=name, src_path=str(source),
                            edition=edition, doc=False, doctest=False, test=False),
                profile=PROFILE, platform=TRIPLE, mode='build', features=[],
                dependencies=[dict(index=index, extern_crate_name=alias, public=False,
                                   noprelude=False, nounused=False) for index, alias in dependencies])


def graph(case):
    """The consumer depends on the library through the fixtures' `one` alias."""
    library = FIXTURES / 'first.rs'
    client = FIXTURES / case['client']
    data = json.dumps(dict(version=1, roots=[1], units=[
        unit('records', case.get('kinds', ['lib']), library, case['edition']),
        unit('client', ['bin'], client, '2024', [(0, 'one')])])).encode()
    plan = parse(data, TRIPLE)
    packages = {}
    for row in plan.units:
        source = Path(row.target.src_path)
        # Authored fixture packages have no manifest; their environment is explicit.
        packages[row.pkg_id] = TargetPackage(
            f'fixture pa29/tests/spec/closed-records/{source.name}', digest(source), FIXTURES, FIXTURES,
            (('CARGO_PKG_NAME', row.target.name), ('CARGO_PKG_VERSION', '1.0.0')), False)
    return plan, packages, data


def observed(receipt):
    failure = receipt['failure']
    dropped = {row['crate']: row['dropped_crate_types'] for row in receipt['completed']
               if row['dropped_crate_types']}
    return {'state': receipt['state'], 'completed': [row['crate'] for row in receipt['completed']],
            'failure': None if failure is None else {'crate': failure['crate'], 'kind': failure['kind']},
            **({'blocked': [row['crate'] for row in receipt['blocked']]} if receipt['blocked'] else {}),
            **({'dropped': dropped} if dropped else {})}


def run(rustgm, out):
    rustgm = rustgm.resolve(strict=True)
    sha = digest(rustgm)
    out.mkdir(parents=True)
    (out / 'declaration.json').write_text(json.dumps({'rustgm_sha256': sha, 'cases': CASES}, indent=2) + '\n')
    results, hashes = {}, {}
    for name, case in CASES.items():
        for attempt in ('first', 'repeat') if name == 'library-and-binary' else ('first',):
            plan, packages, data = graph(case)
            folder = out / name / attempt
            steps = plan_units(plan, packages, folder)
            receipt = execute(steps, packages, rustgm, sha, folder, {'root': name, 'graph': data.decode()})
            results[f'{name}/{attempt}'] = {'observed': observed(receipt), 'expected': case['expected'],
                                            'receipt_sha256': digest(folder / 'receipt.json')}
            hashes[f'{name}/{attempt}'] = [{key: value['sha256'] for key, value in row['outputs'].items()}
                                           for row in receipt['completed']]
    reproducible = hashes['library-and-binary/first'] == hashes['library-and-binary/repeat']
    passed = reproducible and all(row['observed'] == row['expected'] for row in results.values())
    summary = {'state': 'passed' if passed else 'failed', 'reproducible_outputs': reproducible,
               'rustgm': {'path': str(rustgm), 'sha256': sha}, 'unchanged_rustgm': digest(rustgm) == sha,
               'results': results, 'output_sha256': hashes}
    summary['state'] = summary['state'] if summary['unchanged_rustgm'] else 'failed'
    (out / 'result.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))
    return 0 if summary['state'] == 'passed' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rustgm', type=Path, required=True)
    parser.add_argument('--out', type=Path,
                        default=ROOT / 'obj/qualification/target-units' / str(time.time_ns()))
    args = parser.parse_args()
    try:
        return run(args.rustgm, args.out.absolute())
    except (OSError, ValueError, KeyError) as error:
        print(f'target unit check: infrastructure failure: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
