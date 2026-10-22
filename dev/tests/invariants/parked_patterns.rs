//! Parked pattern owners reserve disjoint proof IDs and restore exact query metadata.
use super::*;
use crate::{
    sema::{declarations, dispatch::Dispatcher},
    types::RegionId,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn frame(queries: &mut Queries<'_>, body: crate::resolve::unit::NodeKey) -> Result<FrameId, Error> {
    let id = queries.append_pattern_frame(Frame {
        prepared: None,
        cursor: 0,
        saved: None,
        reservation: None,
        bytes: 0,
    })?;
    queries.install_pattern_frame(
        id,
        Some(PatternScope {
            body,
            owner: None,
            binders: 1,
            end: 0,
        }),
        None,
        queries.work.context,
    )?;
    Ok(id)
}
fn frontier(queries: &Queries<'_>) -> Result<u32, std::num::TryFromIntError> {
    u32::try_from(queries.typing.facts.body_regions.len() + 1)
}
#[test]
fn malformed_discriminant_requirement_does_not_schedule_or_publish() -> TestResult {
    let report = crate::test_source::resolve("struct S;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let member = facts
        .members
        .variants
        .first()
        .ok_or("missing struct member")?;
    assert!(
        facts
            .aggregates
            .get(member.owner)
            .is_some_and(|shape| shape.kind == crate::types::aggregate::Kind::Struct)
    );
    let mut queries = Queries::new(&report, &mut facts);
    for variant in [0, u32::MAX] {
        assert!(
            queries
                .prepare_pattern_requirement(
                    crate::sema::values::execution::Requirement::DiscriminantValue(
                        crate::sema::members::VariantId(variant)
                    )
                )
                .is_err()
        );
    }
    assert!(queries.work.pop().is_none());
    assert!(queries.states.is_empty());
    assert!(queries.pattern_frames.is_none());
    Ok(())
}
#[test]
fn interleaved_dispatchers_keep_foreign_gaps_and_exact_owners() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=7;const B:u8=9;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let a = facts.initializers[0].semantic_source();
    let b = facts.initializers[1].semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    let outer = frame(&mut queries, a)?;
    let first = frontier(&queries)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, first)?);
    queries.reserve_pattern_dispatch()?;
    let inner = frame(&mut queries, b)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, frontier(&queries)?)?);
    queries.reserve_pattern_dispatch()?;
    let (scratch, continuation) = queries.suspend()?;
    let mut queries = Queries::with_scratch(&report, &mut facts, None, scratch);
    queries.resume(*continuation);
    assert!(queries.finish_pattern_frame(outer).is_err());
    assert_eq!(
        queries
            .pattern_frames
            .as_ref()
            .ok_or("missing arena")?
            .active
            .len(),
        2
    );
    queries.finish_pattern_frame(inner)?;
    queries.resume_pattern_dispatch()?;
    queries.finish_pattern_frame(outer)?;
    assert!(queries.pattern_scope.is_none() && queries.pattern_dispatch.is_none());
    let regions = &queries.typing.facts.body_regions;
    assert_eq!(regions[first as usize - 1].body, a);
    assert_eq!(regions[first as usize].body, b);
    let proofs = &queries.typing.facts.quantified_proofs;
    let parent = proofs
        .iter()
        .find(|proof| proof.body == a)
        .ok_or("missing parent proof")?;
    let child = queries
        .typing
        .facts
        .region_equalities
        .iter()
        .filter(|equality| equality.body == b)
        .map(|equality| (equality.left, equality.right))
        .collect::<Vec<_>>();
    assert_eq!(child, [(RegionId(first + 1), RegionId::bound(0, 0)?)]);
    assert!(matches!(
        regions[first as usize].origin,
        crate::sema::model::BodyRegionOrigin::Inference
    ));
    assert_eq!(
        parent.evidence.proof.equalities,
        [(RegionId(first), RegionId::bound(0, 0)?)]
    );
    assert!(parent.evidence.proof.has_gaps());
    assert_eq!(
        queries
            .pattern_frames
            .as_ref()
            .ok_or("missing arena")?
            .children,
        0
    );
    Ok(())
}
#[test]
fn abort_restores_context_without_publishing_parked_proofs() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=7;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let body = facts.initializers[0].semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    let value = ValueScope::Initializer { body, index: 0 };
    queries.value_scope = Some(value);
    queries.work.context = Context(3);
    let outer = frame(&mut queries, body)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, frontier(&queries)?)?);
    let _inner = frame(&mut queries, body)?;
    queries.work.context = Context(17);
    queries.abort_pattern_frames();
    assert_eq!(queries.work.context.0, 3);
    assert!(
        matches!(queries.value_scope,Some(ValueScope::Initializer{body:actual,index:0})if actual==body)
    );
    assert!(queries.pattern_dispatch.is_none() && queries.pattern_scope.is_none());
    assert!(queries.typing.facts.quantified_proofs.is_empty());
    assert!(
        queries
            .pattern_frames
            .as_mut()
            .ok_or("missing arena")?
            .frame(outer)
            .is_err()
    );
    Ok(())
}
#[test]
fn duplicate_entry_and_storage_preflight_do_not_consume_active_state() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=7;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let body = facts.initializers[0].semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    let id = frame(&mut queries, body)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, frontier(&queries)?)?);
    assert!(
        queries
            .install_pattern_frame(id, None, None, Context(0))
            .is_err()
    );
    assert!(queries.pattern_dispatch.is_some());
    let frames = queries.pattern_frames.as_mut().ok_or("missing arena")?;
    frames.children = super::super::pending_patterns::LIMIT;
    let before = frames.slots.len();
    let extra = Frame {
        prepared: None,
        cursor: 0,
        saved: None,
        reservation: None,
        bytes: 0,
    };
    assert!(queries.append_pattern_frame(extra).is_err());
    assert_eq!(
        queries
            .pattern_frames
            .as_ref()
            .ok_or("missing arena")?
            .slots
            .len(),
        before
    );
    Ok(())
}

