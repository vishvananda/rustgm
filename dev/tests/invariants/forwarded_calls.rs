//! Deep unresolved reference inputs retain one shared equation and bounded proof work.
use super::{Context, Decision, Frame, Goal, Search, Selection, Source, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    support::call_traits::CallTrait,
    types::{
        ItemId, Primitive, RegionId, TypeId,
        infer::{Constructor, Domain, arguments::Argument},
    },
};

#[test]
fn reference_chain_selection_keeps_unresolved_work_linear() -> Result<(), Box<dyn std::error::Error>>
{
    for depth in [8, 32, 128, 512] {
        let (_, mut facts) = tests::declarations("fn need<F:Fn(u8)->u8>(){}")?;
        let item = facts
            .associated
            .callable_definition(CallTrait::Fn)
            .ok_or("missing Fn")?;
        let mut context = Context::default();
        let unknown = context.variable(Domain::Any)?;
        let mut subject = context.construct(
            Constructor::Function {
                parameters: 1,
                bound_regions: 0,
                unsafe_: false,
            },
            &[unknown, unknown],
        )?;
        for _ in 0..depth {
            subject = context.construct(
                Constructor::Reference {
                    mutable: false,
                    region: RegionId::STATIC,
                },
                &[subject],
            )?;
        }
        let byte = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
        let tuple = context.construct(Constructor::Tuple, &[byte])?;
        let goal = Goal::trait_application(subject, ItemId(item.0), vec![Argument::Type(tuple)]);
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        let decision = search.run(&mut facts, frame)?;
        let Decision::Proven(_) = decision else {
            return Err("missing forwarded proof".into());
        };
        assert!(
            search.query.steps <= 500 * depth + 10_000,
            "depth {depth}: {} selection steps",
            search.query.steps
        );
        let Selection::Proven(evidence) =
            search.result(&mut facts, &mut context, &goal, decision)?
        else {
            return Err("missing published forwarded evidence".into());
        };
        assert!(
            matches!(evidence.source, Source::Callable { output } if output == TypeId::primitive(Primitive::U8))
        );
        assert_eq!(
            context.resolved(&mut facts.types, unknown)?,
            TypeId::primitive(Primitive::U8)
        );
    }
    Ok(())
}
