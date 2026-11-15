#!/usr/bin/env python3
"""Demand list: the library instances a corpus needs, with pinned rustc as oracle.

rustc is used here only as an oracle and planning tool; nothing produced by this
script is an input to RustGM lexing, parsing, semantics or lowering.

Method. The corpus is built for x86_64-unknown-linux-gnu with Cargo's
`-Zbuild-std` from the pinned 1.98.0 library sources under RustGM's course
profile (opt-level 0, no debug assertions, overflow checks, panic=abort). No
LLVM pass runs (`-Cno-prepopulate-passes`) and MIR inlining is off at opt-level
0, so every instance rustc's monomorphization collector demanded keeps its own
v0-mangled symbol. The linker's `--gc-sections` result is the reachable set: the
linked binary's symbol table names every surviving instance. Unoptimized LLVM IR
(`--emit=llvm-ir`) gives each instance's definition site (DISubprogram file and
line) and the backend-visible operations inside reachable functions: atomics
with widths and orderings, `llvm.*` names, vector, i128 and float operations,
thread-locals, inline assembly and foreign calls. `-Zprint-mono-items` is
captured per crate as an independent count check. Replaying each library
crate's rustc invocation with `-Zdump-mir=LowerIntrinsics` gives the function
and closure bodies that exist (the denominator), their line ranges and the
intrinsic calls before and after rustc's own lowering.

A function is `linked` when the binary keeps it and `unlinked` when only
unreachable blocks of kept functions refer to it (rustc at opt-level 0 emits
the dead arm of a constant `ub_checks()` branch; LLVM's code generator drops
it). A frontend that does not fold those branches needs both.

Outputs per corpus under `obj/parallel/demand/NAME/`: `instances.tsv`,
`source-functions.tsv`, `summary.tsv`, `files.tsv`, `statics.tsv`,
`runtime-surface.tsv`, `receipt.json` (inputs) and `checks.json` (counts and
self-checks). Scratch stays in `obj/parallel/demand-work/`. Run it through the
resource and storage guards:

    scripts/demand_list.py rustgm m1 m2 m3 m4
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import inspect
import json
from multiprocessing import Pool
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.request

import rust_v0_symbols as v0

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / 'obj/parallel/demand-work'
OUT = ROOT / 'obj/parallel/demand'
TARGET = 'x86_64-unknown-linux-gnu'
REVISION = '88d9e12ae178fab0fb5cc050a94da85685d449ea'
# RustGM's course profile (pa34/tests/stdlib-course-profile-outcomes.json) plus
# the settings this oracle needs: line tables for definition sites, no strip.
PROFILE = {
    'OPT_LEVEL': '0', 'DEBUG': 'line-tables-only', 'DEBUG_ASSERTIONS': 'false',
    'OVERFLOW_CHECKS': 'true', 'PANIC': 'abort', 'STRIP': 'none',
    'INCREMENTAL': 'false', 'LTO': 'off', 'CODEGEN_UNITS': '256',
}
RUSTFLAGS = ['-Csymbol-mangling-version=v0', '-Cno-prepopulate-passes',
             '--emit=llvm-ir', '-Zprint-mono-items', '--print=link-args']
BUILD_STD = ['-Zbuild-std=std,panic_abort', '-Zbuild-std-features=']
# The compiler's own locked dependencies; tinyvec as unicode-normalization enables it.
UNICODE = {'unicode-ident': '"=1.0.24"', 'unicode-normalization': '"=0.1.25"',
           'tinyvec': '{ version = "=1.10.0", features = ["alloc"] }'}
CORPORA = {
    'rustgm': {'manifest': 'dev/Cargo.toml', 'bins': ['rustgm'], 'own': ['rustgm']},
    'm1': {'programs': 'pa32/tests/spec/core-consumers'},
    'm2': {'programs': 'pa33/tests/spec/alloc-consumers'},
    'm3': {'programs': 'pa36/tests/spec/std-consumers'},
    'm4': {'programs': 'pa35/tests/spec/unicode-consumers', 'dependencies': UNICODE},
}
# Crate names a consumer program must not reuse: the std graph and unicode-normalization's tinyvec_macros.
LIBRARY_CRATES = {'alloc', 'cfg_if', 'compiler_builtins', 'core', 'hashbrown', 'libc', 'panic_abort',
                  'panic_unwind', 'proc_macro', 'rustc_demangle', 'rustc_std_workspace_alloc',
                  'rustc_std_workspace_core', 'std', 'std_detect', 'test', 'tinyvec_macros', 'unwind'}
LINKAGES = {'internal', 'private', 'hidden', 'weak', 'weak_odr', 'linkonce_odr', 'protected'}
ORDERINGS = {'unordered': 'Unordered', 'monotonic': 'Relaxed', 'acquire': 'Acquire',
             'release': 'Release', 'acq_rel': 'AcqRel', 'seq_cst': 'SeqCst'}
FLOAT_OPS = {'fadd', 'fsub', 'fmul', 'fdiv', 'frem', 'fneg', 'fcmp', 'fptoui', 'fptosi',
             'uitofp', 'sitofp', 'fpext', 'fptrunc'}
SYMBOL = re.compile(r'@(?:"((?:[^"\\]|\\.)*)"|([-\w.$]+))')
INTERESTING = re.compile(
    r'atomic|cmpxchg|\bfence\b|i128|<\d+ x |\bf(?:add|sub|mul|div|rem|neg|cmp|ptoui|ptosi|pext|ptrunc)\b'
    r'|\b[us]itofp\b| asm |\.\.\.\) @')
VECTOR = re.compile(r'<(\d+) x ([\w.]+)>')
SUBPROGRAM = re.compile(
    r'^!(\d+) = distinct !DISubprogram\(name: "(?:[^"\\]|\\.)*", linkageName: "([^"]+)"'
    r'.*?file: !(\d+)(?:, line: (\d+))?')
DIFILE = re.compile(r'^!(\d+) = !DIFile\(filename: "([^"]*)", directory: "([^"]*)"')
DEFINE = re.compile(r'^define ([^@]*)@(?:"((?:[^"\\]|\\.)*)"|([-\w.$]+))\(')
DECLARE = re.compile(r'^declare ([^@]*)@(?:"((?:[^"\\]|\\.)*)"|([-\w.$]+))(\(.*\))')
GLOBAL = re.compile(r'^@(?:"((?:[^"\\]|\\.)*)"|([-\w.$]+)) = ([^{\[<"]*)')
MIR_SPAN = re.compile(r'// .*? at (\S+?):(\d+):\d+: (\d+):\d+(?: \(#\d+\))?\s*$')
MIR_CALL = re.compile(r'= ([A-Za-z_][\w:]*?)(?:::<|\()')


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def write_rows(path, header, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('w') as stream:
        stream.write('\t'.join(header) + '\n')
        for row in rows:
            stream.write('\t'.join(str(cell) for cell in row) + '\n')


# ---------------------------------------------------------------- toolchain


def toolchain():
    """Pinned rustc/cargo paths, hashes and version text; rust-src pin from its manifest."""
    channel = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    tools = {}
    for name in ('rustc', 'cargo'):
        path = Path(subprocess.check_output(
            ['rustup', 'which', '--toolchain', channel, name], cwd=ROOT, text=True).strip())
        version = subprocess.check_output([str(path), '-vV'], text=True).strip()
        tools[name] = {'path': str(path), 'sha256': sha256_file(path), 'version': version}
    if f'commit-hash: {REVISION}' not in tools['rustc']['version']:
        raise SystemExit('pinned rustc is not revision ' + REVISION)
    sysroot = Path(tools['rustc']['path']).parents[1]
    manifest = tomllib.loads((sysroot / 'lib/rustlib/multirust-channel-manifest.toml').read_text())
    source = manifest['pkg']['rust-src']['target']['*']
    nm = subprocess.check_output(['nm', '--version'], text=True).splitlines()[0]
    return {'channel': channel, 'tools': tools, 'nm': nm,
            'rust_src': {'url': source['xz_url'], 'sha256': source['xz_hash'],
                         'version': manifest['pkg']['rust-src']['version']}}


def prepare_source(pin):
    """Fetch and verify the toolchain's own rust-src; nothing is installed into rustup.

    The archive hash comes from the installed channel manifest. Files the frontier
    lane pinned (`scripts/original_core_manifest.json`) must be byte-identical.
    """
    folder = WORK / 'rust-src'
    archive = folder / Path(pin['url']).name
    tree = folder / 'extracted'
    if not archive.exists():
        folder.mkdir(parents=True, exist_ok=True)
        request = urllib.request.Request(pin['url'], headers={'User-Agent': 'rust-src-fetch'})
        with urllib.request.urlopen(request, timeout=120) as response:
            archive.write_bytes(response.read())
    if sha256_file(archive) != pin['sha256']:
        raise SystemExit('rust-src archive differs from the toolchain manifest hash')
    if not tree.exists():
        with tarfile.open(archive) as bundle:
            bundle.extractall(tree, filter='data')
    top = next(tree.iterdir())
    if (top / 'git-commit-hash').read_text().strip() != REVISION:
        raise SystemExit('rust-src is not revision ' + REVISION)
    rust = top / 'rust-src/lib/rustlib/src/rust'
    pinned = json.loads((ROOT / 'scripts/original_core_manifest.json').read_text())
    same, absent = 0, []
    for row in pinned['files']:
        path = rust / row['path']
        if not path.is_file():
            absent.append(row['path'])
        elif sha256_file(path) != row['sha256']:
            raise SystemExit('rust-src differs from the pinned original: ' + row['path'])
        else:
            same += 1
    return rust, {'archive_sha256': pin['sha256'], 'revision': REVISION,
                  'pinned_manifest_files': len(pinned['files']), 'identical': same,
                  'absent_from_rust_src': sorted(absent)}


# ------------------------------------------------------------ rustc wrapper


def rustc_wrapper(argv):
    """RUSTC_WRAPPER: record each target-crate invocation and capture its mono items."""
    arguments = argv[1:]
    capture = os.environ.get('DEMAND_CAPTURE_DIR')
    name = next((arguments[i + 1] for i, a in enumerate(arguments[:-1]) if a == '--crate-name'), None)
    if not capture or name is None or '-' in arguments or '--target' not in arguments:
        os.execv(argv[0], argv)
    extra = next((a.split('=', 1)[1] for a in arguments if a.startswith('extra-filename=')), '')
    folder = Path(capture)
    record = {'crate': name, 'extra': extra, 'argv': argv, 'cwd': os.getcwd(),
              'env': {key: value for key, value in os.environ.items()
                      if key.startswith(('CARGO', 'OUT_DIR', 'RUSTC', 'STD_ENV', 'DEP_'))
                      or key in ('PATH', 'HOME', 'LD_LIBRARY_PATH', '__CARGO_TESTS_ONLY_SRC_ROOT')}}
    write_json(folder / 'invocations' / f'{name}{extra}.json', record)
    (folder / 'mono').mkdir(parents=True, exist_ok=True)
    with (folder / 'mono' / f'{name}{extra}.txt').open('wb') as stream:
        return subprocess.run(argv, stdout=stream, check=False, close_fds=False).returncode


# -------------------------------------------------------------------- build


def program_package(name, corpus):
    """Generate a throwaway Cargo package with one binary per consumer program."""
    folder = WORK / 'packages' / name
    folder.mkdir(parents=True, exist_ok=True)
    programs = sorted((ROOT / corpus['programs']).glob('*.rs'))
    if not programs:
        raise SystemExit(f'{name}: no programs under {corpus["programs"]}')
    lines = ['[package]', f'name = "demand-{name}"', 'version = "0.0.0"', 'edition = "2024"',
             'publish = false', '', '[workspace]', '', '[dependencies]']
    lines += [f'{crate} = {requirement}' for crate, requirement in corpus.get('dependencies', {}).items()]
    taken = LIBRARY_CRATES | {crate.replace('-', '_') for crate in corpus.get('dependencies', {})}
    for program in programs:
        if program.stem.replace('-', '_') in taken:
            raise SystemExit(f'{name}: {program.name} would shadow a library crate; rename it')
        lines += ['', '[[bin]]', f'name = "{program.stem}"', f'path = "{program}"']
    (folder / 'Cargo.toml').write_text('\n'.join(lines) + '\n')
    if corpus.get('dependencies') and not (folder / 'Cargo.lock').exists():
        # Start from the compiler's lockfile so shared dependencies keep its versions.
        (folder / 'Cargo.lock').write_text((ROOT / 'dev/Cargo.lock').read_text())
    return folder / 'Cargo.toml', [program.stem for program in programs]


def build(name, corpus, pin, library):
    """Build one corpus with build-std; returns the linked binaries."""
    if 'manifest' in corpus:
        manifest, bins, locked = ROOT / corpus['manifest'], corpus['bins'], ['--locked']
    else:
        (manifest, bins), locked = program_package(name, corpus), []
    # A frozen copy, so editing this script during a long build cannot break rustc calls.
    wrapper = WORK / 'rustc-wrapper.py'
    wrapper.write_text('#!/usr/bin/env python3\nimport json\nimport os\nfrom pathlib import Path\n'
                       'import subprocess\nimport sys\n\n\n'
                       + inspect.getsource(write_json) + '\n\n' + inspect.getsource(rustc_wrapper)
                       + '\n\nsys.exit(rustc_wrapper(sys.argv[1:]))\n')
    wrapper.chmod(0o755)
    environment = dict(os.environ)
    environment.update({f'CARGO_PROFILE_DEV_{key}': value for key, value in PROFILE.items()})
    environment.update({
        'RUSTC_BOOTSTRAP': '1', '__CARGO_TESTS_ONLY_SRC_ROOT': str(library),
        'CARGO_TARGET_DIR': str(WORK / 'target'), 'CARGO_INCREMENTAL': '0',
        'CARGO_ENCODED_RUSTFLAGS': '\x1f'.join(RUSTFLAGS), 'RUSTC_WRAPPER': str(wrapper),
        'RUSTC': pin['tools']['rustc']['path'], 'DEMAND_CAPTURE_DIR': str(WORK / 'capture'),
        'RUSTC_ICE': '0',
    })
    command = [pin['tools']['cargo']['path'], 'build', '--manifest-path', str(manifest),
               '--target', TARGET, *BUILD_STD, *locked]
    for item in bins:
        command += ['--bin', item]
    log = WORK / 'logs' / f'{name}.build.log'
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open('w') as stream:
        status = subprocess.run(command, env=environment, cwd=ROOT, stdout=stream,
                                stderr=subprocess.STDOUT, check=False).returncode
    if status:
        raise SystemExit(f'{name}: cargo build failed with status {status}; see {log}')
    if corpus.get('dependencies'):
        check_lock(name, manifest.with_name('Cargo.lock'))
    folder = WORK / 'target' / TARGET / 'debug'
    return [folder / item for item in bins], command


def locked_packages(path):
    """Registry packages of a lockfile as {name: (version, checksum)}."""
    return {package['name']: (package['version'], package.get('checksum'))
            for package in tomllib.loads(path.read_text()).get('package', []) if 'source' in package}


def check_lock(name, lock):
    """A consumer package must resolve to exactly the compiler's locked dependencies."""
    ours, theirs = locked_packages(lock), locked_packages(ROOT / 'dev/Cargo.lock')
    if ours != theirs:
        raise SystemExit(f'{name}: {lock} resolves {sorted(ours.items())}, '
                         f'not dev/Cargo.lock\'s {sorted(theirs.items())}')


