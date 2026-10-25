//! Unnamed and named items share ordinary typing, evaluation and export selection.
use super::*;
#[test]
fn unused_initializers_are_checked_and_public_placeholders_do_not_export() -> TestResult {
    for width in [1, 8, 32] {
        let mut source = String::from("fn main(){}");
        for _ in 0..width {
            source.push_str("const _:u8=7;");
        }
        let found = report(&source)?;
        checked_rows(&found, width)?;
        let analysis = pipeline::analyze_source(&found)?;
        let facts = &analysis.facts;
        assert_eq!(facts.initializers.len(), width);
        assert_eq!(facts.constant_work.initializer_indexed, width);
        assert_eq!(facts.constant_work.initializer_visits, width);
        assert_eq!(facts.constant_work.initializer_validations, width);
        println!(
            "unnamed constants width={width} definitions={} bindings={} initializers={} visits={} validations={} steps={}",
            found.resolution.definitions.len(),
            found.resolution.bindings.iter().count(),
            facts.initializers.len(),
            facts.constant_work.initializer_visits,
            facts.constant_work.initializer_validations,
            facts.constant_work.steps
        );
    }
    for name in [
        "unused-wrong-type",
        "unused-divzero",
        "unused-function-divzero",
        "macro-wrong-type",
        "macro-divzero",
    ] {
        let found = report(&fixture(25, name)?)?;
        assert!(!constants(&found).is_empty());
        assert!(pipeline::analyze_source(&found).is_err(), "{name}");
    }
    let found = report(&fixture(25, "public-library")?)?;
    let mut analysis = pipeline::analyze_source(&found)?;
    assert_eq!(analysis.facts.constant_work.initializer_validations, 2);
    assert_eq!(analysis.facts.warnings().count(), 1);
    let identity = crate::support::crates::CrateIdentity::new("unnamed", "public")?;
    let exported = crate::sema::crates::export(&found, &mut analysis.facts, identity)?.interface;
    assert_eq!(exported.constants().len(), 1);
    assert_eq!(exported.constants()[0].definition().path(), ["NAMED"]);
    let bytes = crate::support::crates::encode(&exported)?;
    let decoded = crate::support::crates::decode(&bytes)?;
    assert_eq!(decoded.constants().len(), 1);
    assert_eq!(decoded.constants()[0].definition().path(), ["NAMED"]);
    let minimal = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let source = format!("{minimal}\npub const _:u8=1;\n");
    let error = crate::source_packets::exported(&source)
        .err()
        .ok_or("closed source profile accepted a const row")?;
    assert!(
        error
            .to_string()
            .contains("source declaration kind is outside the initial source profile")
    );
    let alignment =
        crate::test_source::fixture("pa31/tests/spec/unnamed-constants/alignment-reducer.rs")?;
    let found = report(&alignment)?;
    checked_rows(&found, 2)?;
    assert_eq!(
        pipeline::analyze_source(&found)?
            .facts
            .constant_work
            .initializer_validations,
        2
    );
    Ok(())
}
