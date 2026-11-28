#!/usr/bin/env python3
"""PA37 reproducible self-hosting driver (modeled on CPPGM PA35 inception).

Works in both the maintainer repository (`dev/`) and exported student
repositories (`src/` + root `Cargo.toml`).

Stages:
  0. Stage 0 (seed): built once by pinned rustc (`cargo build --release --locked`).
  1. Stage 1 (self): Stage 0 compiles the frozen whole-graph source inventory
     (original Rust 1.98.0 core/alloc/std + locked dependencies + compiler) in
     strict mode with zero bring-up records, linked through CPPGM.
  2. Verification ladder: Stage 1 runs earlier assignment oracle suites via
     `RUSTGM_TEST_APP` (default gate: PA1-PA24; ladder targets reach PA30).
  3. Stage 2 (inception): Stage 1 compiles the exact same frozen graph and
     links with the exact same CPPGM and linker pins.
  4. Byte comparison: Stage 1 and Stage 2 LowIR, native link manifests, and
     linked compiler executables must match byte for byte (`MATCH rustgm`).
  5. Negative controls: verify perturbed stage output and nonzero bring-up
     records are rejected by the strict comparison and receipt checks.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

import build_self_hosting as whole
import compare_self_hosting as cmp_stages
from host_process import digest
import link_self_hosting as link_stage
import setup_rust_src

ROOT = Path(__file__).resolve().parents[1]

STDLIB_LIBRARY_SUBTREES = (
    "core",
    "stdarch",
    "portable-simd",
    "alloc",
    "unwind",
    "std_detect",
    "panic_abort",
    "backtrace",
    "std",
)
STDLIB_VENDOR_CRATES = (
    "cfg-if-1.0.4",
    "libc-0.2.185",
    "hashbrown-0.17.1",
    "rustc-demangle-0.1.27",
)
STDLIB_BASE_CONFIG = {
    "core": {
        "path": "library/core/src/lib.rs",
        "disambiguator": "core_1_98_88d9",
        "cfg": [
            'feature="nightly"',
            'feature="rustc-internal-api"',
            "backtrace_in_libstd",
        ],
    },
    "guests": [
        {"name": "alloc", "edition": "2024", "path": "library/alloc/src/lib.rs"},
        {"name": "cfg_if", "edition": "2018", "path": "library/vendor/cfg-if-1.0.4/src/lib.rs"},
        {"name": "libc", "edition": "2021", "path": "library/vendor/libc-0.2.185/src/lib.rs"},
        {"name": "unwind", "edition": "2024", "path": "library/unwind/src/lib.rs"},
        {"name": "std_detect", "edition": "2024", "path": "library/std_detect/src/lib.rs"},
        {"name": "hashbrown", "edition": "2024", "path": "library/vendor/hashbrown-0.17.1/src/lib.rs"},
        {"name": "rustc_demangle", "edition": "2015", "path": "library/vendor/rustc-demangle-0.1.27/src/lib.rs"},
        {"name": "panic_abort", "edition": "2024", "path": "library/panic_abort/src/lib.rs"},
        {"name": "std", "edition": "2024", "path": "library/std/src/lib.rs"},
    ],
}


def is_maintainer_repo(root):
    return (Path(root) / "dev/Cargo.toml").is_file()


def default_stage0(root):
    root = Path(root).resolve()
    if env_stage0 := os.environ.get("RUSTGM_STAGE0"):
        return Path(env_stage0).resolve()
    if is_maintainer_repo(root):
        return root / "obj/rust/release/rustgm"
    return root / "obj/student/release/rustgm"


def default_linker():
    linker = os.environ.get("RUSTGM_LINKER") or shutil.which("cc")
    if not linker:
        raise ValueError("host C linker `cc` not found; set RUSTGM_LINKER")
    return Path(linker).resolve(strict=True)


def copy_tree_regular(src, dst):
    """Copy a directory tree as regular files (rejecting or dereferencing symlinks cleanly)."""
    src = Path(src).resolve(strict=True)
    dst = Path(dst)
    dst.mkdir(parents=True, exist_ok=True)
    for directory, dirs, files in os.walk(src, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in (".git", "target", "__pycache__"))
        rel = Path(directory).relative_to(src)
        target_dir = dst / rel
        target_dir.mkdir(parents=True, exist_ok=True)
        for name in sorted(files):
            if name == ".rustgm-verified-sha256":
                continue
            source_file = Path(directory) / name
            if source_file.is_symlink():
                continue
            target_file = target_dir / name
            shutil.copyfile(source_file, target_file)
            mode = 0o755 if (source_file.stat().st_mode & 0o111) else 0o644
            target_file.chmod(mode)


def discover_student_configuration(root, sources_dir):
    """Derive a PA37 configuration from a student's Cargo.toml / Cargo.lock."""
    custom = root / "pa37/student-configuration.json"
    if custom.is_file():
        return json.loads(custom.read_text())
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--manifest-path", str(root / "Cargo.toml"),
         "--locked", "--format-version", "1"],
        cwd=root, text=True, timeout=60,
    ))
    packages = {pkg["id"]: pkg for pkg in metadata["packages"]}
    resolve = metadata["resolve"]
    nodes = {node["id"]: node for node in resolve["nodes"]}
    root_id = resolve["root"]
    if not root_id:
        raise ValueError("student Cargo.toml has no root package")
    # Topological post-order over normal target dependencies.
    order, visiting, visited = [], set(), set()

    def visit(pkg_id):
        if pkg_id in visited:
            return
        if pkg_id in visiting:
            raise ValueError(f"dependency cycle involving {pkg_id}")
        visiting.add(pkg_id)
        for dep in nodes[pkg_id]["deps"]:
            kinds = dep.get("dep_kinds", [])
            if not kinds or any(k.get("kind") is None for k in kinds):
                visit(dep["pkg"])
        visiting.remove(pkg_id)
        visited.add(pkg_id)
        order.append(pkg_id)

    visit(root_id)
    guests = list(STDLIB_BASE_CONFIG["guests"])
    vendor_dst = sources_dir / "vendor"
    for pkg_id in order:
        if pkg_id == root_id:
            continue
        pkg = packages[pkg_id]
        lib_target = next(
            (t for t in pkg["targets"] if any(k in ("lib", "rlib") for k in t["kind"])),
            None,
        )
        if lib_target is None:
            continue
        pkg_root = Path(pkg["manifest_path"]).parent
        folder_name = f"{pkg['name']}-{pkg['version']}"
        copy_tree_regular(pkg_root, vendor_dst / folder_name)
        rel_src = Path(lib_target["src_path"]).relative_to(pkg_root).as_posix()
        features = sorted(nodes[pkg_id].get("features", []))
        guests.append({
            "name": lib_target["name"].replace("-", "_"),
            "edition": lib_target.get("edition", pkg.get("edition", "2024")),
            "path": f"vendor/{folder_name}/{rel_src}",
            "cfg": [f'feature="{feat}"' for feat in features],
        })
    root_pkg = packages[root_id]
    root_features = [f'feature="{feat}"' for feat in sorted(nodes[root_id].get("features", []))]
    copy_tree_regular(root / "src", sources_dir / "src")
    lib_target = next(
        (t for t in root_pkg["targets"] if any(k in ("lib", "rlib") for k in t["kind"])),
        None,
    )
    if lib_target is not None:
        rel_lib = Path(lib_target["src_path"]).relative_to(root).as_posix()
        guests.append({
            "name": lib_target["name"].replace("-", "_"),
            "edition": lib_target.get("edition", root_pkg.get("edition", "2024")),
            "path": rel_lib,
            "cfg": root_features,
        })
    bin_target = next(
        (t for t in root_pkg["targets"] if "bin" in t["kind"] and t["name"] == "rustgm"),
        next((t for t in root_pkg["targets"] if "bin" in t["kind"]), None),
    )
    if bin_target is None:
        raise ValueError("student Cargo.toml defines no binary target")
    rel_bin = Path(bin_target["src_path"]).relative_to(root).as_posix()
    guests.append({
        "name": "app",
        "edition": bin_target.get("edition", root_pkg.get("edition", "2024")),
        "path": rel_bin,
        "cfg": root_features,
    })
    return {"core": dict(STDLIB_BASE_CONFIG["core"]), "guests": guests}