# ---------------------------------------------------------------- crate graph


def bin_key(binary):
    """`<crate>-<unit hash>` of the unit Cargo hard-linked to this binary."""
    crate = binary.name.replace('-', '_')
    for candidate in sorted((binary.parent / 'deps').glob(crate + '-*')):
        if not candidate.suffix and candidate.samefile(binary):
            return candidate.name
    raise SystemExit(f'no built unit matches {binary}')


def invocation_graph(bins):
    """Invocation records of the binaries and of every rlib their link lines name.

    Cargo offers each binary crates it never links (`proc_macro`, the unused panic
    runtime); rustc's own `--print=link-args` output is the authority.
    """
    records = {}
    for path in sorted((WORK / 'capture/invocations').glob('*.json')):
        record = json.loads(path.read_text())
        records[record['crate'] + record['extra']] = record
    seen, libraries = {}, set()
    for binary in bins:
        key = bin_key(binary)
        seen[key] = records[key]
        line = next((text for text in (WORK / 'capture/mono' / f'{key}.txt').read_text(
            errors='replace').splitlines() if '.rlib"' in text and not text.startswith('MONO_ITEM')), '')
        for rlib in re.findall(r'/lib([^/"]+)\.rlib"', line):
            if rlib in records:
                seen[rlib] = records[rlib]
        libraries.update(re.findall(r'"(-l[^"]+)"', line))
    return seen, sorted(libraries)


