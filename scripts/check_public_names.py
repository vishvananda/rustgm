#!/usr/bin/env python3
"""Qualify public crate aliases without changing the independently frozen sources."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'pa29/tests/public-name-outcomes.json'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Check:
    def __init__(self, app, folder):
        self.app = app.resolve()
        self.folder = folder.resolve()
        self.folder.mkdir(parents=True, exist_ok=False)
        self.rows = []

    def run(self, name, command, expected=0, codes=()):
        started = time.monotonic()
        command = [str(arg) for arg in command]
        result = subprocess.run(command, capture_output=True, timeout=180)
        stdout = self.folder / (name + '.stdout')
        stderr = self.folder / (name + '.stderr')
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        self.rows.append({'name': name, 'command': command, 'status': result.returncode,
                          'expected': expected, 'seconds': time.monotonic() - started,
                          'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr)})
        (self.folder / 'runs.json').write_text(json.dumps(self.rows, indent=2) + '\n')
        if result.returncode != expected or any(code.encode() not in result.stderr for code in codes):
            raise ValueError(f'{name}: expected {expected}, got {result.returncode}\n'
                             + result.stderr.decode(errors='replace'))
        return result

    def frontend(self, case, libraries, reverse=False):
        name = case['name'] + ('-repeat' if reverse else '')
        command = [self.app, '--emit-lowir', ROOT / 'pa29' / case['path']]
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
        if expected and case.get('frontend_diagnostic', '').encode() not in result.stderr:
            raise ValueError(f'{name}: wrong semantic rejection')
        if expected and (self.folder / (name + '.rgm')).exists():
            raise ValueError(f'{name}: failed export published metadata')
        path = self.folder / (name + '.lowir')
        path.write_bytes(result.stdout)
        return path


def rust_command(case, libraries, folder):
    command = ['rustc', '+1.98.0', '--edition=2024', '--target=x86_64-unknown-linux-gnu',
               '--cap-lints=allow', '-C', 'panic=abort', '-C', 'overflow-checks=yes',
               '--crate-name', case['crate'], '-C',
               'metadata=' + case['name'].replace('-', '_'), '-L', f'dependency={folder}']
    for alias, dependency in case['deps'].items():
        command += ['--extern', f'{alias}={libraries[dependency]}']
    return command


def qualify(app, folder, native):
    plan = json.loads(MANIFEST.read_text())
    check = Check(app, folder)
    version = check.run('rust-version', ['rustc', '+1.98.0', '--version']).stdout
    if not version.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    libraries, rust_libraries, lowir_libraries, metadata = {}, {}, [], {}
    for case in plan['cases']:
        source = ROOT / 'pa29' / case['path']
        if sha(source) != case['source_sha256']:
            raise ValueError(f'changed frozen source: {source}')
        name = case['name']
        library = case['kind'] == 'rlib'
        lowir = check.frontend(case, libraries)
        if case['frontend_expected_after_implementation'] == 0:
            repeated = check.frontend(case, libraries, reverse=True)
            if lowir.read_bytes() != repeated.read_bytes():
                raise ValueError(f'{name}: dependency order changed LowIR')
        command = rust_command(case, rust_libraries, check.folder)
        artifact = check.folder / (f'lib{case["crate"]}-{name}.rlib' if library else name + '.o')
        mode = ['--crate-type=rlib'] if library else ['--emit=obj']
        check.run(name + '-rustc', command + mode + [source, '-o', artifact],
                  case['rustc'], case['rustc_codes'])
        if library and case['rustc'] == 0:
            artifact = check.folder / (name + '.rgm')
            if artifact.read_bytes() != (check.folder / (name + '-repeat.rgm')).read_bytes():
                raise ValueError(f'{name}: repeated publication changed metadata')
            libraries[name] = artifact
            metadata[name] = sha(artifact)
            rust_libraries[name] = check.folder / f'lib{case["crate"]}-{name}.rlib'
            lowir_libraries.append(lowir)
        if native and case['native']:
            candidate = check.folder / (name + '.candidate')
            independent = check.folder / (name + '.rust')
            check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py',
                                         *lowir_libraries, lowir, '-o', candidate])
            check.run(name + '-rust-link', command + [source, '-o', independent])
            left = check.run(name + '-candidate-run', [candidate])
            right = check.run(name + '-rust-run', [independent])
            if (left.stdout, left.stderr) != (right.stdout, right.stderr):
                raise ValueError(f'{name}: native output mismatch')
    receipt = {'manifest_sha256': sha(MANIFEST), 'candidate_sha256': sha(check.app),
               'cases': len(plan['cases']), 'native': native, 'metadata': metadata,
               'runs_sha256': sha(check.folder / 'runs.json')}
    (check.folder / 'complete.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'Qualified {len(plan["cases"])} public-name cases; native={native}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--evidence-root', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    qualify(args.app, args.evidence_root, args.native)


if __name__ == '__main__':
    main()
