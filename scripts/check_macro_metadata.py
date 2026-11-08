#!/usr/bin/env python3
"""Qualify exported macro metadata against the independently frozen PA29 outcomes."""
import argparse
import hashlib
import json
from pathlib import Path

from crate_qualification import Check as CommandCheck

ROOT = Path(__file__).resolve().parents[1]
MANIFESTS = [ROOT / 'pa29/tests/macro-metadata-outcomes.json',
             ROOT / 'pa29/tests/macro-spacing-outcomes.json',
             ROOT / 'pa29/tests/builtin-macro-rows-outcomes.json',
             ROOT / 'pa29/tests/macro-reexport-outcomes.json',
             ROOT / 'pa29/tests/macro-stability-outcomes.json',
             ROOT / 'pa29/tests/imported-panic-outcomes.json',
             ROOT / 'pa29/tests/no-std-client-outcomes.json',
             ROOT / 'pa29/tests/variant-reexport-outcomes.json',
             ROOT / 'pa29/tests/extern-core-outcomes.json',
             ROOT / 'pa29/tests/shim-glob-outcomes.json']
SOURCE_PROFILE = 'module-bodies-v13'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(case):
    return case['crate'] + '@' + case['name'].replace('-', '_')


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
        # Inventory artifacts are visible only by crate identity, never by alias.
        for dependency in case.get('inventory', []):
            command += ['--dependency-metadata', libraries[dependency]]
        if case['kind'] in ('rlib', 'object'):
            command += ['--crate-type=lib', '--crate-name', case['crate'],
                        '--crate-disambiguator', case['name'].replace('-', '_'),
                        '--metadata-output', self.folder / (name + '.rgm')]
        # A source-core provider publishes the module-body profile; its clients
        # name the provider's crate identity as their source owner.
        if case.get('source_core'):
            command += ['--source-core', identity(case), '--source-profile', SOURCE_PROFILE]
        if 'source_client' in case:
            command += ['--source-client', libraries['identity:' + case['source_client']]]
        expected = case['frontend_expected_after_implementation']
        diagnostic = [case['frontend_diagnostic']] if 'frontend_diagnostic' in case else []
        result = self.run(name + '-frontend', command, expected, diagnostic)
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
    # A no_std client of a source provider has no sysroot: rustc finds its
    # injected core by --extern and compiler_builtins in the oracle folder.
    if case.get('rustc_sysroot') == 'empty':
        sysroot = folder / 'empty-sysroot'
        sysroot.mkdir(exist_ok=True)
        command += ['--sysroot', sysroot, '-L', f'crate={folder}']
    for alias, dependency in {**case['deps'], **case.get('rustc_deps', {})}.items():
        command += ['--extern', f'{alias}={libraries[dependency]}']
    return command


def rustc_case(check, case, rust_libraries):
    """Run the independent compiler alone; returns its library or binary."""
    source = ROOT / 'pa29' / case['path']
    name = case['name']
    command = rust_command(case, rust_libraries, check.folder)
    environment = case.get('rustc_environment')
    if case['kind'] == 'rlib':
        artifact = check.folder / f'lib{case["crate"]}-{name}.rlib'
        command += ['--crate-type=rlib', source, '-o', artifact]
    elif case['kind'] == 'object':
        artifact = check.folder / (name + '.rust.o')
        command += ['--crate-type=lib', '--emit=obj', source, '-o', artifact]
    else:
        artifact = check.folder / (name + '.rust')
        command += [source, '-o', artifact]
    check.run(name + '-rustc', command, case['rustc'], case['rustc_codes'],
              environment=environment)
    if case['rustc'] == 0 and case['kind'] == 'rlib':
        rust_libraries[name] = artifact
    return artifact


def driven(check, case, label, objects):
    """Link one object client with its frozen C driver and run it."""
    driver = ROOT / 'pa29' / case['native_driver']
    if sha(driver) != case['native_driver_sha256']:
        raise ValueError(f'changed frozen native driver: {driver}')
    binary = check.folder / (label + '.native')
    check.run(label + '-link', ['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', driver,
                                *objects, '-o', binary])
    result = check.run(label + '-run', [binary])
    if result.stdout.decode() != case['native_stdout']:
        raise ValueError(f'{label}: native output differs from the declared outcome')
    return result


