//! MIR-only prohibition validates source contracts without changing other hints.
use super::*;
use crate::{pipeline, sema::header_attributes, test_source};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-no-mir-inline/{name}.rs"
    ))?)
}
fn checked(name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(&input(name)?, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    Ok((report, analysis))
}
fn definition<'a>(report: &'a Report, name: &str) -> TestResult<(usize, &'a Definition)> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == name
        })
        .ok_or_else(|| "actual function".into())
}
#[test]
fn actual_functions_and_all_method_kinds_keep_independent_inline_hints() -> TestResult {
    for name in [
        "word",
        "with-inline",
        "with-always",
        "with-never",
        "const",
        "unsafe",
        "inherent",
        "required",
        "default",
        "implementation",
        "extern-body",
        "deny-unused",
    ] {
        let (report, analysis) = checked(name)?;
        if name.starts_with("with-") {
            let (index, _) = definition(&report, "f")?;
            let hint = analysis
                .facts
                .inline_hint_for_definition(&report, DefId(u32::try_from(index)?))?
                .ok_or("real inline hint")?;
            let expected = match name {
                "with-always" => crate::sema::inlining::Hint::Always,
                "with-never" => crate::sema::inlining::Hint::Never,
                _ => crate::sema::inlining::Hint::Hint,
            };
            assert_eq!(hint.hint, expected);
        }
        if name == "const" {
            let initializer = analysis
                .facts
                .initializers
                .iter()
                .find(|row| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[row.definition.0 as usize].name)
                        == "V"
                })
                .ok_or("actual const initializer")?;
            let reader = crate::sema::values::Reader::new(&analysis.facts);
            assert!(matches!(
                reader.view(
                    reader
                        .definition(initializer.definition)
                        .ok_or("completed constant")?
                )?,
                crate::sema::values::View::Scalar(crate::sema::model::Scalar::Integer(7))
            ));
        }
        if matches!(
            name,
            "word" | "with-inline" | "with-always" | "with-never" | "extern-body"
        ) {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = analysis;
            crate::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
        }
    }
    Ok(())
}
#[test]
fn malformed_missing_gate_foreign_closure_and_duplicate_markers_are_hard_errors() -> TestResult {
    for name in [
        "foreign",
        "closure",
        "struct",
        "constant",
        "missing-gate",
        "duplicate",
        "duplicate-allowed",
        "empty-list",
        "list",
        "value",
        "unsafe-wrapper",
        "unsafe-denied",
        "nonconst-denied",
    ] {
        let outcome = test_source::resolve(&input(name)?, true)
            .map_err(Into::into)
            .and_then(|report| {
                pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(Into::into)
            });
        let error: Box<dyn std::error::Error> = outcome
            .err()
            .ok_or_else(|| format!("{name}: invalid contract accepted"))?;
        let expected = match name {
            "unsafe-denied" => "unsafe",
            "nonconst-denied" => "const",
            _ => "rustc_no_mir_inline",
        };
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn cfg_removal_macro_allowance_and_distinct_source_owners_remain_real() -> TestResult {
    for name in [
        "cfg-removed-duplicate",
        "cfg-removed-bad-target",
        "macro-allowance",
        "same-owner-distinct",
    ] {
        checked(name)?;
    }
    Ok(())
}
fn consume_first(draft: &mut Draft, report: &Report, facts: &mut Declarations) -> TestResult {
    let (_, row) = definition(report, "f")?;
    let declaration = row.node.ok_or("source function")?;
    let unit = report.package.unit(declaration.file);
    for owner in &unit.configured.owners {
        for (effect, attribute, forest) in
            unit.retained_indexed_attributes(owner.attributes.clone())
        {
            if attribute.name(forest, unit.source.text()) == Some("rustc_no_mir_inline") {
                draft.consume(
                    report,
                    facts,
                    (
                        NodeKey {
                            node: owner.node,
                            ..declaration
                        },
                        declaration,
                    ),
                    (attribute, effect),
                    forest,
                    &mut report.macro_allowances(),
                )?;
                return Ok(());
            }
        }
    }
    Err("actual marker".into())
}
#[test]
fn foreign_incarnations_and_failed_payments_publish_no_checked_owner() -> TestResult {
    let (report, mut analysis) = checked("word")?;
    let mut draft = Draft::default();
    consume_first(&mut draft, &report, &mut analysis.facts)?;
    let other = test_source::resolve(&input("word")?, true)?;
    let (index, row) = definition(&other, "f")?;
    assert!(draft.bind(&other, index, row).is_err());
    assert!(draft.finish().is_err());
    let (index, row) = definition(&report, "f")?;
    draft.prepare_binding(&report, &mut analysis.facts)?;
    draft.bind(&report, index, row)?;
    draft.finish()?;
    for (work, bytes) in [
        (crate::support::crates::MAX_COMPONENTS, 0),
        (0, crate::support::crates::MAX_BYTES),
    ] {
        let (current, mut analysis) = checked("word")?;
        let _ = header_attributes::reserve_source_contracts(&mut analysis.facts, work, bytes);
        let mut failed = Draft::default();
        assert!(consume_first(&mut failed, &current, &mut analysis.facts).is_err());
        assert!(failed.is_empty());
    }
    Ok(())
}
#[test]
fn transient_owner_work_is_linear_and_existing_hot_widths_do_not_grow() -> TestResult {
    let mut previous = (0, 0);
    for width in [8, 32, 128] {
        let mut source = "#![feature(rustc_attrs)]\n".to_owned();
        for index in 0..width {
            writeln!(source, "#[rustc_no_mir_inline]fn f{index}()->u8{{7}}")?;
        }
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let cost = header_attributes::source_contract_costs(&analysis.facts);
        println!(
            "no MIR inline width={width} work={} bytes={}",
            cost.0, cost.1
        );
        if previous.0 != 0 {
            assert!(cost.0 < previous.0 * 6);
            assert!(cost.1 < previous.1 * 6);
        }
        previous = cost;
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn current_imports_preserve_native_inline_hints_without_a_mir_inlining_consumer() -> TestResult {
    let source = input("module-provider")?;
    let interface = crate::module_packets::exported_profile(
        &source,
        crate::lower::instances::SourcePublication::ModuleCallables,
        "mir_boundary",
    )?;
    let bytes = crate::support::crates::encode(&interface)?;
    let decoded = crate::support::crates::decode(&bytes)?;
    assert_eq!(crate::support::crates::encode(&decoded)?, bytes);
    let (report, analysis, _) = crate::source_packets::client_analysis(
        decoded,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observed()->u8{base::marked_hint(7)}",
    )?;
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported declarations")?;
    let id = imported
        .definitions()
        .find_map(|(external, id)| {
            external
                .path()
                .last()
                .is_some_and(|name| name == "marked_hint")
                .then_some(id)
        })
        .ok_or("imported marked function")?;
    assert_eq!(
        analysis
            .facts
            .inline_hint_for_definition(&report, id)?
            .ok_or("transported independent inline hint")?
            .hint,
        crate::sema::inlining::Hint::Always
    );
    Ok(())
}
