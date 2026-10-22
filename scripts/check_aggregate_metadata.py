#!/usr/bin/env python3
"""Qualify fixed arrays and C aggregate types across separately compiled crates."""
import argparse
import json
from pathlib import Path

from crate_qualification import Check, sha

ROOT = Path(__file__).resolve().parents[1]


def rust_command(crate):
    return ['rustc', '+1.98.0', '--edition=2024', '--target=x86_64-unknown-linux-gnu',
            '--cap-lints=allow', '-C', 'panic=abort', '-C', 'overflow-checks=yes',
            '-C', 'debug-assertions=off', '--crate-name', crate]


def frontend(check, stage, plan, case, repeat=False):
    source = ROOT / stage / case['path']
    name = stage + '-' + source.stem + ('-repeat' if repeat else '')
    command = [check.app, '--emit-lowir', source]
    if case['role'] == 'library':
        command += ['--crate-type=lib', '--crate-name', plan['crate'],
                    '--crate-disambiguator', 'aggregate_v1', '--metadata-output',
                    check.folder / (name + '.rgm')]
    else:
        command += ['--extern', f'{plan["crate"]}={check.folder / (stage + "-library.rgm")}']
    result = check.run(name + '-frontend', command, case['frontend_expected_after_implementation'])
    if result.returncode == 0 and result.stderr:
        raise ValueError(f'{name}: unexpected frontend diagnostic')
    if result.returncode != 0:
        diagnostic = result.stderr.decode(errors='replace')
        reason = {
            'reject-length': 'incompatible types',
            'reject-element': 'incompatible types',
            'reject-escape': 'required lifetime does not outlive',
            'reject-move': 'uninitialized or moved at this read',
            'reject-private': 'aggregate field is private',
            'reject-rust-pointer': 'incompatible callable convention or safety',
        }[source.stem]
        if (not diagnostic or result.stdout or
                any(word in diagnostic for word in ['not implemented', 'outside the metadata profile',
                                                    'unknown imported', 'missing imported']) or
                reason not in diagnostic):
            raise ValueError(f'{name}: wrong semantic rejection: {diagnostic}')
    path = check.folder / (name + '.lowir')
    path.write_bytes(result.stdout)
    return path


def source_contract(check, stage, plan, case, library, native):
    source = ROOT / stage / case['path']
    if sha(source) != case['source_sha256']:
        raise ValueError(f'changed frozen source: {source}')
    name = stage + '-' + source.stem
    lowir = frontend(check, stage, plan, case)
    is_library = case['role'] == 'library'
    command = rust_command(plan['crate'] if is_library else 'client')
    if is_library:
        command += ['--crate-type=rlib', '-C', 'metadata=aggregate_v1']
        output = library
    else:
        command += ['--extern', f'{plan["crate"]}={library}', '-L', f'dependency={check.folder}']
        output = check.folder / (name + '.o')
    check.run(name + '-rust', command + ([] if is_library else ['--emit=obj']) +
              [source, '-o', output], case['rustc'], [case['code']] if case['code'] else ())
    if case['frontend_expected_after_implementation'] == 0:
        repeated = frontend(check, stage, plan, case, True)
        if repeated.read_bytes() != lowir.read_bytes():
            raise ValueError(f'{name}: nondeterministic LowIR')
        if is_library and ((check.folder / (name + '.rgm')).read_bytes() !=
                           (check.folder / (name + '-repeat.rgm')).read_bytes()):
            raise ValueError(f'{name}: nondeterministic metadata')
    if native and case['native']:
        candidate, independent = check.folder / (name + '.candidate'), check.folder / (name + '.rust')
        check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py',
                  check.folder / (stage + '-library.lowir'), lowir, '-o', candidate])
        check.run(name + '-rust-link', command + [source, '-o', independent])
        left = check.run(name + '-candidate-run', [candidate])
        right = check.run(name + '-rust-run', [independent])
        if (left.stdout, left.stderr) != (right.stdout, right.stderr):
            raise ValueError(f'{name}: native output mismatch')


def c_caller(check, stage, plan):
    driver = ROOT / stage / plan['c_native']['path']
    if sha(driver) != plan['c_native']['source_sha256']:
        raise ValueError('changed C caller')
    obj = check.folder / 'aggregates.o'
    check.run('c-object', ['python3', ROOT / 'scripts/backend.py', '--emit-object',
                          check.folder / (stage + '-library.lowir'), '-o', obj])
    binary = check.folder / 'c-caller'
    check.run('c-link', ['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', driver, obj, '-lm', '-o', binary])
    result = check.run('c-run', [binary], plan['c_native']['exit_status'])
    if result.stdout or result.stderr:
        raise ValueError('unexpected C caller output')


def qualify(app, folder, native):
    check = Check(app, folder)
    version = check.run('rust-version', ['rustc', '+1.98.0', '--version']).stdout
    if not version.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    manifests, cases = {}, 0
    for stage, name in [('pa29', 'array-metadata-outcomes.json'), ('pa30', 'aggregate-metadata-outcomes.json')]:
        manifest = ROOT / stage / 'tests' / name
        plan = json.loads(manifest.read_text())
        manifests[stage] = sha(manifest)
        library = check.folder / f'lib{plan["crate"]}-aggregate_v1.rlib'
        for case in plan['cases']:
            source_contract(check, stage, plan, case, library, native)
            cases += 1
        if native and 'c_native' in plan:
            c_caller(check, stage, plan)
    receipt = {'manifests': manifests, 'candidate_sha256': sha(check.app), 'cases': cases,
               'native': native, 'runs_sha256': sha(check.folder / 'runs.json')}
    (check.folder / 'complete.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'Qualified {cases} aggregate metadata cases; native={native}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--evidence-root', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    qualify(args.app, args.evidence_root, args.native)


if __name__ == '__main__':
    main()
