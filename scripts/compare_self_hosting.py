#!/usr/bin/env python3
"""Compare two complete strict target builds and their linked RustGM executables."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import build_self_hosting as whole
from host_process import digest
from target_units import bringup_record

CHUNK_BYTES = 1024 * 1024
UNIT_IDENTITY = ('unit', 'package', 'crate_name', 'disambiguator', 'library',
                 'dependencies', 'source', 'environment')
REQUIRED = {'lowir', 'metadata'}


def stage_receipt(folder):
    content = (folder / 'receipt.json').read_bytes()
    receipt = json.loads(content)
    if receipt.get('format') == whole.FORMAT:
        whole.verify_stage(folder, receipt)
        return receipt, hashlib.sha256(content).hexdigest()
    if (receipt['state'] != 'complete' or receipt['profile'] != 'strict'
            or type(receipt['bringup_records']) is not int or receipt['bringup_records'] != 0
            or receipt['failure'] is not None or receipt['blocked'] or receipt['not_run']):
        raise ValueError(f'{folder} requires a complete strict build with zero bring-up records')
    steps = receipt['steps']
    completed = receipt['completed']
    units = [step['unit'] for step in steps]
    if (not units or len(units) != len(set(units))
            or [(step['unit'], step['crate_name']) for step in steps]
            != [(row['unit'], row['crate']) for row in completed]):
        raise ValueError(f'{folder} has inconsistent planned and completed units')
    return receipt, hashlib.sha256(content).hexdigest()


def unit_artifacts(folder, step, completed):
    unit = folder / f"unit-{step['unit']}"
    planned = dict(step['outputs'])
    artifacts = completed['outputs']
    if (len(planned) != len(step['outputs']) or set(planned) != set(artifacts)
            or not REQUIRED <= set(artifacts)):
        raise ValueError(f"{step['crate_name']} requires all planned artifacts, including LowIR and metadata")
    record = bringup_record(unit / 'compile.stderr')
    if record is not None:
        raise ValueError(f"{step['crate_name']} has a bring-up record in {folder}")
    for name, artifact in artifacts.items():
        path = Path(artifact['path'])
        if (planned[name] != str(path) or path.parent != unit or path.is_symlink()
                or not path.is_file() or path.resolve(strict=True).parent != unit):
            raise ValueError(f"{step['crate_name']} has an invalid {name} artifact in {folder}")
    return artifacts


def compare_files(left, right, label):
    if left.samefile(right):
        raise ValueError(f'{label} requires independently built outputs')
    hasher = hashlib.sha256()
    size = 0
    with left.open('rb') as first, right.open('rb') as second:
        while True:
            a, b = first.read(CHUNK_BYTES), second.read(CHUNK_BYTES)
            if a != b:
                raise ValueError(f'{label} differs between stages')
            if not a:
                break
            hasher.update(a)
            size += len(a)
    if not size:
        raise ValueError(f'{label} is empty')
    return {'bytes': size, 'sha256': hasher.hexdigest()}


def argument_paths(receipt):
    return {path: (step['unit'], name)
            for step in receipt['steps'] for name, path in step['outputs']}


def compiler_arguments(step, paths):
    """Replace only executor-owned output and dependency paths with unit coordinates."""
    previous = None
    for argument in step['arguments']:
        if previous == '--extern':
            alias, separator, path = argument.partition('=')
            if separator and path in paths:
                yield alias, paths[path]
                previous = argument
                continue
        if previous in ('-o', '--metadata-output', '--native-links-output', '--dependency-metadata'):
            yield paths.get(argument, argument)
        else:
            yield argument
        previous = argument


def link_receipt(folder, target_sha256, binary, artifact, steps, *, graph_row=None):
    """Bind the compared executable to this strict build's completed link."""
    receipt_path = binary.parent / 'receipt.json'
    receipt = json.loads(receipt_path.read_bytes())
    if (receipt['state'] != 'complete' or receipt['target_receipt'] != str(folder / 'receipt.json')
            or receipt['target_sha256'] != target_sha256):
        raise ValueError(f'{binary} requires its completed link receipt for {folder}')
    executable = {'path': str(binary), **artifact}
    if receipt['linked_compiler'] != executable:
        raise ValueError(f'{binary} differs from its native-link receipt')
    units = receipt['units']
    if not units:
        raise ValueError(f'{binary} link receipt omits its binary target')
    selected = units[0]
    if graph_row is not None and units != [graph_row]:
        raise ValueError(f'{binary} does not link the recorded whole source graph')
    if not any(step['unit'] == selected['unit'] and step['crate_name'] == selected['crate']
               and step['library'] is False for step in steps):
        raise ValueError(f'{binary} link receipt omits its binary target')
    return selected['unit'], receipt['tools'], receipt['implementation_sha256']


