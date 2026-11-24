#!/usr/bin/env python3
"""Compare an imported pinned Drop default with separately built pinned rustc."""
import argparse
import json
from pathlib import Path

import backend
from check_caller_location_forwarding import pinned
from check_source_constants import COMMON, ROOT
from crate_qualification import Check

FIXTURE = ROOT / 'pa31/tests/spec/source-drop-default-transport'
IDENTITY = 'base@drop_defaults'
OPTIONS = [*COMMON, '--emit-lowir', '--cap-lints=allow', '--crate-disambiguator', 'drop_defaults']


def section(path, start, end):
    _, first, tail = path.read_bytes().partition(start.encode())
    body, last, _ = tail.partition(end.encode())
    if not first or not last:
        raise ValueError(f'fixture anchors changed in {path}')
    return first + body


def sources(check, contract, policy):
    base = ROOT / contract['foundation']
    prefix, mark, _ = base.read_bytes().partition(contract['foundation_end'].encode())
    deref = ROOT / contract['deref']
    ops, split, _ = deref.read_bytes().partition(contract['deref_end'].encode())
    if not mark or not split or b',pin_ergonomics' not in prefix:
        raise ValueError('fixture foundation or const Deref boundary changed')
    roles = section(base, contract['drop_start'], contract['drop_end'])
    roles = roles.replace(b'pub(crate) const unsafe fn drop_glue', b'pub const unsafe fn drop_glue')
    if policy:
        old = b'rustc_default_body_unstable(feature = "pin_ergonomics"'
        if roles.count(old) != 1:
            raise ValueError('pinned Drop default body policy changed')
        roles = roles.replace(old, b'rustc_default_body_unstable(feature = "drop_body_fixture"')
    source = check.folder / 'provider.rs'
    source.write_bytes(prefix.replace(b',pin_ergonomics', b',const_convert,pin_ergonomics') +
                       ops + b'\n' + (FIXTURE / contract['pin']).read_bytes() + roles +
                       b'\n' + (FIXTURE / contract['provider']).read_bytes())
    client = FIXTURE / contract['client']
    if policy:
        extra = check.folder / 'permitted.rs'
        extra.write_bytes(b'#![feature(drop_body_fixture)]\n' + client.read_bytes())
        client = extra
    return source, client


def native(check, contract, rustc, policy):
    source, client = sources(check, contract, policy)
    observer = check.folder / 'observer.o'
    check.run('cc-observer', ['cc', '-c', FIXTURE / contract['observer'], '-o', observer], timeout=30)
    expected = contract['stdout'].encode()
    pinned(check, rustc, source, client, observer, expected)
    artifact = check.folder / 'base.rgcrate'
    provider, consumer = (check.folder / path for path in ('provider.lowir', 'client.lowir'))
    exporter = [check.app, *OPTIONS, '--crate-name', 'base', '--source-core', IDENTITY]
    check.run('rustgm-provider', [*exporter, '--source-profile', 'module-body-operations-v14',
              '--metadata-output', artifact, '-o', provider, source], timeout=30)
    importer = [check.app, *OPTIONS, '--crate-name', 'client', '--source-client', IDENTITY]
    check.run('rustgm-client', [*importer, '--extern', f'base={artifact}', '-o', consumer, client],
              timeout=30)
    program = check.folder / 'observe'
    backend.build_many([provider, consumer], program, native_objects=[observer])
    if check.run('rustgm-observe', [program], timeout=30).stdout != expected:
        raise ValueError('fresh RustGM client changed the selected destructor effect')
    if policy:
        denied = FIXTURE / contract['client']
        check.run('rustc-omitted-default', [rustc, '--edition=2024', '--crate-type=lib',
                  '--emit=metadata', '--extern', f'base={check.folder / "libbase.rlib"}',
                  '-o', check.folder / 'denied.rmeta', denied], 1, [contract['denied']], timeout=30,
                  environment={'RUSTC_BOOTSTRAP': '1'})
        check.run('rustgm-omitted-default', [*importer, '--extern', f'base={artifact}', denied],
                  1, [contract['denied']], timeout=30)
    else:
        old = check.folder / 'old.rgcrate'
        check.run('old-profile', [*exporter, '--source-profile', 'module-bodies-v13',
                  '--metadata-output', old, source], 1, timeout=30)
        if old.exists():
            raise ValueError('older profile published the conditional Drop default')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rustc', type=Path, required=True)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    contract = json.loads((FIXTURE / 'outcomes.json').read_text())
    rustc = args.rustc.resolve(strict=True)
    native(Check(args.app, args.out / 'forwarding'), contract, rustc, False)
    native(Check(args.app, args.out / 'default-policy'), contract, rustc, True)
    print(contract['stdout'], end='')


if __name__ == '__main__':
    main()
