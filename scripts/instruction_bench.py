#!/usr/bin/env python3
"""Instruction-count benchmarks of one compiler binary on a frozen input set.

Wall time on a shared host has A/A ranges of tens of percent. Retired user
instructions on frozen inputs repeat far more closely, so they are the primary
metric: this harness counts user instructions and cycles of the compiler
process itself (no wrapper process is counted), records peak resident memory
and hashes every output. Two receipts compare benchmark by benchmark, and any
difference in output, diagnostics or exit status fails the comparison.

    instruction_bench.py freeze PLAN SET        copy and hash a benchmark set
    instruction_bench.py build REVISION OUT     release compiler of one commit
    instruction_bench.py run SET COMPILER OUT   measure every benchmark
    instruction_bench.py compare BASE CANDIDATE per-benchmark deltas

A set is a directory of frozen inputs and `set.json`: the SHA-256 of every
input and the benchmarks, each a compiler argument list in which `{set}` names
the set directory. A benchmark runs in its own empty directory, so outputs are
plain relative names and the arguments do not depend on where results go. Every
input is verified before and after a run. `run` repeats each benchmark, refuses
outputs that differ between repetitions and reports the run-to-run spread of
instructions on that one binary.

`build` archives one commit, builds it with the repository's release profile
and pins the executable, so a base and a candidate are built the same way.

Run measurements and builds through the storage and resource guards, with an
ordinary lease:

    python3 scripts/storage.py run --no-prune -- python3 scripts/instruction_bench.py \\
        build main obj/parallel/instruction-bench/bin/main
    python3 scripts/storage.py run --prune-under obj/parallel/instruction-bench/runs -- \\
        python3 scripts/instruction_bench.py run obj/parallel/instruction-bench/set \\
        obj/parallel/instruction-bench/bin/main/rustgm obj/parallel/instruction-bench/runs/main
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import statistics
import struct
import subprocess
import sys
import tarfile
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = 1
NAME = re.compile(r'[a-z0-9][a-z0-9._-]*')
STREAMS = ('stdout', 'stderr')
DEFAULT_TIMEOUT = 900
# perf_event_open(2): one hardware counter of the compiler, user mode only,
# inherited by its threads. It is opened while the compiler is stopped at its
# first instruction, so it counts that process and nothing else.
PERF_EVENT_OPEN = {'x86_64': 298, 'aarch64': 241}
HARDWARE = 0
CYCLES, INSTRUCTIONS = 0, 1
INHERIT, EXCLUDE_KERNEL, EXCLUDE_HV = 1 << 1, 1 << 5, 1 << 6
TIME_ENABLED_AND_RUNNING = 3
FD_CLOEXEC = 8
ADDR_NO_RANDOMIZE = 0x0040000
# ptrace(2): the compiler stops once after execve and once as it exits, where
# its own address space still reports its peak resident size.
TRACEME, CONTINUE, SETOPTIONS = 0, 7, 0x4200
TRACE_EXIT, EXIT_KILL, EVENT_EXIT = 0x40, 0x100000, 6


class BenchError(ValueError):
    """A refused set, measurement or comparison."""


def sha256(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, indent=1, sort_keys=True) + '\n')


def checked_benchmarks(benchmarks):
    """The benchmarks of a plan or set, refusing names that could leave a run directory."""
    seen = set()
    for row in benchmarks:
        name = row.get('name', '')
        outputs = row.get('outputs', [])
        if not NAME.fullmatch(name) or name in seen:
            raise BenchError(f'benchmark name {name!r} is missing, unsafe or repeated')
        seen.add(name)
        if (not row.get('argv') or not all(isinstance(value, str) for value in row['argv'])
                or not all(isinstance(value, str) for value in row.get('env', {}).values())):
            raise BenchError(f'{name}: arguments and environment must be strings')
        for output in (*outputs, *row.get('empty', [])):
            if not NAME.fullmatch(output) or output in STREAMS:
                raise BenchError(f'{name}: output {output!r} must be a plain file name')
        if not isinstance(row.get('status', 0), int) or row.get('timeout', 1) <= 0:
            raise BenchError(f'{name}: status and timeout must be numbers')
    if not seen:
        raise BenchError('a benchmark set needs at least one benchmark')
    return benchmarks


def regular_files(directory):
    """Every file below a directory by relative path; links are never frozen."""
    found = {}
    for folder, names, files in os.walk(directory, followlinks=False):
        names.sort()
        for name in sorted(files):
            path = Path(folder) / name
            if path.is_symlink() or not path.is_file():
                raise BenchError(f'{path} is not a regular file')
            found[path.relative_to(directory).as_posix()] = path
    return found


def freeze(plan, directory):
    """Copy a plan's inputs into a new set directory and record their hashes.

    `trees` and `files` map a set-relative path to a source directory or
    file. `pins` maps a set-relative file to the SHA-256 it must have, which
    ties the copy to an earlier receipt.
    """
    benchmarks = checked_benchmarks(plan.get('benchmarks', []))
    sources = {}
    for target, source in plan.get('trees', {}).items():
        for relative, path in regular_files(Path(source)).items():
            sources[f'{target}/{relative}'] = path
    for target, source in plan.get('files', {}).items():
        sources[target] = Path(source)
    for target in sources:
        parts = Path(target).parts
        if not parts or Path(target).is_absolute() or '..' in parts or target == 'set.json':
            raise BenchError(f'{target!r} is not a path inside the set')
    directory = Path(directory)
    directory.mkdir(parents=True)
    files = {}
    for target, source in sorted(sources.items()):
        copy = directory / target
        copy.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, copy, follow_symlinks=False)
        copy.chmod(0o444)
        files[target] = sha256(copy)
    for target, expected in plan.get('pins', {}).items():
        if files.get(target) != expected:
            raise BenchError(f'{target} does not have its pinned SHA-256 {expected}')
    write_json(directory / 'set.json', {'schema': SCHEMA, 'files': files, 'benchmarks': benchmarks})
    return files


def load_set(directory):
    """A set's manifest and digest, after checking every frozen input."""
    directory = Path(directory).resolve(strict=True)
    content = (directory / 'set.json').read_bytes()
    manifest = json.loads(content)
    if manifest.get('schema') != SCHEMA:
        raise BenchError(f'{directory} is not a schema {SCHEMA} benchmark set')
    checked_benchmarks(manifest['benchmarks'])
    verify_set(directory, manifest)
    return directory, manifest, hashlib.sha256(content).hexdigest()


