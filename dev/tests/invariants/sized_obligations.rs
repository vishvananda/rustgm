//! Actual Sized applications retain their trait identity and caller/instance debts.
use super::{Result, SOURCE, resolve};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{self, model::Declarations, nominals::Proof},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
use std::sync::Arc;

fn fixture() -> Result<(Report, Declarations)> {
    let report = resolve(&(SOURCE.to_owned() + "fn relaxed<T:?Sized>(){}fn unused<T>(){}"))?;
    let facts = sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn parameter(report: &Report, facts: &Declarations, name: &str) -> Result<(DefId, TypeId)> {
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or("actual function")?;
    let Argument::Type(ty) = facts
        .generics
        .parameters
        .get(&function.definition)
        .and_then(|parameters| parameters.first())
        .ok_or("actual formal")?
        .argument
    else {
        return Err("formal is not a type".into());
    };
    Ok((function.definition, ty))
}
fn goal(facts: &mut Declarations, ty: TypeId) -> Result<Predicate> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .validated_role(LangItem::Sized)?
        .ok_or("actual Sized")?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(application),
    })
}
fn prove(
    facts: &mut Declarations,
    goal: Predicate,
    environment: Option<Arc<Environment>>,
) -> Result<bool> {
    Ok(Proof::for_declaration(environment, 0, 1).prove(facts, goal)?)
}

#[test]
fn actual_sized_trait_goal_requires_concrete_sized_formation() -> Result {
    let (_, mut facts) = fixture()?;
    let u8_ = TypeId::primitive(Primitive::U8);
    let slice = facts.types.intern(Kind::Slice(u8_))?;
    for (ty, expected) in [(u8_, true), (TypeId::UNIT, true), (slice, false)] {
        let goal = goal(&mut facts, ty)?;
        assert!(sema::compiler_items::sizing_predicate(&facts, goal)?);
        assert_eq!(prove(&mut facts, goal, None)?, expected);
        let Bound::Trait(application) = goal.bound else {
            return Err("lost actual trait goal".into());
        };
        let nonordinary = Predicate {
            bound: Bound::ConstTrait(application),
            ..goal
        };
        assert!(!matches!(prove(&mut facts, nonordinary, None), Ok(true)));
    }
    Ok(())
}

#[test]
fn actual_sized_caller_premise_does_not_escape_to_opposed_environment() -> Result {
    let (report, mut facts) = fixture()?;
    let (owner, ty) = parameter(&report, &facts, "relaxed")?;
    let original = Arc::clone(
        facts
            .generics
            .environments
            .get(&owner)
            .ok_or("caller environment")?,
    );
    let goal = goal(&mut facts, ty)?;
    let paid = Arc::new(
        original
            .as_ref()
            .clone()
            .with_clauses(&facts.types, &[goal.into()])?,
    );
    assert!(!prove(&mut facts, goal, Some(Arc::clone(&original)))?);
    assert!(prove(&mut facts, goal, Some(Arc::clone(&paid)))?);
    assert!(!prove(&mut facts, goal, Some(original))?);
    assert!(prove(&mut facts, goal, Some(paid))?);
    Ok(())
}

#[test]
fn unused_actual_sized_formal_is_reproved_after_concrete_substitution() -> Result {
    let (report, mut facts) = fixture()?;
    let (owner, ty) = parameter(&report, &facts, "unused")?;
    let function = facts
        .function_header(owner)?
        .ok_or("actual function header")?;
    assert_eq!(
        facts.types.contains_parameters(function.signature),
        Some(false)
    );
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("actual rigid formal".into());
    };
    let requirement = goal(&mut facts, ty)?;
    let u8_ = TypeId::primitive(Primitive::U8);
    let slice = facts.types.intern(Kind::Slice(u8_))?;
    for (actual, expected) in [(u8_, true), (slice, false)] {
        let mut fold = crate::types::substitute::Bound::parameters(
            &facts.types,
            &[(id, Argument::Type(actual))],
        )?;
        let specialized = fold.predicate(&mut facts.types, requirement)?;
        assert_eq!(specialized.ty, actual);
        assert_eq!(specialized.bound, requirement.bound);
        assert_eq!(prove(&mut facts, specialized, None)?, expected);
    }
    Ok(())
}

#[test]
fn actual_sized_unresolved_subject_defers_without_committing_inference() -> Result {
    use crate::{
        sema::traits::{
            assumptions::Assumptions,
            selection::{self, Caller, Selection},
        },
        types::infer::{Context, Domain, Instantiation, arguments::Argument as Inferred},
    };
    let (report, mut facts) = fixture()?;
    let (_, ty) = parameter(&report, &facts, "relaxed")?;
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("actual formal".into());
    };
    let requirement = goal(&mut facts, ty)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut mapping =
        Instantiation::new(&mut context, &facts.types, &[(id, Inferred::Type(unknown))])?;
    let goal = context.instantiate_predicate(&mut facts.types, requirement, &mut mapping)?;
    let assumptions = Assumptions::default();
    let selected = selection::select(
        &mut facts,
        &mut context,
        &goal,
        Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 1,
            retain_sized: true,
        },
    )?;
    assert!(matches!(selected, Selection::Deferred));
    assert!(matches!(
        context.resolved(&mut facts.types, unknown),
        Err(crate::types::infer::Error::Unresolved)
    ));
    Ok(())
}
