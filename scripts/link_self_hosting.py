#!/usr/bin/env python3
"""Link a completed strict target build through the reviewed CPPGM object driver."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

import backend
import build_self_hosting as whole
from compare_self_hosting import stage_receipt, unit_artifacts
from host_process import DEFAULT_TIMEOUT, CommandLog, digest
import native_links
from stage1_run import measure
from target_units import ENVIRONMENT

REQUIRED = {'lowir', 'metadata', 'native_links'}
SUPPORT = ('backend.py', 'compare_self_hosting.py', 'host_process.py',
           'native_links.py', 'target_units.py')


def closure(receipt, selected):
    """Use the binary's recorded transitive dependencies in frozen target order."""
    by_unit = {step['unit']: (step, completed)
               for step, completed in zip(receipt['steps'], receipt['completed'], strict=True)}
    if selected not in by_unit or by_unit[selected][0]['library'] is not False:
        raise ValueError('native stage link requires its selected binary target unit')
    entry = by_unit[selected][0]
    available = set()
    requested = set(entry['dependencies'])
    if len(requested) != len(entry['dependencies']):
        raise ValueError('binary dependency closure contains a duplicate unit')
    for unit, (step, _) in by_unit.items():
        if unit == selected:
            break
        if unit not in requested:
            continue
        if step['library'] is not True or not set(step['dependencies']) <= available:
            raise ValueError('binary dependency closure lacks its ordered transitive libraries')
        available.add(unit)
    ordered = [unit for unit in by_unit if unit in available]
    if ordered != entry['dependencies']:
        raise ValueError('binary dependency closure differs from frozen target order')
    units = [by_unit[unit] for unit in (selected, *ordered)]
    for step, _ in units:
        expected = {dict(by_unit[unit][0]['outputs'])['metadata'] for unit in step['dependencies']}
        if dependency_paths(step['arguments']) != expected:
            raise ValueError(f"{step['crate_name']} dependency closure differs from compiler inputs")
    return units


def dependency_paths(arguments):
    paths = set()
    for flag, value in zip(arguments, arguments[1:]):
        if flag == '--extern':
            _, separator, path = value.partition('=')
            if not separator or not path:
                raise ValueError('target unit has an invalid extern argument')
            paths.add(path)
        elif flag == '--dependency-metadata':
            paths.add(value)
    return paths


def inputs(folder, units):
    """Read only complete, unchanged artifacts for the selected link."""
    result = []
    for step, completed in units:
        artifacts = unit_artifacts(folder, step, completed)
        if artifacts.keys() != REQUIRED:
            raise ValueError(f"{step['crate_name']} requires its native-link manifest")
        for name, artifact in artifacts.items():
            path = Path(artifact['path'])
            if path.stat().st_size != artifact['bytes'] or digest(path) != artifact['sha256']:
                raise ValueError(f"{step['crate_name']} {name} differs from its target receipt")
        result.append({'unit': step['unit'], 'crate': step['crate_name'], 'artifacts': artifacts})
    return result


def stage_inputs(folder, target, selected):
    if target.get('format') != whole.FORMAT:
        return inputs(folder, closure(target, selected))
    binary = whole.binary_unit(target['graph'])
    if selected is not None and selected != binary:
        raise ValueError('native stage link requires its selected binary target unit')
    outputs = whole.stage_artifacts(folder, target, verify=True)
    return [{'unit': binary, 'crate': 'app', 'artifacts': outputs}]