def crate_types(record):
    argv = record['argv']
    return {argv[i + 1] for i, a in enumerate(argv[:-1]) if a == '--crate-type'}


def crate_inputs(key, record, relative):
    """Version, unit hash and a content hash over every source file rustc read."""
    deps = WORK / 'target' / TARGET / 'debug/deps' / f'{key}.d'
    sources = set()
    if deps.exists():
        for line in deps.read_text().splitlines():
            if line.endswith(':') and not line.startswith(('#', ' ')):
                sources.add(line[:-1].replace('\\ ', ' '))
    digest = hashlib.sha256()
    for source in sorted(sources):
        path = Path(source) if os.path.isabs(source) else Path(record['cwd']) / source
        if path.is_file():
            digest.update(relative(str(path)).encode() + b'\0' + sha256_file(path).encode() + b'\n')
    environment = record['env']
    return {'crate': record['crate'], 'version': environment.get('CARGO_PKG_VERSION', ''),
            'manifest_dir': relative(environment.get('CARGO_MANIFEST_DIR', '')),
            'unit': record['extra'].lstrip('-'), 'source_files': len(sources),
            'sources_sha256': digest.hexdigest(),
            'cfg': sorted(a for a in record['argv'] if a.startswith('feature='))}


def relativizer(library):
    """Stable names for source paths: `library/...`, `registry/...` or repo-relative."""
    rust = str(library.parent) + '/'
    vendor = str(library / 'vendor') + '/'
    root = str(ROOT) + '/'
    registry = re.compile(r'^.*/registry/src/[^/]+/')

    def relative(path):
        path = os.path.normpath(path) if path else path
        if path.startswith(vendor):
            return 'registry/' + path[len(vendor):]
        if path.startswith(rust):
            return path[len(rust):]
        if path.startswith('/rustc/' + REVISION + '/'):
            return path[len('/rustc/' + REVISION + '/'):]
        if registry.match(path):
            return 'registry/' + registry.sub('', path)
        if path.startswith(root):
            return path[len(root):]
        return path
    return relative


