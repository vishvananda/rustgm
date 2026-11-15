#!/usr/bin/env python3
"""Runtime gap probes: observed evidence for the tracker's runtime and backend gap table.

Each probe is a few lines of Rust for the RustGM frontend, or of hand-written
LowIR for the pinned backend, with the expected observation declared beside
it. A frontend probe records the compiler's exit status and diagnostic. A
backend probe is built twice: by the standalone tools (`lowir`, then
`lowir2native -O0`) and by the object bridge (`cppgm++ -c`, then the system
`cc`), and records the failing tool's message or the program's exit status.

The script fails when an observation no longer contains its declaration, so a
closed gap or a new backend pin shows up here before the tracker goes stale.
Frontend probes run under the ordinary library profile; they test attribute
and entry handling, which does not depend on the library.

Run it through the resource and storage guards:

    scripts/runtime_gap_probes.py --app PATH_TO_RUSTGM
"""
import argparse
from pathlib import Path
import subprocess

import backend

ROOT = backend.ROOT
WORK = ROOT / 'obj/parallel/runtime-gaps'
UNRESOLVED = 'unresolved attribute macro is not in path scope'
PASS = 'exit 0'

# name, Rust source, declared part of the observation
FRONTEND = (
    ('control-main', 'fn main() {}\n', 'status 0'),
    ('main-returns-result', 'fn main() -> Result<(), ()> {\n    Ok(())\n}\n',
     'main must be safe, parameterless and return () or !'),
    ('no-main', '#![no_main]\n#[unsafe(no_mangle)]\n'
     'pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {\n    0\n}\n', UNRESOLVED),
    ('global-allocator', 'struct Heap;\n#[global_allocator]\nstatic HEAP: Heap = Heap;\nfn main() {}\n', UNRESOLVED),
    ('thread-local', '#![feature(thread_local)]\n#[thread_local]\nstatic mut COUNT: u32 = 0;\nfn main() {}\n',
     UNRESOLVED),
    ('extern-weak', '#![feature(linkage)]\nunsafe extern "C" {\n    #[linkage = "extern_weak"]\n'
     '    static getrandom: *const u8;\n}\nfn main() {}\n', UNRESOLVED),
    ('link-section', '#[unsafe(link_section = ".init_array")]\nstatic ENTRY: usize = 0;\nfn main() {}\n', UNRESOLVED),
    ('used', '#[used]\nstatic KEPT: u8 = 1;\nfn main() {}\n', UNRESOLVED),
)

ENTRY = 'function @main() -> i64 [role=entry] {\n'
VERDICT = ('    branch %ok, ^pass, ^fail\n  block ^pass:\n    return i64 0\n'
           '  block ^fail:\n    return i64 1\n}\n')

