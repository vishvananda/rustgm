//! Interleaved declarations cannot invalidate storage IDs or erase a caller's proof.
use super::super::{Progress, Query, graph::Recipe};
use super::{
    Argument, Kind, Obligations, Preparation, Primitive, RegionId, ScopeId, TypeId, context,
    projections,
};
use crate::sema::traits::implementations::Availability;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn a_child_can_publish_the_waiting_parents_root_without_replacing_recipe_ids() -> TestResult {
    let (mut facts, first) = context()?;
    let Some(Kind::Adt(owner, _)) = facts.types.kind(first) else {
        return Err("missing nominal".into());
    };
    let input = facts.types.intern(Kind::Reference {
        region: RegionId(10),
        mutable: false,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId(9)), Argument::Type(input)])?;
    let second = facts.types.intern(Kind::Adt(owner, arguments))?;
    let list = facts.types.intern_list(&[first, second])?;
    let root = facts.types.intern(Kind::Tuple(list))?;
    let definition = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing impl")?
        .types[0]
        .1;
    let mut caller = Preparation::default();
    let mut query = Query::new(root);
    let mut proof = Obligations::new(100);
    let mut cursor =
        projections::Cursor::new(None, &proof, ScopeId::ROOT, Availability::Validated)?;
    let mut selections = 0;
    let mut requests = 0;
    let mut interleaved = false;
    loop {
        match caller.advance(&mut facts, &mut query)? {
            Progress::Complete => break,
            Progress::Nominal(_) => return Err("unexpected nominal wait".into()),
            Progress::Normalize(raw) => {
                requests += 1;
                let saved = if requests == 2 {
                    facts.definitions[definition.0 as usize].take()
                } else {
                    None
                };
                cursor.start(&mut facts, raw)?;
                let mut advance = |facts: &mut crate::sema::model::Declarations,
                                   proof: &mut Obligations| {
                    cursor.advance(facts, proof, &mut |_, _| {
                        selections += 1;
                        Ok(None)
                    })
                };
                let progress = advance(&mut facts, &mut proof)?;
                let progress = if let projections::Progress::Definition(wait) = progress {
                    assert_eq!(wait, definition);
                    assert!(!interleaved);
                    for _ in 0..8 {
                        assert_eq!(
                            advance(&mut facts, &mut proof)?,
                            projections::Progress::Definition(wait)
                        );
                        assert!(
                            matches!(caller.advance(&mut facts, &mut query)?, Progress::Normalize(again) if again == raw)
                        );
                    }
                    let prefix = recipes(&facts)?;
                    assert!(!prefix.is_empty());
                    facts.definitions[definition.0 as usize] = saved;
                    let mut child = Obligations::new(proof.regions.end);
                    Preparation::default().prepare(
                        &mut facts,
                        root,
                        &mut projections::concrete(&mut child, ScopeId::ROOT)?,
                    )?;
                    assert!(!child.outlives.is_empty());
                    assert_eq!(&recipes(&facts)?[..prefix.len()], prefix.as_slice());
                    assert!(recipes(&facts)?.len() > prefix.len());
                    proof.resume_at(child.regions.end)?;
                    interleaved = true;
                    advance(&mut facts, &mut proof)?
                } else {
                    progress
                };
                let projections::Progress::Complete { ty, steps } = progress else {
                    return Err("restored definition did not complete".into());
                };
                caller.normalized(&mut facts, &mut query, ty, steps)?;
            }
        }
    }
    assert!(interleaved);
    assert_eq!(selections, 2);
    assert!(proof.outlives.len() >= 2);
    assert!(caller.steps <= 128);
    Ok(())
}

fn recipes(facts: &crate::sema::model::Declarations) -> Result<Vec<String>, &'static str> {
    Ok(facts
        .stored_fields
        .as_ref()
        .ok_or("missing graph")?
        .recipes
        .iter()
        .map(|recipe| match recipe {
            Recipe::Field { raw, normalized } => format!("{raw:?}:{normalized:?}"),
            Recipe::Join(children) => format!("{children:?}"),
        })
        .collect())
}

#[test]
fn an_unfinished_nominal_yields_its_identity_without_expanding_referents() -> TestResult {
    use crate::types::{
        GenericArgs, ItemId,
        aggregate::{Kind as AggregateKind, Repr, Variant},
    };
    let (mut facts, _) = context()?;
    let owner = ItemId(10000);
    let root = facts.types.intern(Kind::Adt(owner, GenericArgs::EMPTY))?;
    let reference = facts.types.intern(Kind::Reference {
        region: RegionId::STATIC,
        mutable: false,
        referent: root,
    })?;
    let mut caller = Preparation::default();
    let mut reference_query = Query::new(reference);
    assert!(matches!(
        caller.advance(&mut facts, &mut reference_query)?,
        Progress::Complete
    ));
    let mut query = Query::new(root);
    for _ in 0..8 {
        assert!(
            matches!(caller.advance(&mut facts, &mut query)?, Progress::Nominal(item) if item == owner)
        );
    }
    let fields = facts
        .types
        .intern_list(&[TypeId::primitive(Primitive::U8)])?;
    facts.aggregates.insert(
        owner,
        AggregateKind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert!(matches!(
        caller.advance(&mut facts, &mut query)?,
        Progress::Complete
    ));
    Ok(())
}

fn nominal(
    facts: &mut crate::sema::model::Declarations,
    root: TypeId,
    index: u32,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Adt(owner, _)) = facts.types.kind(root) else {
        return Err("missing nominal");
    };
    let input = facts.types.intern(Kind::Reference {
        region: RegionId(8 + 2 * index),
        mutable: false,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId(7 + 2 * index)),
        Argument::Type(input),
    ])?;
    facts.types.intern(Kind::Adt(owner, arguments))
}

