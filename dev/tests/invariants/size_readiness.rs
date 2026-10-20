//! Size dependencies preserve their proof frontier and never masquerade as impl headers.
use super::{Decision, Declarations, Dependency, Frame, Search, continuation::Progress, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        aggregate::{self, Repr, Variant},
        parameters::predicates::{Bound, Predicate},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const STR: TypeId = TypeId::primitive(Primitive::Str);
fn steps(search: &Search<'_>) -> usize {
    search.query.sizes.as_ref().map_or(0, |query| query.steps)
}

fn define(facts: &mut Declarations, item: ItemId, tail: TypeId) -> Result {
    let fields = facts.types.intern_list(&[tail])?;
    Ok(facts.aggregates.insert(
        item,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?)
}

#[test]
fn size_waits_retain_search_and_resume_with_the_completed_field_condition() -> Result {
    for (tail, accepted) in [(BYTE, true), (STR, false)] {
        let (_, mut facts) = tests::declarations("fn f(){}")?;
        let item = ItemId(100_000);
        let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Sized,
            },
        )?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        assert_eq!(
            search.begin(&mut facts, first)?,
            Progress::Pending(Dependency::Fields(item))
        );
        let before = (search.query.steps, steps(&search), facts.types.len());
        for _ in 0..8 {
            assert_eq!(
                search.poll(&mut facts)?,
                Progress::Pending(Dependency::Fields(item))
            );
            assert_eq!(
                (search.query.steps, steps(&search), facts.types.len()),
                before
            );
        }
        define(&mut facts, item, tail)?;
        let progress = search.poll(&mut facts)?;
        assert_eq!(
            matches!(progress, Progress::Ready(Decision::Proven(_))),
            accepted
        );
        assert_eq!(
            matches!(progress, Progress::Ready(Decision::Absent)),
            !accepted
        );
    }
    Ok(())
}

#[test]
fn wide_size_queries_do_not_restart_at_each_independent_field_wait() -> Result {
    for count in [128, 256, 512, 1024] {
        let (_, mut facts) = tests::declarations("fn f(){}")?;
        let mut types = Vec::new();
        for index in 0..count {
            types.push(
                facts
                    .types
                    .intern(Kind::Adt(ItemId(100_000 + index), GenericArgs::EMPTY))?,
            );
        }
        let list = facts.types.intern_list(&types)?;
        let ty = facts.types.intern(Kind::Tuple(list))?;
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Sized,
            },
        )?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(
            tests::caller(&assumptions, None),
            Availability::Declarations,
        );
        let first = Frame::new(&facts, &mut context, &goal, 1)?;
        let mut progress = search.begin(&mut facts, first)?;
        for index in 0..count {
            let item = ItemId(100_000 + index);
            assert_eq!(progress, Progress::Pending(Dependency::Fields(item)));
            define(&mut facts, item, BYTE)?;
            progress = search.poll(&mut facts)?;
        }
        assert!(matches!(progress, Progress::Ready(Decision::Proven(_))));
        assert!(steps(&search) < count as usize * 4);
        assert_eq!(facts.aggregates.instantiation_steps(), 0);
    }
    Ok(())
}

#[test]
fn a_positive_recursive_assumption_is_rechecked_after_fields_finish() -> Result {
    let (_, mut facts) = tests::declarations("fn f(){}")?;
    let item = ItemId(100_000);
    let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    facts
        .generics
        .size_rules
        .get_or_insert_with(Default::default)
        .active
        .insert(item);
    let mut query = crate::sema::nominals::sizes::Query::new(None);
    assert!(matches!(
        query.demand(&mut facts, ty, |_, ty| Ok(ty))?,
        crate::sema::nominals::sizes::Progress::Ready {
            sized: true,
            provisional: true
        }
    ));
    define(&mut facts, item, STR)?;
    facts
        .generics
        .size_rules
        .as_mut()
        .ok_or("missing pending sizes")?
        .active
        .remove(&item);
    assert!(matches!(
        query.demand(&mut facts, ty, |_, ty| Ok(ty))?,
        crate::sema::nominals::sizes::Progress::Ready {
            sized: false,
            provisional: false
        }
    ));
    Ok(())
}

#[test]
fn selected_implementation_keeps_provisional_size_evidence_out_of_its_cache() -> Result {
    let (report, mut facts) = tests::declarations("trait A {} impl<T> A for T {}")?;
    let item = ItemId(100_000);
    let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    facts
        .generics
        .size_rules
        .get_or_insert_with(Default::default)
        .active
        .insert(item);
    let predicate = tests::predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        ty,
        GenericArgs::EMPTY,
    )?;
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    let Decision::Proven(root) = search.run(&mut facts, first)? else {
        return Err("recursive positive field assumption was not selected".into());
    };
    assert!(search.query.nodes[root].provisional_size);
    assert!(!search.query.cacheable(root));
    for decision in search.query.completed.values() {
        if let Decision::Proven(index) = decision {
            assert!(!search.query.nodes[*index].provisional_size);
        }
    }
    assert!(search.evidence(root)?.provisional_size);

    define(&mut facts, item, STR)?;
    facts
        .generics
        .size_rules
        .as_mut()
        .ok_or("missing pending sizes")?
        .active
        .remove(&item);
    let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        Availability::Declarations,
    );
    let first = Frame::new(&facts, &mut context, &goal, 1)?;
    assert_eq!(search.run(&mut facts, first)?, Decision::Absent);
    Ok(())
}