def assemble_sources(root, work_dir, *, rust_src=None, crate_cache=None, skip_core_hash=False):
    root = Path(root).resolve()
    work_dir = Path(work_dir).resolve()
    sources = work_dir / "sources"
    config_path = work_dir / "configuration.json"
    if sources.exists():
        shutil.rmtree(sources)
    sources.mkdir(parents=True, exist_ok=True)
    library = setup_rust_src.ensure_rust_library(root, rust_src_override=rust_src, verify=not skip_core_hash)
    for sub in STDLIB_LIBRARY_SUBTREES:
        copy_tree_regular(library / sub, sources / "library" / sub)
    for crate in STDLIB_VENDOR_CRATES:
        copy_tree_regular(library / "vendor" / crate, sources / "library/vendor" / crate)
    if is_maintainer_repo(root):
        vendor = setup_rust_src.ensure_vendor_crates(root, cache_dir=crate_cache)
        for name, version, _ in setup_rust_src.PA35_CRATES:
            folder = f"{name}-{version}"
            copy_tree_regular(vendor / folder, sources / "vendor" / folder)
        copy_tree_regular(root / "dev/src", sources / "dev/src")
        copy_tree_regular(root / "dev/bin", sources / "dev/bin")
        config = json.loads((root / "pa37/configuration.json").read_text())
    else:
        config = discover_student_configuration(root, sources)
    config_path.write_text(json.dumps(config, indent=2) + "\n")
    return sources, config_path, config