#[test]
fn every_field_wait_retains_the_graph_frontier_in_deep_shared_recipes() -> TestResult {
    use std::collections::HashSet;
    for count in [8_u32, 32, 128] {
        let (mut facts, leaf) = context()?;
        let mut root = leaf;
        for index in 1..count {
            let next = nominal(&mut facts, leaf, index)?;
            let fields = facts.types.intern_list(&[root, root, next])?;
            root = facts.types.intern(Kind::Tuple(fields))?;
        }
        let definition = facts
            .generics
            .implementations()
            .next()
            .ok_or("missing impl")?
            .types[0]
            .1;
        let mut query = Query::new(root);
        let mut caller = Preparation::default();
        let mut proof = Obligations::new(1000);
        let mut cursor =
            projections::Cursor::new(None, &proof, ScopeId::ROOT, Availability::Validated)?;
        let mut seen = HashSet::new();
        let mut selections = 0;
        loop {
            let raw = match caller.advance(&mut facts, &mut query)? {
                Progress::Complete => break,
                Progress::Normalize(raw) => raw,
                Progress::Nominal(_) => return Err("unexpected nominal wait".into()),
            };
            let fresh = seen.insert(raw);
            let saved = if fresh {
                facts.definitions[definition.0 as usize].take()
            } else {
                None
            };
            cursor.start(&mut facts, raw)?;
            let mut advance = |facts: &mut crate::sema::model::Declarations,
                               proof: &mut Obligations| {
                cursor.advance(facts, proof, &mut |_, _| {
                    selections += 1;
                    Ok(None)
                })
            };
            if fresh {
                let prior = facts.stored_field_work().steps;
                let caller_prior = caller.steps;
                for _ in 0..8 {
                    assert_eq!(
                        advance(&mut facts, &mut proof)?,
                        projections::Progress::Definition(definition)
                    );
                    assert!(
                        matches!(caller.advance(&mut facts, &mut query)?, Progress::Normalize(again) if again == raw)
                    );
                    assert_eq!(facts.stored_field_work().steps, prior);
                    assert_eq!(caller.steps, caller_prior);
                }
                proof.resume_at(proof.regions.end + 3)?;
                facts.definitions[definition.0 as usize] = saved;
            }
            let projections::Progress::Complete { ty, steps } = advance(&mut facts, &mut proof)?
            else {
                return Err("pending field did not complete".into());
            };
            caller.normalized(&mut facts, &mut query, ty, steps)?;
        }
        let count = count as usize;
        assert_eq!(selections, count);
        assert_eq!(seen.len(), count);
        let owned: usize = proof.owned_regions().map(|range| range.len()).sum();
        assert_eq!(proof.regions.len() - owned, 3 * count);
        assert!(caller.steps <= 64 * count + 128);
        let work = facts.stored_field_work();
        assert!(
            work.steps <= 64 * count + 128,
            "{count}: {} structural steps",
            work.steps
        );
        assert!(work.recipes <= 2 * count);
        assert!(work.retained_bytes <= 512 * count + 512);
    }
    Ok(())
}

#[test]
fn failed_replay_does_not_authorize_a_visited_prefix_for_another_query() -> TestResult {
    let (mut facts, leaf) = context()?;
    let second = nominal(&mut facts, leaf, 1)?;
    let fields = facts.types.intern_list(&[leaf, second])?;
    let root = facts.types.intern(Kind::Tuple(fields))?;
    let mut structural = Obligations::new(100);
    Preparation::default().prepare(
        &mut facts,
        root,
        &mut projections::concrete(&mut structural, ScopeId::ROOT)?,
    )?;
    let mut caller = Preparation::default();
    let mut query = Query::new(root);
    let Progress::Normalize(first) = caller.advance(&mut facts, &mut query)? else {
        return Err("missing first debt".into());
    };
    let mut proof = Obligations::new(structural.regions.end);
    let mut session = projections::concrete(&mut proof, ScopeId::ROOT)?;
    let (normal, steps) = session.normalize(&mut facts, first)?;
    caller.normalized(&mut facts, &mut query, normal, steps)?;
    assert!(matches!(
        caller.advance(&mut facts, &mut query)?,
        Progress::Normalize(_)
    ));
    assert!(
        caller
            .normalized(&mut facts, &mut query, TypeId::primitive(Primitive::U32), 0)
            .is_err()
    );
    assert!(caller.complete.is_empty());
    let mut next = Query::new(root);
    assert!(
        matches!(caller.advance(&mut facts, &mut next)?, Progress::Normalize(raw) if raw == first)
    );
    Ok(())
}
