#!/usr/bin/env python3
"""Freeze and compile one strict PA45 source graph without metadata artifacts."""
import argparse
import json
from pathlib import Path, PurePosixPath
import sys

import backend
from host_inputs import fingerprint, identifier
from host_process import digest
import native_links
from package_snapshot import inventory
from stage1_run import artifact, measure
from target_units import EDITIONS, ENVIRONMENT, bringup_record

FORMAT = 'strict-source-graph/1'
OUTPUTS = {'lowir': 'program.lowir', 'native_links': 'program.native-links.json'}
PROFILE = ('--emit-lowir', '--cap-lints=allow', '--edition=2024', '--crate-name', 'core',
           '--source-contract-attempted-bytes=4294967295',
           '--source-contract-retained-bytes=67108864')
COMPILER_ENVIRONMENT = {**ENVIRONMENT, 'STD_ENV_ARCH': 'x86_64'}
SUPPORT = ('backend.py', 'host_inputs.py', 'host_process.py', 'native_links.py',
           'package_snapshot.py', 'stage1_run.py', 'target_units.py')


def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def source_path(path):
    relative = PurePosixPath(path)
    if (relative.is_absolute() or not relative.parts or '..' in relative.parts
            or relative.as_posix() != path or relative.suffix != '.rs'):
        raise ValueError('source graph requires canonical relative Rust paths')
    return path


def cfg(arguments, flag, name, settings):
    if settings is None and name is not None:
        return
    if not isinstance(settings, list) or any(
            not isinstance(setting, str) or not setting.strip() or '\0' in setting
            for setting in settings):
        raise ValueError('invalid source graph cfg settings')
    if name is not None:
        arguments.extend([flag, name])
    for setting in settings:
        arguments.extend([flag, setting if name is None else f'{name}={setting}'])


def source_arguments(configuration):
    """Build the strict CLI from an ordered crate plan with explicit cfg resets."""
    core = configuration['core']
    disambiguator = core['disambiguator']
    if identifier(disambiguator) != disambiguator:
        raise ValueError('invalid core disambiguator')
    roots = [source_path(core['path'])]
    arguments = [*PROFILE, '--crate-disambiguator', disambiguator,
                 '--source-core', f'core@{disambiguator}']
    cfg(arguments, '--cfg', None, core['cfg'])
    guests = configuration['guests']
    if not isinstance(guests, list) or not guests or guests[-1]['name'] != 'app':
        raise ValueError('the source graph must end with the compiler application')
    names = {'core'}
    for guest in guests:
        name, edition = guest['name'], guest['edition']
        if identifier(name) != name or name in names or edition not in EDITIONS:
            raise ValueError('invalid or repeated source guest')
        names.add(name)
        path = source_path(guest['path'])
        roots.append(path)
        arguments.extend(['--source-guest', f'{name}@{edition}={path}'])
    for guest in guests:
        cfg(arguments, '--source-guest-cfg', guest['name'], guest.get('cfg'))
    return arguments, roots


def tool_pins(linker):
    """Identify the actual reviewed object backend, its lock and the host linker."""
    cppgm = Path(backend.load_tools(('cppgm',))['cppgm']).resolve(strict=True)
    lock = backend.ROOT / 'backend.lock.json'
    return {'cppgm': digest(cppgm), 'linker': digest(linker), 'backend_lock': digest(lock)}


def link_pins(tools):
    return {'cppgm': tools['cppgm']['sha256'], 'linker': tools['linker']['sha256'],
            'backend_lock': tools['backend_lock_sha256']}


def binary_unit(graph):
    # Core is first, followed by each guest in its declared order.
    return len(graph['configuration']['guests'])


def freeze(configuration, sources, stage0, linker, plan):
    sources = Path(sources).resolve(strict=True)
    plan = Path(plan).resolve()
    if plan.is_relative_to(sources):
        raise ValueError('freeze the plan outside its source inventory')
    _, roots = source_arguments(configuration)
    if any(not (sources / root).is_file() for root in roots):
        raise ValueError('source graph is missing a crate root')
    result = {'format': FORMAT, 'configuration': configuration,
              'source_sha256': fingerprint(inventory(sources)),
              'stage0_sha256': digest(Path(stage0).resolve(strict=True)),
              'tools': tool_pins(Path(linker).resolve(strict=True))}
    plan.parent.mkdir(parents=True, exist_ok=True)
    with plan.open('x') as out:
        out.write(json.dumps(result, indent=2, sort_keys=True) + '\n')
    return result


def strict_diagnostics(folder):
    for name in ('compile.stdout', 'compile.stderr'):
        if bringup_record(folder / name) is not None:
            raise ValueError('strict source graph reports a bring-up record')


