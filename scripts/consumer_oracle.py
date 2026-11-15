#!/usr/bin/env python3
"""Consumer oracle: what the milestone consumer programs print under pinned rustc.

Each rung of the self-hosting ladder has a directory of small programs and an
outcomes file that declares, per program, its arguments, environment, standard
input, exit status and standard error. This script builds the programs with
rustc 1.98.0 under the course profile (opt-level 0, overflow checks,
panic=abort, std rebuilt from the pinned sources), runs each one several times
in a fresh empty directory and compares its standard output with the recorded
`NAME.stdout` beside the source. A second build at opt-level 2 must print the
same bytes; a difference there means the program depends on something
unspecified. The second build links the toolchain's prebuilt std, except for
the `no_std` rungs: the prebuilt core refers to an unwinding personality that
only std defines, so those rebuild core and alloc again.

`--record` writes the `.stdout` files and the two hashes of each case. It
refuses when a program does not meet its declared status or standard error, so
the declarations come first and the text oracle second. rustc is an oracle
here; nothing this script produces is an input to RustGM's frontend.

Run it through the resource and storage guards:

    scripts/consumer_oracle.py m1 m2 m3 m4
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile

import demand_list

ROOT = demand_list.ROOT
WORK = ROOT / 'obj/parallel/consumers'
RUNGS = {
    'm1': ('pa32', 'tests/core-consumer-outcomes.json'),
    'm2': ('pa33', 'tests/alloc-consumer-outcomes.json'),
    'm3': ('pa36', 'tests/std-consumer-outcomes.json'),
    'm4': ('pa35', 'tests/unicode-consumer-outcomes.json'),
}
NO_STD = {'m1', 'm2'}
REPEATS = 3
TIMEOUT = 60
OPTIMIZED_PROFILE = {'OPT_LEVEL': '2', 'DEBUG': '0', 'DEBUG_ASSERTIONS': 'false',
                     'OVERFLOW_CHECKS': 'true', 'PANIC': 'abort', 'INCREMENTAL': 'false'}


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def load(rung):
    """The rung's outcomes, checked against the programs that exist."""
    assignment, relative = RUNGS[rung]
    path = ROOT / assignment / relative
    outcomes = json.loads(path.read_text())
    programs = ROOT / demand_list.CORPORA[rung]['programs']
    declared = {}
    for case in outcomes['cases']:
        source = ROOT / assignment / case['path']
        if source.parent != programs or source.stem != case['name'] or case['name'] in declared:
            raise SystemExit(f'{path}: case {case["name"]} does not name one program in {programs}')
        declared[case['name']] = case
    present = {source.stem for source in programs.glob('*.rs')}
    if present != set(declared):
        raise SystemExit(f'{path}: declared and present programs differ: '
                         f'{sorted(present ^ set(declared))}')
    return path, outcomes, ROOT / assignment