def verify_set(directory, manifest):
    for relative, expected in manifest['files'].items():
        path = directory / relative
        if path.is_symlink() or not path.is_file() or sha256(path) != expected:
            raise BenchError(f'frozen input {relative} is missing or changed')


def system():
    libc = ctypes.CDLL(None, use_errno=True)
    libc.syscall.restype = ctypes.c_long
    libc.ptrace.restype = ctypes.c_long
    libc.ptrace.argtypes = [ctypes.c_long, ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p]
    return libc


def trace(request, pid, data=0):
    if system().ptrace(request, pid, None, data) < 0:
        raise BenchError(f'ptrace request {request} failed: {os.strerror(ctypes.get_errno())}')


def counter(pid, event):
    """A descriptor counting one user-mode hardware event of a process tree."""
    number = PERF_EVENT_OPEN.get(platform.machine())
    if number is None:
        raise BenchError(f'no perf_event_open number for {platform.machine()}')
    attributes = struct.pack('IIQQQQQIIQ', HARDWARE, 64, event, 0, 0, TIME_ENABLED_AND_RUNNING,
                             INHERIT | EXCLUDE_KERNEL | EXCLUDE_HV, 0, 0, 0)
    descriptor = system().syscall(
        ctypes.c_long(number), ctypes.create_string_buffer(attributes, len(attributes)),
        ctypes.c_int(pid), ctypes.c_int(-1), ctypes.c_int(-1), ctypes.c_ulong(FD_CLOEXEC))
    if descriptor < 0:
        raise BenchError(f'perf_event_open failed: {os.strerror(ctypes.get_errno())}; '
                         'user-mode hardware counters are required')
    return descriptor


def read_counter(descriptor):
    value, enabled, running = struct.unpack('QQQ', os.read(descriptor, 24))
    if enabled != running:
        raise BenchError('a hardware counter was multiplexed; its value would be an estimate')
    return value


def high_water(pid):
    """Peak resident KiB of a stopped process's own address space."""
    for line in Path(f'/proc/{pid}/status').read_text().splitlines():
        if line.startswith('VmHWM:'):
            return int(line.split()[1])
    raise BenchError('the stopped compiler reports no peak resident size')