# ------------------------------------------------------------------ symbols


def symbol_table(binary):
    """Defined and undefined symbols of a linked binary, by nm type letter."""
    defined, undefined = {}, {}
    text = subprocess.check_output(['nm', str(binary)], text=True)
    for line in text.splitlines():
        parts = line.split()
        if len(parts) == 2:
            undefined[parts[1].split('@')[0]] = parts[0]
        elif len(parts) == 3:
            defined[parts[2]] = parts[1]
    return defined, undefined


# ----------------------------------------------------------------- LLVM IR

REACH = frozenset()


def set_reach(names):
    global REACH  # pylint: disable=global-statement
    REACH = names


def instruction_features(line, features):
    """Backend-visible operation classes of one instruction line."""
    text = line.strip()
    if text.startswith('%') and ' = ' in text:
        text = text.split(' = ', 1)[1]
    words = text.split()
    if not words or words[0] == ';' or re.fullmatch(r'i\d+', words[0]):
        return  # blank, rustc's callee comment or a switch case
    opcode = words[1] if words[0] in ('tail', 'musttail', 'notail') and len(words) > 1 else words[0]
    rest = [word for word in words[1:] if word not in ('volatile', 'weak', 'atomic')]
    if opcode in ('load', 'store') and len(words) > 1 and words[1] == 'atomic':
        kind = rest[0].rstrip(',')
        order = next((ORDERINGS[w.rstrip(',')] for w in words if w.rstrip(',') in ORDERINGS), '?')
        features.add(('atomic', f'{opcode} {kind} {order}'))
    elif opcode == 'atomicrmw':
        match = re.search(r'atomicrmw (?:volatile )?(\w+) ptr [^,]+, (\S+) .*? (\w+)(?:, align \d+)?(?:, !|$)', text)
        if match:
            features.add(('atomic', f'rmw-{match.group(1)} {match.group(2)} '
                          f'{ORDERINGS.get(match.group(3), match.group(3))}'))
    elif opcode == 'cmpxchg':
        match = re.search(r'cmpxchg (weak )?(?:volatile )?ptr [^,]+, (\S+) [^,]+, \S+ \S+ (\w+) (\w+)', text)
        if match:
            features.add(('atomic', f'cmpxchg{"-weak" if match.group(1) else ""} {match.group(2)} '
                          f'{ORDERINGS.get(match.group(3), "?")}/{ORDERINGS.get(match.group(4), "?")}'))
    elif opcode == 'fence':
        order = next((ORDERINGS[w.rstrip(',')] for w in words if w.rstrip(',') in ORDERINGS), '?')
        features.add(('atomic', 'fence ' + order))
    if opcode in FLOAT_OPS:
        kinds = [w.rstrip(',') for w in words[1:] if w.rstrip(',') in
                 ('half', 'float', 'double', 'fp128', 'x86_fp80') or re.fullmatch(r'i\d+,?', w)]
        features.add(('float', opcode + ' ' + '->'.join(dict.fromkeys(kinds))))
    vector = VECTOR.search(text)
    if vector:
        features.add(('vector', f'{opcode} <{vector.group(1)} x {vector.group(2)}>'))
    if 'i128' in text and opcode not in ('call', 'ret', 'br', 'phi', 'alloca', 'getelementptr',
                                         'extractvalue', 'insertvalue', 'load', 'store', 'select'):
        features.add(('i128', opcode))
    if opcode == 'call':
        if ' asm ' in text:
            match = re.search(r' asm (?:\w+ )*"((?:[^"\\]|\\.)*)"', text)
            features.add(('inline-asm', match.group(1)[:120] if match else '?'))
        variadic = re.search(r'\.\.\.\) @(?:"((?:[^"\\]|\\.)*)"|([-\w.$]+))', text)
        if variadic:
            features.add(('variadic-call', variadic.group(1) or variadic.group(2)))


def parse_module(path):
    """One LLVM IR module: facts about the functions the linker kept from it.

    A function with external linkage survived if the binary's symbol table names it.
    `#[inline(always)]` and similar local copies have internal linkage and the same
    name in every module that uses them, so a copy counts only when something kept
    in this module refers to it, directly or through private data such as vtables.
    """
    path = Path(path)
    bodies, data, declares, externs = {}, {}, {}, {}
    names, digests, dropped, tls, used = set(), set(), set(), set(), set()
    subprograms, files, module_asm = {}, {}, 0
    current = None
    with path.open(errors='replace') as stream:
        for line in stream:
            if current is not None:
                if line.startswith('}'):
                    current = None
                    continue
                if not current[0]:
                    continue
                if '@' in line:
                    for quoted, plain in SYMBOL.findall(line):
                        current[1].add(quoted or plain)
                if INTERESTING.search(line):
                    instruction_features(line, current[2])
                continue
            head = line[:1]
            if head == 'd':
                match = DEFINE.match(line)
                if match:
                    name = match.group(2) or match.group(3)
                    names.add(name)
                    digests.add(hashlib.blake2b(name.encode(), digest_size=8).digest())
                    first = match.group(1).split()[:1]
                    linkage = first[0] if first and first[0] in LINKAGES else 'external'
                    wanted = name in REACH
                    current = (wanted, set(), set())
                    if wanted:
                        debug = re.search(r' !dbg !(\d+)', line)
                        bodies[name] = (linkage, debug.group(1) if debug else None, current[1], current[2])
                    elif linkage in ('internal', 'private'):
                        dropped.add(name)
                    continue
                match = DECLARE.match(line)
                if match:
                    declares[match.group(2) or match.group(3)] = (
                        match.group(1).strip() + ' ' + match.group(4))
            elif head == '@':
                match = GLOBAL.match(line)
                if match:
                    name, flags = match.group(1) or match.group(2), match.group(3)
                    if 'thread_local' in flags:
                        tls.add(name)
                    if 'external' in flags or 'extern_weak' in flags:
                        externs[name] = 'extern_weak' if 'extern_weak' in flags else 'external'
                        continue
                    names.add(name)
                    references = set()
                    if 'ptr @' in line:
                        references = {q or p for q, p in SYMBOL.findall(line[match.end(2 if match.group(2) else 1):])}
                    if name in ('llvm.used', 'llvm.compiler.used'):
                        used |= references
                    else:
                        local = flags.split()[:1] in (['private'], ['internal'])
                        if local or name in REACH:
                            data[name] = (local, references)
            elif head == '!':
                match = SUBPROGRAM.match(line)
                if match:
                    subprograms[match.group(1)] = (match.group(2), match.group(3), match.group(4))
                    continue
                match = DIFILE.match(line)
                if match:
                    files[match.group(1)] = os.path.join(match.group(3), match.group(2))
            elif line.startswith('module asm'):
                module_asm += 1

    def is_local(name):
        return (name in bodies and bodies[name][0] in ('internal', 'private')) or (
            name in data and data[name][0])

    pending = [name for name in bodies if not is_local(name)] + [n for n in data if not is_local(n)]
    if pending:
        pending += sorted(used)
    kept = set()
    while pending:
        name = pending.pop()
        if name in kept or (name not in bodies and name not in data):
            continue
        kept.add(name)
        for reference in (bodies[name][2] if name in bodies else data[name][1]):
            if reference not in kept and is_local(reference):
                pending.append(reference)
    result, vanished = {}, set()
    for name in kept:
        if name not in bodies:
            continue
        linkage, debug, references, features = bodies[name]
        site = subprograms.get(debug)
        location = (files.get(site[1], ''), int(site[2] or 0)) if site else ('', 0)
        outside = sorted(r for r in references if r not in REACH and r not in names)
        vanished.update(r for r in references if r in dropped)
        result[name] = (linkage, location, outside, sorted(r for r in references if r in tls),
                        sorted(features))
    return {'module': path.name, 'digests': b''.join(sorted(digests)), 'functions': result,
            'declares': declares, 'externs': externs, 'tls': sorted(tls), 'module_asm': module_asm,
            'vanished': sorted(vanished)}


