//! Feature names are requests; only the actual keyword's frame can add allowance.
use super::*;
#[test]
fn default_features_and_actual_impl_owners_keep_observed_token_authority() -> TestResult {
    for (lane, name) in [
        ("oracle-v1", "single-default-method"),
        ("oracle-v1", "wrong-inherent-owner"),
        ("soft-authority", "literal-specialization-allowance"),
    ] {
        let report = resolved(lane, name)?;
        let analysis = pipeline::analyze_source(&report)?;
        assert!(analysis.facts.warnings().next().is_none(), "{name}");
        assert!(report.package.units.iter().any(|unit| {
            unit.configured
                .ast
                .gate_sites
                .as_ref()
                .is_some_and(|entries| entries.len() == 1)
        }));
    }
    for (lane, name) in [
        ("oracle-v1", "missing-feature"),
        ("oracle-v1", "literal-allowed-keyword"),
        ("soft-authority", "forwarded-specialization-allowance"),
        ("soft-authority", "ancestor-specialization-allowance"),
    ] {
        let error = report(&fixture(lane, name)?)
            .err()
            .ok_or("unauthenticated feature became available")?;
        assert!(
            error.message.contains("specialization feature"),
            "{name}: {error}"
        );
        assert!(
            error.warnings().is_empty(),
            "active hard gate is not an unvisited-syntax warning"
        );
    }
    for name in ["wrong-free-owner", "wrong-trait-owner"] {
        let report = resolved("oracle-v1", name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("default function gained an unrelated owner")?;
        assert!(
            error.message.contains("actual impl owner"),
            "{name}: {error}"
        );
    }
    let nested = "#![feature(min_specialization)]struct S;impl S{fn outer(){default fn inner(){}}}fn main(){}";
    let nested_report = report(nested)?;
    assert!(
        pipeline::analyze_source(&nested_report).is_err(),
        "an impl ancestor is not the nested function's owner"
    );
    let ordinary = report("fn main(){}")?;
    assert!(ordinary.definitions.is_empty(), "unused macros stay lazy");
    assert_eq!(crate::expand::builtin::Builtin::BOOTSTRAP.len(), 24);
    for (slot, builtin) in crate::expand::builtin::Builtin::BOOTSTRAP
        .iter()
        .enumerate()
    {
        assert_eq!(builtin.bootstrap_slot(), Some(slot));
    }
    let mut sources = NoFiles;
    let config = Config::default();
    let engine = Engine::create(
        InputFile {
            identity: "ordinary.rs".into(),
            path: "ordinary.rs".into(),
            source: SourceFile::new(b"fn main(){}".to_vec())?,
        },
        &mut sources,
        &config,
        &crate::resolve::bindings::Context::default(),
    )?;
    assert_eq!(engine.builtin_definitions.len(), 24);
    assert!(engine.builtin_definitions.iter().all(Option::is_none));
    Ok(())
}
