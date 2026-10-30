//! Only genuine required conversion members receive v9 conditional headers.
use super::*;
use crate::{pipeline, resolve::model::DefKind, types::TypeId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn prepared(name: &str) -> Result<(Report, Declarations)> {
    let source = crate::test_source::fixture(&format!("pa31/tests/spec/source-from/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "conversion_headers")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis.facts))
}
fn required(facts: &Declarations) -> Result<crate::sema::model::Function> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .header_role(crate::support::bindings::LangItem::From)?
        .ok_or("From role")?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or("owner")?;
    facts
        .associated
        .owner_members(owner)
        .ok_or("members")?
        .iter()
        .filter_map(|definition| {
            facts
                .function_indices
                .get(definition)
                .and_then(|&index| facts.functions.get(index))
        })
        .find(|function| function.body.is_none())
        .copied()
        .ok_or("required member".into())
}
#[test]
fn actual_const_from_required_header_is_conditional_only_in_version_nine() -> Result {
    let (report, facts) = prepared("original-trait")?;
    let function = required(&facts)?;
    assert_eq!(
        written_constness(&report, &facts, &function, true)?,
        Constness::Conditional
    );
    assert_eq!(
        written_constness(&report, &facts, &function, false)?,
        Constness::Ordinary
    );
    assert!(!facts.is_const_function(function.definition));
    let ordinary = facts
        .functions
        .iter()
        .find(|function| {
            function.body.is_some() && facts.associated.owner(function.definition).is_none()
        })
        .ok_or("ordinary observer")?;
    assert_eq!(
        written_constness(&report, &facts, ordinary, true)?,
        Constness::Ordinary
    );
    let (ordinary_report, ordinary_facts) = prepared("actual-role")?;
    let ordinary_member = required(&ordinary_facts)?;
    assert_eq!(
        written_constness(&ordinary_report, &ordinary_facts, &ordinary_member, true)?,
        Constness::Ordinary
    );
    Ok(())
}
#[test]
fn required_member_cannot_borrow_another_parent_function_or_effect() -> Result {
    let (report, mut facts) = prepared("original-trait")?;
    let original = required(&facts)?;
    let mut changed = original;
    changed.node = facts
        .functions
        .iter()
        .find(|function| function.body.is_some())
        .ok_or("other source function")?
        .node;
    assert!(written_constness(&report, &facts, &changed, true).is_err());
    facts.constant_functions.insert(original.definition);
    assert!(written_constness(&report, &facts, &original, true).is_err());
    facts.constant_functions.remove(&original.definition);
    let mut changed = original;
    changed.body = Some(original.node);
    assert!(written_constness(&report, &facts, &changed, true).is_err());
    assert_eq!(
        written_constness(&report, &facts, &original, true)?,
        Constness::Conditional
    );
    Ok(())
}
#[test]
fn private_alias_closure_keeps_actual_canonical_uses_and_rejects_missing_facts() -> Result {
    let (report, mut facts) = prepared("subject-alias")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let alias = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::TypeAlias)
        .ok_or("actual private alias")?;
    let definition = DefId(u32::try_from(alias)?);
    let ty = facts.definitions[alias].ok_or("actual alias type")?;
    let mut stage = Stage::begin_module_conversions(&report, &mut facts, &identity)?;
    assert!(stage.definitions[alias].is_none());
    assert_eq!(facts.definitions[alias], Some(ty));
    facts.definitions[alias] = None;
    assert!(stage.private_alias(&report, &facts, definition).is_err());
    facts.definitions[alias] = Some(TypeId::UNIT);
    assert!(stage.private_alias(&report, &facts, definition).is_err());
    facts.definitions[alias] = Some(ty);
    stage.private_alias(&report, &facts, definition)?;
    stage.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(stage.private_alias(&report, &facts, definition).is_err());
    assert!(stage.private_alias(&report, &facts, definition).is_err());
    assert!(stage.definitions[alias].is_none());
    Ok(())
}
