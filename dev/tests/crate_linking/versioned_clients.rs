//! The actual driver publishes complete contracts before fresh CLI consumers run.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn qualify(native: bool) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let lane = if native { "native" } else { "frontend" };
    let evidence = root
        .join("obj/parallel/pa31/source-cli-contracts")
        .join(format!("{lane}-{}", std::process::id()));
    std::fs::create_dir_all(evidence.parent().ok_or("evidence parent")?)?;
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check_source_attributes.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&evidence);
    if native {
        command.arg("--native");
    }
    let result = command.output()?;
    std::fs::write(evidence.with_extension("stdout"), &result.stdout)?;
    std::fs::write(evidence.with_extension("stderr"), &result.stderr)?;
    if !result.status.success() {
        return Err(format!(
            "source CLI contracts: {:?}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    if !result.stderr.is_empty() {
        return Err("source CLI contract harness emitted an undeclared diagnostic".into());
    }
    Ok(())
}

#[test]
fn cli_function_contracts_reach_fresh_consumers_with_exact_diagnostics() -> Result {
    qualify(false)
}

#[test]
#[ignore = "requires prebuilt locked object backend, pinned Rust and host C compiler"]
fn cli_function_contracts_reach_fresh_native_consumers_returning_seven() -> Result {
    qualify(true)
}
