//! One header substitution shares all type/const variables and fresh lifetime captures.
#[cfg(test)]
#[path = "../../../../tests/invariants/elided_sugar_parameters.rs"]
mod region_sugar_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/fn_sugar_impl_parameters.rs"]
mod sugar_tests;
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
    /// Fresh regions after the captures, one per bound region of each
    /// determining instance of a quantified condition (see `Determined`).
    pub existentials: u32,
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
    let mut determined = Determined::new(predicate, end);
    let (quantified, lowered) = conditions(
        facts,
        (owner, effect),
        &mut fold,
        &mut determined,
        &mut predicates,
    )?;
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
    context
        .account_work(
            fold.steps
                .saturating_add(lowered)
                .saturating_add(determined.steps),
        )
        .map_err(inference)?;
    Ok(Instance {
        owner,
        goal,
        goals,
        quantified,
        bindings,
        regions,
        outlives,
        existentials: determined.next - end,
    })
}

/// The impl's declared conditions in the selected effect context. Ordinary ones
/// join `predicates`; quantified ones become the trial's scoped conditions, each
/// with its determining instance, if any, among `predicates`. Returns those
/// conditions and the steps that lowering empty binders took.
fn conditions(
    facts: &mut Declarations,
    (owner, effect): (DefId, Option<crate::support::constness::Constness>),
    fold: &mut substitute::Bound,
    determined: &mut Determined,
    predicates: &mut Vec<Predicate>,
) -> Result<(Option<Box<super::search::conditions::Conditions>>, usize), &'static str> {
    let mut quantified = None;
    let mut lowering = None;
    for predicate in
        crate::sema::declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), owner)?
    {
        let value = crate::sema::traits::effects::in_context(
            &mut facts.types,
            predicate.value,
            effect.unwrap_or(crate::support::constness::Constness::Ordinary),
        )?;
        match vacuous(&mut facts.types, &mut lowering, value)? {
            Clause::Predicate(predicate) => predicates.push(predicate),
            clause @ (Clause::ForAll(_) | Clause::UnstableFeature(_)) => {
                if let Some(instance) = determined.instance(&mut facts.types, clause)? {
                    predicates.push(instance);
                }
                let clause = fold.clause(&mut facts.types, clause)?;
                quantified
                    .get_or_insert_with(|| {
                        Box::new(super::search::conditions::Conditions::default())
                    })
                    .push(clause);
            }
        }
    }
    let lowered = lowering.as_ref().map_or(0, |lowering| lowering.steps);
    Ok((quantified, lowered))
}

/// Rustc proves a quantified projection condition of an impl by instantiating
/// its bound regions with placeholders and equating the projection's normal
/// form with the impl's own result. That infers an impl parameter which only
/// the result determines, such as `B` in core's `impl<B, I: Iterator, St, F>
/// Iterator for Scan<I, St, F> where F: FnMut(&mut St, I::Item) -> Option<B>`.
/// Here the universal condition is still proven, in its own scope, after the
/// parent resolves. Its instance at fresh regions is implied by it and is an
/// ordinary condition sharing the trial's inference, so the parameter resolves.
/// An instance is made only when the result names a type or const parameter
/// the header leaves undetermined. A parameter inferred to name a fresh region
/// cannot leave the trial, as rustc's placeholder cannot leave its universe.
struct Determined {
    header: Predicate,
    known: Option<crate::types::parameters::uses::Uses>,
    next: u32,
    steps: usize,
}
impl Determined {
    const fn new(header: Predicate, first: u32) -> Self {
        Self {
            header,
            known: None,
            next: first,
            steps: 0,
        }
    }

    fn instance(
        &mut self,
        types: &mut crate::types::Store,
        clause: Clause,
    ) -> Result<Option<Predicate>, &'static str> {
        use crate::types::parameters::uses::Uses;
        let Clause::ForAll(id) = clause else {
            return Ok(None);
        };
        let scoped = types
            .quantified(id)
            .ok_or("missing quantified impl clause")?;
        let Bound::ProjectionEqual(result) = scoped.predicate.bound else {
            return Ok(None);
        };
        if self.known.is_none() {
            let mut known = Uses::constraining();
            known.predicate(types, self.header)?;
            self.steps = self.steps.saturating_add(known.steps);
            self.known = Some(known);
        }
        let known = self.known.as_ref().ok_or("missing impl header uses")?;
        let mut uses = Uses::default();
        uses.collect(types, result)?;
        self.steps = self.steps.saturating_add(uses.steps);
        if uses.parameters.is_subset(&known.parameters)
            && uses.constants.is_subset(&known.constants)
        {
            return Ok(None);
        }
        let end = self
            .next
            .checked_add(scoped.bound_regions)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("selected impl region identity space exhausted")?;
        let mut opening = substitute::Bound::new((self.next..end).map(RegionId).collect());
        let predicate = opening.predicate(types, scoped.predicate)?;
        self.steps = self
            .steps
            .saturating_add(opening.steps)
            .saturating_add(scoped.bound_regions as usize);
        self.next = end;
        Ok(Some(predicate))
    }
}

/// Parenthesized callable sugar quantifies its clause even when the binder
/// binds no region. Lowered out of that empty binder, the body is an ordinary
/// impl condition: it shares the trial's inference, so an impl parameter that
/// only its `Output` equality determines resolves with the trial instead of
/// waiting for a resolved parent. One lowering serves every such clause.
fn vacuous(
    types: &mut crate::types::Store,
    lowering: &mut Option<substitute::Bound>,
    clause: Clause,
) -> Result<Clause, &'static str> {
    let Clause::ForAll(id) = clause else {
        return Ok(clause);
    };
    let scoped = types
        .quantified(id)
        .ok_or("missing quantified impl clause")?;
    if scoped.bound_regions != 0 {
        return Ok(clause);
    }
    lowering
        .get_or_insert_with(|| substitute::Bound::lower(1))
        .predicate(types, scoped.predicate)
        .map(Clause::Predicate)
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
    /// Captures and existential instance regions, allocated from the trial's first region.
    pub fn allocated(&self) -> Result<u32, &'static str> {
        u32::try_from(self.regions.len())
            .ok()
            .and_then(|captures| captures.checked_add(self.existentials))
            .ok_or("impl region count overflow")
    }

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