def build_stage0(root):
    root = Path(root).resolve()
    if is_maintainer_repo(root):
        cmd = ["cargo", "build", "--manifest-path", str(root / "dev/Cargo.toml"), "--release", "--locked"]
        env = {**os.environ, "CARGO_TARGET_DIR": str(root / "obj/rust"), "CARGO_INCREMENTAL": "0"}
    else:
        cmd = ["cargo", "build", "--release", "--locked"]
        env = {**os.environ, "CARGO_TARGET_DIR": str(root / "obj/student")}
    subprocess.run(cmd, cwd=root, env=env, check=True)
    return default_stage0(root)


def do_freeze(root, work_dir, stage0, linker, *, rust_src=None, crate_cache=None, skip_core_hash=False):
    sources, _, config = assemble_sources(
        root, work_dir, rust_src=rust_src, crate_cache=crate_cache, skip_core_hash=skip_core_hash
    )
    graph_path = work_dir / "graph.json"
    if graph_path.exists():
        graph_path.unlink()
    graph = whole.freeze(config, sources, stage0, linker, graph_path)
    return sources, graph_path, graph


def do_stage1(root, work_dir, stage0, linker, *, optimization=1, timeout=1800,
              rust_src=None, crate_cache=None, skip_core_hash=False):
    stage0 = Path(stage0).resolve(strict=True)
    linker = Path(linker).resolve(strict=True)
    work_dir = Path(work_dir).resolve()
    sources, graph_path, graph = do_freeze(
        root, work_dir, stage0, linker,
        rust_src=rust_src, crate_cache=crate_cache, skip_core_hash=skip_core_hash,
    )
    for sub in ("stage1", "stage2"):
        target = work_dir / sub
        if target.exists():
            shutil.rmtree(target)
    stage0_sha = graph["stage0_sha256"]
    linker_sha = graph["tools"]["linker"]
    print(f"PA37 stage 1: building frozen whole-source graph with {stage0}...", flush=True)
    t_receipt = whole.build(graph_path, sources, stage0, stage0_sha, work_dir / "stage1/target", timeout)
    print(
        f"PA37 stage 1 frontend OK: {t_receipt['command']['seconds']:.1f}s, "
        f"peak {t_receipt['command']['peak_mib']:.1f} MiB, "
        f"lowir {t_receipt['outputs']['lowir']['bytes']} bytes, bringup_records=0",
        flush=True,
    )
    print(f"PA37 stage 1: linking through CPPGM -O{optimization}...", flush=True)
    l_receipt = link_stage.link(
        work_dir / "stage1/target", None, work_dir / "stage1/native",
        linker, linker_sha, timeout=timeout, optimization=optimization,
    )
    linked = l_receipt["linked_compiler"]
    print(f"PA37 stage 1 linked: {linked['path']} ({linked['bytes']} bytes, sha256={linked['sha256'][:16]})", flush=True)
    return t_receipt, l_receipt


