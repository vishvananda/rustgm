//! Failed or unauthorized bodies cannot publish a named alias's hidden type.
use super::*;
use crate::{
    resolve::expansion::Report,
    sema::model::Declarations,
    types::{Kind, Primitive, TypeId},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<Report> {
    let source = crate::test_source::fixture(&format!("pa34/tests/spec/named-opaques/{name}.rs"))?;
    Ok(crate::test_source::source_provider(
        &source,
        "named_publication",
    )?)
}
fn reject(report: &Report, facts: &mut Declarations) -> TestResult<String> {
    facts.prepare_named_opaques(report)?;
    let mut checker = Checker::new(report, facts);
    let error = checker
        .functions(&mut |_, _, _| Ok(()), None)
        .err()
        .ok_or("body opponent accepted")?;
    Ok(error.to_string())
}

fn define_first(report: &Report, facts: &mut Declarations) -> TestResult {
    facts.prepare_named_opaques(report)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "make"
        })
        .copied()
        .ok_or("actual named defining function")?;
    Checker::new(report, facts).function(function)?;
    assert!(facts.typed_function_bodies.contains(&function.definition));
    assert!(
        facts
            .opaques
            .declarations
            .values()
            .all(|declaration| { declaration.hidden == Some(TypeId::primitive(Primitive::U8)) })
    );
    Ok(())
}
#[test]
fn failed_first_definition_keeps_its_hidden_type_unpublished() -> TestResult {
    let report = source("bound-violation")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    reject(&report, &mut facts)?;
    assert!(
        facts
            .opaques
            .declarations
            .values()
            .all(|d| d.hidden.is_none() && d.inferred.is_none())
    );
    assert!(
        facts
            .typed_function_bodies
            .iter()
            .all(|owner| !facts.opaques.owners.contains_key(owner))
    );
    Ok(())
}
#[test]
fn inconsistent_second_definition_preserves_the_first_closed_hidden_type() -> TestResult {
    let report = source("inconsistent-defining-bodies")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let error = reject(&report, &mut facts)?;
    assert!(error.contains("concrete type differs"), "{error}");
    let (&id, declaration) = facts.opaques.declarations.iter().next().ok_or("alias")?;
    assert_eq!(declaration.hidden, Some(TypeId::primitive(Primitive::U8)));
    let complete = facts
        .opaques
        .owners
        .iter()
        .filter(|(owner, ids)| ids.contains(&id) && facts.typed_function_bodies.contains(owner))
        .count();
    assert_eq!(complete, 1);
    Ok(())
}
#[test]
fn a_known_hidden_representation_does_not_authorize_an_ordinary_body_equation() -> TestResult {
    let report = source("unauthorized-body")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    define_first(&report, &mut facts)?;
    reject(&report, &mut facts)?;
    let declaration = facts.opaques.declarations.values().next().ok_or("alias")?;
    assert_eq!(declaration.hidden, Some(TypeId::primitive(Primitive::U8)));
    assert!(matches!(
        facts.types.kind(declaration.application),
        Some(Kind::Opaque(..))
    ));
    assert_eq!(facts.opaques.owners.len(), 1);
    Ok(())
}

#[test]
fn a_forged_function_membership_cannot_create_a_named_defining_permit() -> TestResult {
    let report = source("simple")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    define_first(&report, &mut facts)?;
    let id = *facts.opaques.declarations.keys().next().ok_or("alias")?;
    let ordinary = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "ordinary_use"
        })
        .map(|function| function.definition)
        .ok_or("ordinary function")?;
    facts.opaques.owners.insert(ordinary, vec![id]);
    let error = reject(&report, &mut facts)?;
    assert!(
        error.contains("not authorized in this defining body"),
        "{error}"
    );
    assert_eq!(
        facts.opaques.declarations[&id].hidden,
        Some(TypeId::primitive(Primitive::U8))
    );
    assert!(!facts.typed_function_bodies.contains(&ordinary));
    Ok(())
}