def stage_artifacts(folder, receipt, *, verify=False):
    outputs = receipt['outputs']
    if outputs.keys() != OUTPUTS.keys():
        raise ValueError('strict stage requires exactly its LowIR and native links')
    for name, filename in OUTPUTS.items():
        path = folder / filename
        row = outputs[name]
        if path.is_symlink() or not path.is_file() or row['path'] != str(path):
            raise ValueError(f'strict stage has an invalid {name} artifact')
        if path.stat().st_size <= 0 or path.stat().st_size != row['bytes']:
            raise ValueError(f'strict stage has an empty or changed {name} artifact')
        if verify and digest(path) != row['sha256']:
            raise ValueError(f'strict stage {name} differs from its build receipt')
    return outputs


def verify_stage(folder, receipt):
    if (receipt['format'] != FORMAT or receipt['state'] != 'complete'
            or receipt['profile'] != 'strict' or type(receipt['bringup_records']) is not int
            or receipt['bringup_records'] != 0):
        raise ValueError('source stage requires a complete strict build with zero bring-up records')
    graph = receipt['graph']
    arguments, roots = source_arguments(graph['configuration'])
    command = receipt['command']
    expected = command_line(receipt['rustgm']['path'], arguments, roots[0], folder)
    if (graph['format'] != FORMAT or receipt['environment'] != COMPILER_ENVIRONMENT
            or command['status'] != 0 or command['timed_out']
            or command['signal'] is not None or command['argv'] != expected):
        raise ValueError('strict source stage command differs from its frozen graph')
    strict_diagnostics(folder)
    return stage_artifacts(folder, receipt)


def command_line(compiler, arguments, primary, folder):
    return [str(compiler), *arguments,
            '--native-links-output', str(folder / OUTPUTS['native_links']),
            '-o', str(folder / OUTPUTS['lowir']), primary]


def build(plan, sources, compiler, compiler_sha256, folder, timeout):
    plan = Path(plan).resolve(strict=True)
    sources = Path(sources).resolve(strict=True)
    compiler = Path(compiler).resolve(strict=True)
    folder = Path(folder).resolve()
    if folder.is_relative_to(sources) or timeout <= 0:
        raise ValueError('strict stage requires a separate output directory and positive timeout')
    text = plan.read_bytes()
    graph = json.loads(text)
    arguments, roots = source_arguments(graph['configuration'])
    if graph['format'] != FORMAT or fingerprint(inventory(sources)) != graph['source_sha256']:
        raise ValueError('source inventory differs from its frozen graph')
    used = artifact(compiler)
    if used['sha256'] != compiler_sha256:
        raise ValueError('stage compiler differs from its declared pin')
    files = (Path(__file__), *(Path(__file__).with_name(name) for name in SUPPORT))
    implementation = {path.name: digest(path) for path in files}
    folder.mkdir(parents=True)
    path = folder / 'receipt.json'
    receipt = {'format': FORMAT, 'state': 'running', 'profile': 'strict',
               'graph': graph, 'rustgm': used, 'environment': COMPILER_ENVIRONMENT,
               'implementation_sha256': implementation}
    save(path, receipt)
    try:
        receipt['command'] = measure(
            command_line(compiler, arguments, roots[0], folder), cwd=sources,
            stdout=folder / 'compile.stdout', stderr=folder / 'compile.stderr',
            limit=timeout, environment=COMPILER_ENVIRONMENT,
        )
        strict_diagnostics(folder)
        if receipt['command']['status'] != 0 or receipt['command']['timed_out']:
            raise ValueError('strict compiler failed; see compile.stderr and the command receipt')
        native_links.load(folder / OUTPUTS['native_links'])
        receipt['outputs'] = {name: artifact(folder / filename)
                              for name, filename in OUTPUTS.items()}
        stage_artifacts(folder, receipt)
        if (artifact(compiler) != used or plan.read_bytes() != text
                or fingerprint(inventory(sources)) != graph['source_sha256']
                or any(digest(file) != implementation[file.name] for file in files)):
            raise ValueError('strict stage inputs changed during compilation')
        receipt.update(state='complete', bringup_records=0)
    except (OSError, ValueError) as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        save(path, receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    freezer = commands.add_parser('freeze')
    freezer.add_argument('configuration', type=Path)
    freezer.add_argument('--sources', required=True, type=Path)
    freezer.add_argument('--stage0', required=True, type=Path)
    freezer.add_argument('--linker', required=True, type=Path)
    freezer.add_argument('--out', required=True, type=Path)
    builder = commands.add_parser('build')
    builder.add_argument('plan', type=Path)
    builder.add_argument('--sources', required=True, type=Path)
    builder.add_argument('--compiler', required=True, type=Path)
    builder.add_argument('--compiler-sha256', required=True)
    builder.add_argument('--out', required=True, type=Path)
    builder.add_argument('--timeout', type=int, default=1800)
    args = parser.parse_args()
    try:
        if args.action == 'freeze':
            freeze(json.loads(args.configuration.read_text()), args.sources,
                   args.stage0, args.linker, args.out)
        else:
            receipt = build(args.plan, args.sources, args.compiler, args.compiler_sha256,
                            args.out, args.timeout)
            print(json.dumps(receipt['outputs'], indent=2))
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'strict self-hosting {args.action} failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
