#!/usr/bin/env python3
"""Check logical discriminant result types in a separately compiled client."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_location_role_transport import role_address
from check_source_constants import COMMON, ROOT
from crate_qualification import Check

FIXTURE = ROOT / 'pa31/tests/spec/source-discriminant-transport'
IDENTITY = 'base@discriminant'
OPTIONS = [*COMMON, '--emit-lowir', '--cap-lints=allow', '--crate-disambiguator', 'discriminant']


def opponents(check, tools, paths, contract):
    rustc, exporter, importer = tools
    source, client, artifact = paths
    for name, diagnostic in [('denied', [contract['explicit_impl']]), ('wrong-type', [])]:
        opponent = FIXTURE / contract[name]
        check.run(f'rustc-{name}', [rustc, '--edition=2024', '--crate-type=lib',
                  '--emit=metadata', '--extern', f'base={check.folder / "libbase.rlib"}',
                  '-o', check.folder / f'{name}.rmeta', opponent], 1, timeout=30,
                  environment={'RUSTC_BOOTSTRAP': '1'})
        check.run(f'rustgm-{name}', [*importer, '--extern', f'base={artifact}', opponent],
                  1, diagnostic, timeout=30)
    old = check.folder / 'old.rgcrate'
    check.run('old-profile', [*exporter, '--source-profile', 'module-bodies-v13',
              '--metadata-output', old, source], 1, timeout=30)
    if old.exists():
        raise ValueError('older profile published the discriminant family')
    packet = artifact.read_bytes()
    trait = role_address(packet, 255, 'discriminant_kind')
    member = role_address(packet, 255, 'discriminant_type')
    forged = check.folder / 'wrong-role.rgcrate'
    forged.write_bytes(packet[:member] + packet[trait:trait + 4] + packet[member + 4:])
    check.run('wrong-role', [*importer, '--extern', f'base={forged}', client],
              1, [contract['forged']], timeout=30)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rustc', type=Path, required=True)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / 'outcomes.json').read_text())
    check = Check(args.app, args.out)
    source, client = FIXTURE / contract['provider'], FIXTURE / contract['client']
    observer = check.folder / 'observer.o'
    check.run('cc', ['cc', '-c', FIXTURE / contract['observer'], '-o', observer], timeout=30)
    rustc = args.rustc.resolve(strict=True)
    expected = contract['stdout'].encode()
    pinned(check, rustc, source, client, observer, expected)
    exporter = [check.app, *OPTIONS, '--crate-name', 'base', '--source-core', IDENTITY]
    artifact, provider, consumer = (check.folder / name for name in
                                    ('base.rgcrate', 'provider.lowir', 'client.lowir'))
    check.run('rustgm-provider', [*exporter, '--source-profile', 'module-body-operations-v14',
              '--metadata-output', artifact, '-o', provider, source], timeout=30)
    importer = [check.app, *OPTIONS, '--crate-name', 'client', '--source-client', IDENTITY]
    check.run('rustgm-client', [*importer, '--extern', f'base={artifact}', '-o', consumer, client],
              timeout=30)
    program = check.folder / 'observed'
    backend.build_many([provider, consumer], program, native_objects=[observer])
    if check.run('rustgm-observe', [program], timeout=30).stdout != expected:
        raise ValueError('fresh RustGM client changed the logical discriminant types')
    opponents(check, (rustc, exporter, importer), (source, client, artifact), contract)
    print(expected.decode(), end='')


if __name__ == '__main__':
    main()