def do_stage2(work_dir, linker, *, optimization=1, timeout=3600):
    work_dir = Path(work_dir).resolve()
    linker = Path(linker).resolve(strict=True)
    graph_path = work_dir / "graph.json"
    sources = work_dir / "sources"
    stage1_link = json.loads((work_dir / "stage1/native/receipt.json").read_text())
    stage1_bin = Path(stage1_link["linked_compiler"]["path"])
    stage1_sha = stage1_link["linked_compiler"]["sha256"]
    graph = json.loads(graph_path.read_text())
    linker_sha = graph["tools"]["linker"]
    stage2_dir = work_dir / "stage2"
    if stage2_dir.exists():
        shutil.rmtree(stage2_dir)
    print(f"PA37 stage 2: rebuilding frozen whole-source graph with stage 1 ({stage1_bin})...", flush=True)
    t_receipt = whole.build(graph_path, sources, stage1_bin, stage1_sha, stage2_dir / "target", timeout)
    print(
        f"PA37 stage 2 frontend OK: {t_receipt['command']['seconds']:.1f}s, "
        f"peak {t_receipt['command']['peak_mib']:.1f} MiB, bringup_records=0",
        flush=True,
    )
    # Fail fast if LowIR or native-links differ before spending time on the second native link.
    for filename in whole.OUTPUTS.values():
        left = work_dir / "stage1/target" / filename
        right = work_dir / "stage2/target" / filename
        cmp_stages.compare_files(left, right, filename)
    print("PA37 stage 2 frontend outputs match stage 1 byte for byte; linking stage 2...", flush=True)
    l_receipt = link_stage.link(
        stage2_dir / "target", None, stage2_dir / "native",
        linker, linker_sha, timeout=timeout, optimization=optimization,
    )
    linked = l_receipt["linked_compiler"]
    print(f"PA37 stage 2 linked: {linked['path']} ({linked['bytes']} bytes, sha256={linked['sha256'][:16]})", flush=True)
    return t_receipt, l_receipt


def do_compare(work_dir):
    work_dir = Path(work_dir).resolve()
    result = cmp_stages.compare(
        work_dir / "stage1/target",
        work_dir / "stage2/target",
        work_dir / "stage1/native/rustgm",
        work_dir / "stage2/native/rustgm",
    )
    linked = result["linked_compiler"]
    print(f"MATCH rustgm: {work_dir / 'stage2/native/rustgm'} ({linked['bytes']} bytes, sha256={linked['sha256']})")
    return result


