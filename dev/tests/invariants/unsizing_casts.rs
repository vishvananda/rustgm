//! Coercion casts consume exact selected child adjustments, without new layout.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn mismatched_child_adjustments_do_not_validate_a_slice_cast() -> Result {
    let report =
        crate::test_source::resolve("fn main(){let a=[1u8,2];let _=(&a) as &[u8];}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let source = facts
        .coercions
        .iter()
        .find(|c| c.kind == AdjustmentKind::ArrayToSlice)
        .ok_or("missing selected array conversion")?
        .source;
    let fact = *facts.node(source).ok_or("missing cast child")?;
    let selected = fact.adjustment.ok_or("missing child adjustment")?;
    assert!(selected_reference_cast(&facts, source, selected.target));
    let mut changed = selected;
    changed.target = TypeId::UNIT;
    facts.node_mut(source).ok_or("missing child")?.adjustment = Some(changed);
    assert!(!selected_reference_cast(&facts, source, selected.target));
    changed = selected;
    changed.kind = AdjustmentKind::Equality;
    facts.node_mut(source).ok_or("missing child")?.adjustment = Some(changed);
    assert!(!selected_reference_cast(&facts, source, selected.target));
    facts.node_mut(source).ok_or("missing child")?.adjustment = Some(selected);
    facts.node_mut(source).ok_or("missing child")?.ty = Some(selected.target);
    assert!(!selected_reference_cast(&facts, source, selected.target));
    *facts.node_mut(source).ok_or("missing child")? = fact;
    assert!(selected_reference_cast(&facts, source, selected.target));
    Ok(())
}
#[test]
fn casts_keep_scalar_and_expression_records_unchanged() {
    assert_eq!(size_of::<TypeId>(), 4);
    assert_eq!(size_of::<Type>(), 24);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
}

#[test]
fn nominal_casts_require_the_selected_terminal_conversion() -> Result {
    let report = crate::test_source::resolve(
        "struct S<T:?Sized>{tail:T}fn main(){let x=S{tail:[1u8,2]};let _: &S<[u8]>=&x;}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let (&source, &terminal) = facts
        .unsizing_adjustments
        .iter()
        .next()
        .ok_or("tail conversion")?;
    let original = *facts.node(source).ok_or("source fact")?;
    let target = original.adjustment.ok_or("adjustment")?.target;
    assert!(selected_reference_cast(&facts, source, target));
    facts.unsizing_adjustments.remove(&source);
    assert!(!selected_reference_cast(&facts, source, target));
    facts.unsizing_adjustments.insert(source, terminal);
    facts.node_mut(source).ok_or("source fact")?.ty = Some(target);
    assert!(!selected_reference_cast(&facts, source, target));
    *facts.node_mut(source).ok_or("source fact")? = original;
    assert!(selected_reference_cast(&facts, source, target));
    Ok(())
}