# name, LowIR, declared observation of the standalone tools, declared observation of the object bridge
BACKEND = (
    ('libc-write',
     'global @text : obj<3x1> [storage=readonly] = {\n  i8 104\n  i8 105\n  i8 10\n}\n'
     'declare function @write(%fd : i32, %data : ptr, %length : i64) -> i64\n' + ENTRY +
     '  block ^b0:\n    %p = addr @text\n    %n = call i64 @write(1, %p, 3)\n    %ok = cmp eq i64 %n, 3\n' + VERDICT,
     'undefined symbol: write', "exit 0 stdout b'hi\\n'"),
    ('variadic-open',
     'global @path : obj<10x1> [storage=readonly] = {\n  i8 47\n  i8 100\n  i8 101\n  i8 118\n  i8 47\n'
     '  i8 110\n  i8 117\n  i8 108\n  i8 108\n  i8 0\n}\n'
     'declare function @open(%path : ptr, %flags : i32) -> i32 [arity=variadic]\n'
     'declare function @close(%fd : i32) -> i32\n' + ENTRY +
     '  block ^b0:\n    %p = addr @path\n    %fd = call i32 @open(%p, 0, 0)\n    %ok = cmp ge i32 %fd, 0\n'
     '    branch %ok, ^pass, ^fail\n  block ^pass:\n    %c = call i32 @close(%fd)\n'
     '    %r = convert sext i64 i32 %c\n    return i64 %r\n  block ^fail:\n    return i64 1\n}\n',
     'undefined symbols: open', PASS),
    ('init-array-section',
     'global @seen : i64 = 0\n'
     'function @hook(%argc : i32, %argv : ptr, %envp : ptr) -> void {\n  block ^b0:\n    %p = addr @seen\n'
     '    %n = convert sext i64 i32 %argc\n    store i64 %n, %p\n    return void\n}\n'
     'global @entry : ptr [section=.init_array] = addr @hook\n' + ENTRY +
     '  block ^b0:\n    %p = addr @seen\n    %v = load i64 %p\n    %r = binary sub i64 %v, 1\n    return i64 %r\n}\n',
     'exit 255', PASS),
    ('weak-undefined',
     'declare function @rustgm_probe_absent() -> void [binding=weak]\n' + ENTRY +
     '  block ^b0:\n    %p = addr @rustgm_probe_absent\n    %ok = cmp eq ptr %p, nullptr\n' + VERDICT,
     PASS, PASS),
    ('thread-local-global',
     'global @counter : i64 [storage=thread_local] = 7\n' + ENTRY +
     '  block ^b0:\n    %p = addr @counter\n    %v = load i64 %p\n    %w = binary add i64 %v, 5\n'
     '    store i64 %w, %p\n    %x = load i64 %p\n    %ok = cmp eq i64 %x, 12\n' + VERDICT,
     PASS, PASS),
    ('sse2-byte-compare',
     'global @left : obj<16x16> [storage=readonly] = {\n  i64 72340172838076673\n  i64 144680345676153346\n}\n'
     'global @right : obj<16x16> [storage=readonly] = {\n  i64 72340172838076673\n  i64 217020518514230019\n}\n'
     + ENTRY +
     '  block ^b0:\n    %lp = addr @left\n    %rp = addr @right\n    %l = load v128 %lp\n    %r = load v128 %rp\n'
     '    %e = vector pcmpeqb %l, %r\n    %m = vector pmovmskb %e\n    %ok = cmp eq i32 %m, 255\n' + VERDICT,
     PASS, PASS),
    ('i128-division',
     ENTRY + '  slot $n : i128\n  block ^b0:\n    store i128 1000000000000000000000, $n\n    %a = load i128 $n\n'
     '    %q = binary udiv i128 %a, 1000000000000\n    %ok = cmp eq i128 %q, 1000000000\n' + VERDICT,
     PASS, PASS),
    ('float-remainder',
     ENTRY + '  slot $x : f64\n  block ^b0:\n    store f64 7.5e0, $x\n    %a = load f64 $x\n'
     '    %r = binary mod f64 %a, 2.0e0\n    %ok = cmp eq f64 %r, 1.5e0\n' + VERDICT,
     'unsupported floating binary operation', 'unsupported floating binary operation'),
    ('image-over-2gib',
     'global @first : obj<1200000000x8> = {\n  zero 1200000000\n}\n'
     'global @second : obj<1200000000x8> = {\n  zero 1200000000\n}\n' + ENTRY +
     '  block ^b0:\n    %p = addr @first\n    %q = addr @second\n    %a = load i8 %p\n    %b = load i8 %q\n'
     '    %s = binary add i8 %a, %b\n    %r = convert sext i64 i8 %s\n    return i64 %r\n}\n',
     'PC-relative reference out of range', PASS),
    ('count-zeros-i8',
     ENTRY + '  slot $x : i8\n  block ^b0:\n    store i8 16, $x\n    %a = load i8 $x\n    %z = unary ctz i8 %a\n'
     '    %r = convert sext i64 i8 %z\n    return i64 %r\n}\n',
     'invalid ctz/clz type', 'invalid ctz/clz type'),
)


def frontend(app, source):
    result = subprocess.run([str(app), '--emit-lowir', '--edition=2024'], input=source.encode(),
                            capture_output=True, timeout=60, cwd=ROOT, check=False)
    return f'status {result.returncode} {result.stderr.decode(errors="replace").strip()}'.strip()


def built(build, source, output):
    """The build failure, or what the built program does."""
    try:
        build(source, output)
    except ValueError as error:
        return ' '.join(str(error).split())
    result = subprocess.run([str(output)], capture_output=True, timeout=10, check=False)
    return f'exit {result.returncode}' + (f' stdout {result.stdout!r}' if result.stdout else '')


def clear(folder):
    """Remove what one probe built; the folder holds nothing else."""
    for path in folder.iterdir():
        path.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    parser.add_argument('--app', required=True, type=Path, help='RustGM compiler binary for the frontend probes')
    arguments = parser.parse_args()
    backend.load_tools(('lowir', 'lowir2native', 'cppgm++'))
    scratch = WORK / 'scratch'
    scratch.mkdir(parents=True, exist_ok=True)
    rows = [(name, 'frontend', declared, frontend(arguments.app, source)) for name, source, declared in FRONTEND]
    for name, text, standalone, bridge in BACKEND:
        source = scratch / f'{name}.lowir'
        for path, build, declared in (('standalone', backend.build, standalone),
                                      ('object-bridge', lambda unit, output: backend.build_objects([unit], output),
                                       bridge)):
            clear(scratch)
            source.write_text(text)
            rows.append((name, path, declared, built(build, source, scratch / name)))
    clear(scratch)
    scratch.rmdir()
    differing = 0
    with (WORK / 'observations.tsv').open('w') as out:
        out.write('probe\tpath\tagrees\tdeclared\tobserved\n')
        for name, path, declared, observed in rows:
            agrees = declared in observed
            differing += not agrees
            out.write(f'{name}\t{path}\t{"yes" if agrees else "NO"}\t{declared}\t{observed}\n')
            print(f'{"ok  " if agrees else "DIFF"} {name} [{path}]: {observed[:150]}')
    print(f'{len(rows)} observations, {differing} differ from their declaration')
    raise SystemExit(1 if differing else 0)


if __name__ == '__main__':
    main()
