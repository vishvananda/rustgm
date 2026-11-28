#!/usr/bin/env python3
"""Verify PA34 host build script, generated source, cfg/env propagation, mixed editions and replay."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

import backend

ROOT = Path(__file__).resolve().parents[1]


def sha256_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def parse_build_directives(stdout):
    cfgs, envs = [], {}
    for raw in stdout.splitlines():
        line = raw.strip()
        for prefix in ("cargo::", "cargo:"):
            if line.startswith(prefix):
                rest = line[len(prefix):]
                if rest.startswith("rustc-cfg="):
                    cfgs.append(rest[len("rustc-cfg="):])
                elif rest.startswith("rustc-env="):
                    key, _, value = rest[len("rustc-env="):].partition("=")
                    envs[key] = value
                break
    return cfgs, envs


def run_once(app, fixture_dir, work_dir, tools):
    out_dir = work_dir / "out"
    out_dir.mkdir(parents=True, exist_ok=True)
    # 1. Build and run host build.rs using pinned host rustc.
    script_bin = work_dir / "build-script"
    subprocess.run(
        ["rustc", "--edition=2024", str(fixture_dir / "gen_lib/build.rs"), "-o", str(script_bin)],
        check=True, capture_output=True, text=True, timeout=60,
    )
    script_env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "OUT_DIR": str(out_dir)}
    script_run = subprocess.run(
        [str(script_bin)], env=script_env, cwd=fixture_dir / "gen_lib",
        check=True, capture_output=True, text=True, timeout=30,
    )
    cfgs, envs = parse_build_directives(script_run.stdout)
    if "generated_mode" not in cfgs or envs.get("PA34_TAG") != "build_ok":
        raise ValueError(f"unexpected build script directives: cfgs={cfgs}, envs={envs}")
    if not (out_dir / "generated.rs").is_file():
        raise ValueError("build script did not emit generated.rs in OUT_DIR")

    # 2. Compile 2021 target library with RustGM using build script outputs.
    lib_env = {**os.environ, "OUT_DIR": str(out_dir), **envs}
    lib_meta = work_dir / "gen_lib.rgm"
    lib_lowir = work_dir / "gen_lib.lowir"
    cfg_args = [item for c in cfgs for item in ("--cfg", c)]
    lib_cmd = [
        str(app), "--emit-lowir", "--edition=2021", "--crate-type=lib",
        "--crate-name", "gen_lib", "--crate-disambiguator", "pa34_driver_gen",
        *cfg_args, "--metadata-output", str(lib_meta),
        "-o", str(lib_lowir), str(fixture_dir / "gen_lib/src/lib.rs"),
    ]
    lib_res = subprocess.run(lib_cmd, env=lib_env, capture_output=True, text=True, timeout=60)
    if lib_res.returncode != 0:
        raise ValueError(f"gen_lib compile failed: {lib_res.stderr}")

    # 3. Compile 2024 consumer against renamed dependency `renamed_gen`.
    app_lowir = work_dir / "consumer.lowir"
    app_cmd = [
        str(app), "--emit-lowir", "--edition=2024",
        "--extern", f"renamed_gen={lib_meta}",
        "-o", str(app_lowir), str(fixture_dir / "consumer/src/main.rs"),
    ]
    app_res = subprocess.run(app_cmd, capture_output=True, text=True, timeout=60)
    if app_res.returncode != 0:
        raise ValueError(f"consumer compile failed: {app_res.stderr}")

    # 4. Link through CPPGM and run natively.
    lib_obj = work_dir / "gen_lib.o"
    app_obj = work_dir / "consumer.o"
    exe = work_dir / "consumer"
    backend.emit_object(lib_lowir, lib_obj, tools)
    backend.emit_object(app_lowir, app_obj, tools)
    link_res = subprocess.run(["cc", str(lib_obj), str(app_obj), "-lm", "-o", str(exe)],
                              capture_output=True, text=True, timeout=60)
    if link_res.returncode != 0:
        raise ValueError(f"link failed: {link_res.stderr}")
    exec_res = subprocess.run([str(exe)], capture_output=True, text=True, timeout=30)
    if exec_res.returncode != 0 or exec_res.stdout or exec_res.stderr:
        raise ValueError(f"consumer failed: rc={exec_res.returncode} stderr={exec_res.stderr!r}")

    return {
        "lib_meta_sha256": sha256_file(lib_meta),
        "lib_lowir_sha256": sha256_file(lib_lowir),
        "app_lowir_sha256": sha256_file(app_lowir),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", required=True, type=Path)
    args = parser.parse_args()
    app = args.app.resolve(strict=True)
    fixture_dir = ROOT / "pa34/tests/build-driver"
    tools = backend.load_tools(("cppgm",))
    try:
        with tempfile.TemporaryDirectory(prefix="rustgm-pa34-driver-") as scratch:
            scratch = Path(scratch)
            first = run_once(app, fixture_dir, scratch / "run-a", tools)
            second = run_once(app, fixture_dir, scratch / "run-b", tools)
            if first != second:
                raise ValueError(f"PA34 build-driver replay is not deterministic: {first} != {second}")
        print("pa34/build-driver: PASS (host build.rs + generated source + mixed 2021/2024 + replay)")
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"pa34/build-driver: FAIL — {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
