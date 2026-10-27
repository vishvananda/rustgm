//! Genuine source helper claims are metadata, never a global attribute scope.
use crate::{
    expand::builtin::Builtin,
    resolve::{self, bindings::panic_contracts, model::DefKind, unit::NodeKey},
    support::diagnostics::WarningCode,
};
use std::rc::Rc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/builtin-derive-helpers/{name}.rs"
    ))?)
}
fn provider(name: &str) -> Result<resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "helper_contracts",
    )?)
}
fn key(report: &resolve::expansion::Report, name: &str) -> Result<NodeKey> {
    report
        .resolution
        .definitions
        .iter()
        .find(|d| d.kind == DefKind::Macro && report.package.names.get(d.name) == name)
        .and_then(|d| d.node)
        .ok_or_else(|| "actual source macro".into())
}
fn header<'a>(
    report: &'a resolve::expansion::Report,
    name: &str,
) -> Result<panic_contracts::CheckedHeader<'a>> {
    panic_contracts::checked_definition(&report.package, key(report, name)?)?
        .ok_or_else(|| "positive macro header".into())
}
#[test]
fn actual_helper_declarations_keep_aliases_raw_names_nfc_order_and_checked_empty() -> Result {
    for (name, declared, names) in [
        ("default-helper", "Default", vec!["default"]),
        ("empty-helpers", "Default", vec![]),
        ("implicit-empty", "Default", vec![]),
        ("alias-role", "ChosenDefault", vec!["default"]),
        ("raw-identifiers", "Default", vec!["default", "type"]),
        (
            "duplicate-helpers",
            "Default",
            vec!["default", "default", "other"],
        ),
        ("unicode-helpers", "Default", vec!["é", "é"]),
    ] {
        let report = provider(name)?;
        let checked = header(&report, declared)?;
        let helpers = checked.derive_helpers(Builtin::Default)?;
        assert_eq!(
            helpers
                .iter()
                .map(panic_contracts::HelperAttribute::name)
                .collect::<Vec<_>>(),
            names,
            "{name}"
        );
        let unit = report.package.unit(checked.summary().node.file);
        for helper in helpers {
            assert!(
                unit.source
                    .text()
                    .get(helper.span().start as usize..helper.span().end as usize)
                    .is_some()
            );
        }
        if name == "unicode-helpers" {
            assert_eq!(helpers[0].name().as_ptr(), helpers[1].name().as_ptr());
            assert_ne!(helpers[0].span(), helpers[1].span());
        }
        assert!(report.invocations.is_empty());
        assert!(checked.derive_helpers(Builtin::Clone).is_err());
    }
    let report = provider("bang-helper-without-decl-gate")?;
    assert!(
        header(&report, "line")?
            .derive_helpers(Builtin::Default)
            .is_err()
    );
    let defaults = provider("default-helper")?;
    assert!(Builtin::Default.executable());
    assert!(
        crate::test_source::source_provider(
            &(source("default-helper")? + "#[derive(Default)]pub struct S;"),
            "requires_actual_default_trait"
        )
        .is_err()
    );
    assert!(defaults.invocations.is_empty());
    Ok(())
}
#[test]
fn malformed_helpers_global_use_wrong_roles_and_reincarnated_headers_reject() -> Result {
    for name in [
        "literal-helper",
        "path-helper",
        "argument-helper",
        "path-keyword-helper",
        "keyword-helper",
        "missing-comma",
        "extra-input",
        "wrong-owner",
        "missing-rustc-gate",
        "helper-not-global",
    ] {
        assert!(
            provider(name).is_err(),
            "{name} acquired source helper authority"
        );
    }
    let text = source("alias-role")?
        + "#[rustc_builtin_macro(Default,attributes(other))]pub macro Other(){}";
    let mut report = crate::test_source::source_provider(&text, "separate_claims")?;
    let first = header(&report, "ChosenDefault")?;
    let first_key = first.summary().node;
    let data = first.into_data().ok_or("owned header")?;
    let other = header(&report, "Other")?
        .into_data()
        .ok_or("second header")?;
    assert!(!Rc::ptr_eq(&data, &other));
    assert_eq!(other.derive_helpers(Builtin::Default)?[0].name(), "other");
    assert!(panic_contracts::published(&report.package, first_key, Some(&other), 1).is_err());
    let before = report
        .package
        .macro_context()?
        .ok_or("context")?
        .contracts()
        .usage();
    for _ in 0..32 {
        let repeated = header(&report, "ChosenDefault")?
            .into_data()
            .ok_or("same header")?;
        assert!(Rc::ptr_eq(&data, &repeated));
    }
    let after = report
        .package
        .macro_context()?
        .ok_or("context")?
        .contracts()
        .usage();
    assert!(after.0 > before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    report.package.units[first_key.file.0 as usize].source =
        report.package.unit(first_key.file).source.physical_copy();
    assert!(panic_contracts::published(&report.package, first_key, Some(&data), 1).is_err());
    assert!(panic_contracts::checked_definition(&report.package, first_key).is_err());
    let mut captured = provider("default-helper")?;
    let data = header(&captured, "Default")?
        .into_data()
        .ok_or("captured header")?;
    let summary = data.declaration();
    let effect = data.witnesses[0].index;
    let unit = &mut captured.package.units[summary.node.file.0 as usize];
    let old = unit.configured.effects[effect].capture;
    unit.configured.effects[effect].capture = u32::from(old == 0);
    assert!(panic_contracts::checked_definition(&captured.package, summary.node).is_err());
    let ordinary = crate::test_source::resolve("macro_rules! user{()=>{}}", true)?;
    assert!(panic_contracts::checked(&ordinary.package, key(&ordinary, "user")?)?.is_none());
    let ordinary = crate::test_source::resolve(
        "#[must_use]fn marked()->u8{1}macro_rules! user{()=>{}}",
        true,
    )?;
    assert_eq!(
        crate::pipeline::analyze_source(&ordinary)?
            .facts
            .warnings()
            .count(),
        0
    );
    Ok(())
}
#[test]
fn helper_collision_warnings_use_actual_owner_policy_macro_origin_and_error_carrier() -> Result {
    let bang =
        source("bang-helper-without-decl-gate")?.replace("attributes(helper)", "attributes(doc)");
    let bang = crate::test_source::source_provider(&bang, "bang_helper_collision")?;
    let bang = crate::pipeline::analyze_source(&bang)?;
    assert_eq!(bang.facts.warnings().count(), 1);
    assert_eq!(
        bang.facts
            .warnings()
            .next()
            .ok_or("Bang helper warning")?
            .code,
        WarningCode::AmbiguousDeriveHelpers
    );
    for name in [
        "builtin-collision",
        "collision-warn",
        "collision-macro-warn",
    ] {
        let report = provider(name)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let warnings: Vec<_> = analysis.facts.warnings().collect();
        assert_eq!(warnings.len(), 1, "{name}");
        assert_eq!(warnings[0].code, WarningCode::AmbiguousDeriveHelpers);
        assert_eq!(
            warnings[0].diagnostic.message,
            "there exists a built-in attribute with the same name"
        );
        if name == "collision-macro-warn" {
            assert!(
                !warnings[0].diagnostic.expansions.is_empty(),
                "actual generated macro declaration lost its origin"
            );
        }
    }
    for name in [
        "collision-allow",
        "collision-nested-allow",
        "collision-macro-allow",
    ] {
        let report = provider(name)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        assert_eq!(facts.warnings().count(), 0, "{name}");
    }
    for name in [
        "collision-deny",
        "collision-forbid",
        "collision-future-group",
        "collision-forbid-weaken",
    ] {
        let report = provider(name)?;
        let error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name} lost its actual lint denial"))?;
        assert!(
            error.message.contains("built-in attribute")
                || error.message.contains("forbidden ambiguous_derive_helpers"),
            "{name}: {error}"
        );
    }
    let report = provider("collision-with-type-error")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("invalid body became valid")?;
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(
        error.warnings()[0].code,
        WarningCode::AmbiguousDeriveHelpers
    );
    assert!(
        error.message.contains("type"),
        "primary type failure changed: {error}"
    );
    Ok(())
}
#[test]
fn helper_payloads_warm_failed_work_and_existing_cold_and_dense_costs_are_measured() -> Result {
    use std::fmt::Write;
    let original = source("default-helper")?;
    for width in [8, 32, 128] {
        let mut words = String::new();
        for index in 0..width {
            if index > 0 {
                words.push(',');
            }
            write!(words, "helper_{index}")?;
        }
        let text = original.replace("attributes(default)", &format!("attributes({words})"));
        let report = crate::test_source::source_provider(&text, "helper_width")?;
        let checked = header(&report, "Default")?;
        assert_eq!(checked.derive_helpers(Builtin::Default)?.len(), width);
        let data = checked.into_data().ok_or("actual helper payload")?;
        let usage = report
            .package
            .macro_context()?
            .ok_or("provider")?
            .contracts()
            .usage();
        println!(
            "helper width={width} len={} capacity={} helper-bytes={} header-payload={} source-usage={usage:?}",
            data.helpers.len(),
            data.helpers.capacity(),
            data.helpers.capacity() * size_of::<panic_contracts::HelperAttribute>(),
            data.payload()
        );
    }
    let report = provider("empty-helpers")?;
    let data = header(&report, "Default")?
        .into_data()
        .ok_or("empty owned header")?;
    assert_eq!(data.helpers.capacity(), 0);
    assert_eq!(data.feature_count(), 0);
    let feature_like =
        source("default-helper")?.replace("attributes(default)", "attributes(core_intrinsics)");
    let feature_like = crate::test_source::source_provider(&feature_like, "no_feature_grant")?;
    let data = header(&feature_like, "Default")?
        .into_data()
        .ok_or("helper named like feature")?;
    assert_eq!(data.feature_count(), 0);
    let contracts = report
        .package
        .macro_context()?
        .ok_or("provider")?
        .contracts();
    assert!(contracts.charge(usize::MAX, 0).is_err());
    assert!(header(&report, "Default").is_err());
    assert!(header(&report, "Default").is_err());
    let ordinary = crate::test_source::resolve("fn plain(){}", true)?;
    let ordinary = crate::pipeline::analyze_source(&ordinary)?;
    assert_eq!(ordinary.facts.retained_policy_storage(), (0, 0, 0));
    let affected = crate::test_source::resolve("#![allow(unused_attributes)]fn plain(){}", true)?;
    let affected = crate::pipeline::analyze_source(&affected)?;
    let (nodes, capacity, outer) = affected.facts.retained_policy_storage();
    assert!(nodes > 0);
    assert_eq!(size_of::<resolve::lint_policy::Policy>(), 12);
    println!(
        "helper policy nodes={nodes} capacity={capacity} payload={} previous11bytepayload={} latest-increase={} outer={outer}",
        capacity * size_of::<resolve::lint_policy::Policy>(),
        capacity * 11,
        capacity
    );
    println!(
        "helper layouts Declaration={} HeaderData={} Helper={} SourceCore={} Session={} Policy={} ResultPolicy={} CompositionRow={} Effects={} Producer={} Item={} Node={} Scope={} Definition={} MacroBody={} MacroDefinition={} Report={}",
        size_of::<panic_contracts::Declaration>(),
        size_of::<panic_contracts::HeaderData>(),
        size_of::<panic_contracts::HelperAttribute>(),
        size_of::<resolve::bindings::SourceCore>(),
        size_of::<resolve::expansion::MacroAllowanceSession<'_>>(),
        size_of::<resolve::lint_policy::Policy>(),
        size_of::<std::result::Result<resolve::lint_policy::Policy, resolve::lint_policy::Violation>>(
        ),
        size_of::<(
            resolve::lint_policy::producer::Tail,
            std::result::Result<resolve::lint_policy::Policy, resolve::lint_policy::Violation>
        )>(),
        size_of::<resolve::expansion::attribute_effects::Effects>(),
        size_of::<resolve::lint_policy::producer::Producer>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<resolve::model::Scope>(),
        size_of::<resolve::model::Definition>(),
        size_of::<resolve::expansion::Body>(),
        size_of::<resolve::expansion::Definition>(),
        size_of::<resolve::expansion::Report>()
    );
    Ok(())
}
