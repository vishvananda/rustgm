#!/usr/bin/env python3
"""Compile a no_std consumer in one session with real core, then run it.

Bring-up only. The consumer is mounted as a private module of a writable copy
of core's sources, so that no metadata crosses a crate boundary: the session
compiles core and the program together as one executable. The exported C
``main`` of the program becomes ``guest_main``, called by a Rust ``main`` that
aborts on a nonzero status; the crate-level ``no_std``/``no_main`` attributes
are dropped. Nothing produced here is a qualification.

For each program the driver grows a copy of the core skip list with
``bringup_census.py`` until the session compiles, builds the LowIR through the
pinned backend (the object bridge when the program links native libraries),
runs it and compares its standard output with the recorded rustc output.

Run under the resource and storage guards. ``--work`` must be under ``obj/``.
"""
import argparse
import pathlib
import shutil
import subprocess
import sys

SCRIPTS = pathlib.Path(__file__).resolve().parent
ENTRY = '#[unsafe(no_mangle)]\npub extern "C" fn main('
GUEST_ENTRY = 'pub(crate) extern "C" fn guest_main('
ADAPTER = '''
pub(crate) fn main() {
    if guest_main(0, core::ptr::null()) != 0 {
        unsafe { abort() }
    }
}
'''
MOUNT = '\n#[path = "../../../app/main.rs"]\nmod __guest_app;\nuse __guest_app::main;\n'
CRATE = "core_1_98_88d9_bringup"
FLAGS = [
    "--cap-lints=allow",
    "--source-contract-work-limit=14500624",
    "--source-contract-attempted-bytes=1073741824",
    "--source-contract-retained-bytes=67108864",
    "--emit-lowir", "--edition=2024",
    "--crate-name", "core", "--crate-disambiguator", CRATE,
    "--source-core", f"core@{CRATE}",
]


def guest(text):
    """Return the program as a guest module with a Rust entry point."""
    kept = "".join(line for line in text.splitlines(keepends=True)
                   if line.strip() not in ("#![no_std]", "#![no_main]"))
    if kept.count(ENTRY) != 1:
        raise ValueError("expected exactly one exported C main")
    if "fn abort() -> !" not in kept:
        raise ValueError("the entry adapter needs the program's own abort declaration")
    return kept.replace(ENTRY, GUEST_ENTRY) + ADAPTER


def host(text):
    """Return core's root with the guest module mounted and its main imported."""
    return text + MOUNT


def rows(path):
    """Return the (path, offset) keys and full rows of a skip list."""
    found = {}
    for row in path.read_text().splitlines():
        if row and not row.startswith("#"):
            fields = row.split("\t")
            found[(fields[0], fields[1])] = row
    return found


def prepare(work, library, program):
    """Copy core's sources once, then mount this program in the copy."""
    target = work / "library"
    if not target.is_dir():
        shutil.copytree(library, target)
    root = target / "core" / "src" / "lib.rs"
    root.chmod(0o644)
    root.write_text(host((library / "core" / "src" / "lib.rs").read_text()))
    (work / "app").mkdir(exist_ok=True)
    (work / "app" / "main.rs").write_text(guest(program.read_text()))
    return root


def native(out, expected):
    """Build and run the program; return its status and whether stdout matches."""
    sys.path.insert(0, str(SCRIPTS))
    import backend
    plan = out / "app.native-links.json"
    binary = out / "app"
    backend.build_many([out / "app.lowir"], binary,
                       native_plan=plan if plan.is_file() else None)
    result = subprocess.run([str(binary)], capture_output=True, timeout=60, check=False)
    (out / "app.stdout").write_bytes(result.stdout)
    return result.returncode, result.stdout == expected


def run(options, program):
    """Census, build and run one program; return its summary lines and verdict."""
    name = program.stem
    out = options.work / name
    out.mkdir(parents=True, exist_ok=True)
    root = prepare(options.work, options.library, program)
    skip = out / "skip.tsv"
    shutil.copyfile(options.skip_list, skip)
    census = subprocess.run(
        [sys.executable, str(SCRIPTS / "bringup_census.py"),
         "--compiler", str(options.compiler), "--skip-list", str(skip), "--out", str(out),
         "--timeout", str(options.timeout), "--max-iterations", str(options.max_iterations),
         "--", *FLAGS, "--native-links-output", str(out / "app.native-links.json"),
         "-o", str(out / "app.lowir"), str(root)],
        capture_output=True, text=True, check=False)
    (out / "census.log").write_text(census.stdout + census.stderr)
    base = rows(options.skip_list)
    extra = [row for key, row in rows(skip).items() if key not in base]
    waived = (out / "waived.tsv").read_text().splitlines() if (out / "waived.tsv").exists() else []
    report = [line for line in census.stdout.splitlines()
              if line.startswith(("outcome:", "last failure:", "unattributed:", "repeated:"))]
    lines = [f"### {name}"] + [f"census {line}" for line in report or ["gave no outcome"]]
    lines += [f"  stub: {row}" for row in extra]
    lines += [f"  waived: {fence}" for fence in waived]
    passed = False
    if census.returncode == 0:
        try:
            status, same = native(out, program.with_suffix(".stdout").read_bytes())
            passed = status == 0 and same
            lines.append(f"native: exit {status}; stdout {'matches' if same else 'differs'}")
        except (ValueError, subprocess.TimeoutExpired) as error:
            lines.append(f"native: {str(error).strip().splitlines()[-1][:300]}")
    return lines, passed


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--compiler", required=True, type=pathlib.Path)
    parser.add_argument("--library", required=True, type=pathlib.Path,
                        help="frozen library/ directory holding core, stdarch and portable-simd")
    parser.add_argument("--skip-list", required=True, type=pathlib.Path,
                        help="the skip list with which core alone compiles")
    parser.add_argument("--work", required=True, type=pathlib.Path)
    parser.add_argument("--timeout", type=float, default=900.0)
    parser.add_argument("--max-iterations", type=int, default=30)
    parser.add_argument("programs", nargs="+", type=pathlib.Path)
    options = parser.parse_args()
    options.work.mkdir(parents=True, exist_ok=True)
    summary, failed = [], 0
    for program in options.programs:
        lines, passed = run(options, program)
        failed += not passed
        summary += lines
        print("\n".join(lines), flush=True)
    (options.work / "summary.txt").write_text("\n".join(summary) + "\n")
    print(f"passed {len(options.programs) - failed} of {len(options.programs)}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