#[test]
fn parent_pattern_owner_reserves_before_child_inference_allocations() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=7;const B:u8=9;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let a = facts.initializers[0].semantic_source();
    let b = facts.initializers[1].semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    let outer = frame(&mut queries, a)?;
    let first = frontier(&queries)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, first)?);
    queries.reserve_pattern_dispatch()?;
    assert_eq!(frontier(&queries)?, first + 1);
    let mut foreign = crate::sema::traits::obligations::Reservation::default();
    foreign.reserve(queries.typing.facts, b, b, first + 1..first + 2)?;
    let inner = frame(&mut queries, b)?;
    queries.pattern_dispatch = Some(Dispatcher::new(1, frontier(&queries)?)?);
    queries.finish_pattern_frame(inner)?;
    assert!(
        queries
            .pattern_frames
            .as_ref()
            .ok_or("missing frames")?
            .active
            .last()
            .copied()
            == Some(outer)
    );
    queries.finish_pattern_frame(outer)?;
    let regions = &queries.typing.facts.body_regions;
    assert_eq!(regions[first as usize - 1].body, a);
    assert_eq!(regions[first as usize].body, b);
    assert_eq!(regions[first as usize + 1].body, b);
    let proof = queries
        .typing
        .facts
        .quantified_proofs
        .iter()
        .find(|proof| proof.body == a)
        .ok_or("missing exterior proof")?;
    assert!(proof.evidence.proof.has_gaps());
    Ok(())
}

#[test]
fn selected_value_keeps_caller_scope_and_restores_initializer_owner() -> TestResult {
    let report = crate::test_source::resolve("const A:u8=7;const B:u8=9;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let caller = facts.initializers[0].semantic_source();
    let original = facts.initializers[1].semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    let outer = frame(&mut queries, caller)?;
    queries.begin_pattern_value(1, Context(0))?;
    let inner = *queries
        .pattern_frames
        .as_ref()
        .ok_or("missing frames")?
        .active
        .last()
        .ok_or("missing inner frame")?;
    assert!(inner != outer);
    let scope = queries.pattern_scope.ok_or("missing caller scope")?;
    assert_eq!(scope.body, caller);
    assert_eq!(scope.binders, 1);
    assert!(matches!(queries.value_scope,
        Some(ValueScope::Initializer { body, index:1 }) if body == original));
    queries.finish_pattern_frame(inner)?;
    assert_eq!(
        queries.pattern_scope.ok_or("missing restored scope")?.body,
        caller
    );
    assert!(queries.value_scope.is_none());
    queries.finish_pattern_frame(outer)?;
    assert!(queries.pattern_scope.is_none() && queries.value_scope.is_none());
    Ok(())
}