# -------------------------------------------------------------------- MIR


def intrinsic_declarations(library):
    """Every `#[rustc_intrinsic]` function in core, with whether it has a fallback body."""
    found = {}
    for path in sorted((library / 'core/src/intrinsics').rglob('*.rs')):
        text = path.read_text()
        module = 'simd::' if path.parts[-2:] in (('intrinsics', 'simd.rs'), ('simd', 'mod.rs')) else ''
        if path.name in ('mir.rs', 'bounds.rs', 'fallback.rs', 'gpu.rs'):
            module = path.stem + '::'
        for match in re.finditer(
                r'#\[rustc_intrinsic\](?:\s*(?:#\[[^\]]*\]|//[^\n]*))*\s*'
                r'pub (?:const )?(?:unsafe )?fn (\w+)(?:<[^{;]*?>)?\([^{;]*?(;|\{)', text, re.S):
            found[module + match.group(1)] = match.group(2) == '{'
    return found


def called_intrinsics(code, declared, bare):
    """Intrinsics a MIR statement calls.

    rustc prints a callee by its shortest unambiguous path, so an intrinsic shows as
    `ctpop::<u64>(..)` when no other public item shares the name and as
    `intrinsics::transmute::<..>(..)` or `std::intrinsics::simd::simd_eq(..)` otherwise.
    """
    found = []
    for callee in MIR_CALL.findall(code):
        prefix, _, name = callee.rpartition('::')
        if not prefix:
            if name in bare:
                found.append(bare[name])
        elif prefix.endswith('intrinsics') and name in declared:
            found.append(name)
        elif '::intrinsics::' in '::' + prefix + '::' and prefix.rsplit('::', 1)[-1] + '::' + name in declared:
            found.append(prefix.rsplit('::', 1)[-1] + '::' + name)
    return found


def remove_scratch(folder):
    """Remove one of this script's own MIR dump directories and nothing else."""
    owner = (WORK / 'mir-dump').resolve()
    if folder.is_symlink() or folder.resolve().parent != owner:
        raise SystemExit(f'refusing to remove {folder}: not a directory directly under {owner}')
    if folder.is_dir():
        shutil.rmtree(folder)


def mir_bodies(key, record, relative, declared):
    """Dump MIR around rustc's LowerIntrinsics pass for one crate and tabulate every body.

    `-Zdump-mir` names each file after the definition path (`option-{impl#0}-map`), the
    same path a v0 symbol encodes, so bodies match demanded functions exactly. The
    `before` dump still has every intrinsic call; the `after` dump has those that
    survive to code generation.
    """
    dump = WORK / 'mir-dump' / key
    remove_scratch(dump)
    (dump / 'out').mkdir(parents=True)
    argv, skip = [], False
    for argument in record['argv']:
        if skip or argument.startswith(('--emit', '--error-format', '--json')):
            skip = False
            continue
        skip = argument == '--out-dir'
        if not skip:
            argv.append(argument)
    argv += ['--emit=metadata', '-Zalways-encode-mir', '-Zdump-mir=LowerIntrinsics',
             f'-Zdump-mir-dir={dump}', '-Zdump-mir-exclude-pass-number', '-Zmir-include-spans=on',
             '--out-dir', str(dump / 'out')]
    environment = dict(os.environ)
    environment.update(record['env'])
    environment['RUSTC_ICE'] = '0'
    for name in ('CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS'):
        environment.pop(name, None)
    bare = Counter(name.rsplit('::', 1)[-1] for name in declared)
    bare = {name.rsplit('::', 1)[-1]: name for name in declared if bare[name.rsplit('::', 1)[-1]] == 1}
    log = WORK / 'logs' / f'mir-{key}.stderr'
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open('w') as errors:
        status = subprocess.run(argv, cwd=record['cwd'], env=environment, stdout=errors,
                                stderr=errors, check=False).returncode
    if status:
        raise SystemExit(f'{record["crate"]}: MIR dump failed with status {status}; see {log}')
    prefix, suffix = record['crate'] + '.', '.LowerIntrinsics.before.mir'
    bodies = []
    for path in sorted(dump.glob('*' + suffix)):
        definition = path.name[len(prefix):-len(suffix)].replace('-', '::')
        kind, file, first, last, before = read_body(path, record['cwd'], relative, declared, bare)
        after = read_body(path.with_name(path.name.replace('.before.', '.after.')),
                          record['cwd'], relative, declared, bare)[4]
        if kind == 'fn' and '{closure#' in definition:
            kind = 'closure'
        bodies.append((kind, definition, file, first, last, before, after))
    remove_scratch(dump)
    return bodies


