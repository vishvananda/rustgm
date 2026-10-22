#!/usr/bin/env python3
"""Qualify separately compiled static storage using frozen original sources."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

import backend

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'pa30/tests/storage-metadata-outcomes.json'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Check:
    def __init__(self, app, folder, assignment):
        self.app = app.resolve()
        self.assignment = assignment
        self.folder = folder.resolve()
        self.folder.mkdir(parents=True, exist_ok=False)
        self.rows = []

    def run(self, name, command, expected=0):
        started = time.monotonic()
        output = subprocess.run([str(arg) for arg in command], capture_output=True, timeout=120)
        stdout = self.folder / (name + '.stdout')
        stderr = self.folder / (name + '.stderr')
        stdout.write_bytes(output.stdout)
        stderr.write_bytes(output.stderr)
        self.rows.append({'name': name, 'command': [str(a) for a in command],
                          'status': output.returncode, 'seconds': time.monotonic() - started,
                          'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr)})
        (self.folder / 'runs.json').write_text(json.dumps(self.rows, indent=2) + '\n')
        if output.returncode != expected:
            raise ValueError(f'{name}: expected {expected}, got {output.returncode}\n'
                             + output.stderr.decode(errors='replace'))
        return output

    def frontend(self, case, libraries, *, reverse=False):
        name = case['name'] + ('-reverse' if reverse else '')
        command = [self.app, '--emit-lowir', self.assignment / case['path']]
        entries = list(libraries.items())
        if reverse:
            entries.reverse()
        for alias, path in entries:
            command += ['--extern', f'{alias}={path}']
        if case['role'] == 'library':
            command += ['--crate-type=lib', '--crate-name', case['name'],
                        '--crate-disambiguator', case['name'], '--metadata-output',
                        self.folder / (name + '.rgm')]
        output = self.run(name + '-frontend', command, case['frontend'])
        if case['frontend'] == 0 and output.stderr:
            raise ValueError(f'{name}: unexpected frontend diagnostic')
        if case['diagnostic'] and case['diagnostic'].encode() not in output.stderr:
            raise ValueError(f'{name}: did not diagnose the owning semantic rule')
        lowir = self.folder / (name + '.lowir')
        lowir.write_bytes(output.stdout)
        return lowir


def rust_command(libraries):
    command = ['rustc', '+1.98.0', '--edition=2024', '--target=x86_64-unknown-linux-gnu',
               '-C', 'panic=abort', '-C', 'overflow-checks=yes']
    for name, path in libraries.items():
        command += ['--extern', f'{name}={path}']
    return command


def qualify(app, folder, native, manifest):
    plan = json.loads(manifest.read_text())
    assignment = manifest.resolve().parent.parent
    check = Check(app, folder, assignment)
    sources = [assignment / row['path'] for row in plan['cases']]
    for case, source in zip(plan['cases'], sources, strict=True):
        if sha(source) != case['source_sha256']:
            raise ValueError(f'changed declared source: {source}')
    provider = plan.get('provider')
    if provider and sha(assignment / provider['path']) != provider['sha256']:
        raise ValueError('changed declared C provider')
    version = check.run('rust-version', ['rustc', '+1.98.0', '--version']).stdout
    if not version.startswith(b'rustc 1.98.0 '):
        raise ValueError('unexpected independent toolchain')
    tools = backend.load_tools(('cppgm++',)) if native else None
    provider_objects = []
    if native and provider:
        obj = check.folder / 'provider.o'
        check.run('provider', ['cc', '-c', assignment / provider['path'], '-o', obj])
        provider_objects.append(obj)
    libraries, rust_libraries, objects, metadata = {}, {}, [], {}
    for case, source in zip(plan['cases'], sources, strict=True):
        name = case['name']
        library = case['role'] == 'library'
        command = rust_command(rust_libraries)
        if library:
            independent = check.folder / ('lib' + name + '.rlib')
            command += ['--crate-type=rlib', '--crate-name', name]
        else:
            independent = check.folder / (name + '.rustc.o')
            command += ['--crate-name=client', '--emit=obj', '-L',
                        f'dependency={check.folder}']
        result = check.run(name + '-rustc', command + [source, '-o', independent], case['rustc'])
        if any(code.encode() not in result.stderr for code in case['rustc_codes']):
            raise ValueError(f'{name}: independent rejection has a different cause')
        lowir = check.frontend(case, libraries)
        if case['frontend'] == 0:
            repeated = check.frontend(case, libraries, reverse=True)
            if lowir.read_bytes() != repeated.read_bytes():
                raise ValueError(f'{name}: import order or repeated publication changed LowIR')
        if library:
            artifact = check.folder / (name + '.rgm')
            if artifact.read_bytes() != (check.folder / (name + '-reverse.rgm')).read_bytes():
                raise ValueError(f'{name}: repeated metadata publication changed bytes')
            libraries[name] = artifact
            rust_libraries[name] = independent
            metadata[name] = sha(artifact)
        if native and (library or case['native']):
            obj = check.folder / (name + '.o')
            backend.emit_object(lowir, obj, tools,
                                run=lambda args: check.run(name + '-object', args))
            if library:
                objects.append(obj)
            else:
                executable = check.folder / (name + '.native')
                rust_executable = check.folder / (name + '.rustc')
                check.run(name + '-link', ['cc', *objects, obj, *provider_objects, '-lm', '-o', executable])
                command = rust_command(rust_libraries)
                for provider_object in provider_objects:
                    command += ['-C', f'link-arg={provider_object}']
                check.run(name + '-rust-link', command + ['--crate-name=client', '-L',
                    f'dependency={check.folder}', source, '-o', rust_executable])
                for label, binary in [('native', executable), ('rustc', rust_executable)]:
                    output = check.run(name + '-run-' + label, [binary])
                    if output.stdout or output.stderr:
                        raise ValueError(f'{name}: unexpected {label} runtime output')
    (check.folder / 'complete.json').write_text(json.dumps({
        'manifest_sha256': sha(manifest), 'candidate_sha256': sha(check.app),
        'cases': len(plan['cases']), 'native': sum(c['native'] for c in plan['cases']) if native else 0,
        'metadata': metadata, 'runs_sha256': sha(check.folder / 'runs.json'),
    }, indent=2) + '\n')
    print(f'PASS: {len(plan["cases"])} storage metadata cases, native={native}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--evidence-root', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    parser.add_argument('--manifest', type=Path, default=MANIFEST)
    args = parser.parse_args()
    qualify(args.app, args.evidence_root, args.native, args.manifest)


if __name__ == '__main__':
    main()
