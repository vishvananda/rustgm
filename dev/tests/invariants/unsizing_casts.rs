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

/// A cast to a type parameter that the caller's `T: Unsize<U>` clause proves
/// validates only against its own selected bound conversion of exactly these
/// referents.
#[test]
fn bound_casts_require_their_own_selected_referent_pair() -> Result {
    use crate::types::unsizing::Conversion;
    let source = concat!(
        "#![feature(no_core, lang_items, rustc_attrs)]\n#![no_core]\n",
        "#[lang = \"pointee_sized\"]\npub trait PointeeSized {}\n",
        "#[lang = \"meta_sized\"]\npub trait MetaSized: PointeeSized {}\n",
        "#[lang = \"sized\"]\npub trait Sized: MetaSized {}\n",
        "#[lang = \"legacy_receiver\"]\npub trait LegacyReceiver: PointeeSized {}\n",
        "impl<T: PointeeSized> LegacyReceiver for &T {}\n",
        "#[lang = \"clone\"]\npub trait Clone: Sized {\n",
        "    #[lang = \"clone_fn\"]\n    fn clone(&self) -> Self;\n}\n",
        "#[lang = \"copy\"]\npub trait Copy: Clone {}\n",
        "#[lang = \"unsize\"]\n#[rustc_deny_explicit_impl]\n#[rustc_dyn_incompatible_trait]\n",
        "pub trait Unsize<T: PointeeSized>: PointeeSized {}\n",
        "#[lang = \"coerce_unsized\"]\npub trait CoerceUnsized<T: PointeeSized>: Sized {}\n",
        "impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> ",
        "CoerceUnsized<&'a U> for &'b T {}\n",
        "pub fn view<T: Unsize<U>, U: PointeeSized>(value: &T) -> &U {\n    value as &U\n}\n",
    );
    let report = crate::test_source::source_provider(source, "bound_cast")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let (&child, &terminal) = facts
        .unsizing_adjustments
        .iter()
        .find(|(_, conversion)| matches!(conversion, Conversion::Bound { .. }))
        .ok_or("missing selected bound conversion")?;
    let Conversion::Bound {
        source: from,
        target: to,
    } = terminal
    else {
        return Err("selected conversion changed".into());
    };
    let original = *facts.node(child).ok_or("missing cast child")?;
    let selected = original.adjustment.ok_or("missing child adjustment")?;
    assert!(selected_reference_cast(&facts, child, selected.target));
    for changed in [
        Conversion::Bound {
            source: to,
            target: from,
        },
        Conversion::Bound {
            source: from,
            target: from,
        },
        Conversion::Object {
            concrete: from,
            target: to,
        },
        Conversion::Nominal {
            source: from,
            target: to,
        },
    ] {
        facts.unsizing_adjustments.insert(child, changed);
        assert!(!selected_reference_cast(&facts, child, selected.target));
    }
    facts.unsizing_adjustments.remove(&child);
    assert!(!selected_reference_cast(&facts, child, selected.target));
    facts.unsizing_adjustments.insert(child, terminal);
    let mut changed = selected;
    changed.kind = AdjustmentKind::Equality;
    facts.node_mut(child).ok_or("missing child")?.adjustment = Some(changed);
    assert!(!selected_reference_cast(&facts, child, selected.target));
    changed.kind = AdjustmentKind::ArrayToSlice;
    facts.node_mut(child).ok_or("missing child")?.adjustment = Some(changed);
    assert!(!selected_reference_cast(&facts, child, selected.target));
    facts.node_mut(child).ok_or("missing child")?.adjustment = Some(selected);
    facts.node_mut(child).ok_or("missing child")?.ty = Some(selected.target);
    assert!(!selected_reference_cast(&facts, child, selected.target));
    *facts.node_mut(child).ok_or("missing child")? = original;
    assert!(selected_reference_cast(&facts, child, selected.target));
    Ok(())
}
