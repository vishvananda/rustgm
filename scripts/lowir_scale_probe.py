#!/usr/bin/env python3
"""LowIR scale probe: the pinned backend's time and peak memory on large valid LowIR.

Self-hosting hands the backend hundreds of thousands of function instances in
one program, far more than any fixture. This probe builds large valid programs
from RustGM's own output. Every executable native fixture that passes on its
own is a unit. A round is one replica of every unit with the unit's symbols
renamed, and a generated entry calls every replica's former entry, so the
combined program still checks itself: exit status 0 means every replica ran
and passed. Each size is built and run on both link paths: the pinned `lowir`
validator and standalone `lowir2native -O0`, and the object bridge
(`cppgm++ -O0 -c`, then the system `cc`). The probe records wall time, CPU time
and peak resident memory per process.

Each backend process runs under an address-space limit, so a size that needs
more memory fails by itself and is recorded as a failing row; a path's first
failure ends that path for the size. Rows are written as they are observed.

`--density` compares code size instead: every unit is compiled to an object by
the backend and, from its Rust source, by pinned rustc under the course
profile, and the bytes of their executable sections go to `density.tsv`. rustc
is a yardstick here, as in the native checks.

Nothing here changes the backend or the frontend, and the combined programs are
removed after they are measured. Run it through the resource and storage guards:

    scripts/lowir_scale_probe.py --app PATH_TO_RUSTGM 16 64 128
    scripts/lowir_scale_probe.py --app PATH_TO_RUSTGM --density
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import struct
import subprocess
import sys
import tempfile
import threading
import time

from backend import load_tools
from check_syntax import rustc_arguments
from course import ROOT, run_case

WORK = ROOT / 'obj/parallel/lowir-scale'
NAME = rb'[A-Za-z_][A-Za-z0-9_.$]*'
DEFINITION = re.compile(rb'(?:function|global) @(' + NAME + rb')')
ENTRY = re.compile(rb'function @(' + NAME + rb')\(\) -> i64 \[role=entry\] \{')
ROLE = b' [role=entry]'
SPAN = re.compile(rb'obj<(\d+)x')
LARGEST_OBJECT = 1 << 20
FAN_OUT = 256
COLUMNS = ('rounds', 'replicas', 'functions', 'globals', 'lines', 'bytes', 'repeat', 'tool',
           'wall_seconds', 'user_seconds', 'system_seconds', 'peak_rss_kib', 'status', 'output_bytes')


class Unsuitable(ValueError):
    """A fixture's LowIR this probe cannot replicate."""


def candidates():
    """Executable native fixtures declared to pass, in stage and manifest order."""
    manifests = sorted(ROOT.glob('pa*/tests/manifest.json'), key=lambda path: int(path.parts[-3][2:]))
    for manifest in manifests:
        document = json.loads(manifest.read_text())
        for case in document['cases']:
            native = case.get('native', {})
            if (case.get('exit_status') == 0 and case.get('rustc', {}).get('kind') == 'program'
                    and native.get('kind', 'execute') == 'execute' and native.get('outcome') == 'pass'
                    and not case.get('auxiliary') and not case.get('environment')):
                yield manifest.parts[-3], document['mode'], case


def parse(text):
    """Split one unit into globals, declarations and functions, with what it defines."""
    if any(int(size) > LARGEST_OBJECT for size in SPAN.findall(text)):
        raise Unsuitable('an object over 1 MiB: the probe measures code, not constant data')
    globals_, declarations, functions, defined, entry = [], [], [], set(), None
    lines = text.splitlines(keepends=True)
    index = 0
    while index < len(lines):
        line = lines[index]
        if line.startswith(b'declare '):
            declarations.append(line)
            index += 1
            continue
        match = DEFINITION.match(line)
        if not match or not line.rstrip().endswith(b'{'):
            raise Unsuitable(f'unsupported top-level item: {line[:60]!r}')
        end = index
        while end < len(lines) and lines[end] != b'}\n':
            end += 1
        if end == len(lines):
            raise Unsuitable('unterminated item')
        defined.add(match.group(1))
        if ROLE in line:
            found = ENTRY.fullmatch(line.rstrip())
            if not found or entry is not None:
                raise Unsuitable(f'unsupported entry: {line[:60]!r}')
            entry = found.group(1)
            lines[index] = line.replace(ROLE, b'')
        (globals_ if line.startswith(b'global ') else functions).append(b''.join(lines[index:end + 1]))
        index = end + 1
    if entry is None:
        raise Unsuitable('no entry function')
    return {'globals': pieces(b''.join(globals_), defined), 'functions': pieces(b''.join(functions), defined),
            'declarations': declarations, 'entry': entry,
            'counts': (len(functions), len(globals_), len(lines) - len(declarations))}


