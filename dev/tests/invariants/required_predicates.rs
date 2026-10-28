//! Whole local-source outcomes frozen and independently qualified before implementation.
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn semantic_view_prints_canonical_feature_names_without_type_subjects_or_foreign_ids() -> TestResult
{
    let source = crate::test_source::fixture(
        "pa31/tests/spec/feature-bound-predicates/trait-declaration.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let mut output = Vec::new();
    crate::sema::semantic_view::write(&report, &facts, &mut output)?;
    let text = String::from_utf8(output)?;
    let feature_rows = text
        .lines()
        .filter(|line| line.contains(" feature="))
        .collect::<Vec<_>>();
    assert_eq!(feature_rows.len(), 1);
    assert!(feature_rows[0].ends_with("feature=\"feature_bound_probe\""));
    assert!(!feature_rows[0].contains("subject="));
    let mut foreign = crate::types::Store::default();
    let id = foreign.intern_feature_charged("feature_bound_probe", &|_, _| Ok(()))?;
    let clause = facts
        .generics
        .predicates
        .values_mut()
        .flatten()
        .find(|predicate| {
            matches!(
                predicate.value,
                crate::types::parameters::clauses::Clause::UnstableFeature(_)
            )
        })
        .ok_or("genuine declared feature")?;
    clause.value = crate::types::parameters::clauses::Clause::UnstableFeature(id);
    assert!(crate::sema::semantic_view::write(&report, &facts, &mut Vec::new()).is_err());
    Ok(())
}

#[test]
fn original_local_feature_predicate_sources_match_the_pinned_metadata_contract() -> TestResult {
    let cases = [
        ("trait-declaration", 0),
        ("impl-declaration", 0),
        ("matching-caller", 0),
        ("missing-caller", 1),
        ("unrelated-caller", 1),
        ("global-only-staged", 1),
        ("stable-pair", 1),
        ("nested-function", 1),
        ("closure-inheritance", 0),
        ("function-chain", 0),
        ("function-chain-missing", 1),
        ("impl-missing", 1),
        ("word-form", 1),
        ("empty-list", 0),
        ("wrong-owner", 1),
        ("missing-request", 1),
        ("malformed-name", 1),
        ("unsafe-wrapper", 1),
        ("reified-matching", 0),
        ("reified-missing", 1),
        ("generic-forward", 1),
        ("generic-forward-missing", 1),
        ("generic-forward-guarded", 0),
        ("generic-call-missing", 1),
        ("generic-bound-unused", 1),
        ("generic-bound-in-scope", 0),
    ];
    let mut disagreements = Vec::new();
    for (name, expected) in cases {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/feature-bound-predicates/{name}.rs"
        ))?;
        let outcome = crate::test_source::resolve(&source, true).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        let actual = i32::from(outcome.is_err());
        let error = outcome.err();
        println!("feature-source {name} expected={expected} actual={actual} error={error:?}");
        let unsupported = error.as_deref().is_some_and(|message| {
            [
                "not implemented",
                "not supported",
                "budget exceeded",
                "infrastructure",
                "non-type clause requires its feature consumer",
            ]
            .iter()
            .any(|needle| message.contains(needle))
        });
        if actual != expected || unsupported {
            disagreements.push(format!(
                "{name}: expected {expected}, actual {actual}, {error:?}"
            ));
        }
    }
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n"));
    Ok(())
}
