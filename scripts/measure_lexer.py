#!/usr/bin/env python3
"""Retain raw scaling evidence; optionally compare immutable A/B binaries."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
LINE = b'fn measured(value: u64) -> u64 { value + 17 }\n'
LITERAL_LINE = br'''42u64 0.1f32 'x' '\u{1f980}' b'\xff' "plain" "escaped\n" br##"raw"## c"\xe6"''' + b"\n"
AST_LINE = b"let value: u64 = 1 + 2 * 3; value;\n"
CRATE_LINE = b"#[cfg_attr(unix, allow(unused))] fn measured<T: Copy>(value: T) -> T { value }\n"
TREE_LINE = b'#[cfg_attr(unix, allow(unused))] fn measured(x: u64) -> u64 { x + 17 }\n'


def sample(binary, source, destination, stage):
    started = time.perf_counter_ns()
    result = subprocess.run(["/usr/bin/time", "-f", "%M", "-o", str(destination),
                             str(binary), f"--count-{stage}", str(source)],
                            capture_output=True, timeout=120)
    elapsed = time.perf_counter_ns() - started
    if result.returncode or result.stderr:
        raise ValueError(f"measurement failed: {result.stderr!r}")
    return {"elapsed_ns": elapsed, "peak_rss_kb": int(destination.read_text()),
            "output": result.stdout.decode(), "output_sha256": hashlib.sha256(result.stdout).hexdigest()}


def full_output_hash(binary, source, directory, stage):
    path = directory / "rendered-output.tmp"
    with path.open("wb") as stream:
        result = subprocess.run([str(binary), f"--emit-{stage}", str(source)],
                                stdout=stream, stderr=subprocess.PIPE, timeout=120)
    if result.returncode or result.stderr:
        raise ValueError("full rendering failed during equivalence validation")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    path.unlink()
    return digest.hexdigest()


def float_pattern_source(lines, mode):
    declarations = "const LO:f64=-1.0/0.0;const HI:f64=1.0/0.0;enum E{A([f64;2]),B}\n"
    if mode == "scalar":
        parameter = "x:f64"
        body = "match x{0.0=>1,LO..0.0=>2,0.0..=HI=>3,_=>4}"
        argument = "0.5"
    elif mode == "product":
        parameter = "x:E"
        body = "match x{E::A([0.0|1.0,LO..0.0])=>1,E::A([LO..=HI,0.0..=HI])=>2,E::A(_)=>3,E::B=>4}"
        argument = "E::A([0.5,1.0])"
    else:
        parameter = "x:f64"
        body = "let mut calls=0u8;match x{0.0|0.0|LO..=HI if {calls+=1;false}=>0,_=>calls}"
        argument = "-0.0"
    functions = "".join(f"fn measured_{i}({parameter})->u8{{{body}}}\n" for i in range(lines))
    calls = "".join(f"measured_{i}({argument});" for i in range(lines))
    return declarations + functions + "fn main(){" + calls + "}\n"


def slice_pattern_source(lines, mode):
    if mode == "sparse":
        pattern = ",".join(["_"] * lines + ["true"])
        return f"fn measured(x:&[bool]){{match x{{[{pattern}]=>(),[..,true]=>(),[..,false]=>(),[]=>()}}}}\n"
    if mode == "nested":
        parameter = "x:&[&[bool]]"
        arms = "[]=>(),[[],..]=>(),[[true,..],..]=>(),[..,[..,false]]=>(),_=>()"
    else:
        parameter = "x:&[bool]"
        arms = "[]=>(),[true,..]=>(),[..,false]=>(),[false,..,true]=>()"
    return "".join(f"fn measured_{i}({parameter}){{match x{{{arms}}}}}\n" for i in range(lines))


def product_constant_source(lines, mode):
    declarations = ["const ROOT:([u8;2],bool)=([2,7],true);\n"]
    if mode != "shared":
        for i in range(lines):
            value = f"N{i + 1}" if mode == "chain" and i + 1 < lines else "ROOT"
            declarations.append(f"const N{i}:([u8;2],bool)={value};\n")
    for i in range(lines):
        name = "ROOT" if mode == "shared" else f"N{i}"
        declarations.append(f"fn measured_{i}(x:([u8;2],bool))->u8{{match x{{v@{name}=>v.0[1],_=>0}}}}\n")
    calls = "".join(f"measured_{i}(([2,7],true));" for i in range(lines))
    return "".join(declarations) + "fn main(){" + calls + "}\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--candidate", type=Path, default=ROOT / "obj/rust/release/rustgm")
    workloads = parser.add_mutually_exclusive_group()
    workloads.add_argument("--aggregate-input", action="store_true", help="exercise owned aggregate LowIR at 1000/4000/16000 procedures")
    workloads.add_argument("--discriminant-input", action="store_true", help="exercise typed discriminant queries at 1000/4000/16000 enums")
    workloads.add_argument("--pattern-input", action="store_true", help="exercise nested pattern binding identities at 1000/4000/16000 procedures")
    workloads.add_argument("--product-input", action="store_true", help="exercise owned product bindings at 1000/4000/16000 procedures")
    workloads.add_argument("--array-pattern-input", action="store_true", help="exercise owned array ranges at 1000/4000/16000 procedures")
    workloads.add_argument("--borrowed-pattern-input", action="store_true", help="exercise borrowed arrays/slices and occurrence captures at 1000/4000/16000 procedures")
    workloads.add_argument("--scalar-match-input", action="store_true", help="exercise ordered scalar match decisions at 1000/4000/16000 procedures")
    workloads.add_argument("--constructor-pattern-input", action="store_true", help="exercise shared enum/product/array coverage at 1000/4000/16000 procedures")
    workloads.add_argument("--constructor-match-input", action="store_true", help="exercise constructor tests and delayed alternative captures at 1000/4000/16000 procedures")
    workloads.add_argument("--conditional-pattern-input", action="store_true", help="exercise let-else, conditional chains and while-let at 1000/4000/16000 procedures")
    workloads.add_argument("--storage-width-input", choices=("chain", "let-else"), help="exercise persistent cleanup at 128/512/2048 bindings in one procedure")
    workloads.add_argument("--guard-input", choices=("boolean", "let-chain", "retry"), help="exercise guard scopes or execution at 1000/4000/16000 procedures")
    workloads.add_argument("--constant-pattern-input", choices=("shared", "unique", "chain"), help="exercise cached or independent constant patterns at 1000/4000/16000 procedures")
    workloads.add_argument("--float-pattern-input", choices=("scalar", "product", "retry"), help="exercise floating ranges, composite coverage or guard retries at 1000/4000/16000 procedures")
    workloads.add_argument("--slice-pattern-input", choices=("scalar", "nested", "sparse"), help="exercise dynamic slice coverage or sparse wide-pattern aliases")
    workloads.add_argument("--product-constant-input", choices=("shared", "unique", "chain"), help="exercise shared composite values and constant-pattern translation")
    parser.add_argument("--stage", choices=("tokens", "literals", "token-trees", "ast", "crate-ast", "names", "expanded", "expanded-crate", "types", "patterns", "layouts", "body-ir", "ownership", "lowir"), default="tokens")
    args = parser.parse_args()
    if args.slice_pattern_input and args.stage != "patterns":
        parser.error("slice workloads require --stage patterns until reference execution has loan proofs")
    if (args.aggregate_input or args.discriminant_input or args.product_input or args.array_pattern_input or args.scalar_match_input or args.constructor_match_input or args.conditional_pattern_input or args.storage_width_input or args.constant_pattern_input or args.float_pattern_input or args.product_constant_input) and args.stage != "lowir":
        parser.error("aggregate, discriminant and owned-pattern workloads require --stage lowir")
    if args.pattern_input and args.stage != "names":
        parser.error("pattern identity workloads require --stage names")
    if args.guard_input and args.stage not in ("names", "lowir"):
        parser.error("guard workloads require --stage names or --stage lowir")
    if bool(args.borrowed_pattern_input or args.constructor_pattern_input or args.slice_pattern_input) != (args.stage == "patterns"):
        parser.error("--stage patterns requires a borrowed, constructor or slice workload")
    directory = ROOT / "obj/perf" / str(time.time_ns())
    directory.mkdir(parents=True)
    candidate = args.candidate.resolve()
    baseline = args.baseline.resolve() if args.baseline else candidate
    # Snapshot binaries before measurement so a concurrent build cannot change A/B.
    binaries = {}
    for name, binary in [("A", baseline), ("B", candidate)]:
        target = directory / name
        target.write_bytes(binary.read_bytes())
        target.chmod(0o755)
        binaries[name] = target
    receipt = {"kind": "ABBA" if args.baseline else "A/A scaling calibration",
               "stage": args.stage,
               "aggregate_input": args.aggregate_input,
               "discriminant_input": args.discriminant_input,
               "pattern_input": args.pattern_input,
               "product_input": args.product_input,
               "array_pattern_input": args.array_pattern_input,
               "borrowed_pattern_input": args.borrowed_pattern_input,
               "scalar_match_input": args.scalar_match_input,
               "constructor_pattern_input": args.constructor_pattern_input,
               "constructor_match_input": args.constructor_match_input,
               "conditional_pattern_input": args.conditional_pattern_input,
               "storage_width_input": args.storage_width_input,
               "guard_input": args.guard_input,
               "constant_pattern_input": args.constant_pattern_input,
               "float_pattern_input": args.float_pattern_input,
               "slice_pattern_input": args.slice_pattern_input,
               "product_constant_input": args.product_constant_input,
               "binary_sha256": {name: hashlib.sha256(p.read_bytes()).hexdigest() for name, p in binaries.items()},
               "samples": [], "full_output_sha256": {}}
    scales = (4000, 16000, 64000) if args.stage in ("token-trees", "ast", "crate-ast", "names", "expanded", "expanded-crate", "types", "body-ir", "ownership", "lowir") else (16000, 64000, 256000)
    if args.aggregate_input or args.discriminant_input or args.pattern_input or args.product_input or args.array_pattern_input or args.borrowed_pattern_input or args.scalar_match_input or args.constructor_pattern_input or args.constructor_match_input or args.conditional_pattern_input or args.guard_input or args.constant_pattern_input or args.float_pattern_input or args.product_constant_input or args.stage == "layouts":
        scales = (1000, 4000, 16000)
    if args.storage_width_input:
        scales = (128, 512, 2048)
    if args.slice_pattern_input:
        scales = (128, 512, 2048) if args.slice_pattern_input == "sparse" else (1000, 4000, 16000)
    line = {"tokens": LINE, "literals": LITERAL_LINE, "token-trees": TREE_LINE, "ast": AST_LINE, "crate-ast": CRATE_LINE, "names": b"", "expanded": b"", "expanded-crate": b"", "types": b"", "patterns": b"", "layouts": b"", "body-ir": b"", "ownership": b"", "lowir": b""}[args.stage]
    for lines in scales:
        source = directory / f"input-{lines}.rs"
        if args.product_constant_input:
            source.write_text(product_constant_source(lines, args.product_constant_input))
        elif args.slice_pattern_input:
            source.write_text(slice_pattern_source(lines, args.slice_pattern_input))
        elif args.float_pattern_input:
            source.write_text(float_pattern_source(lines, args.float_pattern_input))
        elif args.constant_pattern_input:
            constants = "const ROOT:u64=7;\n"
            if args.constant_pattern_input != "shared":
                for i in range(lines):
                    value = f"N{i+1}+1" if args.constant_pattern_input == "chain" and i + 1 < lines else f"ROOT+{i}"
                    constants += f"const N{i}:u64={value};\n"
            functions = []
            for i in range(lines):
                name = "ROOT" if args.constant_pattern_input == "shared" else f"N{i}"
                functions.append(f"fn measured_{i}(x:u64)->u64{{match x{{{name}=>1,0=>2,_=>3}}}}\n")
            source.write_text(constants + "".join(functions) + "fn main(){" + "".join(
                f"measured_{i}(7);" for i in range(lines)) + "}\n")
        elif args.guard_input:
            guard = "n>0&&n<10" if args.guard_input == "boolean" else "let n=n&&n>0&&let m=n+1"
            value = "m" if args.guard_input == "let-chain" else "n"
            prefix, pattern = "", "n"
            if args.guard_input == "retry":
                prefix, pattern = "let mut calls=0;", "n@(0..=9|5..=15|_)"
                guard = "{calls+=1;calls==2}"
            source.write_text("".join(
                f"fn measured_{i}(x:u8)->u8{{{prefix}match x{{{pattern} if {guard}=>{value},_=>0}}}}\n"
                for i in range(lines)) + ("fn main(){" + "".join(
                f"measured_{i}(7);" for i in range(lines)) + "}\n" if args.stage == "lowir" else ""))
        elif args.storage_width_input:
            patterns = [f"n{i}@0..=9=" + ("x" if i == 0 else f"n{i-1}") for i in range(lines)]
            if args.storage_width_input == "chain":
                body = "if " + "&&".join("let " + p for p in patterns) + f"{{n{lines-1}}}else{{99}}"
            else:
                body = "".join("let " + p + " else{return 99;};" for p in patterns) + f"n{lines-1}"
            source.write_text("fn measured(x:u8)->u8{" + body + "}fn main(){measured(7);}")
        elif args.conditional_pattern_input:
            source.write_text("enum E{A(u64),B}\n" + "".join(
                f"fn measured_{i}(x:E,mut count:u8)->u64{{let E::A(n)=x else{{return 99;}};"
                "let mut sum=0u64;while let k@1..=9=count&&k>0{sum+=k as u64;count-=1;}"
                "if let a@0..=99=n&&a<90&&let b@0..=99=sum{a+b}else{99}}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}(E::A(7),3);" for i in range(lines)) + "}\n")
        elif args.constructor_match_input:
            source.write_text("enum E{A(bool,u64),B(u64,bool),C}\n" + "".join(
                f"fn measured_{i}(x:(E,[u8;2]))->u64{{match x{{"
                "(E::A(true,n)|E::B(n,true),[0,m]|[m,0])=>n+m as u64,"
                "(E::A(_,n)|E::B(n,_),_)=>n,(E::C,_)=>0}}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}((E::B(7,true),[3,0]));" for i in range(lines)) + "}\n")
        elif args.constructor_pattern_input:
            source.write_text("enum E{A(bool,u8),B{flag:bool},C}\n" + "".join(
                f"fn measured_{i}(x:(E,[bool;2])){{match x{{"
                "(E::A(true,_),_)|(E::B{flag:true},_)=>(),"
                "(E::A(false,0..=127),[_,true])=>(),"
                "(E::A(false,_),_)|(E::B{flag:false},_)|(E::C,_)=>()}}\n"
                for i in range(lines)))
        elif args.scalar_match_input:
            source.write_text("".join(
                f"fn measured_{i}(x:u64)->u64{{match x{{"
                "n@(0|2|4)=>n+1,1..=9=>7,10..100=>9,n=>n}}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}(3);" for i in range(lines)) + "}\n")
        elif args.borrowed_pattern_input:
            source.write_text("".join(
                f"fn measured_{i}(xs:&&mut[u8;5],ys:&mut[u8],zs:(&u8,&[u8;1])){{"
                "let[first,tail@..,last]=xs;let[all@..]=ys;let((x,_)|(_,[x]))=zs;}\n"
                for i in range(lines)))
        elif args.array_pattern_input:
            source.write_text("struct N(u64);\n" + "".join(
                f"fn measured_{i}(xs:[N;5])->u64{{let[a,..,e]=xs;let[_,middle@..,_]=xs;"
                "let[b,c,d]=middle;a.0+b.0+c.0+d.0+e.0}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}([N(1),N(2),N(3),N(4),N(5)]);" for i in range(lines)) + "}\n")
        elif args.product_input:
            source.write_text("struct N(u64);struct S{a:N,b:N}\n" + "".join(
                f"fn measured_{i}(S{{a:N(x),b:N(y)}}:S)->u64{{"
                "let whole@(a,..,b)=(x,0u8,y);let ((c,d)|(d,c))=(a,b);whole.0+c+d}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}(S{{a:N(1),b:N(2)}});" for i in range(lines)) + "}\n")
        elif args.pattern_input:
            source.write_text("".join(
                f"fn measured_{i}(v:(u8,u8)){{match v{{((x,0)|(x,1))|((0,x)|(1,x))=>{{x;}},_=>{{}}}}}}\n"
                for i in range(lines)))
        elif args.discriminant_input:
            source.write_text("const SEED:u64=7;\n" + "".join(
                f"#[repr(u64)]enum E{i}{{A=SEED+{i},B=E{i}::A as u64+1}}"
                f"fn measured_{i}()->u64{{E{i}::B as u64}}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}();" for i in range(lines)) + "}\n")
        elif args.aggregate_input:
            source.write_text("struct N(u64);struct Pair{left:N,right:N}fn consume(x:N)->u64{x.0}\n" + "".join(
                f"fn measured_{i}(mut x:Pair,c:bool)->u64{{let n=consume(x.left);"
                "if c{x.left=N(n+1);}else{x.left=N(2);}consume(x.left)+consume(x.right)}\n"
                for i in range(lines)) + "fn main(){" + "".join(
                f"measured_{i}(Pair{{left:N(1),right:N(2)}},false);" for i in range(lines)) + "}\n")
        elif args.stage == "layouts":
            source.write_text("".join(
                f"#[repr(C)]struct T{i}{{a:u8,b:u64,c:[u32;3]}}type A{i}=[T{i};4];\n"
                for i in range(lines)))
        elif args.stage in ("types", "body-ir", "lowir"):
            source.write_text("fn identity(x:u64)->u64{x}\n" + "".join(
                f"fn measured_{i}(x:u64)->u64{{let n={i};let v=identity(n);if x>v{{x+v}}else{{v-x}}}}\n"
                for i in range(lines)))
            if args.stage == "lowir":
                with source.open("a") as stream:
                    stream.write("fn main(){" + "".join(f"measured_{i}(0);" for i in range(lines)) + "}\n")
        elif args.stage == "ownership":
            source.write_text("struct N(u64);struct Pair{left:N,right:N}fn consume(x:N)->u64{x.0}\n" + "".join(
                f"fn measured_{i}(mut x:Pair,c:bool)->u64{{let n=consume(x.left);"
                "if c{x.left=N(n);}else{x.left=N(1);}consume(x.left)+consume(x.right)}\n"
                for i in range(lines)))
        elif args.stage == "expanded-crate":
            source.write_text("macro_rules! measured {($e:expr,wrong)=>{};($e:expr,right)=>{$e+$e}}\n" + "".join(
                f"fn measured_{i}()->u64{{measured!({i},right)}}\n" for i in range(lines)))
        elif args.stage == "expanded":
            source.write_bytes(b"macro_rules! measured {($e:expr, wrong)=>{};($e:expr, right)=>{$e+$e;}}\n"
                               + b"measured!(1+2, right);\n" * lines)
        elif args.stage == "names":
            source.write_text("mod origin {\n" + "".join(
                f"pub fn measured_{i}(value:u64)->u64{{let next=value+17;next}}\n"
                for i in range(lines)) + "}\nuse origin::*;\n")
        else:
            source.write_bytes(b"{\n" + line * lines + b"}\n" if args.stage == "ast" else line * lines)
        full_outputs = {name: full_output_hash(binary, source, directory, args.stage) for name, binary in binaries.items()}
        if full_outputs["A"] != full_outputs["B"]:
            raise ValueError("full output differs between immutable binaries")
        receipt["full_output_sha256"][str(lines)] = full_outputs
        expected = None
        # Warm both snapshots, calibrate A/A, then retain three ABBA blocks.
        order = "ABAAAA" + "ABBA" * 3
        for index, name in enumerate(order):
            value = sample(binaries[name], source, directory / "rss.txt", args.stage)
            expected = expected or value["output"]
            if expected != value["output"]:
                raise ValueError("count/output mismatch between immutable binaries")
            observation = {"lines": lines, "bytes": source.stat().st_size,
                                       "input_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                                       "binary": name, "phase": "warmup" if index < 2 else "calibration" if index < 6 else "paired",
                                       **value}
            receipt["samples"].append(observation)
            with (directory / "observations.jsonl").open("a") as stream:
                stream.write(json.dumps(observation) + "\n")
        data = [s for s in receipt["samples"] if s["lines"] == lines and s["phase"] == "paired" and s["binary"] == "B"]
        print(f"{source.stat().st_size} bytes: median {statistics.median(s['elapsed_ns'] for s in data)/1e6:.2f} ms; "
              f"peak RSS {max(s['peak_rss_kb'] for s in data)} KiB")
        (directory / "samples.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"Raw observations: {directory / 'samples.json'}")
    print("No regression claim is made from calibration/scaling alone.")


if __name__ == "__main__":
    main()
