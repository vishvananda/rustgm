//! Ordinary method paths must preserve lifetime-only prefixes and caller debts.
use super::{cases, native};
use std::path::Path;
const CASES: &[(&str, bool)] = &[
    ("inherent-lifetime-only-turbofish", true),
    ("trait-lifetime-only-turbofish", true),
    ("inherent-lifetime-only-arity", false),
    ("inherent-lifetime-only-debt", false),
];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa19/tests/spec/implicit-paths/{name}.rs")),
    )
}
#[test]
fn lifetime_only_prefixes_keep_arity_and_lifetime_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        cases(&[(name, &source(name)?, accepted)])?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn lifetime_only_prefixes_execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = Vec::new();
    for &(name, accepted) in CASES {
        if accepted {
            sources.push((name, source(name)?));
        }
    }
    let rows = sources
        .iter()
        .map(|(name, source)| {
            source
                .strip_prefix(native::CHECK)
                .map(|body| (*name, body, false))
                .ok_or("missing fixture check helper")
        })
        .collect::<Result<Vec<_>, _>>()?;
    native::check_cases("prefix-lifetimes", &rows)
}
