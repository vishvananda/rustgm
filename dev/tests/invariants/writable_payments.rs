//! Actual owner authentication and paid transient restriction storage.
use super::*;
use crate::sema::header_attributes;
use std::fmt::Write as _;
fn consume_first(
    draft: &mut Draft,
    report: &Report,
    facts: &mut Declarations,
    wrong_owner: bool,
) -> TestResult {
    let (_, row) = function(report, "f")?;
    let declaration = row.node.ok_or("actual source function node")?;
    let unit = report.package.unit(declaration.file);
    let other = if wrong_owner {
        let (_, row) = function(report, "g")?;
        let key = row.node.ok_or("actual second function")?;
        Some(
            unit.configured
                .owners
                .iter()
                .find(|owner| {
                    let mut node = owner.node;
                    while let crate::syntax::ast::Kind::Attributed { value, .. } =
                        unit.configured.ast.node(node).kind
                    {
                        node = value;
                    }
                    node == key.node && owner.enabled
                })
                .ok_or("actual second configured owner")?
                .node,
        )
    } else {
        None
    };
    for owner in &unit.configured.owners {
        for (effect, attribute, forest) in
            unit.retained_indexed_attributes(owner.attributes.clone())
        {
            if attribute.name(forest, unit.source.text()) == Some("rustc_no_writable") {
                draft.consume(
                    report,
                    facts,
                    (
                        NodeKey {
                            node: other.unwrap_or(owner.node),
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
    Err("actual writable suppression marker".into())
}
#[test]
fn source_incarnation_actual_definition_owner_and_failed_payments_remain_checked() -> TestResult {
    let (report, mut analysis) = checked("owner-replay")?;
    let mut draft = Draft::default();
    consume_first(&mut draft, &report, &mut analysis.facts, false)?;
    let other = test_source::resolve(&input("owner-replay")?, true)?;
    let (index, row) = function(&other, "f")?;
    assert!(draft.bind(&other, index, row).is_err());
    assert!(draft.finish().is_err());
    let (index, row) = function(&report, "f")?;
    assert!(draft.bind(&report, index + 1, row).is_err());
    draft.prepare_binding(&report, &mut analysis.facts)?;
    draft.bind(&report, index, row)?;
    draft.finish()?;
    let mut wrong = Draft::default();
    assert!(consume_first(&mut wrong, &report, &mut analysis.facts, true).is_err());
    assert!(wrong.is_empty());
    for (work, bytes) in [
        (crate::support::crates::MAX_COMPONENTS, 0),
        (0, crate::support::crates::MAX_BYTES),
    ] {
        let (report, mut analysis) = checked("word")?;
        let _ = header_attributes::reserve_source_contracts(&mut analysis.facts, work, bytes);
        let mut failed = Draft::default();
        let before = header_attributes::source_contract_costs(&analysis.facts).0;
        assert!(consume_first(&mut failed, &report, &mut analysis.facts, false).is_err());
        assert!(failed.is_empty());
        let attempted = header_attributes::source_contract_costs(&analysis.facts).0;
        assert!(attempted > before);
        assert!(consume_first(&mut failed, &report, &mut analysis.facts, false).is_err());
        assert!(failed.is_empty());
        assert!(header_attributes::source_contract_costs(&analysis.facts).0 > attempted);
    }
    Ok(())
}
#[test]
fn transient_restrictions_scale_linearly_without_common_layout_growth() -> TestResult {
    let mut previous = (0, 0);
    for width in [8, 32, 128] {
        let mut source = "#![feature(rustc_attrs)]\n".to_owned();
        for index in 0..width {
            writeln!(
                source,
                "#[rustc_no_mir_inline]#[rustc_no_writable]fn f{index}()->u32{{7}}"
            )?;
        }
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let cost = header_attributes::source_contract_costs(&analysis.facts);
        println!(
            "optimization restrictions width={width} work={} bytes={}",
            cost.0, cost.1
        );
        if previous.0 != 0 {
            assert!(cost.0 < previous.0 * 6);
            assert!(cost.1 < previous.1 * 6);
        }
        previous = cost;
    }
    println!(
        "restriction Draft={} Entry={} key={}",
        size_of::<Draft>(),
        size_of::<Entry>(),
        size_of::<(NodeKey, Restriction)>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