def child(executable, argv, env, cwd):
    """Become the compiler, stopped for the parent after execve. Never returns."""
    try:
        os.setsid()
        os.chdir(cwd)
        for descriptor, name, flags in ((0, os.devnull, os.O_RDONLY),
                                        (1, 'stdout', os.O_WRONLY | os.O_CREAT | os.O_EXCL),
                                        (2, 'stderr', os.O_WRONLY | os.O_CREAT | os.O_EXCL)):
            opened = os.open(name, flags, 0o644)
            os.dup2(opened, descriptor)
            os.close(opened)
        # A fixed address-space layout removes one source of run-to-run spread.
        libc = system()
        libc.personality(ctypes.c_ulong(libc.personality(ctypes.c_ulong(0xFFFFFFFF)) | ADDR_NO_RANDOMIZE))
        trace(TRACEME, 0)
        os.execve(executable, ['rustgm', *argv], env)
    except BaseException as error:  # noqa: BLE001 - the forked child must not unwind into the harness
        os.write(2, f'instruction_bench: cannot start {executable}: {error}\n'.encode())
    finally:
        os._exit(127)


def reap(pid):
    """End a child that may be stopped and collect it."""
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    while True:
        try:
            _, status, _ = os.wait4(pid, 0)
        except ChildProcessError:
            return
        if not os.WIFSTOPPED(status):
            return


def measure(executable, argv, env, cwd, timeout):
    """Run the compiler in `cwd`; its streams become `stdout` and `stderr` there."""
    sys.stdout.flush()
    sys.stderr.flush()
    pid = os.fork()
    if pid == 0:
        child(str(executable), argv, env, cwd)
    expired = []
    watchdog = threading.Timer(timeout, lambda: (expired.append(True), os.killpg(pid, signal.SIGKILL)))
    watchdog.start()
    descriptors = []
    peak = None
    live = True
    try:
        _, status, usage = os.wait4(pid, 0)
        live = os.WIFSTOPPED(status)
        if not live:
            raise BenchError(f'cannot start {executable}; see {cwd}/stderr')
        descriptors = [counter(pid, event) for event in (INSTRUCTIONS, CYCLES)]
        trace(SETOPTIONS, pid, TRACE_EXIT | EXIT_KILL)
        start = time.monotonic()
        deliver = 0
        while live:
            trace(CONTINUE, pid, deliver)
            _, status, usage = os.wait4(pid, 0)
            live = os.WIFSTOPPED(status)
            if live and status >> 8 == signal.SIGTRAP | EVENT_EXIT << 8:
                peak, deliver = high_water(pid), 0
            elif live:
                deliver = os.WSTOPSIG(status)
        wall = time.monotonic() - start
        if expired:
            raise BenchError(f'the compiler exceeded {timeout} seconds')
        if peak is None:
            raise BenchError('the compiler ended without reporting its peak resident size')
        instructions, cycles = (read_counter(descriptor) for descriptor in descriptors)
    finally:
        watchdog.cancel()
        watchdog.join()
        if live:
            reap(pid)
        for descriptor in descriptors:
            os.close(descriptor)
    return {'status': os.waitstatus_to_exitcode(status), 'instructions': instructions, 'cycles': cycles,
            'peak_rss_kib': peak, 'wall_seconds': round(wall, 6),
            'user_seconds': usage.ru_utime, 'system_seconds': usage.ru_stime}


def identity(folder, outputs, status):
    """What a compilation produced: its exit status and the hash of every output."""
    rows = {}
    for name in (*outputs, *STREAMS):
        path = folder / name
        rows[name] = {'bytes': path.stat().st_size, 'sha256': sha256(path)} if path.is_file() else None
    return {'status': status, 'outputs': rows}


def spread(values):
    """The range of repeated observations relative to their smallest."""
    return (max(values) - min(values)) / min(values) if min(values) else 0.0


def summarize(samples):
    instructions = [sample['instructions'] for sample in samples]
    return {'instructions': statistics.median(instructions), 'instructions_min': min(instructions),
            'instructions_max': max(instructions), 'instructions_spread': spread(instructions),
            'cycles': statistics.median(sample['cycles'] for sample in samples),
            'peak_rss_kib': max(sample['peak_rss_kib'] for sample in samples)}


