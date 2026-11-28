#!/usr/bin/env python3
"""Grade real upstream library and crate consumer suites (PA32, PA33, PA35, PA36)."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time

import backend
import native_links
import setup_rust_src
from target_units import ENVIRONMENT, bringup_record

ROOT = Path(__file__).resolve().parents[1]
CORE_DISAMBIGUATOR = "core_1_98_88d9"
CORE_IDENTITY = f"core@{CORE_DISAMBIGUATOR}"
SOURCE_LIMITS = (
    "--source-contract-attempted-bytes=4294967295",
    "--source-contract-retained-bytes=67108864",
)
CORE_CFGS = (
    "--cfg", 'feature="nightly"',
    "--cfg", 'feature="rustc-internal-api"',
)
STD_CFGS = (
    *CORE_CFGS,
    "--cfg", "backtrace_in_libstd",
)
RUN_TIMEOUT = 60
COMPILE_TIMEOUT = 600


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def load_suite(root, stage):
    folder = Path(root).resolve() / stage
    suite_path = folder / "tests/suite.json"
    if not suite_path.is_file():
        raise ValueError(f"{stage}: missing consumer suite manifest {suite_path}")
    suite = json.loads(suite_path.read_text())
    if suite.get("schema") != 1 or suite.get("kind") != "consumers" or suite.get("stage") != stage:
        raise ValueError(f"{stage}: malformed consumer suite manifest")
    outcomes_path = folder / suite["outcomes"]
    if not outcomes_path.is_file():
        raise ValueError(f"{stage}: missing outcomes file {outcomes_path}")
    outcomes = json.loads(outcomes_path.read_text())
    programs_dir = (folder / suite["programs_dir"]).resolve()
    declared = {}
    for case in outcomes["cases"]:
        source = (folder / case["path"]).resolve()
        stdout_path = (folder / case["stdout"]).resolve()
        if source.parent != programs_dir or source.stem != case["name"] or case["name"] in declared:
            raise ValueError(f"{stage}: invalid consumer case {case['name']}")
        if not source.is_file() or not stdout_path.is_file():
            raise ValueError(f"{stage}: missing source or stdout oracle for {case['name']}")
        if sha256_file(source) != case["source_sha256"]:
            raise ValueError(f"{stage}: source SHA-256 mismatch for {case['path']}")
        if sha256_file(stdout_path) != case["stdout_sha256"]:
            raise ValueError(f"{stage}: stdout SHA-256 mismatch for {case['stdout']}")
        declared[case["name"]] = case
    present = {p.stem for p in programs_dir.glob("*.rs")}
    if present != set(declared):
        raise ValueError(f"{stage}: declared and present consumer programs differ: {sorted(present ^ set(declared))}")
    return suite, outcomes, folder


def check_strict_diagnostics(stdout_path, stderr_path):
    for path in (stdout_path, stderr_path):
        if path.is_file() and bringup_record(path) is not None:
            raise ValueError(f"strict build emitted a bring-up record in {path.name}")


def run_compiler(command, *, cwd, env, stdout_path, stderr_path, timeout=COMPILE_TIMEOUT):
    started = time.monotonic()
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        try:
            result = subprocess.run(
                [str(arg) for arg in command],
                cwd=cwd, env=env, stdout=out, stderr=err, timeout=timeout, check=False,
            )
        except subprocess.TimeoutExpired as exc:
            raise ValueError(f"compiler timed out after {timeout}s") from exc
    elapsed = round(time.monotonic() - started, 3)
    check_strict_diagnostics(stdout_path, stderr_path)
    if result.returncode != 0:
        tail = stderr_path.read_text(errors="replace").strip().splitlines()[:12]
        raise ValueError(f"compiler failed (exit {result.returncode}): " + " | ".join(tail))
    return elapsed


def verify_execution(binary, case, scratch, repeats):
    expected_stdout = (scratch["folder"] / case["stdout"]).read_bytes()
    wanted_status = -signal.SIGABRT if case["status"] == "abort" else case["status"]
    expected_stderr = case["stderr"]
    observed = []
    for index in range(repeats):
        with tempfile.TemporaryDirectory(dir=scratch["work"], prefix=f"run-{case['name']}-{index}-") as rundir:
            try:
                proc = subprocess.run(
                    [str(binary), *case["args"]],
                    input=case["stdin"].encode(),
                    env=case["env"],
                    cwd=rundir,
                    capture_output=True,
                    timeout=RUN_TIMEOUT,
                    check=False,
                )
            except subprocess.TimeoutExpired as exc:
                raise ValueError(f"{case['name']}: execution timed out after {RUN_TIMEOUT}s") from exc
            left = sorted(os.listdir(rundir))
        if proc.returncode != wanted_status:
            raise ValueError(
                f"{case['name']} run {index}: exit {proc.returncode}, expected {wanted_status} "
                f"(stderr={proc.stderr.decode(errors='replace')[:200]!r})"
            )
        if isinstance(expected_stderr, str):
            if proc.stderr.decode(errors="replace") != expected_stderr:
                raise ValueError(
                    f"{case['name']} run {index}: stderr differs: {proc.stderr!r} != {expected_stderr!r}"
                )
        else:
            for piece in expected_stderr["contains"]:
                if piece.encode() not in proc.stderr:
                    raise ValueError(
                        f"{case['name']} run {index}: stderr missing {piece!r}: {proc.stderr!r}"
                    )
        if proc.stdout != expected_stdout:
            raise ValueError(
                f"{case['name']} run {index}: stdout differs ({len(proc.stdout)} vs {len(expected_stdout)} bytes)"
            )
        if left:
            raise ValueError(f"{case['name']} run {index}: left files in working directory: {left}")
        observed.append({"status": proc.returncode, "stdout_sha256": sha256_bytes(proc.stdout)})
    return observed


def link_executable(objects, native_plan_path, executable, tools):
    libraries = native_links.load(native_plan_path) if native_plan_path.is_file() else []
    command = ["cc", *[str(o) for o in objects], *native_links.arguments(libraries, ()), "-lm", "-o", str(executable)]
    result = subprocess.run(command, capture_output=True, timeout=120, check=False)
    if result.returncode != 0:
        raise ValueError(f"link failed ({result.returncode}): {result.stderr.decode(errors='replace')[:500]}")
    return libraries


def run_lint_manifest(app, folder, lint_rel, work):
    manifest = folder / lint_rel
    plan = json.loads(manifest.read_text())
    source_core = plan.get("source_core", False)
    edition = plan.get("edition", "2024")
    results = []
    for index, case in enumerate(plan["cases"]):
        source = manifest.parent / case["path"]
        cfg = ["--cfg", case["cfg"]] if "cfg" in case else []
        args = [str(app), "--emit-types", f"--edition={edition}", *cfg]
        if source_core:
            args += ["--source-core", "lint_case@lintcase",
                     "--crate-name", "lint_case", "--crate-disambiguator", "lintcase"]
        args.append(str(source))
        proc = subprocess.run(args, capture_output=True, timeout=30, check=False)
        status = case["status"]
        lint = case["lint"]
        if proc.returncode != status:
            raise ValueError(
                f"lint {case['path']}#{index}: exit {proc.returncode}, expected {status}: "
                f"{proc.stderr.decode(errors='replace')[:300]}"
            )
        if lint and case["diagnostic"].encode() not in proc.stderr:
            raise ValueError(f"lint {case['path']}#{index}: missing diagnostic {case['diagnostic']!r}")
        if re.search(rb"^warning\[", proc.stderr, re.M):
            raise ValueError(f"lint {case['path']}#{index}: unexpected warning: {proc.stderr!r}")
        results.append({"case": f"{case['path']}#{index}", "status": proc.returncode, "passed": True})
    return results


def build_standalone_core(app, library, out_dir, tools, *, check_reproducibility=True):
    compiler_env = {**ENVIRONMENT}
    core_src = library / "core/src/lib.rs"
    runs = {}
    for label in (("run-a", "run-b") if check_reproducibility else ("run-a",)):
        run_dir = out_dir / label
        run_dir.mkdir(parents=True, exist_ok=True)
        cmd = [
            app, "--emit-lowir", "--cap-lints=allow", "--edition=2024",
            "--crate-type=lib", "--crate-name", "core",
            "--crate-disambiguator", CORE_DISAMBIGUATOR,
            "--source-core", CORE_IDENTITY,
            "--source-profile", "module-body-operations-v14",
            *SOURCE_LIMITS, *CORE_CFGS,
            "--metadata-output", run_dir / "core.rgcrate",
            "--native-links-output", run_dir / "core.native-links.json",
            "-o", run_dir / "core.lowir",
            core_src,
        ]
        elapsed = run_compiler(
            cmd, cwd=library.parent, env=compiler_env,
            stdout_path=run_dir / "compile.stdout", stderr_path=run_dir / "compile.stderr",
        )
        runs[label] = {
            "seconds": elapsed,
            "rgcrate_sha256": sha256_file(run_dir / "core.rgcrate"),
            "lowir_sha256": sha256_file(run_dir / "core.lowir"),
            "native_links_sha256": sha256_file(run_dir / "core.native-links.json"),
        }
    if check_reproducibility:
        for key in ("rgcrate_sha256", "lowir_sha256", "native_links_sha256"):
            if runs["run-a"][key] != runs["run-b"][key]:
                raise ValueError(f"standalone core is not reproducible: {key} differs between run-a and run-b")
        # Clean duplicate run-b artifacts after verifying byte equality.
        for name in ("core.rgcrate", "core.lowir", "core.native-links.json"):
            (out_dir / "run-b" / name).unlink(missing_ok=True)
    run_a = out_dir / "run-a"
    core_obj = run_a / "core.o"
    backend.emit_object(run_a / "core.lowir", core_obj, tools)
    # Build minimal compiler_builtins loader metadata.
    builtins_src = out_dir / "loader-builtins.rs"
    builtins_src.write_text("#![feature(compiler_builtins)]\n#![compiler_builtins]\n#![no_std]\n")
    builtins_meta = out_dir / "loader-builtins.rgcrate"
    run_compiler(
        [
            app, "--emit-lowir", "--edition=2024",
            "--crate-type=lib", "--crate-name", "compiler_builtins",
            "--crate-disambiguator", "pa32_loader_builtins",
            "--source-client", CORE_IDENTITY,
            "--extern", f"core={run_a / 'core.rgcrate'}",
            "--metadata-output", builtins_meta,
            "-o", out_dir / "loader-builtins.lowir",
            builtins_src,
        ],
        cwd=out_dir, env=compiler_env,
        stdout_path=out_dir / "builtins.stdout", stderr_path=out_dir / "builtins.stderr",
    )
    return {
        "runs": runs,
        "core_meta": run_a / "core.rgcrate",
        "core_obj": core_obj,
        "builtins_meta": builtins_meta,
        "reproducible": check_reproducibility,
    }


def whole_graph_guests(strategy, library, vendor):
    guests = [("--source-guest", f"alloc@2024={library}/alloc/src/lib.rs")]
    cfgs = list(CORE_CFGS)
    env = {**ENVIRONMENT}
    if strategy in ("whole-graph-std", "whole-graph-unicode"):
        cfgs = list(STD_CFGS)
        env["STD_ENV_ARCH"] = "x86_64"
        guests += [
            ("--source-guest", f"cfg_if@2018={library}/vendor/cfg-if-1.0.4/src/lib.rs"),
            ("--source-guest", f"libc@2021={library}/vendor/libc-0.2.185/src/lib.rs"),
            ("--source-guest", f"unwind@2024={library}/unwind/src/lib.rs"),
            ("--source-guest", f"std_detect@2024={library}/std_detect/src/lib.rs"),
            ("--source-guest", f"hashbrown@2024={library}/vendor/hashbrown-0.17.1/src/lib.rs"),
            ("--source-guest", f"rustc_demangle@2015={library}/vendor/rustc-demangle-0.1.27/src/lib.rs"),
            ("--source-guest", f"panic_abort@2024={library}/panic_abort/src/lib.rs"),
            ("--source-guest", f"std@2024={library}/std/src/lib.rs"),
        ]
    guest_cfgs = []
    if strategy == "whole-graph-unicode":
        guests += [
            ("--source-guest", f"unicode_ident@2021={vendor}/unicode-ident-1.0.24/src/lib.rs"),
            ("--source-guest", f"tinyvec_macros@2018={vendor}/tinyvec_macros-0.1.1/src/lib.rs"),
            ("--source-guest", f"tinyvec@2018={vendor}/tinyvec-1.10.0/src/lib.rs"),
            ("--source-guest", f"unicode_normalization@2018={vendor}/unicode-normalization-0.1.25/src/lib.rs"),
        ]
        guest_cfgs += [
            "--source-guest-cfg", "unicode_ident",
            "--source-guest-cfg", "tinyvec_macros",
            "--source-guest-cfg", "tinyvec",
            "--source-guest-cfg", 'tinyvec=feature="alloc"',
            "--source-guest-cfg", 'tinyvec=feature="default"',
            "--source-guest-cfg", 'tinyvec=feature="tinyvec_macros"',
            "--source-guest-cfg", "unicode_normalization",
            "--source-guest-cfg", 'unicode_normalization=feature="default"',
            "--source-guest-cfg", 'unicode_normalization=feature="std"',
        ]
    flat_guests = [item for pair in guests for item in pair]
    return cfgs, flat_guests, guest_cfgs, env


def run_consumer_case(strategy, app, folder, case, work_root, tools, repeats, *,
                      core_ctx=None, library=None, vendor=None, keep_artifacts=False):
    name = case["name"]
    case_dir = work_root / name
    case_dir.mkdir(parents=True, exist_ok=True)
    source = folder / case["path"]
    lowir = case_dir / f"{name}.lowir"
    links = case_dir / f"{name}.native-links.json"
    obj = case_dir / f"{name}.o"
    exe = case_dir / name
    try:
        if strategy == "standalone-core":
            cmd = [
                app, "--emit-lowir", "--cap-lints=allow", "--edition=2024",
                "--crate-name", name.replace("-", "_"),
                "--crate-disambiguator", f"pa32_{name.replace('-', '_')}",
                "--source-client", CORE_IDENTITY,
                "--extern", f"core={core_ctx['core_meta']}",
                "--extern", f"compiler_builtins={core_ctx['builtins_meta']}",
                "--native-links-output", links,
                "-o", lowir, source,
            ]
            compile_s = run_compiler(
                cmd, cwd=folder, env={**ENVIRONMENT},
                stdout_path=case_dir / "compile.stdout", stderr_path=case_dir / "compile.stderr",
            )
            backend.emit_object(lowir, obj, tools)
            link_executable([obj, core_ctx["core_obj"]], links, exe, tools)
        else:
            cfgs, guests, guest_cfgs, env = whole_graph_guests(strategy, library, vendor)
            cmd = [
                app, "--emit-lowir", "--cap-lints=allow", "--edition=2024",
                "--crate-name", "core", "--crate-disambiguator", CORE_DISAMBIGUATOR,
                "--source-core", CORE_IDENTITY,
                *SOURCE_LIMITS, *cfgs,
                *guests,
                "--source-guest", f"app@2024={source}",
                *guest_cfgs,
                "--source-guest-cfg", "app",
                "--native-links-output", links,
                "-o", lowir,
                library / "core/src/lib.rs",
            ]
            compile_s = run_compiler(
                cmd, cwd=folder, env=env,
                stdout_path=case_dir / "compile.stdout", stderr_path=case_dir / "compile.stderr",
            )
            backend.emit_object(lowir, obj, tools)
            link_executable([obj], links, exe, tools)
        lowir_sha = sha256_file(lowir)
        links_sha = sha256_file(links) if links.is_file() else None
        runs = verify_execution(exe, case, {"folder": folder, "work": case_dir}, repeats)
        result = {
            "name": name, "passed": True, "compile_seconds": compile_s,
            "lowir_sha256": lowir_sha, "native_links_sha256": links_sha, "runs": runs,
        }
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result = {"name": name, "passed": False, "error": str(error)}
    finally:
        if not keep_artifacts:
            for artifact in (lowir, obj, exe):
                artifact.unlink(missing_ok=True)
    return result


def check_whole_graph_reproducibility(strategy, app, folder, case, first_result, work_root, library, vendor):
    repro_dir = work_root / f"repro-{case['name']}"
    repro_dir.mkdir(parents=True, exist_ok=True)
    cfgs, guests, guest_cfgs, env = whole_graph_guests(strategy, library, vendor)
    lowir = repro_dir / "program.lowir"
    links = repro_dir / "program.native-links.json"
    cmd = [
        app, "--emit-lowir", "--cap-lints=allow", "--edition=2024",
        "--crate-name", "core", "--crate-disambiguator", CORE_DISAMBIGUATOR,
        "--source-core", CORE_IDENTITY,
        *SOURCE_LIMITS, *cfgs,
        *guests,
        "--source-guest", f"app@2024={folder / case['path']}",
        *guest_cfgs,
        "--source-guest-cfg", "app",
        "--native-links-output", links,
        "-o", lowir,
        library / "core/src/lib.rs",
    ]
    try:
        elapsed = run_compiler(
            cmd, cwd=folder, env=env,
            stdout_path=repro_dir / "compile.stdout", stderr_path=repro_dir / "compile.stderr",
        )
        lowir_sha = sha256_file(lowir)
        links_sha = sha256_file(links) if links.is_file() else None
        if lowir_sha != first_result["lowir_sha256"] or links_sha != first_result["native_links_sha256"]:
            raise ValueError(
                f"reproducibility failed for {case['name']}: second compile differed from first"
            )
        return {"case": case["name"], "reproducible": True, "seconds": elapsed, "lowir_sha256": lowir_sha}
    finally:
        lowir.unlink(missing_ok=True)


def run_stage(root, stage, app, *, jobs=2, repeats=3, selected=None, rust_src=None,
              crate_cache=None, evidence_root=None, skip_repro=False, skip_core_hash=False):
    suite, outcomes, folder = load_suite(root, stage)
    app = Path(app).resolve(strict=True)
    tools = backend.load_tools(("cppgm",))
    library = setup_rust_src.ensure_rust_library(root, rust_src_override=rust_src, verify=not skip_core_hash)
    vendor = (
        setup_rust_src.ensure_vendor_crates(root, cache_dir=crate_cache)
        if suite.get("requires_pa35_crates")
        else None
    )
    cases = outcomes["cases"]
    if selected:
        by_name = {c["name"]: c for c in cases}
        unknown = [name for name in selected if name not in by_name]
        if unknown:
            raise ValueError(f"{stage}: unknown consumer case(s): {', '.join(unknown)}")
        cases = [by_name[name] for name in selected]
    artifact_base = Path(evidence_root) if evidence_root else (Path(root).resolve() / "obj/tests")
    if not artifact_base.is_absolute():
        artifact_base = Path(root).resolve() / artifact_base
    out = artifact_base / stage / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=True)
    strategy = suite["strategy"]
    started = time.monotonic()
    receipt = {
        "schema": 1,
        "stage": stage,
        "strategy": strategy,
        "app": str(app),
        "app_sha256": sha256_file(app),
        "library": str(library),
        "repeats": repeats,
        "cases": [],
    }
    core_ctx = None
    try:
        if strategy == "standalone-core":
            print(f"{stage}: building standalone core ({'2x reproducibility' if not skip_repro else '1x'})...", flush=True)
            core_ctx = build_standalone_core(
                app, library, out / "core", tools, check_reproducibility=not skip_repro
            )
            receipt["core"] = {
                "runs": core_ctx["runs"],
                "reproducible": core_ctx["reproducible"],
            }
            print(
                f"{stage}: standalone core OK ({core_ctx['runs']['run-a']['seconds']:.2f}s"
                + (f", run-b {core_ctx['runs']['run-b']['seconds']:.2f}s byte-identical" if not skip_repro else "")
                + ")",
                flush=True,
            )

        def worker(case):
            return run_consumer_case(
                strategy, app, folder, case, out / "consumers", tools, repeats,
                core_ctx=core_ctx, library=library, vendor=vendor,
            )

        with ThreadPoolExecutor(max_workers=jobs) as pool:
            results = list(pool.map(worker, cases))
        receipt["cases"] = results
        passed = 0
        for row in results:
            if row["passed"]:
                passed += 1
                print(f"{stage}/{row['name']}: PASS ({row['compile_seconds']:.2f}s)", flush=True)
            else:
                print(f"{stage}/{row['name']}: FAIL — {row['error']}", flush=True)

        extra_ok = True
        if not skip_repro and strategy != "standalone-core" and not selected:
            repro_name = suite.get("reproducibility_case")
            if repro_name:
                target_case = next((c for c in cases if c["name"] == repro_name), None)
                first_row = next((r for r in results if r["name"] == repro_name and r["passed"]), None)
                if target_case and first_row:
                    try:
                        receipt["reproducibility"] = check_whole_graph_reproducibility(
                            strategy, app, folder, target_case, first_row, out, library, vendor
                        )
                        print(f"{stage}/reproducibility ({repro_name}): PASS", flush=True)
                    except ValueError as error:
                        receipt["reproducibility"] = {"case": repro_name, "reproducible": False, "error": str(error)}
                        print(f"{stage}/reproducibility ({repro_name}): FAIL — {error}", flush=True)
                        extra_ok = False
        if lint_rel := suite.get("lint_manifest"):
            try:
                lint_rows = run_lint_manifest(app, folder, lint_rel, out)
                receipt["lints"] = {"passed": True, "cases": lint_rows}
                print(f"{stage}/lints: PASS ({len(lint_rows)}/{len(lint_rows)})", flush=True)
            except ValueError as error:
                receipt["lints"] = {"passed": False, "error": str(error)}
                print(f"{stage}/lints: FAIL — {error}", flush=True)
                extra_ok = False
        receipt["passed"] = passed == len(cases) and extra_ok
        receipt["total_seconds"] = round(time.monotonic() - started, 3)
    finally:
        if core_ctx is not None:
            for artifact in (core_ctx["core_obj"], core_ctx["core_meta"],
                             core_ctx["core_meta"].with_suffix(".lowir"), core_ctx["builtins_meta"]):
                artifact.unlink(missing_ok=True)
        (out / "results.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"{stage}: {passed}/{len(cases)} consumers passed in {receipt.get('total_seconds', 0):.1f}s")
    print(f"Evidence: {out.relative_to(Path(root).resolve())}")
    return receipt.get("passed", False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("pa32", "pa33", "pa35", "pa36"))
    parser.add_argument("--app", default=str(ROOT / "dev/rustgm"))
    parser.add_argument("--jobs", type=int, default=int(os.environ.get("RUSTGM_TEST_JOBS", "2")))
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--case", dest="cases", action="append", help="run only selected consumer name(s)")
    parser.add_argument("--rust-src", type=Path, help="override pinned Rust 1.98.0 library path")
    parser.add_argument("--crate-cache", type=Path, help="override .crate cache directory for PA35")
    parser.add_argument("--evidence-root", type=Path, help="store receipts under this directory")
    parser.add_argument("--skip-reproducibility", action="store_true")
    parser.add_argument("--skip-core-hash", action="store_true")
    args = parser.parse_args()
    if args.jobs < 1 or args.repeats < 1:
        parser.error("jobs and repeats must be positive")
    try:
        ok = run_stage(
            ROOT, args.stage, args.app,
            jobs=args.jobs, repeats=args.repeats, selected=args.cases,
            rust_src=args.rust_src, crate_cache=args.crate_cache,
            evidence_root=args.evidence_root,
            skip_repro=args.skip_reproducibility, skip_core_hash=args.skip_core_hash,
        )
        return 0 if ok else 1
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"consumer grading error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
