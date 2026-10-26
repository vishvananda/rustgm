//! A real method body is selected without relaxing the existing overlap rules.
use super::*;
fn selected_impl_body() -> TestResult {
    let report = resolved("oracle-v1", "single-default-method")?;
    let analysis = pipeline::analyze_source(&report)?;
    let trait_id = definition(&report, DefKind::Trait, "T")?;
    let declaration = analysis
        .facts
        .associated
        .members()
        .iter()
        .find(|member| analysis.facts.associated.trait_owner(member.definition) == Some(trait_id))
        .ok_or("actual required trait method")?
        .definition;
    let header = analysis
        .facts
        .generics
        .implementations()
        .find(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .and_then(|(application, _)| analysis.facts.types.trait_application(application))
                .is_some_and(|application| application.item.0 == trait_id.0)
        })
        .ok_or("validated actual implementation")?;
    let selected = header
        .methods
        .iter()
        .find(|(actual, _)| *actual == declaration)
        .and_then(|(_, body)| *body)
        .ok_or("selected actual method body")?;
    assert_ne!(
        selected, declaration,
        "required trait declaration cannot supply a body"
    );
    let key = report.resolution.definitions[selected.0 as usize]
        .node
        .ok_or("selected local function")?;
    let modifier = report
        .active_defaultness(key)?
        .ok_or("selected default body lost its modifier")?;
    assert_eq!(modifier.kind, DefaultKind::Function);
    let node = report.package.unit(key.file).configured.ast.node(key.node);
    let crate::syntax::ast::Kind::Item(index) = node.kind else {
        return Err("selected body is not an item".into());
    };
    assert!(matches!(
        report.package.unit(key.file).configured.ast.items[index as usize].kind,
        ItemKind::Function { body: Some(_), .. }
    ));
    Ok(())
}
fn unsupported_families_stay_closed() -> TestResult {
    for name in [
        "child-overrides",
        "inherited-impl-not-trait-body",
        "final-sibling-cannot-override",
        "new-copy-predicate",
        "repeated-child-input",
        "static-specialization",
        "ordinary-overlap-still-rejected",
    ] {
        let found = resolved("oracle-v1", name)?;
        let error = pipeline::analyze_source(&found)
            .err()
            .ok_or("overlapping family gained an unproved ordering")?;
        assert!(
            error.message.contains("overlapping trait implementation"),
            "{name}: {error}"
        );
    }
    for name in [
        "partial-impl-not-obligation",
        "partial-impl-child-inherits",
        "full-default-constant",
    ] {
        let error = report(&fixture("oracle-v1", name)?)
            .err()
            .ok_or("unsupported default graph/value gained semantics")?;
        assert!(
            error.message.contains("specialization") && error.message.contains("not implemented"),
            "{name}: {error}"
        );
    }
    let associated_type = report(&fixture("soft-edges", "full-default-associated-type")?)
        .err()
        .ok_or("default associated type acquired unsupported semantics")?;
    assert!(
        associated_type
            .message
            .contains("default associated value specialization semantics")
    );
    for name in ["min-default-impl", "min-default-constant"] {
        let error = report(&fixture("oracle-v1", name)?)
            .err()
            .ok_or("min request became full specialization")?;
        assert!(
            error.message.contains("specialization feature"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn nonoverlap_keeps_real_bodies_and_unsupported_specialization_stays_explicit() -> TestResult {
    selected_impl_body()?;
    let invalid = resolved("oracle-v1", "unused-invalid-default-body")?;
    assert!(
        pipeline::analyze_source(&invalid).is_err(),
        "unused default method body was skipped"
    );
    let ordinary = resolved("oracle-v1", "ordinary-trait-default")?;
    pipeline::analyze_source(&ordinary)?;
    assert!(
        ordinary
            .package
            .units
            .iter()
            .all(|unit| unit.configured.ast.gate_sites.is_none())
    );
    unsupported_families_stay_closed()?;
    Ok(())
}
