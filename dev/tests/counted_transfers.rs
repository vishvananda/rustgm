//! Counted copies preserve span proofs, snapshots and explicit byte-profile boundaries.
use crate::checked_sources as checked;
use crate::counted_cases as declared;
use crate::native_references as native;
use crate::qualify as rustc;
use crate::resolve as source;
#[path = "counted_transfers/work.rs"]
mod work;
use std::path::Path;
#[test]
fn counted_memory_matches_predeclared_language_and_profile_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/counted-memory/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, frontend, reason) in declared::CASES {
        checked::check(
            &root.join(format!("pa28/tests/spec/counted-memory/{name}.rs")),
            &out,
            name,
            (rust, frontend, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn counted_memory_executes_owned_and_reified_transfers() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let inputs = [
        "backend-loop",
        "scalars",
        "disjoint",
        "generic",
        "reified",
        "argument-once",
        "reference-elements",
        "rest-destination",
        "owned-drops",
        "zst-large",
        "shared-zero",
    ]
    .into_iter()
    .map(|name| {
        std::fs::read_to_string(root.join(format!("pa28/tests/counted-memory-native/{name}.rs")))
            .map(|source| (name, source))
    })
    .collect::<Result<Vec<_>, _>>()?;
    let cases = inputs
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/counted-memory/native", &cases)
}