def build_optimized(rung, pin, library):
    """The same package at opt-level 2; returns a label and the binaries' folder."""
    manifest = demand_list.WORK / 'packages' / rung / 'Cargo.toml'
    environment = dict(os.environ)
    for name in ('RUSTC_BOOTSTRAP', 'RUSTC_WRAPPER', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'):
        environment.pop(name, None)
    environment.update({f'CARGO_PROFILE_RELEASE_{key}': value for key, value in OPTIMIZED_PROFILE.items()})
    environment.update({'CARGO_INCREMENTAL': '0', 'RUSTC': pin['tools']['rustc']['path']})
    command = [pin['tools']['cargo']['path'], 'build', '--release', '--manifest-path', str(manifest),
               '--target', demand_list.TARGET]
    if rung in NO_STD:
        label, target = 'rebuilt core and alloc, opt-level 2', WORK / 'target-rebuilt'
        environment.update({'RUSTC_BOOTSTRAP': '1', '__CARGO_TESTS_ONLY_SRC_ROOT': str(library)})
        command += demand_list.BUILD_STD
    else:
        label, target = 'prebuilt std, opt-level 2', WORK / 'target-prebuilt'
    environment['CARGO_TARGET_DIR'] = str(target)
    log = WORK / 'logs' / f'{rung}.optimized.log'
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open('w') as stream:
        status = subprocess.run(command, env=environment, cwd=ROOT, stdout=stream,
                                stderr=subprocess.STDOUT, check=False).returncode
    if status:
        raise SystemExit(f'{rung}: {label} build failed with status {status}; see {log}')
    return label, target / demand_list.TARGET / 'release'


def execute(binary, case, scratch):
    """One run in a fresh empty directory with exactly the declared inputs."""
    with tempfile.TemporaryDirectory(dir=scratch, prefix=case['name'] + '.') as folder:
        try:
            result = subprocess.run([str(binary), *case['args']], input=case['stdin'].encode(),
                                    env=case['env'], cwd=folder, capture_output=True,
                                    timeout=TIMEOUT, check=False)
        except subprocess.TimeoutExpired:
            raise SystemExit(f'{case["name"]}: no exit within {TIMEOUT} seconds') from None
        left = sorted(os.listdir(folder))
    return result.returncode, result.stdout, result.stderr, left


def mismatch(case, status, stderr, left):
    """Why a run does not meet the case's declarations, or None."""
    wanted = -signal.SIGABRT if case['status'] == 'abort' else case['status']
    if status != wanted:
        return f'status {status}, declared {case["status"]}'
    expected = case['stderr']
    if isinstance(expected, str):
        if stderr.decode(errors='replace') != expected:
            return f'standard error {stderr!r}, declared {expected!r}'
    elif any(piece.encode() not in stderr for piece in expected['contains']):
        return f'standard error {stderr!r} lacks one of {expected["contains"]}'
    if left:
        return f'left files in its working directory: {left}'
    return None


def observe(case, folders, scratch):
    """Standard output every build and every repeat agree on."""
    seen = None
    for label, folder in folders:
        for _ in range(REPEATS):
            status, stdout, stderr, left = execute(folder / case['name'], case, scratch)
            problem = mismatch(case, status, stderr, left)
            if problem:
                raise SystemExit(f'{case["name"]} ({label}): {problem}')
            if seen is not None and stdout != seen:
                raise SystemExit(f'{case["name"]} ({label}): standard output is not stable '
                                 f'across builds and repeats')
            seen = stdout
    return seen


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    parser.add_argument('rung', nargs='+', choices=sorted(RUNGS))
    parser.add_argument('--record', action='store_true',
                        help='write the .stdout files and hashes instead of comparing')
    arguments = parser.parse_args()
    pin = demand_list.toolchain()
    rust, _ = demand_list.prepare_source(pin['rust_src'])
    scratch = WORK / 'run'
    scratch.mkdir(parents=True, exist_ok=True)
    failures = 0
    for rung in arguments.rung:
        path, outcomes, assignment = load(rung)
        corpus = demand_list.CORPORA[rung]
        demand_list.build(rung, corpus, pin, rust / 'library')
        folders = [('course profile', demand_list.WORK / 'target' / demand_list.TARGET / 'debug'),
                   build_optimized(rung, pin, rust / 'library')]
        for case in outcomes['cases']:
            stdout = observe(case, folders, scratch)
            source = (assignment / case['path']).read_bytes()
            reference = assignment / case['stdout']
            if arguments.record:
                reference.write_bytes(stdout)
                case['source_sha256'], case['stdout_sha256'] = sha256(source), sha256(stdout)
            elif (not reference.exists() or reference.read_bytes() != stdout
                  or case.get('stdout_sha256') != sha256(stdout)
                  or case.get('source_sha256') != sha256(source)):
                print(f'{rung} {case["name"]}: differs from the recorded oracle')
                failures += 1
        if arguments.record:
            path.write_text(json.dumps(outcomes, indent=2, ensure_ascii=False) + '\n')
        print(f'{rung}: {len(outcomes["cases"])} programs '
              f'{"recorded" if arguments.record else "checked"}')
    raise SystemExit(1 if failures else 0)


if __name__ == '__main__':
    main()