def compare_whole_graph(stages, receipts, receipt_hashes, binaries):
    first, second = receipts
    graph = first['graph']
    if first['rustgm']['sha256'] != graph['stage0_sha256']:
        raise ValueError('stage 1 did not use the frozen stage 0 compiler')
    linked = compare_files(*binaries, 'linked RustGM compiler')
    if linked['sha256'] != second['rustgm']['sha256']:
        raise ValueError('stage 2 did not use the linked stage 1 compiler')
    outputs = {}
    for name in whole.OUTPUTS:
        entries = [receipt['outputs'][name] for receipt in receipts]
        result = compare_files(*(Path(row['path']) for row in entries), name)
        if any(row[key] != value for row in entries for key, value in result.items()):
            raise ValueError(f'{name} differs from its build receipt')
        outputs[name] = result
    selected = whole.binary_unit(graph)
    steps = [{'unit': selected, 'crate_name': 'app', 'library': False}]
    links = []
    for folder, sha256, binary, receipt in zip(stages, receipt_hashes, binaries, receipts, strict=True):
        if digest(folder / 'receipt.json') != sha256:
            raise ValueError('a strict target receipt changed during comparison')
        graph_row = {'unit': selected, 'crate': 'app', 'artifacts': receipt['outputs']}
        link = link_receipt(folder, sha256, binary, linked, steps, graph_row=graph_row)
        if whole.link_pins(link[1]) != graph['tools']:
            raise ValueError('stage link tools differ from the frozen source graph')
        links.append(link)
    if links[0] != links[1]:
        raise ValueError('the stages used different native link tools or implementations')
    return {'outputs': outputs, 'linked_compiler': linked, 'bringup_records': 0}


def compare(stage1, stage2, linked1, linked2):
    stage1, stage2 = (Path(folder).resolve(strict=True) for folder in (stage1, stage2))
    linked1, linked2 = (Path(path).resolve(strict=True) for path in (linked1, linked2))
    if stage1 == stage2:
        raise ValueError('self-hosting comparison requires independently built outputs')
    first, first_sha256 = stage_receipt(stage1)
    second, second_sha256 = stage_receipt(stage2)
    if first['graph'] != second['graph']:
        raise ValueError('the stages used different frozen build graphs')
    if first['implementation_sha256'] != second['implementation_sha256']:
        raise ValueError('the stages used different target-unit executors')
    graph = first.get('format') == whole.FORMAT
    if graph != (second.get('format') == whole.FORMAT):
        raise ValueError('the stages used different kinds of target build')
    if graph:
        return compare_whole_graph((stage1, stage2), (first, second),
                                   (first_sha256, second_sha256), (linked1, linked2))
    input_hash = digest(linked1)
    if input_hash != second['rustgm']['sha256']:
        raise ValueError('stage 2 did not use the linked stage 1 compiler')
    if len(first['steps']) != len(second['steps']):
        raise ValueError('the stages built different numbers of target units')
    paths1, paths2 = argument_paths(first), argument_paths(second)
    units = []
    for left, right, left_row, right_row in zip(
            first['steps'], second['steps'], first['completed'], second['completed'], strict=True):
        if any(left[key] != right[key] for key in UNIT_IDENTITY):
            raise ValueError(f"stage unit identity differs at {left['crate_name']}")
        artifacts1 = unit_artifacts(stage1, left, left_row)
        artifacts2 = unit_artifacts(stage2, right, right_row)
        if set(artifacts1) != set(artifacts2):
            raise ValueError(f"{left['crate_name']} has different artifact sets")
        if tuple(compiler_arguments(left, paths1)) != tuple(compiler_arguments(right, paths2)):
            raise ValueError(f"stage compiler arguments differ at {left['crate_name']}")
        outputs = {}
        for name, _ in left['outputs']:
            a, b = artifacts1[name], artifacts2[name]
            label = f"{left['crate_name']} {name}"
            result = compare_files(Path(a['path']), Path(b['path']), label)
            if any(item[key] != value for item in (a, b) for key, value in result.items()):
                raise ValueError(f'{label} differs from its build receipt')
            outputs[name] = result
        units.append({'unit': left['unit'], 'crate': left['crate_name'], 'outputs': outputs})
    linked = compare_files(linked1, linked2, 'linked RustGM compiler')
    if linked['sha256'] != input_hash:
        raise ValueError('the linked stage 1 compiler changed during comparison')
    if (first_sha256 != digest(stage1 / 'receipt.json')
            or second_sha256 != digest(stage2 / 'receipt.json')):
        raise ValueError('a strict target receipt changed during comparison')
    original_link = link_receipt(stage1, first_sha256, linked1, linked, first['steps'])
    rebuilt_link = link_receipt(stage2, second_sha256, linked2, linked, second['steps'])
    if original_link != rebuilt_link:
        raise ValueError('the stages linked different binary targets or used different native-link tools')
    return {'units': units, 'linked_compiler': linked, 'bringup_records': 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage1', type=Path, help='stage 1 target output directory')
    parser.add_argument('stage2', type=Path, help='stage 2 target output directory')
    parser.add_argument('--stage1-binary', required=True, type=Path)
    parser.add_argument('--stage2-binary', required=True, type=Path)
    args = parser.parse_args()
    try:
        result = compare(args.stage1, args.stage2, args.stage1_binary, args.stage2_binary)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'self-hosting comparison failed: {error}', file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
