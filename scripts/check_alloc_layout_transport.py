#!/usr/bin/env python3
"""Check the separately imported allocation layout against pinned rustc."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, ROOT
from crate_qualification import Check

FIXTURE = ROOT / 'pa31/tests/spec/source-alloc-layout-transport'
IDENTITY = 'base@alloc_layout'
OPTIONS = [*COMMON, '--emit-lowir', '--cap-lints=allow', '--crate-disambiguator', 'alloc_layout']
PROFILE = 'module-body-operations-v14'


def provider(check, contract):
    start = contract['layout_start'].encode()
    _, anchor, fields = (ROOT / contract['layout']).read_bytes().partition(start)
    text = (FIXTURE / contract['provider']).read_bytes()
    marker = contract['layout_marker'].encode()
    if not anchor or text.count(marker) != 1:
        raise ValueError('allocation layout fixture lost its original field declaration')
    source = check.folder / 'provider.rs'
    source.write_bytes(text.replace(marker, anchor + fields))
    return source


def opponents(check, commands, paths, contract):
    exporter, importer = commands
    source, client, artifact = paths
    old = check.folder / 'old.rgcrate'
    check.run('old-profile', [*exporter, '--source-profile', 'module-bodies-v13',
              '--metadata-output', old, source], 1, [contract['old_profile']], timeout=30)
    wrong = check.folder / 'wrong-shape.rs'
    text = source.read_bytes()
    original = b'_Align1Shl0 = 1,'
    if text.count(original) != 1:
        raise ValueError('allocation layout fixture lost its first alignment variant')
    wrong.write_bytes(text.replace(original, b'_Align1Shl0 = 3,'))
    unsupported = check.folder / 'wrong-shape.rgcrate'
    ordinary = ['--emit-semantics' if option == '--emit-lowir' else option
                for option in exporter if option != '--crate-type=lib']
    check.run('wrong-shape-source', [*ordinary, wrong], timeout=30)
    check.run('wrong-shape-export', [*exporter, '--source-profile', PROFILE,
              '--metadata-output', unsupported, wrong], 1, [contract['shape_error']], timeout=30)
    if old.exists() or unsupported.exists():
        raise ValueError('unsupported allocation layout published a source artifact')
    packet = artifact.read_bytes()
    offset = role_address(packet, 255, 'alloc_layout') - 4
    forged = check.folder / 'foreign-layout.rgcrate'
    forged.write_bytes(packet[:offset] + (1).to_bytes(4, 'little') + packet[offset + 4:])
    check.run('foreign-role', [*importer, '--extern', f'base={forged}', client],
              1, [contract['forged']], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rustc', type=Path, required=True)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / 'outcomes.json').read_text())
    check = Check(args.app, args.out)
    source, client = provider(check, contract), FIXTURE / contract['client']
    observer = check.folder / 'observer.o'
    check.run('cc', ['cc', '-c', FIXTURE / contract['observer'], '-o', observer], timeout=30)
    expected = contract['stdout'].encode()
    pinned(check, args.rustc.resolve(strict=True), source, client, observer, expected)
    exporter = [check.app, *OPTIONS, '--crate-name', 'base', '--source-core', IDENTITY]
    artifact, exported, consumer = (check.folder / name for name in
                                    ('base.rgcrate', 'provider.lowir', 'client.lowir'))
    check.run('rustgm-provider', [*exporter, '--source-profile', PROFILE,
              '--metadata-output', artifact, '-o', exported, source], timeout=30)
    importer = [check.app, *OPTIONS, '--crate-name', 'client', '--source-client', IDENTITY]
    check.run('rustgm-client', [*importer, '--extern', f'base={artifact}', '-o', consumer, client],
              timeout=30)
    program = check.folder / 'observed'
    backend.build_many([exported, consumer], program, native_objects=[observer])
    if check.run('rustgm-observe', [program], timeout=30).stdout != expected:
        raise ValueError('fresh RustGM client changed the allocation layout')
    opponents(check, (exporter, importer), (source, client, artifact), contract)
    print(expected.decode(), end='')


if __name__ == '__main__':
    main()