def run_benchmark(row, set_directory, compiler, out, repeat, measurer):
    argv = [value.replace('{set}', str(set_directory)) for value in row['argv']]
    outputs = row.get('outputs', [])
    samples = []
    produced = None
    for repetition in range(repeat):
        folder = out / row['name'] / str(repetition)
        folder.mkdir(parents=True)
        for name in row.get('empty', []):
            (folder / name).touch()
        sample = measurer(compiler, argv, dict(row.get('env', {})), folder, row.get('timeout', DEFAULT_TIMEOUT))
        observed = identity(folder, outputs, sample.pop('status'))
        if observed['status'] != row.get('status', 0):
            raise BenchError(f"{row['name']}: exit status {observed['status']}, expected "
                             f"{row.get('status', 0)}; see {folder}")
        if produced is None:
            produced = observed
        elif observed != produced:
            raise BenchError(f"{row['name']}: repetition {repetition} produced different output; see {folder}")
        else:
            # Equal to the first repetition's retained files.
            for name in (*outputs, *row.get('empty', [])):
                (folder / name).unlink(missing_ok=True)
        samples.append(sample)
    return {'argv': argv, 'env': row.get('env', {}), 'produced': produced, 'samples': samples,
            **summarize(samples)}


def run_set(set_directory, compiler, out, *, repeat=3, only=(), measurer=measure):
    """Measure a compiler on a set and write `receipt.json` under a new directory."""
    set_directory, manifest, digest = load_set(set_directory)
    compiler = Path(compiler).resolve(strict=True)
    rows = [row for row in manifest['benchmarks'] if not only or row['name'] in only]
    missing = set(only) - {row['name'] for row in rows}
    if missing or repeat < 1:
        raise BenchError(f'unknown benchmarks {sorted(missing)} or no repetitions')
    out = Path(out)
    out.mkdir(parents=True)
    out = out.resolve()
    receipt = {'schema': SCHEMA, 'set': {'path': str(set_directory), 'sha256': digest},
               'compiler': {'path': str(compiler), 'sha256': sha256(compiler), 'bytes': compiler.stat().st_size},
               'host': {'machine': platform.machine(), 'kernel': platform.release()},
               'address_space_randomization': False, 'repeat': repeat, 'benchmarks': {}}
    for row in rows:
        receipt['benchmarks'][row['name']] = run_benchmark(row, set_directory, compiler, out, repeat, measurer)
        write_json(out / 'receipt.json', receipt)
        print(render_run(row['name'], receipt['benchmarks'][row['name']]), flush=True)
    verify_set(set_directory, manifest)
    if sha256(compiler) != receipt['compiler']['sha256']:
        raise BenchError('the compiler changed while it was measured')
    receipt['complete'] = True
    write_json(out / 'receipt.json', receipt)
    return receipt


def render_run(name, row):
    return (f"{name:<24} {row['instructions']:>16,.0f} instructions  spread {row['instructions_spread']:.4%}  "
            f"{row['cycles']:>16,.0f} cycles  {row['peak_rss_kib']:>9,} KiB")


def change(base, candidate):
    return (candidate - base) / base if base else 0.0


def compare(base, candidate):
    """Rows of per-benchmark deltas, and every reason the comparison fails."""
    problems = []
    for receipt, label in ((base, 'base'), (candidate, 'candidate')):
        if receipt.get('schema') != SCHEMA or not receipt.get('complete'):
            problems.append(f'the {label} receipt is incomplete or has another schema')
    if base['set']['sha256'] != candidate['set']['sha256']:
        problems.append('the receipts measured different benchmark sets')
    shared = [name for name in base['benchmarks'] if name in candidate['benchmarks']]
    if not shared:
        problems.append('the receipts share no benchmark')
    rows = []
    for name in shared:
        old, new = base['benchmarks'][name], candidate['benchmarks'][name]
        differing = [] if old['argv'] == new['argv'] and old['env'] == new['env'] else ['arguments']
        if old['produced']['status'] != new['produced']['status']:
            differing.append('status')
        differing += [output for output, value in old['produced']['outputs'].items()
                      if new['produced']['outputs'].get(output) != value]
        differing += [output for output in new['produced']['outputs'] if output not in old['produced']['outputs']]
        if differing:
            problems.append(f"{name}: {', '.join(differing)} differ")
        rows.append({'name': name, 'base': old['instructions'], 'candidate': new['instructions'],
                     'instructions': change(old['instructions'], new['instructions']),
                     'cycles': change(old['cycles'], new['cycles']),
                     'peak_rss': change(old['peak_rss_kib'], new['peak_rss_kib']),
                     'spread': max(old['instructions_spread'], new['instructions_spread']),
                     'same_output': not differing})
    unshared = sorted(set(base['benchmarks']) ^ set(candidate['benchmarks']))
    return rows, problems, unshared


