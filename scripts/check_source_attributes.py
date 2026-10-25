#!/usr/bin/env python3
"""Qualify real CLI source-header artifacts, warnings and fresh native clients."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time

import backend
from crate_qualification import Check, sha

ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / 'pa31/tests/spec/source-attribute-transport/cli-contract.json'


class Gate:
    def __init__(self, app, folder, seconds):
        self.check = Check(app, folder)
        self.deadline = time.monotonic() + seconds

    def run(self, name, command, expected=0, *, stderr=b'', contains=(), environment=None):
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('source attribute qualification deadline exhausted')
        result = self.check.run(name, command, expected, contains,
                                timeout=min(10, remaining), environment=environment)
        if stderr is not None and result.stderr != stderr:
            raise ValueError(f'{name}: undeclared diagnostic bytes\n{result.stderr!r}')
        return result


def validate_inputs(plan):
    rows = [*plan['providers'].values(), *plan['clients'].values(),
            plan['ordinary'], plan['legacy'], plan['unsupported'], plan['native_driver']]
    for row in rows:
        if sha(ROOT / row['path']) != row['sha256']:
            raise ValueError(f"frozen input changed: {row['path']}")
    if sha(ROOT / 'backend.lock.json') != plan['backend']['lock_sha256']:
        raise ValueError('frozen backend contract changed')


def identity_arguments(plan):
    identity = plan['provider_identity']
    return ['--crate-name', identity['name'], '--crate-disambiguator', identity['disambiguator']]


def library(gate, plan, name, source, diagnostic, *, source_core=True, expected=0,
            contains=()):
    folder = gate.check.folder
    metadata = folder / (name + '.rgm')
    command = [gate.check.app, '--emit-lowir', '--edition=2024', '--crate-type=lib',
               *identity_arguments(plan), '--metadata-output', metadata]
    if source_core:
        identity = plan['provider_identity']
        command += ['--source-core', identity['name'] + '@' + identity['disambiguator']]
    result = gate.run(name + '-provider', command + [source], expected,
                      stderr=diagnostic, contains=contains)
    lowir = folder / (name + '.lowir')
    lowir.write_bytes(result.stdout)
    if expected:
        if metadata.exists() or result.stdout:
            raise ValueError(f'{name}: rejected provider published output')
        return None
    encoded = metadata.read_bytes()
    if source_core and (encoded[:8] != b'RGMETA\r\n' or encoded[12:16] != b'\x0a\x01\x00\x00'):
        raise ValueError(f'{name}: SourceCore did not publish complete row version one')
    return metadata


def client(gate, plan, case, metadata, *, diagnostic=None, contains=()):
    folder = gate.check.folder
    name = case['name']
    artifact = folder / (name + '-client.rgm')
    identity = plan['provider_identity']
    source = ROOT / plan['clients'][case['client']]['path']
    command = [gate.check.app, '--emit-lowir', '--edition=2024', '--crate-type=lib',
               '--crate-name', 'client', '--crate-disambiguator', name,
               '--source-client', identity['name'] + '@' + identity['disambiguator'],
               '--extern', f'base={metadata}', '--metadata-output', artifact, source]
    expected = case['status']
    result = gate.run(name + '-client', command, expected,
                      stderr=case['stderr'].encode() if diagnostic is None else diagnostic,
                      contains=contains)
    if expected == 0:
        encoded = artifact.read_bytes() if artifact.is_file() else b''
        if (not result.stdout or len(encoded) < 13 or encoded[:8] != b'RGMETA\r\n'
                or encoded[12] != plan['client_metadata_tag']):
            raise ValueError(f'{name}: successful client changed its ordinary library output')
    elif result.stdout or artifact.exists():
        raise ValueError(f'{name}: LowIR/artifact output disagrees with its declared status')
    lowir = folder / (name + '.lowir')
    lowir.write_bytes(result.stdout)
    return lowir


def candidate(gate, plan, native):
    providers = {}
    for name, row in plan['providers'].items():
        providers[name] = library(gate, plan, name, ROOT / row['path'], row['stderr'].encode())
    for case in plan['accepted']:
        lowir = client(gate, plan, case, providers[case['provider']])
        if native:
            execute_native(gate, plan, case, lowir)
    for name in plan['frontend_only_consumers']['providers']:
        client(gate, plan, {'name': name + '_consumer', 'client': 'used', 'status': 0,
                           'stderr': ''}, providers[name])
    for case in plan['rejected']:
        client(gate, plan, case, providers[case['provider']])
    opponents(gate, plan, providers)


def opponents(gate, plan, providers):
    folder = gate.check.folder
    golden = (ROOT / plan['legacy']['path']).read_text()
    legacy = folder / 'legacy.rgm'
    legacy.write_bytes(bytes.fromhex(' '.join(line for line in golden.splitlines()
                                               if not line.startswith('#'))))
    if sha(legacy) != plan['legacy']['decoded_sha256']:
        raise ValueError('legacy golden bytes changed')
    client(gate, plan, {'name': 'legacy_unknown', **plan['legacy']}, legacy)
    corrupt = folder / 'invalid-version.rgm'
    encoded = bytearray(providers['marked'].read_bytes())
    encoded[13:15] = b'\xff\xff'
    corrupt.write_bytes(encoded)
    # This is an owning decoder error, distinct from consumer warning formatting.
    source = ROOT / plan['clients']['used']['path']
    result = gate.run('invalid-version', [gate.check.app, '--emit-lowir', source,
                      '--extern', f'base={corrupt}'], 1, stderr=None,
                      contains=(plan['corruption']['diagnostic'],))
    if result.stdout:
        raise ValueError('invalid version emitted LowIR')
    unsupported = plan['unsupported']
    library(gate, plan, 'unsupported', ROOT / unsupported['path'], None,
            expected=1, contains=(unsupported['diagnostic'],))
    ordinary = ROOT / plan['ordinary']['path']
    first = library(gate, plan, 'ordinary', ordinary, b'', source_core=False)
    again = library(gate, plan, 'ordinary-repeat', ordinary, b'', source_core=False)
    if first.read_bytes() != again.read_bytes() or first.read_bytes()[12] != plan['ordinary']['tag']:
        raise ValueError('ordinary publication changed profiles or reproducible bytes')


def rust_command(tool):
    return [tool, '--edition=2024', '--target=x86_64-unknown-linux-gnu',
            '-Cpanic=abort', '--error-format=json']


def independent_providers(gate, plan):
    environment = plan['independent']['environment']
    location = gate.run('rust-tool', ['rustup', 'which', '--toolchain', '1.98.0', 'rustc'])
    tool = Path(location.stdout.decode().strip()).resolve()
    if sha(tool) != plan['independent']['rustc_sha256']:
        raise ValueError('independent compiler bytes differ from the frozen tool')
    version = gate.run('rust-version', [tool, '--version'])
    if not version.stdout.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    libraries = {}
    for name in ['plain', 'marked']:
        output = gate.check.folder / ('libminicore-' + name + '.rlib')
        gate.run(name + '-rust-provider', [*rust_command(tool), '--crate-type=rlib',
                 '--crate-name', 'minicore', '-Cmetadata=source_attribute_' + name,
                 ROOT / plan['providers'][name]['path'], '-o', output], environment=environment)
        libraries[name] = output
    return tool, libraries


def execute_native(gate, plan, case, lowir):
    name, folder = case['name'], gate.check.folder
    output = folder / (name + '.candidate.o')
    gate.run(name + '-backend', [sys.executable, ROOT / 'scripts/backend.py',
             '--emit-object', lowir, '-o', output])
    link_and_run(gate, plan, name + '-candidate', output)
    output.unlink()


def link_and_run(gate, plan, name, obj):
    binary = gate.check.folder / (name + '.native')
    gate.run(name + '-link', ['cc', '-std=c11', '-Wall', '-Wextra', '-Werror',
             ROOT / plan['native_driver']['path'], obj, '-o', binary])
    result = gate.run(name + '-run', [binary])
    if result.stdout != b'7\n':
        raise ValueError(f'{name}: native result differs from frozen seven-newline')
    binary.unlink()


def independent(gate, plan, tool, libraries):
    for case in plan['accepted']:
        name = case['name']
        obj = gate.check.folder / (name + '.rust.o')
        result = gate.run(name + '-rust-client', [*rust_command(tool), '--crate-type=lib',
                          '--crate-name', 'client', '--emit=obj', '--extern',
                          f"base={libraries[case['provider']]}",
                          ROOT / plan['clients'][case['client']]['path'], '-o', obj],
                          stderr=None, environment=plan['independent']['environment'])
        diagnostics = [json.loads(line) for line in result.stderr.splitlines()]
        warnings = [row for row in diagnostics if row.get('level') == 'warning' and row.get('code')]
        if [row['code']['code'] for row in warnings] != case['rustc_warning_codes']:
            raise ValueError(f'{name}: independent warning contract disagrees')
        if warnings and not any(child.get('message') == 'keep result'
                                for row in warnings for child in row.get('children', [])):
            raise ValueError(f'{name}: independent warning lost the actual provider reason')
        link_and_run(gate, plan, name + '-rust', obj)
        obj.unlink()
    for case in plan['rejected']:
        output = gate.check.folder / (case['name'] + '.rejected.rmeta')
        result = gate.run(case['name'] + '-rust-rejected', [*rust_command(tool),
                          '--crate-type=lib', '--crate-name', 'client', '--emit=metadata',
                          '--extern', f"base={libraries['marked']}",
                          ROOT / plan['clients'][case['client']]['path'], '-o', output],
                          1, stderr=None, environment=plan['independent']['environment'])
        errors = [json.loads(line) for line in result.stderr.splitlines()]
        codes = [row['code']['code'] for row in errors
                 if row.get('level') == 'error' and row.get('code')]
        if case['rustc_required_error'] not in codes or result.stdout or output.exists():
            raise ValueError(f"{case['name']}: independent rejection differs from its lint contract")
    for library_path in libraries.values():
        library_path.unlink()


def qualify(app, folder, native, seconds):
    plan = json.loads(PLAN.read_text())
    validate_inputs(plan)
    plan_sha = sha(PLAN)
    gate = Gate(app, folder, seconds)
    binary_sha = sha(gate.check.app)
    tools = backend.load_tools(('cppgm++',)) if native else {}
    declaration = {'plan_sha256': plan_sha, 'cli_sha256': binary_sha, 'native': native,
                   'seconds': seconds, 'prebuilt_backend': tools,
                   'native_stdout': '7\n', 'accepted_cases': len(plan['accepted'])}
    (gate.check.folder / 'declaration.json').write_text(json.dumps(declaration, indent=2) + '\n')
    tool, libraries = independent_providers(gate, plan) if native else (None, {})
    candidate(gate, plan, native)
    if native:
        independent(gate, plan, tool, libraries)
    validate_inputs(plan)
    if (sha(PLAN) != plan_sha or sha(gate.check.app) != binary_sha
            or (native and sha(tool) != plan['independent']['rustc_sha256'])):
        raise ValueError('qualification input changed during observation')
    (gate.check.folder / 'complete.json').write_text(json.dumps({
        **declaration, 'state': 'passed', 'runs_sha256': sha(gate.check.folder / 'runs.json'),
        'metadata': {path.name: {'bytes': path.stat().st_size, 'sha256': sha(path)}
                     for path in sorted(gate.check.folder.glob('*.rgm'))},
        'cost_scope': 'serialized metadata byte sizes and command times; no heap allocation/capacity claim',
    }, indent=2) + '\n')
    print(f"Qualified {len(plan['accepted'])} actual CLI source-contract cases; native={native}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', required=True, type=Path)
    parser.add_argument('--evidence-root', required=True, type=Path)
    parser.add_argument('--native', action='store_true')
    parser.add_argument('--seconds', type=float, default=240)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 240:
        parser.error('seconds must be between one and 240')
    try:
        qualify(args.app, args.evidence_root, args.native, args.seconds)
    except (OSError, ValueError, TimeoutError, subprocess.TimeoutExpired) as error:
        args.evidence_root.mkdir(parents=True, exist_ok=True)
        (args.evidence_root / 'failure.json').write_text(json.dumps({
            'state': 'failed', 'error': str(error), 'kind': type(error).__name__,
        }, indent=2) + '\n')
        raise


if __name__ == '__main__':
    main()
