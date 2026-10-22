//! Only the actual caller's inputs can pay selected declared-premise debts.
use super::{checked_sources, native};
use std::path::{Path, PathBuf};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "paid-ordinary",
    "paid-pattern",
    "paid-inline",
    "paid-nested",
    "static-control",
    "distinct-instances",
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/caller-input-premises")
        .join(format!("{name}.rs"))
}
#[test]
fn actual_caller_inputs_pay_premises_in_each_source_context() -> Result {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-paid-premises/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for name in [
        "unpaid-pattern",
        "unpaid-inline",
        "foreign-bound",
        "body-debt",
        "cache-unpaid",
        "nested-binder-unpaid",
    ] {
        let types = i32::from(matches!(name, "unpaid-pattern" | "unpaid-inline"));
        checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (1, [types, 1, 1], "not paid by caller inputs"),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn caller_paid_values_execute() -> Result {
    let sources = ACCEPTED
        .iter()
        .map(|name| Ok((*name, std::fs::read_to_string(fixture(name))?)))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| {
            Ok((
                *name,
                text.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    native::check_cases("parallel/pa25-paid-premises/native", &cases)
}
