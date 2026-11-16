//! Link the pinned CPPGM `LowIR` library, found only through the verified pin.
//!
//! `scripts/backend.py --library-config` checks every pinned library file and
//! header against the hashes of `backend.lock.json` before it names the
//! folder, and states the release and interface version the lock reviews. The
//! build refuses an interface version other than the one `src/records.rs`
//! states records for.
use std::{env, path::PathBuf, process::Command};

/// The interface version of `src/records.rs`.
const ABI_VERSION: &str = "4";

fn main() {
    if let Err(message) = link() {
        println!("cargo::error=in-process LowIR bridge: {message}");
    }
}

fn link() -> Result<(), String> {
    let manifest = env::var("CARGO_MANIFEST_DIR").map_err(|error| error.to_string())?;
    let root = PathBuf::from(manifest).join("../..");
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    for input in [
        "backend.local.json",
        "backend.lock.json",
        "scripts/backend.py",
    ] {
        println!("cargo::rerun-if-changed={}", root.join(input).display());
    }
    let output = Command::new("python3")
        .arg(root.join("scripts/backend.py"))
        .arg("--library-config")
        .current_dir(&root)
        .output()
        .map_err(|error| format!("cannot run scripts/backend.py: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "the pinned library is unavailable: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let field = |key: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
            .ok_or_else(|| format!("scripts/backend.py states no {key}"))
    };
    let (folder, release, abi) = (field("folder")?, field("release")?, field("abi_version")?);
    if abi != ABI_VERSION {
        return Err(format!(
            "the lock reviews interface version {abi}; this bridge states version {ABI_VERSION}"
        ));
    }
    println!("cargo::rerun-if-changed={folder}/libcppgm_lowir.a");
    println!("cargo::rustc-link-search=native={folder}");
    println!("cargo::rustc-link-lib=static=cppgm_lowir");
    // The library is C++ inside: its runtime and threads link after it.
    for library in ["stdc++", "pthread", "m"] {
        println!("cargo::rustc-link-lib=dylib={library}");
    }
    println!("cargo::rustc-env=RUSTGM_BRIDGE_RELEASE={release}");
    println!("cargo::rustc-env=RUSTGM_BRIDGE_ROOT={}", root.display());
    Ok(())
}
