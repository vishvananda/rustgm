#!/usr/bin/env python3
"""Qualify exported macro metadata against the independently frozen PA29 outcomes."""
import argparse
import hashlib
import json
from pathlib import Path

from crate_qualification import Check as CommandCheck

ROOT = Path(__file__).resolve().parents[1]
MANIFESTS = [ROOT / 'pa29/tests/macro-metadata-outcomes.json',
             ROOT / 'pa29/tests/macro-spacing-outcomes.json']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Check(CommandCheck):
    def frontend(self, case, libraries, reverse=False):
        name = case['name'] + ('-repeat' if reverse else '')
        command = [self.app, '--emit-lowir', f'--edition={case["edition"]}',
                   ROOT / 'pa29' / case['path']]
        entries = list(case['deps'].items())
        if reverse:
            entries.reverse()
        for alias, dependency in entries:
            command += ['--extern', f'{alias}={libraries[dependency]}']
        if case['kind'] == 'rlib':
            command += ['--crate-type=lib', '--crate-name', case['crate'],
                        '--crate-disambiguator', case['name'].replace('-', '_'),
                        '--metadata-output', self.folder / (name + '.rgm')]
        expected = case['frontend_expected_after_implementation']
        result = self.run(name + '-frontend', command, expected)
        if expected == 0 and result.stderr:
            raise ValueError(f'{name}: unexpected frontend diagnostic')
        if expected and (self.folder / (name + '.rgm')).exists():
            raise ValueError(f'{name}: failed export published metadata')
        path = self.folder / (name + '.lowir')
        path.write_bytes(result.stdout)
        return path


def rust_command(case, libraries, folder):
    command = ['rustc', '+1.98.0', f'--edition={case["edition"]}',
               '--target=x86_64-unknown-linux-gnu', '--cap-lints=allow', '-C', 'panic=abort',
               '--crate-name', case['crate'], '-C', 'metadata=' + case['name'].replace('-', '_'),
               '-L', f'dependency={folder}']
    for alias, dependency in case['deps'].items():
        command += ['--extern', f'{alias}={libraries[dependency]}']
    return command


def qualify_case(check, case, state, native):
    libraries, rust_libraries, lowirs, metadata = state
    source = ROOT / 'pa29' / case['path']
    if sha(source) != case['source_sha256']:
        raise ValueError(f'changed frozen source: {source}')
    name = case['name']
    library = case['kind'] == 'rlib'
    lowir = check.frontend(case, libraries)
    passing = case['frontend_expected_after_implementation'] == 0
    if passing:
        repeated = check.frontend(case, libraries, reverse=True)
        if lowir.read_bytes() != repeated.read_bytes():
            raise ValueError(f'{name}: dependency order changed LowIR')
    command = rust_command(case, rust_libraries, check.folder)
    if library:
        artifact = check.folder / f'lib{case["crate"]}-{name}.rlib'
        check.run(name + '-rustc', command + ['--crate-type=rlib', source, '-o', artifact],
                  case['rustc'], case['rustc_codes'])
    else:
        artifact = check.folder / (name + '.rust')
        check.run(name + '-rustc', command + [source, '-o', artifact],
                  case['rustc'], case['rustc_codes'])
    if library and passing:
        published = check.folder / (name + '.rgm')
        if published.read_bytes() != (check.folder / (name + '-repeat.rgm')).read_bytes():
            raise ValueError(f'{name}: repeated publication changed metadata')
        libraries[name] = published
        metadata[name] = sha(published)
        rust_libraries[name] = artifact
        lowirs[name] = lowir
    if native and case['native'] and passing:
        candidate = check.folder / (name + '.candidate')
        linked = [lowirs[dependency] for dependency in case['deps'].values()]
        check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py', *linked, lowir,
                                     '-o', candidate])
        exit_status = case['rustc_native_exit']
        left = check.run(name + '-candidate-run', [candidate], exit_status)
        right = check.run(name + '-rust-run', [artifact], exit_status)
        if (left.stdout, left.stderr) != (right.stdout, right.stderr):
            raise ValueError(f'{name}: native output mismatch')


def qualify(app, folder, native):
    check = Check(app, folder)
    version = check.run('rust-version', ['rustc', '+1.98.0', '--version']).stdout
    if not version.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    state = ({}, {}, {}, {})
    count = 0
    for manifest in MANIFESTS:
        for case in json.loads(manifest.read_text())['cases']:
            qualify_case(check, case, state, native)
            count += 1
    receipt = {'manifests_sha256': {path.name: sha(path) for path in MANIFESTS},
               'candidate_sha256': sha(check.app), 'cases': count, 'native': native,
               'metadata': state[3], 'runs_sha256': sha(check.folder / 'runs.json')}
    (check.folder / 'complete.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'Qualified {count} macro-metadata cases; native={native}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--evidence-root', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    qualify(args.app, args.evidence_root, args.native)


if __name__ == '__main__':
    main()
