//! One header substitution shares all type/const variables and fresh lifetime captures.
use super::{Context, Declarations, DefId, Goal, RegionId, inference};
use crate::types::{
    arguments::Argument as Canonical,
    infer::{Instantiation, arguments::Argument},
    outlives::Pair,
    parameters::{
        ParameterId, ParameterKind,
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    substitute,
};

pub(super) struct Instance {
    pub owner: DefId,
    pub goal: Goal,
    pub goals: Vec<Goal>,
    pub quantified: Option<Box<super::search::conditions::Conditions>>,
    pub bindings: Vec<(ParameterId, Argument)>,
    pub regions: Vec<RegionId>,
    pub outlives: Vec<Pair>,
}

pub(super) fn instantiate(
    facts: &mut Declarations,
    context: &mut Context,
    owner: DefId,
    first: u32,
    effect: Option<crate::support::constness::Constness>,
) -> Result<Instance, String> {
    let predicate = capability(facts, owner, effect)?;
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("missing selected impl schema")?;
    let end = first
        .checked_add(schema.captures)
        .filter(|end| *end < 0x8000_0000)
        .ok_or("selected impl region identity space exhausted")?;
    if first == 0 {
        return Err("selected impl requires a free region allocator".into());
    }
    context
        .account_work(schema.captures as usize)
        .map_err(inference)?;
    let regions: Vec<_> = (first..end).map(RegionId).collect();
    let mut fold = substitute::Bound::parameters_captured(&facts.types, &[], regions.clone())?;
    let mut outlives = Vec::with_capacity(schema.bounds.len());
    for &(a, b) in &schema.bounds {
        let (Canonical::Lifetime(a), Canonical::Lifetime(b)) = (
            fold.apply_argument(&mut facts.types, Canonical::Lifetime(a))?,
            fold.apply_argument(&mut facts.types, Canonical::Lifetime(b))?,
        ) else {
            return Err("invalid impl region bound".into());
        };
        outlives.push((a, b));
    }
    let ids = facts.generics.parameter_ids(&facts.types, owner)?;
    let mut bindings = Vec::with_capacity(ids.len());
    let mut predicates = Vec::new();
    for id in ids {
        bindings.push((
            id,
            context
                .parameter_variable(&facts.types, id)
                .map_err(inference)?,
        ));
        if matches!(
            facts.types.parameter(id).map(|p| p.kind),
            Some(ParameterKind::Type { sized: true })
        ) {
            let ty = facts.types.intern(crate::types::Kind::Parameter(id))?;
            predicates.push(Predicate {
                ty,
                bound: Bound::Sized,
            });
        }
    }
    let mut quantified = None;
    for predicate in
        crate::sema::declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), owner)?
    {
        let value = crate::sema::traits::effects::in_context(
            &mut facts.types,
            predicate.value,
            effect.unwrap_or(crate::support::constness::Constness::Ordinary),
        )?;
        match value {
            Clause::Predicate(predicate) => predicates.push(predicate),
            clause @ (Clause::ForAll(_) | Clause::UnstableFeature(_)) => {
                let clause = fold.clause(&mut facts.types, clause)?;
                quantified
                    .get_or_insert_with(|| {
                        Box::new(super::search::conditions::Conditions::default())
                    })
                    .push(clause);
            }
        }
    }
    let mut substitution =
        Instantiation::new(context, &facts.types, &bindings).map_err(inference)?;
    let predicate = fold.predicate(&mut facts.types, predicate)?;
    let goal = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(inference)?;
    let mut goals = Vec::with_capacity(predicates.len());
    for predicate in predicates {
        let predicate = fold.predicate(&mut facts.types, predicate)?;
        goals.push(
            context
                .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
                .map_err(inference)?,
        );
    }
    context.account_work(fold.steps).map_err(inference)?;
    Ok(Instance {
        owner,
        goal,
        goals,
        quantified,
        bindings,
        regions,
        outlives,
    })
}

fn capability(
    facts: &Declarations,
    owner: DefId,
    effect: Option<crate::support::constness::Constness>,
) -> Result<Predicate, String> {
    let registry = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing selected impl registry")?;
    registry.polarity(&facts.types, &facts.associated, owner)?;
    let mut predicate = registry
        .header(owner)
        .ok_or("missing selected impl header")?
        .predicate;
    // A checked actual const impl is a capability, not a weaker stored header.
    // Only this candidate view adopts the requested use effect. Ordinary impls
    // retain Ordinary and cannot match a Const or Conditional goal.
    if let Some((application, crate::support::constness::Constness::Const)) =
        predicate.bound.trait_parts()
        && let Some(effect) = effect
    {
        predicate.bound = Bound::trait_application(application, effect);
    }
    Ok(predicate)
}

impl Instance {
    pub fn arguments(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
    ) -> Result<crate::types::GenericArgs, crate::types::infer::Error> {
        let mut arguments = self
            .regions
            .iter()
            .copied()
            .map(Canonical::Lifetime)
            .collect::<Vec<_>>();
        for &(_, argument) in &self.bindings {
            arguments.push(match argument {
                Argument::Type(term) => Canonical::Type(context.resolved(&mut facts.types, term)?),
                Argument::Const(term) => {
                    Canonical::Const(context.resolved_const(&mut facts.types, term)?)
                }
                Argument::Lifetime(_) => return Err(crate::types::infer::Error::InvalidIdentity),
            });
        }
        facts
            .types
            .intern_arguments(&arguments)
            .map_err(|_| crate::types::infer::Error::InvalidIdentity)
    }
}