def native_case(check, case, lowir, lowirs, artifact):
    name = case['name']
    if case['kind'] == 'object':
        candidate = check.folder / (name + '.candidate.o')
        check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py',
                                     '--emit-object', lowir, '-o', candidate])
        left = driven(check, case, name + '-candidate', [candidate])
        right = driven(check, case, name + '-rust', [artifact])
    else:
        candidate = check.folder / (name + '.candidate')
        linked = [lowirs[dependency]
                  for dependency in [*case['deps'].values(), *case.get('inventory', [])]]
        check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py', *linked, lowir,
                                     '-o', candidate])
        exit_status = case['rustc_native_exit']
        left = check.run(name + '-candidate-run', [candidate], exit_status)
        right = check.run(name + '-rust-run', [artifact], exit_status)
    if (left.stdout, left.stderr) != (right.stdout, right.stderr):
        raise ValueError(f'{name}: native output mismatch')


def qualify_case(check, case, state, native):
    libraries, rust_libraries, lowirs, metadata = state
    source = ROOT / 'pa29' / case['path']
    if sha(source) != case['source_sha256']:
        raise ValueError(f'changed frozen source: {source}')
    name = case['name']
    # A support crate that only the independent compiler's injection needs.
    if case.get('rustc_only'):
        rustc_case(check, case, rust_libraries)
        return
    library = case['kind'] == 'rlib'
    libraries['identity:' + name] = identity(case)
    lowir = check.frontend(case, libraries)
    passing = case['frontend_expected_after_implementation'] == 0
    if passing:
        repeated = check.frontend(case, libraries, reverse=True)
        if lowir.read_bytes() != repeated.read_bytes():
            raise ValueError(f'{name}: dependency order changed LowIR')
    artifact = rustc_case(check, case, rust_libraries)
    if library and passing:
        published = check.folder / (name + '.rgm')
        encoded = published.read_bytes()
        if encoded != (check.folder / (name + '-repeat.rgm')).read_bytes():
            raise ValueError(f'{name}: repeated publication changed metadata')
        if 'metadata_version' in case and encoded[12:14] != bytes([14, case['metadata_version']]):
            raise ValueError(f'{name}: macro table version differs from the declared outcome')
        libraries[name] = published
        metadata[name] = sha(published)
        lowirs[name] = lowir
    if native and case['native'] and passing:
        native_case(check, case, lowir, lowirs, artifact)


def oracle(folder, native):
    """Freeze the independent compiler's outcomes before any candidate run."""
    check = CommandCheck(Path('/bin/true'), folder)
    rust_libraries, count = {}, 0
    for manifest in MANIFESTS:
        for case in json.loads(manifest.read_text())['cases']:
            if sha(ROOT / 'pa29' / case['path']) != case['source_sha256']:
                raise ValueError(f'changed frozen source: {case["path"]}')
            artifact = rustc_case(check, case, rust_libraries)
            if native and case['native'] and case['kind'] == 'object':
                driven(check, case, case['name'] + '-rust', [artifact])
            elif native and case['native']:
                check.run(case['name'] + '-rust-run', [artifact], case['rustc_native_exit'])
            count += 1
    receipt = {'manifests_sha256': {path.name: sha(path) for path in MANIFESTS},
               'cases': count, 'native': native, 'runs_sha256': sha(check.folder / 'runs.json')}
    (check.folder / 'oracle.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'Oracle matched {count} declared rustc outcomes; native={native}')


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
    parser.add_argument('--oracle-only', action='store_true',
                        help='run only the independent compiler against the declarations')
    args = parser.parse_args()
    if args.oracle_only:
        oracle(args.evidence_root, args.native)
    else:
        qualify(args.app, args.evidence_root, args.native)


if __name__ == '__main__':
    main()
