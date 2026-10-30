//! Touched declarations, transactional cold growth and canonical type identities.
use super::*;
use crate::{sema::header_attributes, syntax::ast::Kind};
use std::fmt::Write as _;
type Site<'a> = (NodeKey, NodeKey, Attribute, usize, &'a Forest);
fn sites(report: &Report) -> TestResult<Vec<Site<'_>>> {
    let mut result = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = crate::resolve::unit::FileId(u32::try_from(file)?);
        for owner in &unit.configured.owners {
            if !owner.enabled {
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            for (effect, attribute, forest) in
                unit.retained_indexed_attributes(owner.attributes.clone())
            {
                if matches!(
                    attribute.name(forest, unit.source.text()),
                    Some(
                        "rustc_no_mir_inline"
                            | "rustc_no_writable"
                            | "rustc_should_not_be_called_on_const_items"
                            | "rustc_confusables"
                            | "must_not_suspend"
                    )
                ) {
                    result.push((
                        NodeKey {
                            file,
                            node: owner.node,
                        },
                        NodeKey { file, node },
                        attribute,
                        effect,
                        forest,
                    ));
                }
            }
        }
    }
    Ok(result)
}
fn collected(report: &Report, facts: &mut Declarations) -> TestResult<Draft> {
    let mut draft = Draft::default();
    let mut allowances = report.macro_allowances();
    for (owner, declaration, attribute, effect, forest) in sites(report)? {
        draft.consume(
            report,
            facts,
            (owner, declaration),
            (attribute, effect),
            forest,
            &mut allowances,
        )?;
    }
    Ok(draft)
}
#[test]
fn repeated_confusables_and_nominal_constructor_self_rows_keep_exact_completion() -> TestResult {
    let (report, mut analysis) = checked("source-confusable-methods", "repeat-attributes")?;
    let mut draft = collected(&report, &mut analysis.facts)?;
    assert_eq!(draft.entries.len(), 1);
    assert_eq!(draft.annotations, 1);
    draft.prepare_binding(&report, &mut analysis.facts)?;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        draft.bind(&report, &analysis.facts, index, definition)?;
    }
    draft.finish()?;
    for (name, kind) in [("structure", DefKind::Struct), ("trait", DefKind::Trait)] {
        let (report, mut analysis) = checked("source-suspend-advisory", name)?;
        let mut draft = collected(&report, &mut analysis.facts)?;
        let key = *draft
            .entries
            .keys()
            .next()
            .ok_or("actual annotated source")?;
        draft.prepare_binding(&report, &mut analysis.facts)?;
        for (index, row) in report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.node == Some(key)
                    && matches!(row.kind, DefKind::Constructor | DefKind::SelfType)
            })
        {
            draft.bind(&report, &analysis.facts, index, row)?;
        }
        assert!(
            draft.finish().is_err(),
            "auxiliary identity completed the annotation"
        );
        let (index, row) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, row)| row.kind == kind && row.node == Some(key))
            .ok_or("canonical type identity")?;
        draft.bind(&report, &analysis.facts, index, row)?;
        draft.finish()?;
    }
    Ok(())
}
#[test]
fn foreign_images_wrong_actual_owners_and_namespace_cannot_complete_the_draft() -> TestResult {
    let (report, mut analysis) = checked("source-confusable-methods", "owner-replay")?;
    let mut draft = collected(&report, &mut analysis.facts)?;
    draft.prepare_binding(&report, &mut analysis.facts)?;
    let key = *draft.entries.keys().next().ok_or("marked method")?;
    let (index, definition) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| row.node == Some(key) && row.kind == DefKind::Function)
        .ok_or("actual function")?;
    let foreign = test_source::resolve(&input("source-confusable-methods", "owner-replay")?, true)?;
    assert!(
        draft
            .bind(
                &foreign,
                &analysis.facts,
                index,
                &foreign.resolution.definitions[index]
            )
            .is_err()
    );
    assert!(
        draft
            .bind(&report, &analysis.facts, index + 1, definition)
            .is_err()
    );
    let owner = analysis
        .facts
        .associated
        .owner(DefId(u32::try_from(index)?))
        .ok_or("actual impl")?;
    let old = *analysis
        .facts
        .associated
        .owner_for_test(owner)
        .ok_or("actual owner")?;
    let other = report
        .resolution
        .definitions
        .iter()
        .find(|row| {
            row.kind == DefKind::Function
                && row.node.is_some()
                && report.package.names.get(row.name) == "other"
        })
        .and_then(|row| row.node)
        .ok_or("actual other source")?;
    analysis
        .facts
        .associated
        .owner_for_test(owner)
        .ok_or("actual owner")?
        .source = Some(other);
    assert!(
        draft
            .bind(&report, &analysis.facts, index, definition)
            .is_err()
    );
    assert!(draft.finish().is_err());
    *analysis
        .facts
        .associated
        .owner_for_test(owner)
        .ok_or("actual owner")? = old;
    draft.bind(&report, &analysis.facts, index, definition)?;
    draft.finish()?;
    Ok(())
}
#[test]
fn canonical_nominal_namespace_changed_cannot_publish_type_completion() -> TestResult {
    let (mut report, mut analysis) = checked("source-suspend-advisory", "structure")?;
    let mut draft = collected(&report, &mut analysis.facts)?;
    draft.prepare_binding(&report, &mut analysis.facts)?;
    let key = *draft.entries.keys().next().ok_or("marked type")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| row.kind == DefKind::Struct && row.node == Some(key))
        .ok_or("actual struct")?;
    let old = report.resolution.definitions[index].namespace;
    report.resolution.definitions[index].namespace = Namespace::Value;
    assert!(
        draft
            .bind(
                &report,
                &analysis.facts,
                index,
                &report.resolution.definitions[index]
            )
            .is_err()
    );
    assert!(draft.finish().is_err());
    report.resolution.definitions[index].namespace = old;
    draft.bind(
        &report,
        &analysis.facts,
        index,
        &report.resolution.definitions[index],
    )?;
    draft.finish()?;
    let old = report.resolution.definitions[index].kind;
    report.resolution.definitions[index].kind = DefKind::SelfType;
    assert!(
        draft
            .bind(
                &report,
                &analysis.facts,
                index,
                &report.resolution.definitions[index]
            )
            .is_err()
    );
    assert!(
        draft.finish().is_err(),
        "a forged auxiliary kind kept completion"
    );
    report.resolution.definitions[index].kind = old;
    draft.bind(
        &report,
        &analysis.facts,
        index,
        &report.resolution.definitions[index],
    )?;
    draft.finish()?;
    Ok(())
}
#[test]
fn failed_mixed_annotation_growth_keeps_the_inline_record_and_sticky_payment() -> TestResult {
    let source = "#![feature(rustc_attrs)]struct S;impl S{#[rustc_no_mir_inline]#[rustc_confusables(\"swap\")]fn f()->u32{7}}";
    for (work, bytes) in [
        (crate::support::crates::MAX_COMPONENTS, 0),
        (0, crate::support::crates::MAX_BYTES),
    ] {
        let report = test_source::resolve(source, true)?;
        let mut analysis = pipeline::analyze_source(&report)?;
        let rows = sites(&report)?;
        let (owner, key, attribute, effect, forest) = rows[0];
        let mut draft = Draft::default();
        draft.consume(
            &report,
            &mut analysis.facts,
            (owner, key),
            (attribute, effect),
            forest,
            &mut report.macro_allowances(),
        )?;
        let (owner, key, attribute, effect, forest) = rows[1];
        let (restriction, _) = restrictions::authenticate(
            &report,
            &mut analysis.facts,
            (owner, key),
            (attribute, effect),
            forest,
            &mut report.macro_allowances(),
        )?;
        let _ = header_attributes::reserve_source_contracts(&mut analysis.facts, work, bytes);
        let before = header_attributes::source_contract_costs(&analysis.facts).0;
        let entry = draft
            .entries
            .get_mut(&key)
            .ok_or("real inline annotation")?;
        assert!(
            entry
                .append(&mut analysis.facts, key, restriction, attribute.span)
                .is_err()
        );
        assert_eq!(entry.annotations().count(), 1);
        assert!(!entry.bound);
        assert_eq!(draft.annotations, 1);
        let attempted = header_attributes::source_contract_costs(&analysis.facts).0;
        assert!(attempted > before);
        let entry = draft
            .entries
            .get_mut(&key)
            .ok_or("retained inline annotation")?;
        assert!(
            entry
                .append(&mut analysis.facts, key, restriction, attribute.span)
                .is_err()
        );
        assert_eq!(entry.annotations().count(), 1);
        assert!(header_attributes::source_contract_costs(&analysis.facts).0 > attempted);
    }
    Ok(())
}
#[test]
fn unrelated_definitions_do_not_multiply_the_present_annotation_frontier() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut costs = Vec::new();
        for mixed in [false, true] {
            let mut source = "#![feature(rustc_attrs,must_not_suspend)]".to_owned();
            if mixed {
                source.push_str("#[must_not_suspend]");
            }
            source.push_str("struct S;impl S{#[rustc_no_mir_inline]");
            if mixed {
                source.push_str("#[rustc_no_writable]#[rustc_should_not_be_called_on_const_items]#[rustc_confusables(\"swap\")]");
            }
            source.push_str("fn marked()->u32{7}}");
            for index in 0..width {
                writeln!(source, "fn plain{index}()->u32{{7}}")?;
            }
            let report = test_source::resolve(&source, true)?;
            let analysis = pipeline::analyze_source(&report)?;
            costs.push(header_attributes::source_contract_costs(&analysis.facts));
        }
        let delta = costs[1]
            .0
            .checked_sub(costs[0].0)
            .ok_or("mixed annotation work decreased")?;
        println!(
            "annotations width={width} single={:?} mixed={:?} delta={delta}",
            costs[0], costs[1]
        );
        if let Some(before) = previous {
            assert!(
                delta < before * 2,
                "annotation kinds multiplied unrelated definition work"
            );
        }
        previous = Some(delta);
    }
    println!(
        "Draft={} Entry={} Annotation={} NodeKey={} initial_buckets={}",
        size_of::<Draft>(),
        size_of::<Entry>(),
        size_of::<boundary_entries::Annotation>(),
        size_of::<NodeKey>(),
        Growth::map::<NodeKey, Entry>(0, 0, 1).bytes
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