def read_body(path, cwd, relative, declared, bare):
    """Kind, first-span file and line range, and intrinsic calls of one dumped body."""
    kind, file, first, last, intrinsics = '', '', 0, 0, set()
    with path.open(errors='replace') as stream:
        for line in stream:
            if not kind:
                if line.startswith(('fn ', 'const ', 'static ')):
                    kind = line.split(' ', 1)[0]
                elif line.strip() and not line.startswith('//'):
                    kind = 'other'
                continue
            span = MIR_SPAN.search(line)
            if span:
                here = relative(os.path.join(cwd, span.group(1)))
                if not file:
                    file, first = here, int(span.group(2))
                if here == file:
                    first, last = min(first, int(span.group(2))), max(last, int(span.group(3)))
            code = line.split(' // ', 1)[0]
            if ' = ' in code and '(' in code and not code.lstrip().startswith('//'):
                intrinsics.update(called_intrinsics(code, declared, bare))
    return kind, file, first, last, sorted(intrinsics)


def library_bodies(graph, wanted, relative, refresh, declared):
    """Cached MIR body tables for the library crates that supply demanded functions."""
    tables = {}
    for key, record in sorted(graph.items()):
        if record['crate'] not in wanted or 'bin' in crate_types(record):
            continue
        cache = WORK / 'mir' / f'{key}.bodies-v3.json'
        if refresh or not cache.exists():
            write_json(cache, mir_bodies(key, record, relative, declared))
        tables[record['crate']] = json.loads(cache.read_text())
    return tables


# ----------------------------------------------------------------- analysis


