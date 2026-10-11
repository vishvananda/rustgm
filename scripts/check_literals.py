#!/usr/bin/env python3
"""Evaluate PA2 vectors with pinned rustc, independently of reference generation."""
import argparse
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import subprocess
import sys

from course import execute, load_cases

ROOT = Path(__file__).resolve().parents[1]
QUOTE = r'''
fn quoted(bytes: &[u8]) {
    print!("\"");
    for b in bytes {
        match b {
            b'"' => print!("\\\""), b'\\' => print!("\\\\"),
            b'\n' => print!("\\n"), b'\r' => print!("\\r"), b'\t' => print!("\\t"),
            32..=126 => print!("{}", char::from(*b)), _ => print!("\\x{b:02x}"),
        }
    }
    println!("\"");
}
'''


def runtime_statement(case, source):
    """Only the output adapter is shared in concept; rustc evaluates the value."""
    metadata = case["rustc"]
    kind = metadata["kind"]
    literal = source.removesuffix("\n")
    end = len(literal.encode())
    prefix = f"{kind} 0 {end}"
    ty = metadata.get("type")
    if kind == "integer":
        declared = "u128" if ty == "infer" else ty
        statement = f'let v: {declared} = {literal}; println!("{prefix} {ty} {{}}", v);'
    elif kind == "float":
        width = 8 if ty == "f32" else 16
        statement = f'let v: {ty} = {literal}; println!("{prefix} {ty} 0x{{:0{width}x}}", v.to_bits());'
    elif kind == "character":
        statement = f'println!("{prefix} U+{{:06X}}", u32::from({literal}));'
    elif kind == "byte":
        statement = f'println!("{prefix} 0x{{:02x}}", {literal});'
    elif kind == "boolean":
        statement = f'println!("{prefix} {{}}", {literal});'
    elif kind in ("string", "byte-string", "c-string"):
        expression = f"({literal}).as_bytes()" if kind == "string" else (
            f"({literal}).to_bytes_with_nul()" if kind == "c-string" else literal)
        statement = f'let v: &[u8] = {expression}; print!("{prefix} {{}} ", v.len()); quoted(v);'
    else:
        return None
    return "{ " + statement + f' println!("eof {len(source.encode())}");' + " }"


def rustc(source, output, metadata=False):
    command = ["rustc", "--edition=2024", "--crate-name", "literal_check",
               "--cap-lints", "allow", "-o", str(output), str(source)]
    if metadata:
        command.insert(1, "--emit=metadata")
    result = subprocess.run(command, capture_output=True, timeout=30)
    output.with_suffix(".stderr").write_bytes(result.stderr)
    return result


def check(app):
    version = subprocess.check_output(["rustc", "--version"], text=True)
    if not version.startswith("rustc 1.98.0 "):
        raise ValueError(f"expected rustc 1.98.0, got {version.strip()}")
    _, cases = load_cases(ROOT, "pa2")
    directory = ROOT / "obj/rustc-literals"
    directory.mkdir(parents=True, exist_ok=True)
    program = ["#![allow(overflowing_literals)]", QUOTE, "fn main() {"]
    values = []
    outcomes = []
    excluded = []
    for case in cases:
        path = ROOT / "pa2" / case["path"]
        # read_bytes preserves bare CR and NUL rejection fixtures.
        source = path.read_bytes().decode()
        statement = runtime_statement(case, source)
        if statement is not None:
            values.append(case)
            program.append(statement)
        elif case["rustc"]["kind"] in ("accept", "reject"):
            outcomes.append((case, source))
        else:
            excluded.append(case)
    program.append("}")
    source_path = directory / "values.rs"
    source_path.write_text("\n".join(program) + "\n")
    binary = directory / "values"
    result = rustc(source_path, binary)
    if result.returncode:
        raise ValueError(f"rustc value program failed: {result.stderr.decode(errors='replace')}")
    result = subprocess.run([str(binary)], capture_output=True, timeout=30)
    if result.returncode or result.stderr:
        raise ValueError("rustc value program failed at runtime")
    (directory / "values.stdout").write_bytes(result.stdout)
    lines = result.stdout.splitlines(keepends=True)
    if len(lines) != len(values) * 2 or not values:
        raise ValueError("unexpected independent value output shape")
    for index, case in enumerate(values):
        fixture = ROOT / "pa2" / case["path"]
        expected = b"".join(lines[2 * index:2 * index + 2])
        actual = execute([str(app), "--emit-literals"], fixture.read_bytes(), 10)
        if actual != (expected, b"", b"0\n"):
            raise ValueError(f"value disagreement: {case['path']}:\nrustc {expected!r}\nrustgm {actual!r}")
        if Path(str(fixture) + ".ref").read_bytes() != expected:
            raise ValueError(f"committed oracle disagrees with rustc: {case['path']}")

    def outcome(item):
        case, expression = item
        name = Path(case["path"]).parent.name + "-" + Path(case["path"]).stem
        source = directory / (name + ".rs")
        source.write_text("#![allow(overflowing_literals)]\nfn main() { let _ = " + expression + "; }\n")
        result = rustc(source, directory / (name + ".rmeta"), metadata=True)
        if result.returncode != case["exit_status"]:
            raise ValueError(f"independent rustc outcome disagrees: {case['path']}: {result.returncode}")
        actual = execute([str(app), "--emit-literals"], expression.encode(), 10)
        if actual[2] != f"{case['exit_status']}\n".encode():
            raise ValueError(f"candidate outcome disagrees: {case['path']}")

    with ThreadPoolExecutor(max_workers=4) as pool:
        list(pool.map(outcome, outcomes))
    print(f"PASS: {len(values)} independent rustc value/bit comparisons; {len(outcomes)} expression outcomes")
    print(f"{len(excluded)} stream/upstream cases excluded here; upstream programs use check_rustc.py")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, default=ROOT / "dev/rustgm")
    try:
        check(parser.parse_args().app.resolve())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