def render_comparison(rows, unshared):
    lines = [f"{'benchmark':<24} {'base instructions':>18} {'candidate':>18} {'instr':>9} {'cycles':>9} "
             f"{'peak RSS':>9} {'spread':>9}  output"]
    for row in rows:
        lines.append(f"{row['name']:<24} {row['base']:>18,.0f} {row['candidate']:>18,.0f} "
                     f"{row['instructions']:>+9.3%} {row['cycles']:>+9.3%} {row['peak_rss']:>+9.3%} "
                     f"{row['spread']:>9.4%}  {'same' if row['same_output'] else 'DIFFERENT'}")
    if unshared:
        lines.append('measured on one side only: ' + ', '.join(unshared))
    return '\n'.join(lines)


def git(*arguments):
    return subprocess.run(['git', *arguments], cwd=ROOT, check=True, capture_output=True,
                          text=True).stdout.strip()


def build(revision, out):
    """Build one commit's compiler with the repository's release profile and pin it."""
    commit = git('rev-parse', '--verify', f'{revision}^{{commit}}')
    out = Path(out)
    out.mkdir(parents=True)
    sources = out / 'sources'
    sources.mkdir()
    archive = out / 'sources.tar'
    with archive.open('wb') as stream:
        subprocess.run(['git', 'archive', commit, 'dev'], cwd=ROOT, stdout=stream, check=True)
    with tarfile.open(archive) as stream:
        stream.extractall(sources, filter='data')
    target = ROOT / 'obj/parallel/instruction-bench/target'
    command = ['cargo', 'build', '--locked', '--offline', '--release', '--bin', 'rustgm',
               '--manifest-path', str(sources / 'dev/Cargo.toml')]
    # Incremental caches and profile overrides would make two builds differ.
    environment = {name: value for name, value in os.environ.items()
                   if not name.startswith(('CARGO_PROFILE_', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'))}
    environment.update(CARGO_INCREMENTAL='0', CARGO_TARGET_DIR=str(target))
    with (out / 'build.stdout').open('wb') as stdout, (out / 'build.stderr').open('wb') as stderr:
        status = subprocess.run(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr,
                                check=False).returncode
    if status:
        raise BenchError(f'the release build failed with status {status}; see {out}/build.stderr')
    (out / '.keep-artifacts').touch()
    executable = out / 'rustgm'
    shutil.copyfile(target / 'release/rustgm', executable)
    executable.chmod(0o555)
    receipt = {'commit': commit, 'tree': git('rev-parse', f'{commit}^{{tree}}'), 'command': command,
               'profile': 'release', 'jobs': environment.get('CARGO_BUILD_JOBS'),
               'rustc': subprocess.run(['rustc', '-vV'], capture_output=True, text=True, check=True).stdout,
               'compiler_sha256': sha256(executable), 'archive_sha256': sha256(archive)}
    write_json(out / 'receipt.json', receipt)
    return receipt


def main(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest='action', required=True)
    freezer = commands.add_parser('freeze')
    freezer.add_argument('plan', type=Path)
    freezer.add_argument('set', type=Path)
    builder = commands.add_parser('build')
    builder.add_argument('revision')
    builder.add_argument('out', type=Path)
    runner = commands.add_parser('run')
    runner.add_argument('set', type=Path)
    runner.add_argument('compiler', type=Path)
    runner.add_argument('out', type=Path)
    runner.add_argument('--repeat', type=int, default=3)
    runner.add_argument('--only', action='append', default=[])
    comparer = commands.add_parser('compare')
    comparer.add_argument('base', type=Path)
    comparer.add_argument('candidate', type=Path)
    args = parser.parse_args(arguments)
    try:
        if args.action == 'freeze':
            files = freeze(json.loads(args.plan.read_text()), args.set)
            print(f'froze {len(files)} files in {args.set}')
        elif args.action == 'build':
            receipt = build(args.revision, args.out)
            print(f"built {receipt['commit']} as {args.out}/rustgm sha256 {receipt['compiler_sha256']}")
        elif args.action == 'run':
            run_set(args.set, args.compiler, args.out, repeat=args.repeat, only=args.only)
        else:
            rows, problems, unshared = compare(*(json.loads(path.read_text())
                                                 for path in (args.base, args.candidate)))
            print(render_comparison(rows, unshared))
            for problem in problems:
                print(f'FAILED: {problem}', file=sys.stderr)
            return 1 if problems else 0
    except (BenchError, OSError, subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as error:
        print(f'instruction_bench: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
