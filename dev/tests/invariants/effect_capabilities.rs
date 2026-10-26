//! A source const impl capability is immutable; each use keeps its own effect key.
use super::*;
use crate::{
    sema::traits::{
        assumptions::Assumptions,
        implementations::{Availability, Implementation, Implementations},
        selection,
    },
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, ItemId, Primitive, TypeId,
        infer::{Context, Instantiation},
        parameters::predicates::Bound,
        traits::Application,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn provider(constant: bool) -> Result<Declarations> {
    let (prefix, _) = crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID
        .split_once("const fn unused")
        .ok_or("prefix")?;
    let source = if constant {
        prefix.to_owned()
    } else {
        prefix.replace("const impl Clone for u8", "impl Clone for u8")
    };
    let source = source + "struct CacheWitness;";
    let report = crate::test_source::source_provider(&source, "const_impl_capability")?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
fn predicate(facts: &mut Declarations, effect: Constness) -> Result<Predicate> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Clone)?
        .ok_or("Clone")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(Predicate {
        ty: TypeId::primitive(Primitive::U8),
        bound: Bound::trait_application(application, effect),
    })
}
fn select(
    facts: &mut Declarations,
    predicate: Predicate,
    reuse: &mut Reuse,
) -> Result<selection::Selection> {
    let mut context = Context::default();
    let mut fold = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut fold)?;
    let assumptions = Assumptions::default();
    Ok(selection::select_reusing(
        facts,
        &mut context,
        &goal,
        selection::Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 1,
            retain_sized: false,
        },
        (Availability::Validated, false),
        reuse,
    )?)
}
#[test]
fn actual_const_impl_uses_preserve_canonical_capability_and_effect_cache_keys() -> Result {
    for constant in [false, true] {
        for order in [
            [Constness::Ordinary, Constness::Const],
            [Constness::Const, Constness::Ordinary],
        ] {
            let mut facts = provider(constant)?;
            let mut reuse = Reuse::default();
            for effect in order
                .into_iter()
                .chain(order)
                .chain([Constness::Conditional])
            {
                let goal = predicate(&mut facts, effect)?;
                let selected = select(&mut facts, goal, &mut reuse)?;
                assert_eq!(
                    matches!(selected, selection::Selection::Proven(_)),
                    constant || effect == Constness::Ordinary
                );
                if let selection::Selection::Proven(evidence) = selected {
                    // Explicitly retain this successful complete source to cover
                    // the direct lookup even if a conservative child was not exported.
                    assert!(evidence.proof.is_empty() && evidence.sized.is_empty());
                    reuse.completed.insert(goal, evidence.source);
                    assert!(reuse.selection(&facts, goal, 1)?.is_some());
                }
            }
            let ordinary = predicate(&mut facts, Constness::Ordinary)?;
            let required = predicate(&mut facts, Constness::Const)?;
            let mut ordinary_only = Reuse::default();
            ordinary_only.completed.insert(
                ordinary,
                reuse
                    .completed
                    .get(&ordinary)
                    .ok_or("ordinary source")?
                    .clone(),
            );
            assert!(ordinary_only.selection(&facts, required, 1)?.is_none());
            let (application, _) = ordinary.bound.trait_parts().ok_or("application")?;
            let header = facts
                .generics
                .implementations
                .as_ref()
                .ok_or("registry")?
                .headers()
                .find(|h| {
                    h.predicate.ty == ordinary.ty
                        && h.predicate
                            .bound
                            .trait_parts()
                            .is_some_and(|(id, _)| id == application)
                })
                .ok_or("Clone impl")?;
            assert_eq!(
                header
                    .predicate
                    .bound
                    .trait_parts()
                    .map(|(_, effect)| effect),
                Some(if constant {
                    Constness::Const
                } else {
                    Constness::Ordinary
                })
            );
        }
    }
    Ok(())
}
#[test]
fn source_const_header_mismatch_fails_publication_and_direct_completed_reuse() -> Result {
    let mut facts = provider(true)?;
    let goal = predicate(&mut facts, Constness::Ordinary)?;
    let selection::Selection::Proven(evidence) = select(&mut facts, goal, &mut Reuse::default())?
    else {
        return Err("source".into());
    };
    let Source::Implementation { owner, .. } = evidence.source else {
        return Err("impl".into());
    };
    let header = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .header(owner)
        .ok_or("header")?;
    let (application, _) = header.predicate.bound.trait_parts().ok_or("effect")?;
    let source = header.source;
    let actual = *facts
        .associated
        .owner(owner)
        .and_then(|id| facts.associated.get(id))
        .ok_or("owner")?;
    let make = || Implementation {
        owner,
        source,
        predicate: Predicate {
            ty: goal.ty,
            bound: Bound::Trait(application),
        },
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    };
    let trait_item = facts
        .types
        .trait_application(application)
        .ok_or("trait")?
        .item;
    let mut registry = Implementations::default();
    assert!(
        registry
            .prepare_source(
                &mut facts.types,
                &facts.associated,
                trait_item.into(),
                actual,
                make()
            )
            .is_err()
    );
    assert_eq!(registry.headers().count(), 0);
    let mut wrong = actual;
    if let crate::sema::associated::OwnerKind::Impl { constant, .. } = &mut wrong.kind {
        *constant = false;
    }
    assert!(
        registry
            .prepare_source(
                &mut facts.types,
                &facts.associated,
                trait_item.into(),
                wrong,
                make()
            )
            .is_err()
    );
    // Test-only synthetic publication simulates corrupted retained state. The
    // production preparation API above refuses both variants before mutation.
    registry.prepare(&mut facts.types, trait_item, make())?;
    facts.generics.implementations = Some(Box::new(registry));
    let mut reuse = Reuse::default();
    reuse.completed.insert(goal, evidence.source);
    assert!(reuse.selection(&facts, goal, 1).is_err());
    Ok(())
}

#[test]
fn const_impl_conditional_clauses_are_paid_under_each_requested_effect() -> Result {
    for clone_constant in [false, true] {
        let (prefix, _) = crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID
            .split_once("const fn unused")
            .ok_or("prefix")?;
        let mut source = prefix.to_owned();
        if !clone_constant {
            source = source.replace("const impl Clone for u8", "impl Clone for u8");
        }
        source
            .push_str("pub const trait C{}const impl<T:[const]Clone>C for T{}struct CacheWitness;");
        let report = crate::test_source::source_provider(&source, "const_impl_conditions")?;
        let definition = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "C")
            .ok_or("C")?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let application = facts.types.intern_trait_application(Application {
            item: ItemId(u32::try_from(definition)?),
            arguments: GenericArgs::EMPTY,
        })?;
        let mut reuse = Reuse::default();
        for effect in [
            Constness::Ordinary,
            Constness::Const,
            Constness::Conditional,
            Constness::Ordinary,
            Constness::Const,
        ] {
            let goal = Predicate {
                ty: TypeId::primitive(Primitive::U8),
                bound: Bound::trait_application(application, effect),
            };
            let selected = select(&mut facts, goal, &mut reuse)?;
            assert_eq!(
                matches!(selected, selection::Selection::Proven(_)),
                clone_constant || effect == Constness::Ordinary
            );
        }
        assert!(
            facts
                .generics
                .implementations
                .as_ref()
                .ok_or("registry")?
                .headers()
                .any(|header| header.predicate.bound == Bound::ConstTrait(application))
        );
    }
    Ok(())
}
