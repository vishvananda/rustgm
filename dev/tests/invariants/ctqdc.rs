//! Completed absence remains separate from failed or effect-mismatched queries.
use super::*;
use crate::{
    sema,
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, ItemId, Kind, TypeId, parameters::predicates::Bound, traits::Application,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn facts() -> Result<(
    Declarations,
    TypeId,
    TypeId,
    crate::types::traits::ApplicationId,
)> {
    let text = crate::test_source::fixture("pa31/tests/spec/source-const-cleanup/prknow.rs")?;
    let report = crate::test_source::source_provider(&text, "scoped_const_decisions")?;
    let mut facts = sema::declarations::analyze(&report)?;
    let find = |name: &str| {
        report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find_map(|(index, definition)| {
                let ty = facts.definitions.get(index).copied().flatten()?;
                (report.package.names.get(definition.name) == name
                    && matches!(facts.types.kind(ty), Some(Kind::Adt(..))))
                .then_some(ty)
            })
            .ok_or_else(|| format!("nominal {name}"))
    };
    let ordinary = find("N")?;
    let constant = find("C")?;
    let item = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Destruct)?
        .ok_or("Destruct")?
        .definition;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(item.0),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok((facts, ordinary, constant, application))
}
#[test]
fn completed_false_cannot_contaminate_a_later_true_or_stronger_effect() -> Result {
    let (mut facts, ordinary, constant, application) = facts()?;
    for effect in [Constness::Const, Constness::Conditional] {
        let mut state = State::body(&mut facts, None, 0, 1)?;
        let goal = |ty, effect| Predicate {
            ty,
            bound: Bound::trait_application(application, effect),
        };
        assert_eq!(
            state.decide(&mut facts, goal(ordinary, Constness::Ordinary))?,
            Decision::Proven
        );
        for _ in 0..3 {
            assert_eq!(
                state.decide(&mut facts, goal(ordinary, effect))?,
                Decision::Absent
            );
            assert_eq!(
                state.decide(&mut facts, goal(constant, effect))?,
                Decision::Proven
            );
        }
    }
    Ok(())
}
#[test]
fn malformed_type_identity_never_becomes_completed_absence() -> Result {
    let (mut facts, _, _, application) = facts()?;
    let mut state = State::body(&mut facts, None, 0, 1)?;
    assert!(
        state
            .decide(
                &mut facts,
                Predicate {
                    ty: TypeId(u32::MAX),
                    bound: Bound::ConstTrait(application)
                }
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn temporary_selection_contexts_forward_their_nested_work() -> Result {
    use crate::types::{
        arguments::Argument,
        parameters::clauses::{Clause, Quantified},
    };
    let text =
        crate::test_source::fixture("pa31/tests/spec/negative-premise-sides/nested-tuple-head.rs")?;
    let prefix = text.split("pub trait Seal").next().ok_or("source prefix")?;
    let text = format!(
        "{prefix}pub trait Tr {{ type Out: ?Sized; }}
         impl<T: ?Sized> Tr for T {{ type Out = T; }}
         pub struct P<T: ?Sized + Tr>(<T as Tr>::Out);"
    );
    let report = crate::test_source::source_provider(&text, "nested_selection_contexts")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let item = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "P")
        .ok_or("P")?;
    let item = ItemId(u32::try_from(item)?);
    let mut ty = TypeId::primitive(crate::types::Primitive::U8);
    for _ in 0..4 {
        let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
        ty = facts.types.intern(Kind::Adt(item, arguments))?;
    }
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_role(LangItem::MetaSized)?
        .ok_or("MetaSized")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    let predicate = Predicate {
        ty,
        bound: Bound::Trait(application),
    };
    for quantified in [false, true] {
        let mut state = State::body(&mut facts, None, 0, 1)?;
        selection::take_nested_observation();
        if quantified {
            let clause = Clause::ForAll(facts.types.intern_quantified(Quantified {
                bound_regions: 0,
                predicate,
            })?);
            assert!(state.prove_clause(&mut facts, clause, false)?);
        } else {
            assert_eq!(state.decide(&mut facts, predicate)?, Decision::Proven);
        }
        let (_, tally) = selection::take_nested_observation();
        assert!(tally > 0);
        assert_eq!(state.context.nested(), tally);
    }
    Ok(())
}