def do_ladder(root, work_dir, stages, *, jobs=4, evidence_root=None):
    root = Path(root).resolve()
    work_dir = Path(work_dir).resolve()
    stage1_bin = work_dir / "stage1/native/rustgm"
    if not stage1_bin.is_file():
        raise ValueError(f"missing stage 1 executable: {stage1_bin}; run `stage1` first")
    ev_root = Path(evidence_root) if evidence_root else (work_dir / "ladder-tests")
    results = []
    failed = []
    for stage in stages:
        started = time.monotonic()
        cmd = [
            sys.executable, str(root / "scripts/course.py"), stage,
            "--app", str(stage1_bin), "--jobs", str(jobs),
            "--evidence-root", str(ev_root),
        ]
        proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True, check=False)
        elapsed = round(time.monotonic() - started, 2)
        summary_line = next(
            (line for line in reversed(proc.stdout.splitlines()) if line.startswith(f"{stage}: ")),
            f"{stage}: exit {proc.returncode}",
        )
        ok = proc.returncode == 0
        results.append({"stage": stage, "passed": ok, "summary": summary_line, "seconds": elapsed})
        print(f"PA37 stage 1 {summary_line} ({elapsed:.2f}s)" + ("" if ok else " [FAIL]"), flush=True)
        if not ok:
            failed.append(stage)
            if proc.stderr.strip():
                print(proc.stderr.strip()[:400], file=sys.stderr)
    receipt = {
        "schema": 1,
        "stage1_binary": str(stage1_bin),
        "stage1_sha256": digest(stage1_bin),
        "stages": results,
        "passed": not failed,
    }
    (work_dir / "ladder-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if failed:
        raise ValueError(f"stage 1 failed oracle suite(s): {', '.join(failed)}")
    return receipt


def do_negative_controls(work_dir):
    """Verify perturbed stage output and nonzero bring-up records are rejected."""
    work_dir = Path(work_dir).resolve()
    s1_target = work_dir / "stage1/target"
    s2_target = work_dir / "stage2/target"
    s1_bin = work_dir / "stage1/native/rustgm"
    s2_bin = work_dir / "stage2/native/rustgm"
    with tempfile.TemporaryDirectory(dir=work_dir, prefix=".neg-control-") as scratch:
        scratch = Path(scratch)
        # 1. Perturbed stage 2 LowIR must fail byte comparison.
        perturbed = scratch / "stage2-perturbed"
        perturbed.mkdir()
        for name in ("compile.stdout", "compile.stderr", "program.native-links.json"):
            shutil.copyfile(s2_target / name, perturbed / name)
        lowir = perturbed / "program.lowir"
        data = bytearray((s2_target / "program.lowir").read_bytes())
        data[-1] ^= 0x01
        lowir.write_bytes(data)
        receipt = json.loads((s2_target / "receipt.json").read_text())
        if "command" in receipt and "argv" in receipt["command"]:
            receipt["command"]["argv"] = [
                arg.replace(str(s2_target), str(perturbed)) for arg in receipt["command"]["argv"]
            ]
        receipt["outputs"]["lowir"]["path"] = str(lowir)
        receipt["outputs"]["lowir"]["sha256"] = digest(lowir)
        receipt["outputs"]["native_links"]["path"] = str(perturbed / "program.native-links.json")
        (perturbed / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        try:
            cmp_stages.compare(s1_target, perturbed, s1_bin, s2_bin)
        except ValueError as err:
            if "differs" not in str(err):
                raise ValueError(f"perturbed stage 2 failed for unexpected reason: {err}") from err
        else:
            raise ValueError("negative control failed: perturbed stage 2 LowIR was accepted")

        # 2. Nonzero bring-up record must fail strict verification.
        dirty = scratch / "stage1-bringup"
        dirty.mkdir()
        for name in ("compile.stdout", "program.lowir", "program.native-links.json"):
            shutil.copyfile(s1_target / name, dirty / name)
        (dirty / "compile.stderr").write_text("bringup-blocker\tsynthetic\tnegative-control\n")
        dirty_receipt = json.loads((s1_target / "receipt.json").read_text())
        if "command" in dirty_receipt and "argv" in dirty_receipt["command"]:
            dirty_receipt["command"]["argv"] = [
                arg.replace(str(s1_target), str(dirty)) for arg in dirty_receipt["command"]["argv"]
            ]
        dirty_receipt["outputs"]["lowir"]["path"] = str(dirty / "program.lowir")
        dirty_receipt["outputs"]["native_links"]["path"] = str(dirty / "program.native-links.json")
        try:
            whole.verify_stage(dirty, dirty_receipt)
        except ValueError as err:
            if "bring-up" not in str(err):
                raise ValueError(f"bring-up negative control failed for unexpected reason: {err}") from err
        else:
            raise ValueError("negative control failed: nonzero bring-up record was accepted")
    print("PA37 negative controls: PASS (perturbed output + bring-up record rejected)", flush=True)
    return {"perturbed_stage2_rejected": True, "bringup_record_rejected": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--work", type=Path, default=ROOT / "obj/pa37", help="working directory under obj/")
    parser.add_argument("--stage0", type=Path, help="stage 0 compiler binary")
    parser.add_argument("--linker", type=Path, help="host C linker (default: cc)")
    parser.add_argument("--rust-src", type=Path, help="pinned Rust 1.98.0 library directory")
    parser.add_argument("--crate-cache", type=Path, help=".crate cache directory")
    parser.add_argument("--optimization", type=int, choices=(0, 1), default=1)
    parser.add_argument("--jobs", type=int, default=int(os.environ.get("RUSTGM_TEST_JOBS", "4")))
    parser.add_argument("--stage1-timeout", type=int, default=1800)
    parser.add_argument("--stage2-timeout", type=int, default=3600)
    parser.add_argument("--skip-core-hash", action="store_true")
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("stage0", help="build stage 0 compiler with pinned rustc")
    sub.add_parser("stage1", help="freeze source graph and build/link stage 1")
    sub.add_parser("stage2", help="build/link stage 2 using stage 1")
    sub.add_parser("compare", help="byte-compare stage 1 and stage 2 artifacts")
    sub.add_parser("negative-controls", help="verify perturbed input and bring-up record fail")
    ladder = sub.add_parser("ladder", help="run assignment oracle suites with stage 1")
    ladder_sel = ladder.add_mutually_exclusive_group(required=True)
    ladder_sel.add_argument("--stage", help="single assignment, e.g. pa12")
    ladder_sel.add_argument("--through", type=int, help="run pa1..paN with stage 1")
    test_cmd = sub.add_parser("test", help="full PA37 gate: stage1 + PA1..PA24 + stage2 + compare + negative controls")
    test_cmd.add_argument("--through", type=int, default=24, help="ladder ceiling (default: 24)")
    test_cmd.add_argument("--build-stage0", action="store_true", help="build stage 0 first if missing")
    args = parser.parse_args()

    work_dir = args.work.resolve()
    try:
        if args.action == "stage0":
            binary = build_stage0(ROOT)
            print(f"Stage 0 ready: {binary}")
            return 0
        linker = args.linker.resolve(strict=True) if args.linker else default_linker()
        if args.action == "stage1":
            stage0 = args.stage0 or default_stage0(ROOT)
            do_stage1(
                ROOT, work_dir, stage0, linker,
                optimization=args.optimization, timeout=args.stage1_timeout,
                rust_src=args.rust_src, crate_cache=args.crate_cache, skip_core_hash=args.skip_core_hash,
            )
        elif args.action == "stage2":
            do_stage2(work_dir, linker, optimization=args.optimization, timeout=args.stage2_timeout)
        elif args.action == "compare":
            do_compare(work_dir)
        elif args.action == "negative-controls":
            do_negative_controls(work_dir)
        elif args.action == "ladder":
            stages = [args.stage] if args.stage else [f"pa{n}" for n in range(1, args.through + 1)]
            do_ladder(ROOT, work_dir, stages, jobs=args.jobs)
        elif args.action == "test":
            started = time.monotonic()
            stage0 = args.stage0 or default_stage0(ROOT)
            if args.build_stage0 and not Path(stage0).is_file():
                stage0 = build_stage0(ROOT)
            s1_target, s1_link = do_stage1(
                ROOT, work_dir, stage0, linker,
                optimization=args.optimization, timeout=args.stage1_timeout,
                rust_src=args.rust_src, crate_cache=args.crate_cache, skip_core_hash=args.skip_core_hash,
            )
            ladder_receipt = do_ladder(
                ROOT, work_dir, [f"pa{n}" for n in range(1, args.through + 1)], jobs=args.jobs
            )
            s2_target, s2_link = do_stage2(
                work_dir, linker, optimization=args.optimization, timeout=args.stage2_timeout
            )
            cmp_result = do_compare(work_dir)
            neg_result = do_negative_controls(work_dir)
            summary = {
                "schema": 1,
                "stage": "pa37",
                "passed": True,
                "total_seconds": round(time.monotonic() - started, 2),
                "stage1": {"frontend": s1_target["command"], "linked": s1_link["linked_compiler"]},
                "stage2": {"frontend": s2_target["command"], "linked": s2_link["linked_compiler"]},
                "ladder": {"through": args.through, "passed": ladder_receipt["passed"]},
                "comparison": cmp_result,
                "negative_controls": neg_result,
            }
            (work_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
            print(f"PA37: PASS in {summary['total_seconds']:.1f}s (summary: {work_dir / 'summary.json'})")
        return 0
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"PA37 self-hosting failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
