//! Real impl/member identities supply constness; policy and accounting grant none.
use super::*;
use crate::{pipeline, resolve::stability::Library, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/const-inherent-policy/{name}.rs"
    ))?)
}

#[test]
fn declared_inherent_const_sources_match_independent_semantic_and_execution_outcomes() -> Result {
    let cases = test_source::fixture("pa31/tests/spec/const-inherent-policy/cases-v4.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("source case fields")?;
        let text = test_source::fixture(path)?;
        let result = (|| -> Result {
            let report = test_source::resolve(&text, true)?;
            let mut analysis = pipeline::analyze_source(&report)?;
            crate::lower::instances::inspect_prepared(
                &report,
                &mut analysis.facts,
                analysis.prepared,
                |_, _| (),
            )?;
            Ok(())
        })();
        if result.is_ok() != (status == "0") {
            failures.push(format!("{path}: {:?}", result.err()));
        }
        count += 1;
    }
    assert_eq!(count, 51);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn actual_const_inventory_distinguishes_group_policy_and_function_eligibility() -> Result {
    for (name, constant, policy) in [
        ("unstable-block", true, Some("inherent_const_api")),
        (
            "ordinary-group-const-member",
            true,
            Some("inherent_const_api"),
        ),
        (
            "ordinary-impl-annotation",
            false,
            Some("inherent_const_api"),
        ),
        ("private-unmarked-inherent", true, None),
    ] {
        let report = test_source::resolve(&source(name)?, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let function = analysis
            .facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == "read"
            })
            .ok_or("actual read function")?;
        assert_eq!(
            analysis.facts.is_const_function(function.definition),
            constant,
            "{name}"
        );
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("inventory")?;
        assert_eq!(
            inventory
                .const_declaration(function.definition)
                .map(|p| p.feature.as_ref()),
            policy,
            "{name}"
        );
        if name != "private-unmarked-inherent" {
            assert_eq!(
                inventory
                    .declaration(function.definition)
                    .ok_or("regular API")?
                    .feature
                    .as_ref(),
                "root_api"
            );
        }
    }
    assert_eq!(size_of::<Frame>(), 24);
    assert_eq!(size_of::<Role>(), 2);
    assert_eq!(size_of::<Inventory>(), 120);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn ordinary_const_call_facts_keep_exact_owner_without_a_compiler_catalog() -> Result {
    let text = test_source::fixture("pa25/tests/spec/const-inherent/conditional-const-bound.rs")?;
    let report = test_source::resolve(&text, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(analysis.facts.source_bindings.is_none());
    let function = analysis
        .facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "read"
        })
        .ok_or("actual generic read")?;
    let body = function.body.ok_or("actual body")?;
    let call = analysis
        .facts
        .nodes
        .iter()
        .flat_map(|file| file.iter())
        .filter_map(|node| match node.operation {
            Some(crate::sema::model::Operation::Call {
                source_use: Some(use_),
                ..
            }) => Some(use_),
            _ => None,
        })
        .find(|use_| use_.owner(&analysis.facts) == Ok(body))
        .ok_or("owned const call")?;
    assert_eq!(
        call.effect_for(&analysis.facts, body)?,
        crate::support::constness::Constness::Conditional
    );
    assert!(call.effect_for(&analysis.facts, function.node).is_err());
    let report = test_source::resolve("fn target()->u8{7}fn main(){let _=target();}", true)?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(analysis.facts.source_bindings.is_none());
    assert!(
        analysis
            .facts
            .nodes
            .iter()
            .flat_map(|file| file.iter())
            .all(|node| {
                !matches!(
                    node.operation,
                    Some(crate::sema::model::Operation::Call {
                        source_use: Some(_),
                        ..
                    })
                )
            })
    );
    Ok(())
}
