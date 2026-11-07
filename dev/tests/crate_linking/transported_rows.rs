//! Imported `macro_rules!` rows are authenticated once and bind `$crate` by identity.
use super::{artifacts, exports::compile, imports::resolve};
use rustgm::support::crates::{self, Dependencies, Dependency, Interface};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PROVIDER: &str = "pub fn f()->u8{40}#[macro_export]macro_rules! call_f{()=>{$crate::f()};}";

fn qualify(native: bool) -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join(format!(
        "obj/parallel/pa29-macro-metadata/maintainer-{}-{native}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let mut command = Command::new("python3");
    command
        .current_dir(root)
        .arg(root.join("scripts/check_macro_metadata.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&folder);
    if native {
        command.arg("--native");
    }
    artifacts::checked("harness", &folder, &mut command)?;
    Ok(folder)
}

fn client(text: &str, alias: &str, artifact: Interface) -> std::result::Result<(), String> {
    let identity = artifact.identity().clone();
    let dependencies = Dependencies::new(
        vec![artifact],
        vec![Dependency::new(alias.into(), identity)?],
    )?;
    resolve(text, Arc::new(dependencies)).map(drop)
}

/// The encoded artifact with one byte run respelled in place.
fn forged(bytes: &[u8], from: &[u8], to: &[u8]) -> Result<Vec<u8>> {
    let at = bytes
        .windows(from.len())
        .position(|window| window == from)
        .ok_or("forged bytes not found")?;
    let mut copy = bytes.to_vec();
    copy[at..at + to.len()].copy_from_slice(to);
    Ok(copy)
}

#[test]
fn frozen_macro_metadata_outcomes_hold_for_every_case() -> Result {
    let folder = qualify(false)?;
    let receipt = std::fs::read_to_string(folder.join("complete.json"))?;
    // Fifteen declarative cases and fourteen builtin-row and allowance cases.
    assert!(receipt.contains("\"cases\": 29"), "{receipt}");
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM object backend and pinned Rust toolchain"]
fn frozen_macro_metadata_clients_match_rustc_natively() -> Result {
    qualify(true).map(drop)
}

#[test]
fn transported_rows_must_rebuild_from_one_lex_and_keep_their_rule_count() -> Result {
    let (_, interface) = compile(PROVIDER, "provider")?;
    let bytes = crates::encode(&interface)?;
    let call = "fn f()->u8{0}fn main(){let _:u8=renamed::call_f!();}";
    // `$crate` names the provider through its crate id, whatever its alias,
    // and a local `f` cannot capture it.
    client(call, "renamed", crates::decode(&bytes)?)?;
    // A failed match names the definition in its image unit, not a builtin.
    let error = client(
        "fn main(){let _:u8=renamed::call_f!(1);}",
        "renamed",
        crates::decode(&bytes)?,
    )
    .err()
    .ok_or("unmatched imported call accepted")?;
    assert!(
        error.contains("no macro rule matches") && error.contains("; defined at file 1: 0"),
        "{error}"
    );
    // An inner space passes the decoder's layout check, but the one re-lex
    // splits the token, so the rebuilt rows differ.
    let respelled = crates::decode(&forged(&bytes, b"$crate::f", b"$cr te::f")?)?;
    let error = client(call, "renamed", respelled)
        .err()
        .ok_or("forged image accepted")?;
    assert!(
        error.contains("does not rebuild its canonical rows"),
        "{error}"
    );
    // The rule count sits just before the image's length prefix.
    let image = b"{()=>{$crate::f()};}";
    let at = bytes
        .windows(image.len())
        .position(|window| window == image)
        .ok_or("missing image")?;
    let mut recounted = bytes.clone();
    recounted[at - 8..at - 4].copy_from_slice(&2u32.to_le_bytes());
    let error = client(call, "renamed", crates::decode(&recounted)?)
        .err()
        .ok_or("forged rule count accepted")?;
    assert!(
        error.contains("imported macro rule count changed"),
        "{error}"
    );
    Ok(())
}

#[test]
fn macro_use_rows_enter_the_prelude_and_later_crates_shadow_earlier_ones() -> Result {
    let (_, first) = compile(PROVIDER, "first")?;
    let (_, second) = compile(
        "pub fn f()->u16{9}#[macro_export]macro_rules! call_f{()=>{$crate::f()};}",
        "second",
    )?;
    let (first_bytes, second_bytes) = (crates::encode(&first)?, crates::encode(&second)?);
    let dependencies = |reverse: bool| -> Result<Arc<Dependencies>> {
        let mut aliases = vec![
            Dependency::new("first".into(), first.identity().clone())?,
            Dependency::new("second".into(), second.identity().clone())?,
        ];
        if reverse {
            aliases.reverse();
        }
        let artifacts = vec![
            crates::decode(&first_bytes)?,
            crates::decode(&second_bytes)?,
        ];
        Ok(Arc::new(Dependencies::new(artifacts, aliases)?))
    };
    // The later `#[macro_use]` crate supplies `call_f!`, so its `u16` result
    // type-checks, whatever the dependency order. The path form is unaffected.
    let text = "#[macro_use]extern crate first;#[macro_use]extern crate second;fn main(){let _:u16=call_f!();let _:u8=first::call_f!();}";
    for reverse in [false, true] {
        let report = resolve(text, dependencies(reverse)?)?;
        rustgm::pipeline::analyze_source(&report)?;
    }
    let swapped = "#[macro_use]extern crate second;#[macro_use]extern crate first;fn main(){let _:u16=call_f!();}";
    let report = resolve(swapped, dependencies(false)?)?;
    assert!(rustgm::pipeline::analyze_source(&report).is_err());
    Ok(())
}
