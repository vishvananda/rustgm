//! Half-open integer ranges retain canonical methods and ordinary iteration proofs.
use super::{mutation::frontend, native, rustc};
type SourceCase = (&'static str, String, bool);
fn cases() -> Result<Vec<SourceCase>, Box<dyn std::error::Error>> {
    // Required unimplemented Rust positives are tracked separately in the outcome manifest.
    let mut result = Vec::new();
    for &(name, accepted) in CASES {
        let text = std::fs::read_to_string(format!(
            "{}/../pa26/tests/spec/iteration/integer-ranges/{name}.rs",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        result.push((name, text, accepted));
    }
    Ok(result)
}
const CASES: &[(&str, bool)] = &[
    ("range-exclusive", true),
    ("range-reversed", true),
    ("named-range", true),
    ("range-float-reject", false),
    ("range-to-reject", false),
    ("range-full-reject", false),
    ("source-lookalike-range", false),
    ("integer-u8", true),
    ("integer-u16", true),
    ("integer-u32", true),
    ("integer-u64", true),
    ("integer-u128", true),
    ("integer-usize", true),
    ("integer-i8", true),
    ("integer-i16", true),
    ("integer-i32", true),
    ("integer-i64", true),
    ("integer-i128", true),
    ("integer-isize", true),
    ("default-inference", true),
    ("exclusive-max", true),
    ("qualified-next", true),
    ("reified-next", true),
    ("alias-fields", true),
    ("generic-iterator", true),
    ("mutable-blanket", true),
    ("break-continue", true),
    ("formation-once", true),
    ("copy-range", false),
    ("source-shadow-syntax", true),
    ("source-range-iterator", true),
    ("empty", true),
    ("mismatched-endpoint", false),
    ("nominal-no-step", false),
    ("generic-no-step", false),
    ("shared-range-not-iterator", false),
    ("source-range-canonical-impl-conflict", false),
    ("range-bool-reject", false),
    ("body-selects-u8", true),
    ("body-selects-index", true),
    ("bound-selects-usize", true),
    ("no-prelude-loop", true),
    ("named-alias", true),
    ("method-ambiguity", false),
    ("source-primitive-name-shadow", true),
    ("constant-range", true),
    ("const-next-reject", false),
];
#[test]
fn integer_range_outcomes_preserve_variants_inference_and_lookup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (name, text, accepted) in cases()? {
        rustc::qualify(
            &format!("parallel/pa26/integer-ranges/source/{name}"),
            &text,
            accepted,
        )?;
        let result = frontend::checked(&text);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn integer_ranges_execute_selected_next_and_control_flow() -> Result<(), Box<dyn std::error::Error>>
{
    let sources = cases()?
        .into_iter()
        .filter(|(_, _, accepted)| *accepted)
        .collect::<Vec<_>>();
    let cases = sources
        .iter()
        .map(|(name, text, _)| (*name, text.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa26/integer-ranges/native", &cases)
}
