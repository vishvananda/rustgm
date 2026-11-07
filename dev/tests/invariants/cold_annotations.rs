//! Stable cold words retain independent header data without changing const rules.
use crate::test_source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa26/tests/spec/attributes/cold/{name}.rs"
    ))?)
}
#[test]
fn cold_words_preserve_inline_and_const_semantics_with_explicit_transport_debt() -> Result {
    let report = test_source::resolve(&fixture("valid")?, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let (id, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "ANSWER")
        .ok_or("actual const ANSWER")?;
    assert!(
        crate::sema::values::Reader::new(&facts)
            .definition(crate::resolve::model::DefId(u32::try_from(id)?))
            .is_some()
    );
    for name in ["malformed", "unsafe-wrapper", "nonconst"] {
        let result: Result<crate::pipeline::Analysis> = test_source::resolve(&fixture(name)?, true)
            .map_err(Into::into)
            .and_then(|report| crate::pipeline::analyze_source(&report).map_err(Into::into));
        assert!(result.is_err(), "{name}");
    }
    for name in ["required-prototype", "required-foreign"] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        crate::pipeline::analyze_source(&report)?;
    }
    let source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            .replace(
                "pub fn identity<T>",
                "#[cold]#[inline(never)]pub fn identity<T>",
            );
    let report = test_source::source_provider(&source, "cold_source")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "identity")
        .ok_or("actual source identity")?;
    let hint = facts
        .inline_hint_for_definition(&report, crate::resolve::model::DefId(u32::try_from(index)?))?
        .ok_or("independent inline hint")?;
    assert_eq!(hint.hint, super::Hint::Never);
    for profile in [
        crate::lower::instances::SourcePublication::LegacyFunctions,
        crate::lower::instances::SourcePublication::FunctionContracts,
    ] {
        let error = crate::module_packets::exported_profile(&source, profile, "cold_publication")
            .err()
            .ok_or("cold data dropped")?;
        assert!(
            error
                .to_string()
                .contains("cold advisory marker transport is not implemented"),
            "{error}"
        );
    }
    Ok(())
}

fn no_body_cost(width: usize) -> Result<(usize, usize)> {
    use std::fmt::Write;
    let mut source = String::from("unsafe extern \"C\"{#[cold]fn external();}");
    for index in 0..width {
        write!(source, "#[cold]pub fn ordinary{index}(){{}}")?;
    }
    let report = test_source::resolve(&source, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok(crate::sema::header_attributes::source_contract_costs(
        &facts,
    ))
}
#[test]
fn no_body_cold_accounting_does_not_retain_ordinary_function_rows() -> Result {
    let small = no_body_cost(32)?;
    let large = no_body_cost(128)?;
    assert!(large.0 > small.0 && large.0 <= small.0 * 4);
    assert_eq!(large.1, small.1);
    Ok(())
}
#[test]
fn no_body_cold_validation_rejects_foreign_owners_and_source_replays() -> Result {
    use crate::resolve::model::{DefId, DefKind, ScopeId};
    let source = "unsafe extern \"C\"{#[cold]fn external();}";
    let mut report = test_source::resolve(source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let (index, row) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == "external"
        })
        .ok_or("actual foreign prototype")?;
    let key = row.node.ok_or("actual foreign syntax")?;
    let id = DefId(u32::try_from(index)?);
    crate::sema::header_attributes::bind_cold_prototype(&report, &mut facts, id, key)?;
    let replay = test_source::resolve(source, true)?;
    assert!(
        crate::sema::header_attributes::bind_cold_prototype(&replay, &mut facts, id, key).is_err()
    );
    let block = report
        .resolution
        .foreign_owners
        .remove(&id)
        .ok_or("actual foreign owner")?;
    assert!(
        crate::sema::header_attributes::bind_cold_prototype(&report, &mut facts, id, key).is_err()
    );
    report.resolution.foreign_owners.insert(id, block);
    report.resolution.definitions[index].scope = ScopeId(u32::MAX);
    assert!(
        crate::sema::header_attributes::bind_cold_prototype(&report, &mut facts, id, key).is_err()
    );
    Ok(())
}
#[test]
fn no_body_cold_requires_its_actual_trait_member_scope() -> Result {
    let mut report = test_source::resolve("trait Owner{#[cold]fn required();}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let (index, row) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "required")
        .ok_or("actual required method")?;
    let id = crate::resolve::model::DefId(u32::try_from(index)?);
    let key = row.node.ok_or("required method source")?;
    let parent = facts
        .associated
        .trait_owner(id)
        .ok_or("actual trait parent")?;
    report.resolution.definitions[parent.0 as usize].members = None;
    assert!(
        crate::sema::header_attributes::bind_cold_prototype(&report, &mut facts, id, key).is_err()
    );
    Ok(())
}