def pieces(text, defined):
    """The text cut after every use of a defined symbol; joining with a suffix renames them all."""
    cuts, last = [], 0
    for match in re.finditer(rb'@(' + NAME + rb')', text):
        if match.group(1) in defined:
            cuts.append(text[last:match.end()])
            last = match.end()
    cuts.append(text[last:])
    return cuts


def driver(entries):
    """Functions that call every entry once through a tree; the root is the new entry."""
    level, out = 0, []
    while True:
        groups = [entries[start:start + FAN_OUT] for start in range(0, len(entries), FAN_OUT)]
        root = len(groups) == 1
        names = []
        for number, group in enumerate(groups):
            name = b'main' if root else b'probe.g%d.%d' % (level, number)
            names.append(name)
            body = [b'function @%s() -> i64%s {\n  block ^b0:\n' % (name, ROLE if root else b''),
                    b'    %%t0 = call i64 @%s()\n' % group[0]]
            for position, callee in enumerate(group[1:], 1):
                body.append(b'    %%t%d = call i64 @%s()\n' % (2 * position - 1, callee))
                body.append(b'    %%t%d = binary add i64 %%t%d, %%t%d\n'
                            % (2 * position, 2 * position - 2, 2 * position - 1))
            body.append(b'    return i64 %%t%d\n}\n' % (2 * len(group) - 2))
            out.append(b''.join(body))
        if root:
            return out
        entries, level = names, level + 1


def compose(units, rounds, path):
    """Write `rounds` replicas of every unit as one program; returns its counts and hash."""
    declared = {}
    for unit in units:
        for line in unit['declarations']:
            name = DEFINITION.search(line).group(1)
            if declared.setdefault(name, line) != line:
                raise Unsuitable(f'conflicting declarations of {name.decode()}')
    hasher, size = hashlib.sha256(), 0
    with path.open('wb') as out:
        def emit(data):
            nonlocal size
            out.write(data)
            hasher.update(data)
            size += len(data)

        entries = []
        for part in ('globals', 'functions'):
            if part == 'functions':
                emit(b''.join(declared[name] for name in sorted(declared)))
            for replica in range(rounds):
                for number, unit in enumerate(units):
                    suffix = b'.r%d' % (replica * len(units) + number)
                    emit(suffix.join(unit[part]))
                    if part == 'functions':
                        entries.append(unit['entry'] + suffix)
        tree = driver(entries)
        emit(b''.join(tree))
    functions, globals_, lines = (rounds * sum(unit['counts'][index] for unit in units) for index in range(3))
    return {'rounds': rounds, 'replicas': rounds * len(units), 'functions': functions + len(tree),
            'globals': globals_, 'lines': lines + len(declared) + sum(item.count(b'\n') for item in tree),
            'bytes': size, 'sha256': hasher.hexdigest()}


def measured(command, log, memory_gib, timeout):
    """One process under an address-space limit: wall and CPU seconds, peak resident KiB, status."""
    limited = ['prlimit', f'--as={memory_gib << 30}', *command]
    start = time.monotonic()
    with log.open('wb') as stream:
        process = subprocess.Popen(limited, stdout=stream, stderr=subprocess.STDOUT, cwd=ROOT)
        timer = threading.Timer(timeout, process.kill)
        timer.start()
        try:
            _, status, usage = os.wait4(process.pid, 0)
        finally:
            timer.cancel()
    process.returncode = os.waitstatus_to_exitcode(status)
    if not log.stat().st_size:
        log.unlink()
    return {'wall_seconds': round(time.monotonic() - start, 3), 'user_seconds': round(usage.ru_utime, 3),
            'system_seconds': round(usage.ru_stime, 3), 'peak_rss_kib': usage.ru_maxrss,
            'status': process.returncode}


