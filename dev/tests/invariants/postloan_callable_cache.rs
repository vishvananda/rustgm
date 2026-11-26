//! Loan-derived callable identities are rebuilt from fresh metadata, never transported proof.
use super::*;
use crate::{body::ir::Owner, lower::instances::SourcePublication, types::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source() -> Result<String> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/source-enum-role-transport/provider.rs")?;
    let prefix = foundation
        .split_once("#[rustc_diagnostic_item = \"Option\"]")
        .ok_or("actual foundation boundary")?
        .0;
    let tail = crate::test_source::fixture(
        "pa31/tests/spec/source-captured-callable-equalities/provider.rs",
    )?;
    Ok(format!(
        "#![feature(auto_traits,const_destruct)]\n{prefix}{tail}"
    ))
}

fn observe_solved_targets(text: &str) -> Result {
    let report = crate::test_source::source_provider(text, "captured_target_inspection")?;
    let nested = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "nested")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual nested source function")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, checked| -> Result {
            let body = checked.program().bodies.iter().find(|body|
            body.instance.is_none() && matches!(body.owner, Owner::Function { definition, .. } if definition == nested))
            .ok_or("actual checked source original")?;
            let targets = body.callable_targets();
            assert!(
                !targets.is_empty(),
                "the fixture must need solved captured-call identities"
            );
            assert!(body.has_only_transportable_inputs());
            assert!(!body.has_only_caller_sites());
            for (original, selected) in targets {
                assert_ne!(original, selected);
                let item = |ty| match facts.types.kind(ty) {
                    Some(Kind::FunctionItem { item, .. }) => Some(item),
                    _ => None,
                };
                assert_eq!(item(original.ty()), item(selected.ty()));
                assert!(body.values.iter().any(|value| value.ty == original.ty()));
            }
            Ok(())
        },
    )??;
    Ok(())
}

#[test]
fn solved_original_targets_stay_out_of_fresh_raw_bodies() -> Result {
    let text = source()?;
    observe_solved_targets(&text)?;
    let artifact = crate::module_packets::exported_profile(
        &text,
        SourcePublication::ModuleBodyOperations,
        "captured_targets",
    )?;
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    let (report, analysis, _) = crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe(x:&u32)->u32{let mut slot=x;base::nested(&mut slot)}",
    )?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| row.node.is_none() && report.package.names.get(row.name) == "nested")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual imported nested function")?;
    let mut steps = 0;
    let view = crate::sema::crates::source_original(&analysis.facts, definition, &mut steps)?
        .ok_or("actual fresh source original")?;
    let body = rebuilt_operations::build(&view, &mut ImportedOriginals::default())?;
    rebuilt_operations::validate(&analysis.facts, &view, &body, &mut steps)?;
    assert!(body.callable_targets().is_empty());
    assert!(body.has_only_caller_sites());
    Ok(())
}