def analyse(name, corpus, bins, pin, library, source_check, command, refresh_mir, jobs):
    relative = relativizer(library)
    graph, libraries = invocation_graph(bins)
    defined, undefined, binaries = {}, {}, []
    for binary in bins:
        here, missing = symbol_table(binary)
        defined.update(here)
        undefined.update(missing)
        binaries.append({'name': binary.name, 'sha256': sha256_file(binary),
                         'bytes': binary.stat().st_size, 'symbols': len(here)})
    deps = WORK / 'target' / TARGET / 'debug/deps'
    modules = sorted(str(path) for key in graph for path in deps.glob(f'{key}*.ll'))
    # Functions rustc referenced from code the linker kept, but which are not in the
    # binary: their only callers sit in blocks LLVM's code generator dropped (constant
    # `ub_checks()` branches and the like). They are demanded unless such branches are
    # folded, so they are found by iterating and reported as `unlinked`.
    unlinked, rounds = set(), 0
    while True:
        rounds += 1
        with Pool(jobs, initializer=set_reach, initargs=(frozenset(defined) | unlinked,)) as pool:
            parsed = pool.map(parse_module, modules, chunksize=1)
        known = set()
        for module in parsed:
            blob = module['digests']
            known.update(blob[i:i + 8] for i in range(0, len(blob), 8))
        more = set()
        for module in parsed:
            more.update(module['vanished'])
            for facts in module['functions'].values():
                more.update(r for r in facts[2] if not r.startswith('llvm.') and r not in undefined
                            and hashlib.blake2b(r.encode(), digest_size=8).digest() in known)
        if more <= unlinked:
            break
        unlinked |= more

    module_crate = {}
    for key in graph:
        for path in deps.glob(f'{key}*.ll'):
            module_crate[path.name] = key
    functions, declares, tls_names, extern_data = {}, {}, set(), {}
    digests, module_asm = defaultdict(set), Counter()
    for module in parsed:
        unit = module_crate[module['module']]
        crate = graph[unit]['crate']
        blob = module['digests']
        digests[unit].update(blob[i:i + 8] for i in range(0, len(blob), 8))
        if module['functions']:
            module_asm[crate] += module['module_asm']
        declares.update(module['declares'])
        tls_names.update(module['tls'])
        extern_data.update(module['externs'])
        for symbol, facts in module['functions'].items():
            entry = functions.setdefault(symbol, {'crates': set(), 'site': ('', 0), 'outside': set(),
                                                  'tls': set(), 'features': set()})
            entry['crates'].add(crate)
            if facts[1][0]:
                entry['site'] = (relative(facts[1][0]), facts[1][1])
            entry['outside'].update(facts[2])
            entry['tls'].update(facts[3])
            entry['features'].update(tuple(item) for item in facts[4])

    own = set(corpus.get('own', [item.name.replace('-', '_') for item in bins]))
    instances, natives, statics, failures = {}, [], [], []
    for symbol, letter in sorted(list(defined.items()) + [(name, '-') for name in unlinked]):
        if not symbol.startswith('_R'):
            natives.append((symbol, letter, ','.join(sorted(functions.get(symbol, {}).get('crates', ())))))
            continue
        try:
            path, _, suffix = v0.parse(symbol)
            text = v0.render_path(path, True, [0])
        except (v0.Invalid, RecursionError) as error:
            failures.append((symbol, str(error)))
            continue
        crate = v0.defining_crate(path)
        if letter in 'tTwW-':
            row = instances.setdefault(text, {'crate': crate, 'kind': v0.classify(path), 'linked': False,
                                              'definition': v0.defining_path(path),
                                              'module': v0.module_path(path), 'site': ('', 0)})
            row['linked'] = row['linked'] or letter != '-'
            site = functions.get(symbol, {}).get('site', ('', 0))
            if site[0]:
                row['site'] = site
        else:
            statics.append((crate, text + suffix, letter, 'tls' if symbol in tls_names else ''))

    out = OUT / name
    write_rows(out / 'instances.tsv', ('crate', 'kind', 'reach', 'instance'),
               sorted((row['crate'], row['kind'], 'linked' if row['linked'] else 'unlinked', text)
                      for text, row in instances.items()))
    write_rows(out / 'statics.tsv', ('crate', 'static', 'nm', 'tls'), sorted(statics))

    sources = {}
    for text, row in instances.items():
        entry = sources.setdefault((row['crate'], row['definition']), {
            'kind': row['kind'], 'site': row['site'], 'instances': 0, 'linked': 0, 'example': text,
            'module': row['module']})
        entry['instances'] += 1
        entry['linked'] += row['linked']
        if row['site'][0] and not entry['site'][0]:
            entry['site'] = row['site']
        if (len(text), text) < (len(entry['example']), entry['example']):
            entry['example'] = text

    wanted = {crate for crate, _ in sources} - own
    declared = intrinsic_declarations(library)
    tables = library_bodies(graph, wanted, relative, refresh_mir, declared)
    indexes = {crate: {row[1]: row for row in table} for crate, table in tables.items()}
    intrinsic_users, surviving_users, unmatched = defaultdict(set), defaultdict(set), Counter()
    unmatched_names = []
    for (crate, definition), entry in sources.items():
        entry['end'] = 0
        if crate not in indexes or entry['kind'] in ('shim', 'drop-glue'):
            continue
        body = indexes[crate].get(definition.split('::', 1)[-1])
        if body is None:
            # No source body: a tuple constructor used as a function, or a shim the
            # compiler builds for a bodiless trait method (Clone, Fn* on fn pointers).
            entry['kind'] = 'builtin'
            unmatched[crate] += 1
            unmatched_names.append(definition)
            continue
        entry['end'] = body[4]
        if not entry['site'][0]:
            entry['site'] = (body[2], body[3])
        for intrinsic in body[5]:
            intrinsic_users[intrinsic].add(definition)
        for intrinsic in body[6]:
            surviving_users[intrinsic].add(definition)

    write_rows(out / 'source-functions.tsv',
               ('crate', 'kind', 'reach', 'definition', 'file', 'line', 'end_line', 'instances',
                'linked_instances', 'example'),
               sorted((crate, entry['kind'], 'linked' if entry['linked'] else 'unlinked', definition,
                       entry['site'][0], entry['site'][1], entry['end'], entry['instances'],
                       entry['linked'], entry['example'])
                      for (crate, definition), entry in sources.items() if crate not in own))

    summary = defaultdict(lambda: [0, 0, 0, 0])
    for row in instances.values():
        summary[(row['crate'], row['module'])][0] += 1
        summary[(row['crate'], row['module'])][1] += row['linked']
    for (crate, _), entry in sources.items():
        summary[(crate, entry['module'])][3 if entry['kind'] == 'closure' else 2] += 1
    write_rows(out / 'summary.tsv',
               ('crate', 'module', 'instances', 'linked_instances', 'source_functions', 'closures'),
               sorted((crate, module, *counts) for (crate, module), counts in summary.items()))

    totals = library_totals(tables, sources, own)
    write_rows(out / 'files.tsv', ('crate', 'file', 'demanded_functions', 'demanded_closures',
                                   'linked_functions', 'linked_closures',
                                   'existing_functions', 'existing_closures'), totals['files'])

    surface = runtime_surface(functions, instances, undefined, declares, extern_data,
                              natives, intrinsic_users, surviving_users, declared)
    write_rows(out / 'runtime-surface.tsv', ('section', 'item', 'detail', 'users', 'crates', 'example'),
               surface)

    text_symbols = [s for s, letter in defined.items() if letter in 'tTwW']
    unresolved = sorted({r for entry in functions.values() for r in entry['outside']
                         if not r.startswith('llvm.') and r not in undefined and r not in defined
                         and r not in unlinked})
    library_sources = [entry for (crate, _), entry in sources.items()
                       if crate not in own and entry['kind'] in ('fn', 'closure')]
    receipt = {
        'corpus': name, 'binaries': binaries, 'cargo_command': [relative(part) for part in command],
        'profile': PROFILE, 'rustflags': RUSTFLAGS, 'build_std': BUILD_STD, 'target': TARGET,
        'toolchain': pin, 'source': source_check, 'linked_native_libraries': libraries,
        'repository_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'script_sha256': {path.name: sha256_file(path) for path in
                          (Path(__file__).resolve(), Path(v0.__file__).resolve())},
        'crates': sorted((crate_inputs(key, record, relative) for key, record in graph.items()),
                         key=lambda row: (row['crate'], row['unit'])),
    }
    write_json(out / 'receipt.json', receipt)
    checks = {
        'instances': len(instances),
        'linked_instances': sum(row['linked'] for row in instances.values()),
        'instances_by_crate': count_by(instances.values(), lambda row: row['crate']),
        'instance_kinds': count_by(instances.values(), lambda row: row['kind']),
        'source_definitions_by_kind': count_by(
            (entry for (crate, _), entry in sources.items() if crate not in own), lambda entry: entry['kind']),
        'library': totals['crates'],
        'stdarch_source_functions': sum(e['site'][0].startswith('library/stdarch/') for e in library_sources),
        'portable_simd_source_functions': sum(
            e['site'][0].startswith('library/portable-simd/') for e in library_sources),
        'verification': {
            'undemangled_symbols': failures[:50],
            'text_symbols_in_binary': len(text_symbols),
            'text_symbols_without_reachable_ir': sorted(s for s in text_symbols if s not in functions),
            'unlinked_reference_rounds': rounds,
            'ir_references_unresolved': unresolved[:200],
            'units': unit_counts(graph, digests),
            'module_asm_blocks_in_linked_crates': {k: v for k, v in sorted(module_asm.items()) if v},
            'definitions_without_source_body': sorted(unmatched_names)[:80],
        },
        'intrinsics_declared': len(declared),
        'intrinsics_with_fallback_body': sum(1 for body in declared.values() if body),
    }
    write_json(out / 'checks.json', checks)
    return checks


def count_by(rows, key):
    return dict(sorted(Counter(key(row) for row in rows).items()))