def code_bytes(path):
    """Bytes in the executable sections of an ELF64 relocatable object."""
    data = path.read_bytes()
    if data[:6] != b'\x7fELF\x02\x01':
        raise Unsuitable(f'{path.name} is not a little-endian ELF64 file')
    offset, = struct.unpack_from('<Q', data, 40)
    size, count = struct.unpack_from('<HH', data, 58)
    total = 0
    for index in range(count):
        flags, _, _, length = struct.unpack_from('<QQQQ', data, offset + index * size + 8)
        if flags & 4:
            total += length
    return total


def density(tools, folder, source, stage, case):
    """Machine code for one fixture: pinned rustc's object against the backend's object."""
    fixture, ours, theirs = ROOT / stage / case['path'], folder / 'unit.o', folder / 'rustc.o'
    for command in ([tools['cppgm++'], '-O0', '-c', str(source), '-o', str(ours)],
                    ['rustc', '--edition=2024', '--target', 'x86_64-unknown-linux-gnu',
                     *rustc_arguments(case, 'density'), '-C', 'panic=abort', '-C', 'overflow-checks=yes',
                     '-C', 'debug-assertions=off', '-C', 'codegen-units=1', '--cap-lints', 'allow',
                     '--emit=obj', '-o', str(theirs), str(fixture)]):
        result = subprocess.run(command, capture_output=True, timeout=120, cwd=ROOT, check=False)
        if result.returncode:
            raise Unsuitable(f'{Path(command[0]).name} status {result.returncode} for the density object')
    return {'backend_code': code_bytes(ours), 'rustc_code': code_bytes(theirs)}


def qualify(app, tools, scratch, dense, stage, mode, case):
    """The fixture's LowIR if it validates, compiles and exits 0 by itself; otherwise why not."""
    try:
        _, (text, _, status) = run_case(ROOT, stage, mode, case, [str(app)], 60)
        if status != b'0\n' or not text:
            raise Unsuitable(f'frontend status {status!r}')
        unit = parse(text)
        with tempfile.TemporaryDirectory(dir=scratch) as folder:
            source, checked, native = (Path(folder) / name for name in ('unit.lowir', 'checked.lowir', 'native'))
            source.write_bytes(text)
            for command in ([tools['lowir'], '-o', str(checked), str(source)],
                            [tools['lowir2native'], '-O0', '-o', str(native), str(checked)], [str(native)]):
                result = subprocess.run(command, capture_output=True, timeout=60, cwd=ROOT, check=False)
                if result.returncode or (command[0] == str(native) and (result.stdout or result.stderr)):
                    raise Unsuitable(f'{Path(command[0]).name} status {result.returncode}')
            if dense:
                unit.update(density(tools, Path(folder), source, stage, case))
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        return f'{stage}/{case["path"]}', None, str(error)
    unit.update(path=f'{stage}/{case["path"]}', bytes=len(text), sha256=hashlib.sha256(text).hexdigest())
    return unit['path'], unit, None


def observe(tools, rounds, units, arguments, record):
    """Compose one size and build and run it on both link paths; one row per process.

    A path's first failure ends that path for this size; the other path goes on.
    """
    folder = arguments.output
    program, checked, native, unit, linked = (folder / f'rounds-{rounds}.{suffix}'
                                              for suffix in ('lowir', 'checked.lowir', 'native', 'o', 'linked'))
    paths = ((('lowir', [tools['lowir'], '-o', str(checked), str(program)], checked),
              ('lowir2native', [tools['lowir2native'], '-O0', '-o', str(native), str(checked)], native),
              ('run', [str(native)], None)),
             (('cppgm++', [tools['cppgm++'], '-O0', '-c', str(program), '-o', str(unit)], unit),
              ('cc', ['cc', str(unit), '-lm', '-o', str(linked)], linked),
              ('run-linked', [str(linked)], None)))
    passing = list(paths)
    try:
        shape = compose(units, rounds, program)
        print(f'rounds {rounds}: {shape["functions"]} functions, {shape["bytes"]} bytes', flush=True)
        for repeat in range(arguments.repeats):
            for steps in tuple(passing):
                for tool, command, output in steps:
                    result = measured(command, folder / f'rounds-{rounds}.{tool}.log', arguments.memory_gib,
                                      arguments.timeout)
                    produced = output.stat().st_size if output is not None and output.exists() else 0
                    record({**shape, 'repeat': repeat, 'tool': tool, **result, 'output_bytes': produced})
                    print(f'  {tool}: {result}', flush=True)
                    if result['status']:
                        passing.remove(steps)
                        break
    finally:
        if not arguments.keep:
            for generated in (program, checked, native, unit, linked):
                generated.unlink(missing_ok=True)
    return shape, len(passing) == len(paths)


