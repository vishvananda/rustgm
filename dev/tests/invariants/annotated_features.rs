//! Source feature clauses retain exact owners without publishing a body permission.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str =
    "#![feature(staged_api,rustc_attrs)]#![stable(feature=\"source_fixture\",since=\"1.0.0\")]";
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/feature-bound-predicates/{name}.rs"
    ))?)
}
fn definition(report: &Report, name: &str) -> Result<(DefId, NodeKey)> {
    let (index, row) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            report.package.names.get(row.name) == name
                && matches!(row.kind, DefKind::Trait | DefKind::Function)
        })
        .ok_or("actual source declaration")?;
    Ok((
        DefId(u32::try_from(index)?),
        row.node.ok_or("actual source key")?,
    ))
}
#[test]
fn actual_trait_function_and_impl_annotations_form_paid_non_type_clauses() -> Result {
    for (name, declaration) in [
        ("trait-declaration", "NeedsGate"),
        ("function-chain", "guarded"),
        ("matching-caller", "observed"),
    ] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let (owner, key) = definition(&report, declaration)?;
        let producer = report.lint_producer().ok_or("actual producer")?;
        assert!(
            producer
                .impl_annotation(&report.package, key)
                .map_err(Error::from)?
                .is_none()
        );
        assert!(
            producer
                .feature_bound_keys()
                .any(|candidate| candidate == key)
        );
        let before = producer.work();
        let clauses = declaration_clauses(&report, &mut facts, owner, key)?
            .ok_or("actual feature annotation")?;
        assert_eq!(clauses.len(), 1);
        let Clause::UnstableFeature(id) = clauses[0].value else {
            return Err("missing real non-type clause".into());
        };
        assert_eq!(facts.types.feature_name(id), Some("feature_bound_probe"));
        let cold = producer.work();
        let warm = declaration_clauses(&report, &mut facts, owner, key)?.ok_or("warm clause")?;
        assert_eq!(warm[0].value, clauses[0].value);
        assert!(cold.steps > before.steps && producer.work().steps > cold.steps);
        assert!(declaration_clauses(&report, &mut facts, DefId(u32::MAX), key).is_err());
        let (other, other_key) = definition(&report, "value")?;
        assert!(declaration_clauses(&report, &mut facts, other, other_key)?.is_none());
        assert!(declaration_clauses(&report, &mut facts, owner, other_key).is_err());
    }
    let report = crate::test_source::resolve(&fixture("impl-declaration")?, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let owner = *facts
        .associated
        .owners()
        .iter()
        .find(|owner| {
            matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: Some(_),
                    ..
                }
            )
        })
        .ok_or("actual impl")?;
    let key = owner.source.ok_or("source impl")?;
    assert!(
        report
            .lint_producer()
            .ok_or("producer")?
            .impl_annotation(&report.package, key)
            .map_err(Error::from)?
            .is_some()
    );
    assert_eq!(
        declaration_clauses(&report, &mut facts, owner.self_definition, key)?
            .ok_or("impl requirements")?
            .len(),
        1
    );
    Ok(())
}
#[test]
fn feature_owner_validation_is_independent_of_empty_lists_and_const_policy() -> Result {
    let mut report = crate::test_source::resolve(&fixture("empty-list")?, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let (owner, key) = definition(&report, "declaration")?;
    assert!(
        declaration_clauses(&report, &mut facts, owner, key)?
            .ok_or("empty annotation")?
            .is_empty()
    );
    assert!(
        report
            .lint_producer()
            .ok_or("actual producer")?
            .require_impl_bound_transport(&report.package)
            .is_err()
    );
    let producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("actual producer")?;
    let error = declaration_clauses(&report, &mut facts, owner, key)
        .err()
        .ok_or("missing producer became checked absence")?;
    assert!(
        error
            .to_string()
            .contains("authenticated attribute producer")
    );
    report
        .attribute_effects
        .as_deref_mut()
        .ok_or("actual effects")?
        .policy = Some(producer);
    let missing = crate::test_source::resolve(&fixture("missing-request")?, true)?;
    let (missing_owner, missing_key) = definition(&missing, "declaration")?;
    let error = declaration_clauses(&missing, &mut facts, missing_owner, missing_key)
        .err()
        .ok_or("unstaged source formed feature clauses")?;
    assert!(error.to_string().contains("staged_api"), "{error}");
    for source in [
        format!(
            "{ROOT}#[stable(feature=\"source_fixture\",since=\"1.0.0\")]#[unstable_feature_bound(probe)]fn f(){{}}"
        ),
        format!(
            "{ROOT}#[stable(feature=\"source_fixture\",since=\"1.0.0\")]#[rustc_const_unstable(feature=\"probe\",issue=\"none\")]#[unstable_feature_bound(probe)]const fn f(){{}}"
        ),
        format!(
            "{ROOT}#[stable(feature=\"source_fixture\",since=\"1.0.0\")]trait T{{#[unstable(feature=\"probe\",issue=\"none\")]#[unstable_feature_bound(probe)]fn f();}}"
        ),
        format!(
            "{ROOT}unsafe extern \"C\"{{#[unstable(feature=\"probe\",issue=\"none\")]#[unstable_feature_bound(probe)]fn f();}}"
        ),
    ] {
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let (owner, key) = definition(&report, "f")?;
        assert!(declaration_clauses(&report, &mut facts, owner, key).is_err());
    }
    for name in [
        "word-form",
        "wrong-owner",
        "malformed-name",
        "unsafe-wrapper",
    ] {
        assert!(
            crate::test_source::resolve(&fixture(name)?, true).is_err(),
            "{name}"
        );
    }
    let repeated = format!(
        "{ROOT}#[unstable(feature=\"probe\",issue=\"none\")]#[unstable_feature_bound(probe)]#[unstable_feature_bound(other)]fn f(){{}}"
    );
    assert!(crate::test_source::resolve(&repeated, true).is_err());
    Ok(())
}
#[test]
fn paid_clause_materialization_tracks_actual_feature_arity_and_stays_data_only() -> Result {
    use std::fmt::Write as _;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut names = String::new();
        for index in 0..width {
            write!(names, "probe_{index},")?;
        }
        let source = format!(
            "{ROOT}#[unstable(feature=\"probe\",issue=\"none\")]#[unstable_feature_bound({names})]fn f(){{}}"
        );
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let (owner, key) = definition(&report, "f")?;
        let count = facts.generics.predicates.get(&owner).map_or(0, Vec::len);
        let producer = report.lint_producer().ok_or("actual source payer")?;
        let before = producer.work();
        let clauses =
            declaration_clauses(&report, &mut facts, owner, key)?.ok_or("actual clauses")?;
        assert_eq!(clauses.len(), width);
        let cold = producer.work();
        let warm = declaration_clauses(&report, &mut facts, owner, key)?.ok_or("warm clauses")?;
        assert_eq!(
            warm.iter().map(|p| p.value).collect::<Vec<_>>(),
            clauses.iter().map(|p| p.value).collect::<Vec<_>>()
        );
        let after = producer.work();
        assert_eq!(
            facts.generics.predicates.get(&owner).map_or(0, Vec::len),
            count
        );
        assert!(after.bytes >= cold.bytes + width * size_of::<Predicate>());
        observations.push((cold.steps - before.steps, after.steps - cold.steps));
        eprintln!(
            "source feature arity={width} cold={} warm={} bytes={}",
            cold.steps - before.steps,
            after.steps - cold.steps,
            after.bytes - before.bytes
        );
    }
    for axis in 0..2 {
        let values: Vec<_> = observations
            .iter()
            .map(|&(cold, warm)| if axis == 0 { cold } else { warm })
            .collect();
        let first = values[1]
            .checked_sub(values[0])
            .ok_or("nonmonotone first interval")?;
        let second = values[2]
            .checked_sub(values[1])
            .ok_or("nonmonotone second interval")?;
        assert!(first > 0 && second <= 6 * first, "{values:?}");
    }
    Ok(())
}
