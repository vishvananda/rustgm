//! Signature inference cannot publish a callable permission before capture analysis.
use super::{
    Context, Decision, Frame, Goal, Search, Source, continuation::Progress, frames::Inference,
    tests,
};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    support::call_traits::CallTrait,
    types::{
        ItemId, Primitive, TypeId,
        closure::CallKind,
        infer::{Constructor, Domain, arguments::Argument},
    },
};

#[test]
fn pending_capture_kind_retains_signature_inference_without_granting_permission()
-> Result<(), Box<dyn std::error::Error>> {
    for actual in [CallKind::Shared, CallKind::Mutable, CallKind::Once] {
        let (_, mut facts) = tests::declarations("fn need<F:Fn(u8)->u8>(){}")?;
        let identity = facts.types.reserve_closure()?;
        let item = facts
            .associated
            .callable_definition(CallTrait::Fn)
            .ok_or("missing Fn")?;
        let mut context = Context::default();
        let unknown = context.variable(Domain::Any)?;
        let signature = context.construct(
            Constructor::Function {
                parameters: 1,
                bound_regions: 0,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            },
            &[unknown, unknown],
        )?;
        let environment = context.variable(Domain::Any)?;
        let subject = context.closure(identity, signature, environment, &[])?;
        let byte = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
        let tuple = context.construct(Constructor::Tuple, &[byte])?;
        let goal = Goal::trait_application(subject, ItemId(item.0), vec![Argument::Type(tuple)]);
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        search.query.refining = true;
        let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
        frame.inference = Inference::Selecting;
        assert_eq!(search.begin(&mut facts, frame)?, Progress::Inferred);
        assert!(search.publish_inputs(&mut facts, &mut context)?);
        assert!(search.query.nodes.is_empty());
        assert_eq!(
            context.resolved(&mut facts.types, unknown)?,
            TypeId::primitive(Primitive::U8)
        );
        assert!(matches!(
            context.resolved(&mut facts.types, subject),
            Err(crate::types::infer::Error::Unresolved)
        ));
        facts.types.complete_closure(identity, actual)?;
        let empty = context.known(&facts.types, TypeId::UNIT)?;
        context.equate(&facts.types, environment, empty)?;
        search.resume_at(search.query.allocations.end())?;
        search.refresh_inputs(&mut facts, &mut context)?;
        match search.poll(&mut facts)? {
            Progress::Ready(Decision::Proven(root)) if actual == CallKind::Shared => {
                let evidence = search.evidence(root)?;
                assert!(
                    matches!(evidence.source, Source::Callable { output } if output == TypeId::primitive(Primitive::U8))
                );
            }
            Progress::Ready(Decision::Absent) if actual != CallKind::Shared => {}
            _ => return Err("pending body escaped its completed capture permission".into()),
        }
    }
    Ok(())
}