def link(folder, selected, output, linker, linker_sha256, timeout=DEFAULT_TIMEOUT, *, optimization=0):
    folder = Path(folder).resolve(strict=True)
    output = Path(output).resolve()
    if output.is_relative_to(folder):
        raise ValueError('native stage output must be outside the target receipts')
    linker = Path(linker).resolve(strict=True)
    if digest(linker) != linker_sha256:
        raise ValueError('native linker differs from its declared pin')
    target_receipt = folder / 'receipt.json'
    target, target_sha256 = stage_receipt(folder)
    source_graph = target.get('format') == whole.FORMAT
    verified = stage_inputs(folder, target, selected)
    libraries = native_links.load(verified[0]['artifacts']['native_links']['path'])
    tools = backend.load_tools(('cppgm',))
    cppgm = Path(tools['cppgm']).resolve(strict=True)
    cppgm_sha256 = digest(cppgm)
    lock = backend.ROOT / 'backend.lock.json'
    lock_sha256 = digest(lock)
    sources = {name: Path(backend.__file__).with_name(name) for name in SUPPORT}
    if source_graph:
        for name in ('build_self_hosting.py', 'stage1_run.py'):
            sources[name] = Path(backend.__file__).with_name(name)
    sources[Path(__file__).name] = Path(__file__).resolve()
    implementation = {name: digest(path) for name, path in sources.items()}
    tools = {'cppgm': {'path': str(cppgm), 'sha256': cppgm_sha256},
             'linker': {'path': str(linker), 'sha256': linker_sha256},
             'backend_lock_sha256': lock_sha256, 'object_optimization': optimization}
    if source_graph and whole.link_pins(tools) != target['graph']['tools']:
        raise ValueError('stage link tools differ from the frozen source graph')
    if type(timeout) is not int or timeout <= 0:
        raise ValueError('native stage link requires a positive whole-number timeout')
    if type(optimization) is not int or optimization not in (0, 1):
        raise ValueError('native stage link supports CPPGM optimization levels 0 and 1')
    output.mkdir(parents=True)
    receipt_path = output / 'receipt.json'
    receipt = {
        'state': 'linking', 'target_receipt': str(target_receipt),
        'target_sha256': target_sha256, 'units': verified, 'libraries': libraries,
        'implementation_sha256': implementation,
        'tools': tools,
        'commands': [], 'objects': [],
    }

    def save():
        receipt_path.write_text(json.dumps(receipt, indent=2) + '\n')

    log = CommandLog(output, output, receipt['commands'], save)

    def run(name, command):
        if command[0] == cppgm:
            if command[1:2] != ['-O0']:
                raise ValueError('the object bridge changed its default optimization argument')
            command = [cppgm, f'-O{optimization}', *command[2:]]
        if source_graph:
            row = measure(command, cwd=output, stdout=output / f'{name}.stdout',
                          stderr=output / f'{name}.stderr', limit=timeout, environment=ENVIRONMENT)
            receipt['commands'].append({'name': name, **row})
            save()
            status = row['status'] if row['signal'] is None and not row['timed_out'] else -1
        else:
            status = log.run(name, command, ENVIRONMENT, timeout)
        if status:
            raise ValueError(f'{name} failed with status {status}')

    save()
    try:
        objects = []
        for item in verified:
            unit = item['unit']
            source = Path(item['artifacts']['lowir']['path'])
            object_path = output / f'unit-{unit}.o'
            backend.emit_object(source, object_path, {'cppgm': cppgm},
                                run=lambda command: run(f'unit-{unit}-object', command))
            objects.append(object_path)
            receipt['objects'].append({'unit': unit, 'path': str(object_path),
                                       'sha256': digest(object_path)})
        pending = output / 'rustgm.incomplete'
        run('link', [linker, *objects, *native_links.arguments(libraries, ()), '-lm', '-o', pending])
        if (digest(target_receipt) != target_sha256 or digest(linker) != linker_sha256
                or digest(cppgm) != cppgm_sha256 or digest(lock) != lock_sha256
                or any(digest(path) != implementation[name] for name, path in sources.items())
                or stage_inputs(folder, target, selected) != verified):
            raise ValueError('stage link inputs changed during execution')
        if any(digest(Path(row['path'])) != row['sha256'] for row in receipt['objects']):
            raise ValueError('staged native objects changed during execution')
        linked = output / 'rustgm'
        result = {'path': str(linked), 'bytes': pending.stat().st_size, 'sha256': digest(pending)}
        if not result['bytes']:
            raise ValueError('linked compiler is empty')
        pending.rename(linked)
        receipt.update(state='complete', linked_compiler=result)
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        receipt.update(state='failed', error=str(error))
        raise
    finally:
        save()
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('target', type=Path)
    parser.add_argument('--unit', type=int, help='binary target index for a per-unit build')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--linker', type=Path, required=True)
    parser.add_argument('--linker-sha256', required=True)
    parser.add_argument('--timeout', type=int, default=DEFAULT_TIMEOUT)
    parser.add_argument('--optimization', type=int, choices=(0, 1), default=0,
                        help='CPPGM object optimization level recorded in the link receipt')
    args = parser.parse_args()
    try:
        receipt = link(args.target, args.unit, args.out, args.linker,
                       args.linker_sha256, args.timeout, optimization=args.optimization)
    except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
        print(f'strict native stage link failed: {error}', file=sys.stderr)
        return 1
    print(json.dumps(receipt['linked_compiler'], indent=2))
    return 0


if __name__ == '__main__':
    sys.exit(main())
