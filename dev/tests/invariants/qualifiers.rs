//! Identity transport never converts ordinary trait evidence into const evidence.
use super::*;
use crate::{
    support::constness::Constness,
    types::{GenericArgs, Primitive, substitute},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_trait_effects_retain_compact_ordinary_records() {
    println!(
        "ItemKind={} Item={} OwnerKind={} Owner={} Bound={} CanonicalPredicate={} InferredPredicate={} Convention={}",
        size_of::<crate::syntax::item::ItemKind>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<crate::sema::associated::OwnerKind>(),
        size_of::<crate::sema::associated::Owner>(),
        size_of::<Bound>(),
        size_of::<Canonical>(),
        size_of::<Predicate>(),
        size_of::<crate::types::callable::Convention>(),
    );
    assert_eq!(size_of::<Bound>(), 8);
    assert_eq!(size_of::<Canonical>(), 12);
    assert_eq!(size_of::<Predicate>(), 24);
    assert_eq!(size_of::<crate::syntax::item::ItemKind>(), 28);
    assert_eq!(size_of::<crate::syntax::item::Item>(), 64);
    assert_eq!(size_of::<crate::sema::associated::OwnerKind>(), 20);
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<crate::types::callable::Convention>(), 2);
}

#[test]
fn qualifiers_survive_inference_substitution_and_shape_keys() -> TestResult {
    let mut store = Store::default();
    let ty = TypeId::primitive(Primitive::U8);
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    let mut keys = Vec::new();
    for constness in [
        Constness::Ordinary,
        Constness::Const,
        Constness::Conditional,
    ] {
        let canonical = Canonical {
            ty,
            bound: Bound::trait_application(application, constness),
        };
        let mut context = Context::default();
        let mut substitution = Instantiation::new(&mut context, &store, &[])?;
        let goal = context.instantiate_predicate(&mut store, canonical, &mut substitution)?;
        let mapped = goal.map_arguments(Ok)?;
        assert_eq!(mapped.constness(), Some(constness));
        assert_eq!(context.resolved_predicate(&mut store, &mapped)?, canonical);
        let mut fold = substitute::Bound::parameters(&store, &[])?;
        assert_eq!(fold.predicate(&mut store, canonical)?, canonical);
        keys.push(store.shape_key(canonical)?.0);
        for other in [
            Constness::Ordinary,
            Constness::Const,
            Constness::Conditional,
        ] {
            let proof = Canonical {
                ty,
                bound: Bound::trait_application(application, other),
            };
            assert_eq!(
                context
                    .equate_trait_predicate(&mut store, &mapped, proof)
                    .is_ok(),
                constness == Constness::Ordinary || constness == other
            );
        }
    }
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[0], keys[2]);
    assert_ne!(keys[1], keys[2]);
    Ok(())
}

#[test]
fn actual_const_trait_and_impl_syntax_keeps_independent_flags() -> TestResult {
    use crate::syntax::{expression::parse_crate, item::ItemKind};
    let ast =
        parse_crate("const trait C{fn f();}const impl C for u8{fn f(){}}trait D{}impl D for u8{}")
            .map_err(|e| std::io::Error::other(e.message))?;
    let flags: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| match item.kind {
            ItemKind::Trait { constant, .. } | ItemKind::Impl { constant, .. } => Some(constant),
            _ => None,
        })
        .collect();
    assert_eq!(flags, [true, true, false, false]);
    Ok(())
}
