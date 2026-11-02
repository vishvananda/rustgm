//! Source dispatch policy stays owned, counted and separate from receiver execution.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa34/tests/spec/method-dispatch-policy/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, Declarations)> {
    let report = test_source::resolve(source, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn trait_id(report: &Report, name: &str) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, row)| {
            (row.kind == DefKind::Trait && report.package.names.get(row.name) == name).then_some(i)
        })
        .map(|index| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or_else(|| "missing actual trait".into())
}
#[test]
fn actual_declaration_forms_gates_targets_and_macros_match_independent_rust() -> TestResult {
    for name in [
        "array",
        "boxed-slice",
        "both",
        "reversed",
        "trailing",
        "raw-key",
        "const-trait",
        "cfg-removed",
        "macro-allowance",
    ] {
        analyze(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "word",
        "value",
        "empty",
        "duplicate-key",
        "duplicate-boxed",
        "unknown",
        "literal",
        "key-value",
        "nested",
        "qualified",
        "number",
        "delimiter",
        "duplicate-attribute",
        "unsafe-wrapper",
        "missing-gate",
        "struct",
        "enum",
        "union",
        "function",
        "alias",
        "impl",
        "associated",
        "field",
        "inner-crate",
        "macro-no-allowance",
    ] {
        let error = analyze(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(
            !error.contains("not in path scope"),
            "{name}: never validated its actual attribute: {error}"
        );
    }
    Ok(())
}
#[test]
fn actual_source_auto_trait_keeps_its_role_and_dispatch_declaration() -> TestResult {
    let input = fixture("source-auto-trait")?;
    let report = test_source::source_provider(&input, "dispatch_auto")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = trait_id(&report, "Send")?;
    let ContractKnowledge::KnownPresent(view) = facts.source_method_dispatch_policy(&report, id)?
    else {
        return Err("missing actual auto trait policy".into());
    };
    assert_eq!(
        view.policy,
        DispatchPolicy {
            array: true,
            boxed_slice: true
        }
    );
    Ok(())
}
#[test]
fn known_absence_still_requires_its_actual_trait_item() -> TestResult {
    let input = format!("{}trait Other{{}}", fixture("source-auto-trait")?);
    let mut report = test_source::source_provider(&input, "dispatch_absence")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let id = trait_id(&report, "Other")?;
    assert!(matches!(
        facts.source_method_dispatch_policy(&report, id)?,
        ContractKnowledge::KnownAbsent
    ));
    let key = report.resolution.definitions[id.0 as usize]
        .node
        .ok_or("trait source")?;
    let Kind::Item(index) = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .kind
    else {
        return Err("trait item".into());
    };
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .items[index as usize]
        .kind = ItemKind::Enum {
        variants: crate::syntax::ast::List { start: 0, end: 0 },
    };
    assert!(
        facts.source_method_dispatch_policy(&report, id).is_err(),
        "absence survived replacing the trait item"
    );
    Ok(())
}
#[test]
fn completed_trait_query_retains_both_policy_bits_without_creating_trait_evidence() -> TestResult {
    for (name, array, boxed_slice) in [
        ("array", true, false),
        ("boxed-slice", false, true),
        ("both", true, true),
        ("raw-key", true, false),
        ("macro-allowance", true, true),
    ] {
        let (report, facts) = analyze(&fixture(name)?)?;
        let id = trait_id(&report, "Pick")?;
        let ContractKnowledge::KnownPresent(view) =
            facts.source_method_dispatch_policy(&report, id)?
        else {
            return Err(format!("missing policy: {name}").into());
        };
        assert_eq!(view.policy, DispatchPolicy { array, boxed_slice });
        assert_eq!(
            Some(view.source.declaration),
            report.resolution.definitions[id.0 as usize].node
        );
    }
    let source = "#![feature(rustc_attrs)]#[rustc_skip_during_method_dispatch(array)]trait Pick{fn pick(self)->u64;}fn main(){7u64.pick();}";
    assert!(
        analyze(source).is_err(),
        "dispatch flags cannot supply an implementation"
    );
    Ok(())
}
#[test]
fn source_images_trait_headers_and_unknown_imports_keep_distinct_authority() -> TestResult {
    let input = fixture("both")?;
    let (mut report, facts) = analyze(&input)?;
    let id = trait_id(&report, "Pick")?;
    let key = report.resolution.definitions[id.0 as usize]
        .node
        .ok_or("source declaration")?;
    let Kind::Item(index) = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .kind
    else {
        return Err("source trait".into());
    };
    let fresh = test_source::resolve(&input, true)?;
    assert!(facts.source_method_dispatch_policy(&fresh, id).is_err());
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .items[index as usize]
        .visibility = crate::syntax::item::Visibility::Public;
    assert!(
        facts.source_method_dispatch_policy(&report, id).is_err(),
        "changed actual trait was accepted"
    );
    let (mut report, facts) = analyze(&input)?;
    let id = trait_id(&report, "Pick")?;
    report.resolution.definitions[id.0 as usize].node = None;
    assert!(matches!(
        facts.source_method_dispatch_policy(&report, id)?,
        ContractKnowledge::Unknown
    ));
    Ok(())
}
#[test]
fn older_metadata_profiles_do_not_erase_the_checked_trait_policy() -> TestResult {
    let (report, mut facts) = analyze(&fixture("both")?)?;
    let error = crate::sema::crates::export(
        &report,
        &mut facts,
        crate::support::crates::CrateIdentity::new("dispatch", "owned")?,
    )
    .err()
    .ok_or("trait policy erased by metadata")?
    .to_string();
    assert!(
        error.contains("method dispatch policy requires authenticated trait metadata transport"),
        "{error}"
    );
    assert!(facts.require_dispatch_policy_transport(&report).is_err());
    Ok(())
}
#[test]
fn query_and_retention_payments_are_sticky_and_shared_headers_stay_compact() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut source = String::from("#![feature(rustc_attrs)]");
        for index in 0..width {
            write!(
                source,
                "#[rustc_skip_during_method_dispatch(array,boxed_slice)]trait T{index}{{}}"
            )?;
        }
        let (report, facts) = analyze(&source)?;
        let before = super::super::source_contract_costs(&facts);
        for index in 0..width {
            let id = trait_id(&report, &format!("T{index}"))?;
            assert!(matches!(
                facts.source_method_dispatch_policy(&report, id)?,
                ContractKnowledge::KnownPresent(_)
            ));
        }
        let after = super::super::source_contract_costs(&facts);
        assert_eq!(after.1, before.1);
        observations.push(before);
        eprintln!(
            "dispatch width={width} work={} bytes={}",
            before.0, before.1
        );
    }
    assert_eq!(
        observations[2].0 - observations[1].0,
        4 * (observations[1].0 - observations[0].0)
    );
    assert_eq!(
        observations[2].1 - observations[1].1,
        4 * (observations[1].1 - observations[0].1)
    );
    let (report, facts) = analyze(&fixture("both")?)?;
    let id = trait_id(&report, "Pick")?;
    let headers = &facts
        .must_use_contracts
        .as_ref()
        .ok_or("contracts")?
        .headers
        .as_ref()
        .ok_or("ledger")?
        .local;
    headers.steps.set(crate::support::crates::MAX_COMPONENTS);
    assert!(facts.source_method_dispatch_policy(&report, id).is_err());
    assert!(facts.source_method_dispatch_policy(&report, id).is_err());
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
