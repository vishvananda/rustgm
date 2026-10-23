#!/usr/bin/env python3
"""Qualify transitive artifact inventory independently of source-visible aliases."""
import argparse
import json
from pathlib import Path

from crate_qualification import Check, sha

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'pa29/tests/transitive-metadata-outcomes.json'


def rust_command(check, case):
    command = ['rustc', '+1.98.0', '--edition=2024', '--target=x86_64-unknown-linux-gnu',
               '--cap-lints=allow', '-C', 'panic=abort', '-C', 'overflow-checks=yes',
               '--crate-name', case['crate'], '-L', f'dependency={check.folder}']
    for alias, name in case['deps'].items():
        command += ['--extern', f'{alias}={check.folder / ("libshared-" + name + ".rlib")}']
    return command


def independent(check, cases, native):
    version = check.run('rust-version', ['rustc', '+1.98.0', '--version']).stdout
    if not version.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    for case in cases:
        name, source = case['name'], ROOT / 'pa29' / case['path']
        command = rust_command(check, case)
        if case['kind'] == 'rlib':
            output = check.folder / ('libshared-' + name + '.rlib')
            command += ['--crate-type=rlib', '-C', 'metadata=' + name]
        else:
            output = check.folder / (name + '.o')
            command += ['--emit=obj']
        check.run(name + '-rust-object', command + [source, '-o', output], case['rustc'],
                  () if case['code'] is None else (case['code'],))
        if native and case['native']:
            binary = check.folder / (name + '.rust')
            check.run(name + '-rust-link', rust_command(check, case) + [source, '-o', binary])
            result = check.run(name + '-rust-run', [binary])
            if result.stdout or result.stderr:
                raise ValueError('unexpected independent native output')


def frontend(check, case, name):
    command = [check.app, '--emit-lowir', ROOT / 'pa29' / case['path']]
    if case['kind'] == 'rlib':
        command += ['--crate-type=lib', '--crate-name', case['crate'], '--crate-disambiguator', case['name'],
                    '--metadata-output', check.folder / (name + '.rgm')]
    for alias, dependency in case['deps'].items():
        command += ['--extern', f'{alias}={check.folder / (dependency + ".rgm")}']
    for dependency in case['inventory']:
        command += ['--dependency-metadata', check.folder / (dependency + '.rgm')]
    result = check.run(name + '-frontend', command, case['frontend_expected_after_implementation'])
    if result.returncode == 0:
        if result.stderr:
            raise ValueError('unexpected successful frontend diagnostic')
    elif result.stdout or b'unresolved' not in result.stderr:
        raise ValueError('inaccessible crate must fail ordinary name resolution')
    lowir = check.folder / (name + '.lowir')
    lowir.write_bytes(result.stdout)
    return lowir


def failures(check, client):
    base = [check.app, '--emit-lowir', ROOT / 'pa29' / client['path'],
            '--extern', f'renamed={check.folder / "second.rgm"}']
    check.run('missing-inventory', base, 1, ('missing crate dependency metadata',))
    malformed = check.folder / 'truncated.rgm'
    malformed.write_bytes((check.folder / 'first.rgm').read_bytes()[:7])
    check.run('malformed-inventory', base + ['--dependency-metadata', malformed], 1, ('truncated',))
    check.run('combined-budget', base + ['--dependency-metadata', check.folder / 'first.rgm'] * 1024,
              2, ('external crate arguments exceed profile budget',))
    check.run('inventory-mode', [check.app, '--emit-types', ROOT / 'pa29' / client['path'],
                                 '--dependency-metadata', check.folder / 'first.rgm'],
              2, ('crate artifact options require --emit-lowir',))
    check.run('missing-argument', [check.app, '--emit-lowir', '--dependency-metadata'],
              2, ('--dependency-metadata requires a path',))


def candidate(check, cases, native):
    for case in cases:
        name = case['name']
        lowir = frontend(check, case, name)
        if case['frontend_expected_after_implementation']:
            continue
        # Duplicate canonical inventory paths must share decoding, including a
        # path already loaded through a visible --extern binding.
        repeated = {**case, 'inventory': list(reversed(case['inventory'] + list(case['deps'].values())))
                     + case['inventory']}
        again = frontend(check, repeated, name + '-repeat')
        if again.read_bytes() != lowir.read_bytes():
            raise ValueError('repeated/reordered artifact inputs changed LowIR')
        if case['kind'] == 'rlib' and ((check.folder / (name + '.rgm')).read_bytes()
                                      != (check.folder / (name + '-repeat.rgm')).read_bytes()):
            raise ValueError('repeated metadata changed bytes')
        if native and case['native']:
            binary = check.folder / (name + '.candidate')
            check.run(name + '-backend', ['python3', ROOT / 'scripts/backend.py',
                      check.folder / 'first.lowir', check.folder / 'second.lowir', lowir, '-o', binary])
            result = check.run(name + '-candidate-run', [binary])
            if result.stdout or result.stderr:
                raise ValueError('candidate native output differs from the declared empty result')
    failures(check, next(case for case in cases if case['name'] == 'client'))


def qualify(app, folder, native):
    manifest_sha = sha(MANIFEST)
    plan = json.loads(MANIFEST.read_text())
    pins = {ROOT / 'pa29' / case['path']: case['source_sha256'] for case in plan['cases']}
    if any(sha(path) != expected for path, expected in pins.items()):
        raise ValueError('frozen transitive metadata source changed')
    check = Check(app or Path(__file__), folder)
    binary_sha = None if app is None else sha(check.app)
    (check.folder / 'declaration.json').write_text(json.dumps({
        'manifest_sha256': manifest_sha, 'candidate_sha256': binary_sha,
        'native': native, 'scope': 'independent-only' if app is None else 'independent and candidate',
    }, indent=2) + '\n')
    independent(check, plan['cases'], native)
    if app is not None:
        candidate(check, plan['cases'], native)
    if sha(MANIFEST) != manifest_sha or any(sha(path) != expected for path, expected in pins.items()):
        raise ValueError('qualification inputs changed during execution')
    if app is not None and sha(check.app) != binary_sha:
        raise ValueError('candidate changed during execution')
    (check.folder / 'complete.json').write_text(json.dumps({
        'state': 'passed', 'manifest_sha256': manifest_sha, 'candidate_sha256': binary_sha,
        'cases': len(plan['cases']), 'native': native, 'runs_sha256': sha(check.folder / 'runs.json'),
    }, indent=2) + '\n')
    print(f'Qualified{len(plan["cases"])} transitive metadata cases; candidate={app is not None}, native={native}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path)
    parser.add_argument('--independent-only', action='store_true')
    parser.add_argument('--evidence-root', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    if (args.app is None) != args.independent_only:
        parser.error('select --app or --independent-only')
    qualify(args.app, args.evidence_root, args.native)


if __name__ == '__main__':
    main()