def write_density(units, folder):
    """Per-unit and total machine-code bytes of the two compilers, as density.tsv."""
    columns = ('path', 'bytes', 'functions', 'backend_code', 'rustc_code')
    with (folder / 'density.tsv').open('w') as out:
        out.write('\t'.join(columns) + '\n')
        for unit in units:
            out.write('\t'.join(str(unit['counts'][0] if column == 'functions' else unit[column])
                                for column in columns) + '\n')
    totals = {column: sum(unit['counts'][0] if column == 'functions' else unit[column] for unit in units)
              for column in columns[1:]}
    print(f'density: {totals}', flush=True)
    return totals


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    parser.add_argument('rounds', nargs='*', type=int, help='replicas of every unit, one size each')
    parser.add_argument('--density', action='store_true',
                        help='also compile every unit to an object with pinned rustc and with the backend '
                             'and record their machine-code bytes in density.tsv')
    parser.add_argument('--keep', action='store_true', help='keep the composed programs for inspection')
    parser.add_argument('--output', type=Path, default=WORK,
                        help='folder for observations and receipts; name another for a side run')
    parser.add_argument('--app', required=True, type=Path, help='RustGM compiler binary that emits the units')
    parser.add_argument('--jobs', type=int, default=8)
    parser.add_argument('--repeats', type=int, default=3)
    parser.add_argument('--memory-gib', type=int, default=10,
                        help='address-space limit per backend process; keep it below the job memory limit, '
                             'which stops the whole probe instead of failing one process')
    parser.add_argument('--timeout', type=int, default=1800, help='seconds per backend process')
    parser.add_argument('--limit', type=int, help='use only the first N candidate fixtures')
    arguments = parser.parse_args()
    # A stopped probe must still remove the large programs it composed.
    signal.signal(signal.SIGTERM, lambda number, _frame: sys.exit(128 + number))
    wanted = ('lowir', 'lowir2native', 'cppgm++')
    tools = {name: str(path) for name, path in load_tools(wanted).items() if name in wanted}
    folder = arguments.output = arguments.output.resolve()
    scratch = folder / 'scratch'
    scratch.mkdir(parents=True, exist_ok=True)
    cases = list(candidates())[:arguments.limit]
    with ThreadPoolExecutor(max_workers=arguments.jobs) as pool:
        results = list(pool.map(lambda item: qualify(arguments.app, tools, scratch, arguments.density, *item), cases))
    units = [unit for _, unit, _ in results if unit is not None]
    print(f'{len(units)} of {len(cases)} fixtures qualify as units', flush=True)
    if not units:
        raise SystemExit('no fixture qualifies')
    # Hashed in pieces: this process's own peak memory is the floor of every child's reported peak.
    with arguments.app.open('rb') as stream:
        compiler = hashlib.file_digest(stream, 'sha256').hexdigest()
    receipt = {'schema': 1, 'backend': json.loads((ROOT / 'backend.lock.json').read_text()),
               'compiler': str(arguments.app), 'compiler_sha256': compiler,
               'repeats': arguments.repeats, 'memory_gib': arguments.memory_gib, 'timeout': arguments.timeout,
               'programs': [], 'rejected': {path: reason for path, unit, reason in results if unit is None},
               'units': [{key: unit[key] for key in ('path', 'bytes', 'sha256')} for unit in units]}
    if arguments.density:
        receipt['rustc'] = subprocess.check_output(['rustc', '--version'], text=True, cwd=ROOT).strip()
        receipt['density'] = write_density(units, folder)
        (folder / 'density.json').write_text(json.dumps(receipt, indent=2) + '\n')
    if not arguments.rounds:
        return
    passed = True
    with (folder / 'observations.tsv').open('w') as out:
        def record(row):
            out.write('\t'.join(str(row[column]) for column in COLUMNS) + '\n')
            out.flush()

        out.write('\t'.join(COLUMNS) + '\n')
        for rounds in arguments.rounds:
            shape, complete = observe(tools, rounds, units, arguments, record)
            passed = passed and complete
            receipt['programs'].append({**shape, 'complete': complete})
            (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    raise SystemExit(0 if passed else 1)


if __name__ == '__main__':
    main()