def library_totals(tables, sources, own):
    """Existing bodies (MIR oracle) against demanded source functions, per file and crate."""
    existing = defaultdict(lambda: [0, 0])
    kinds = defaultdict(Counter)
    for crate, table in tables.items():
        for kind, _, file, _, _, _, _ in table:
            kinds[crate][kind] += 1
            if kind in ('fn', 'closure'):
                existing[(crate, file)][kind == 'closure'] += 1
    demanded = defaultdict(lambda: [0, 0, 0, 0])
    per_crate = defaultdict(lambda: [0, 0, 0, 0])
    builtin = Counter()
    for (crate, _), entry in sources.items():
        if crate in own:
            continue
        if entry['kind'] not in ('fn', 'closure'):
            builtin[crate] += 1
            continue
        closure = entry['kind'] == 'closure'
        for counts in (demanded[(crate, entry['site'][0])], per_crate[crate]):
            counts[closure] += 1
            counts[2 + closure] += bool(entry['linked'])
    files = sorted((crate, file, *demanded.get((crate, file), [0, 0, 0, 0]),
                    *existing.get((crate, file), [0, 0]))
                   for crate, file in set(demanded) | set(existing))
    crates = {crate: {'demanded_functions': per_crate[crate][0], 'demanded_closures': per_crate[crate][1],
                      'linked_functions': per_crate[crate][2], 'linked_closures': per_crate[crate][3],
                      'existing_functions': kinds[crate]['fn'], 'existing_closures': kinds[crate]['closure'],
                      'demanded_without_source_body': builtin[crate],
                      'const_and_static_bodies': kinds[crate]['const'] + kinds[crate]['static']}
              for crate in sorted(tables)}
    return {'files': files, 'crates': crates}


def unit_counts(graph, digests):
    """Per compiled unit: `MONO_ITEM fn` lines against distinct functions defined in its IR."""
    counts = {}
    for key in sorted(graph):
        path = WORK / 'capture/mono' / f'{key}.txt'
        items = 0
        if path.exists():
            with path.open(errors='replace') as stream:
                items = sum(1 for line in stream if line.startswith('MONO_ITEM fn '))
        counts[key] = {'mono_item_functions': items, 'ir_defined_functions': len(digests.get(key, ()))}
    return counts


def runtime_surface(functions, instances, undefined, declares, extern_data, natives,
                    intrinsic_users, surviving_users, declared):
    """Rows naming what the reachable code needs from the backend and the platform."""
    def show(symbol):
        try:
            return v0.demangle(symbol)
        except (v0.Invalid, RecursionError):
            return symbol

    grouped = defaultdict(lambda: defaultdict(set))
    foreign = defaultdict(set)
    for symbol, entry in functions.items():
        for section, item in entry['features']:
            grouped[section][item].add(symbol)
        for reference in entry['outside']:
            if reference.startswith('llvm.'):
                grouped['llvm-name'][reference].add(symbol)
            else:
                foreign[reference].add(symbol)
        for reference in entry['tls']:
            grouped['thread-local'][show(reference)].add(symbol)
    rows = []
    for section in sorted(grouped):
        for item, users in sorted(grouped[section].items()):
            crates = sorted({crate for user in users for crate in functions[user]['crates']})
            rows.append((section, item, '', len(users), ','.join(crates), show(min(users, key=len))))
    for symbol, letter in sorted(undefined.items()):
        users = foreign.get(symbol, set())
        crates = sorted({crate for user in users for crate in functions[user]['crates']})
        detail = ('weak ' if letter in 'wv' else '') + (declares.get(symbol) or extern_data.get(symbol)
                                                        or 'no IR reference: codegen or startup libcall')
        rows.append(('undefined-symbol', symbol, detail, len(users), ','.join(crates),
                     show(min(users, key=len)) if users else ''))
    for symbol, letter, crates in natives:
        rows.append(('unmangled-defined-symbol', symbol, 'nm ' + letter, '', crates, ''))
    for intrinsic, users in sorted(intrinsic_users.items()):
        known = intrinsic in declared
        detail = ('fallback body' if declared.get(intrinsic) else 'no fallback body') if known \
            else 'not a #[rustc_intrinsic] item'
        detail += f'; reaches codegen in {len(surviving_users.get(intrinsic, ()))}'
        rows.append(('rust-intrinsic-call', 'core::intrinsics::' + intrinsic, detail, len(users), '',
                     min(users, key=len)))
    arch = defaultdict(list)
    for text, row in instances.items():
        definition = row['definition']
        for prefix, section in (('core::core_arch::', 'stdarch-function'), ('core::core_simd::', 'portable-simd-function'),
                                ('core::intrinsics::', 'intrinsic-fallback-instance'),
                                ('core::sync::atomic::', 'atomic-instance'),
                                ('compiler_builtins::', 'compiler-builtins-function')):
            if definition.startswith(prefix):
                arch[(section, definition)].append(text)
    for (section, definition), texts in sorted(arch.items()):
        rows.append((section, definition, '', len(texts), '', min(texts, key=len)))
    return rows


# --------------------------------------------------------------------- main


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    parser.add_argument('corpus', nargs='+', help='one of: ' + ', '.join(sorted(CORPORA)))
    parser.add_argument('--adhoc', action='append', default=[], metavar='NAME=DIRECTORY',
                        help='extra corpus: every *.rs program in a repository directory')
    parser.add_argument('--no-build', action='store_true', help='reuse the existing build')
    parser.add_argument('--build-only', action='store_true', help='build without analysing')
    parser.add_argument('--refresh-mir', action='store_true', help='recompute cached MIR body tables')
    parser.add_argument('--jobs', type=int, default=8)
    arguments = parser.parse_args()
    pin = toolchain()
    rust, source_check = prepare_source(pin['rust_src'])
    library = rust / 'library'
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / '.keep-artifacts').touch()
    for item in arguments.adhoc:
        name, _, folder = item.partition('=')
        CORPORA[name] = {'programs': folder}
    for name in arguments.corpus:
        corpus = CORPORA[name]
        if arguments.no_build:
            bins = corpus.get('bins') or [p.stem for p in sorted((ROOT / corpus['programs']).glob('*.rs'))]
            binaries, command = [WORK / 'target' / TARGET / 'debug' / item for item in bins], []
        else:
            binaries, command = build(name, corpus, pin, library)
            write_json(WORK / 'logs' / f'{name}.command.json', command)
        if arguments.build_only:
            continue
        if not command and (WORK / 'logs' / f'{name}.command.json').exists():
            command = json.loads((WORK / 'logs' / f'{name}.command.json').read_text())
        checks = analyse(name, corpus, binaries, pin, library, source_check, command,
                         arguments.refresh_mir, arguments.jobs)
        print(json.dumps({'corpus': name, 'instances': checks['instances'], 'library': checks['library']},
                         sort_keys=True))


if __name__ == '__main__':
    main()
