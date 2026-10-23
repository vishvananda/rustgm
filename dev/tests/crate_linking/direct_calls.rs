//! Independent source cases exercise safe generic dependencies across metadata.
use super::{exports, imports};
use rustgm::{
    lower, pipeline,
    support::crates::{self, Dependencies, Dependency},
};
use std::{path::Path, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa29/tests/spec/generic-contracts/{name}.rs")),
    )?)
}
fn dependencies(provider: &str) -> Result<Arc<Dependencies>> {
    let (_, interface) = exports::compile(&fixture(provider)?, "direct_calls")?;
    let encoded = crates::encode(&interface)?;
    let interface = crates::decode(&encoded)?;
    assert_eq!(crates::encode(&interface)?, encoded);
    let payload = interface.generic_payload().ok_or("generic payload")?;
    assert!(payload.rows().calls.is_some());
    let identity = interface.identity().clone();
    Ok(Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), identity)?],
    )?))
}
fn client(name: &str, dependencies: Arc<Dependencies>) -> Result {
    let mut report = imports::resolve(&fixture(name)?, dependencies)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    Ok(())
}
#[test]
fn imported_safe_generic_helpers_pay_complete_caller_contracts() -> Result {
    let deps = dependencies("call-library")?;
    client("pass-helper-unused", Arc::clone(&deps))?;
    client("call-const", Arc::clone(&deps))?;
    for name in [
        "call-reject-copy",
        "call-reject-unused",
        "call-reject-private",
    ] {
        assert!(client(name, Arc::clone(&deps)).is_err(), "{name}");
    }
    assert!(exports::compile(&fixture("call-reject-source")?, "invalid_unused").is_err());
    assert!(exports::compile(&fixture("call-unsafe-library")?, "unsafe_required").is_err());
    let deps = dependencies("call-runtime-library")?;
    client("pass-helper-unused", Arc::clone(&deps))?;
    assert!(client("call-const", deps).is_err());
    Ok(())
}
#[test]
fn recursive_generic_originals_are_finite_until_actual_constant_demand() -> Result {
    let deps = dependencies("call-recursive-library")?;
    client("call-unused", Arc::clone(&deps))?;
    let Err(error) = client("call-reject-recursion", deps) else {
        return Err("demanded recursive constant completed".into());
    };
    assert!(
        error
            .to_string()
            .contains("constant execution frame limit exceeded"),
        "{error}"
    );
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM backend; generic-call-outcomes.json independently qualifies original sources"]
fn imported_generic_helpers_execute_native_runtime_and_constant_clients() -> Result {
    use std::process::Command;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa29-crate-linking/direct-calls-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (provider, clients) in [
        ("call-library", vec!["pass-helper-unused", "call-const"]),
        ("call-runtime-library", vec!["pass-helper-unused"]),
        ("call-recursive-library", vec!["call-unused"]),
    ] {
        let metadata = folder.join(format!("{provider}.rgm"));
        let disambiguator = provider.replace('-', "_");
        super::artifacts::checked(
            provider,
            &folder,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .args([
                    "--emit-lowir",
                    "--crate-type=lib",
                    "--crate-name",
                    "base",
                    "--crate-disambiguator",
                    &disambiguator,
                    "--metadata-output",
                ])
                .arg(&metadata)
                .arg("-o")
                .arg(folder.join(format!("{provider}.lowir")))
                .arg(root.join(format!("pa29/tests/spec/generic-contracts/{provider}.rs"))),
        )?;
        for client in clients {
            let name = format!("{provider}-{client}");
            let lowir = folder.join(format!("{name}.lowir"));
            super::artifacts::checked(
                &name,
                &folder,
                Command::new(env!("CARGO_BIN_EXE_rustgm"))
                    .arg("--emit-lowir")
                    .arg(root.join(format!("pa29/tests/spec/generic-contracts/{client}.rs")))
                    .arg("--extern")
                    .arg(format!("base={}", metadata.display()))
                    .arg("-o")
                    .arg(&lowir),
            )?;
            let native = folder.join(format!("{name}.native"));
            super::artifacts::checked(
                &format!("{name}.backend"),
                &folder,
                Command::new("python3")
                    .arg(root.join("scripts/backend.py"))
                    .arg(lowir)
                    .arg("-o")
                    .arg(&native),
            )?;
            super::artifacts::checked(
                &format!("{name}.execute"),
                &folder,
                &mut Command::new(native),
            )?;
        }
    }
    Ok(())
}
