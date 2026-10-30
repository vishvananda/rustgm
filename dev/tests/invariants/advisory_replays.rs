//! Paid transient source ownership is checked before publication and then discarded.
use super::*;
use crate::sema::header_attributes;
use std::fmt::Write as _;
fn consume_first(
    draft: &mut Draft,
    report: &Report,
    facts: &mut Declarations,
    wrong: bool,
) -> TestResult {
    let (_, method) = function(report, "f")?;
    let declaration = method.node.ok_or("actual method source")?;
    let unit = report.package.unit(declaration.file);
    for owner in &unit.configured.owners {
        for (effect, attribute, forest) in
            unit.retained_indexed_attributes(owner.attributes.clone())
        {
            if attribute.name(forest, unit.source.text())
                != Some("rustc_should_not_be_called_on_const_items")
            {
                continue;
            }
            let owner_node = if wrong {
                let (_, other) = function(report, "g")?;
                let other = other.node.ok_or("actual other method source")?;
                unit.configured
                    .owners
                    .iter()
                    .find(|row| {
                        let mut node = row.node;
                        while let crate::syntax::ast::Kind::Attributed { value, .. } =
                            unit.configured.ast.node(node).kind
                        {
                            node = value;
                        }
                        row.enabled && node == other.node
                    })
                    .ok_or("other actual owner")?
                    .node
            } else {
                owner.node
            };
            draft.consume(
                report,
                facts,
                (
                    NodeKey {
                        node: owner_node,
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
    Err("actual const-item method advisory".into())
}
#[test]
fn current_owner_scope_incarnation_and_failed_work_cannot_publish_an_advisory() -> TestResult {
    let (report, mut analysis) = checked("owner-replay")?;
    let mut draft = Draft::default();
    consume_first(&mut draft, &report, &mut analysis.facts, false)?;
    let other = test_source::resolve(&input("owner-replay")?, true)?;
    let (index, definition) = function(&other, "f")?;
    assert!(
        draft
            .bind(&other, &analysis.facts, index, definition)
            .is_err()
    );
    assert!(draft.finish().is_err());
    let (index, definition) = function(&report, "f")?;
    draft.prepare_binding(&report, &mut analysis.facts)?;
    let id = DefId(u32::try_from(index)?);
    let owner = analysis
        .facts
        .associated
        .owner(id)
        .ok_or("actual impl owner")?;
    let old = *analysis
        .facts
        .associated
        .owner_for_test(owner)
        .ok_or("actual owner row")?;
    analysis
        .facts
        .associated
        .owner_for_test(owner)
        .ok_or("actual owner row")?
        .source = function(&report, "g")?.1.node;
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
        .ok_or("actual owner row")? = old;
    draft.bind(&report, &analysis.facts, index, definition)?;
    draft.finish()?;
    let mut wrong = Draft::default();
    assert!(consume_first(&mut wrong, &report, &mut analysis.facts, true).is_err());
    assert!(wrong.is_empty());
    for (work, bytes) in [
        (crate::support::crates::MAX_COMPONENTS, 0),
        (0, crate::support::crates::MAX_BYTES),
    ] {
        let (report, mut analysis) = checked("owner-replay")?;
        let _ = header_attributes::reserve_source_contracts(&mut analysis.facts, work, bytes);
        let mut failed = Draft::default();
        let before = header_attributes::source_contract_costs(&analysis.facts).0;
        assert!(consume_first(&mut failed, &report, &mut analysis.facts, false).is_err());
        assert!(failed.is_empty());
        let attempted = header_attributes::source_contract_costs(&analysis.facts).0;
        assert!(attempted > before);
        assert!(consume_first(&mut failed, &report, &mut analysis.facts, false).is_err());
        assert!(header_attributes::source_contract_costs(&analysis.facts).0 > attempted);
    }
    Ok(())
}
#[test]
fn many_actual_method_advisories_keep_one_paid_pass_and_common_layouts() -> TestResult {
    let mut previous = (0, 0);
    for width in [8, 32, 128] {
        let mut source = "#![feature(rustc_attrs)]struct S;impl S{".to_owned();
        for index in 0..width {
            writeln!(
                source,
                "#[rustc_should_not_be_called_on_const_items]pub fn f{index}()->u32{{7}}"
            )?;
        }
        source.push('}');
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let cost = header_attributes::source_contract_costs(&analysis.facts);
        println!(
            "const-item advisories width={width} work={} bytes={}",
            cost.0, cost.1
        );
        if previous.0 != 0 {
            assert!(cost.0 < previous.0 * 6);
            assert!(cost.1 < previous.1 * 6);
        }
        previous = cost;
    }
    println!(
        "transient Draft={} Entry={} key={}",
        size_of::<Draft>(),
        size_of::<Entry>(),
        size_of::<(NodeKey, Restriction)>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
