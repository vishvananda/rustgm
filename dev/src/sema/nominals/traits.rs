//! Canonical declaration goals enter inference only after their outer binder becomes free.
pub(in crate::sema) mod normal_form;
mod normalization;
mod quantified;
pub(in crate::sema) use quantified::{Goal as QuantifiedGoal, Queries as QuantifiedQueries};
mod shape_normalization;
mod term_normalization;
use crate::{
    sema::{
        model::Declarations,
        traits::{
            assumptions::Assumptions, implementations::Availability, obligations::Obligations,
            selection,
        },
    },
    types::{
        RegionId,
        infer::{Context, Instantiation},
        parameters::{clauses::Clause, environment::Environment, predicates::Predicate},
        substitute,
    },
};
use std::sync::Arc;

pub(crate) struct State {
    normalized: std::collections::HashMap<crate::types::TypeId, crate::types::TypeId>,
    assumptions: Assumptions,
    environment: Option<Arc<Environment>>,
    context: Context,
    fold: Option<substitute::Bound>,
    obligations: Obligations,
    sized: Vec<crate::types::TypeId>,
    retain_sized: bool,
    provisional_size: bool,
    availability: Availability,
    quantified: Option<Box<quantified::Queries>>,
    terms: Option<Box<term_normalization::Terms>>,
    shapes: Option<Box<shape_normalization::Sizes>>,
}
pub(crate) struct Projected {
    pub provisional_size: bool,
    pub source: Option<selection::Source>,
    pub resolution: Option<substitute::Resolution>,
    pub proof: Obligations,
    pub sized: Vec<crate::types::TypeId>,
    /// Member inputs in the retained selection proof's lifetime namespace.
    pub family: crate::types::GenericArgs,
}
impl State {
    pub(super) fn append_proof(&mut self, proof: Obligations) -> Result<(), &'static str> {
        self.obligations.extend(proof)
    }
    pub(super) const fn region_frontier(&self) -> u32 {
        self.obligations.regions.end
    }
    #[cfg(test)]
    pub(crate) const fn selection_steps(&self) -> usize {
        self.context.steps
    }

    pub(crate) fn instance(
        facts: &mut Declarations,
        universals: u32,
    ) -> Result<Self, &'static str> {
        let mut state = Self::new(facts, None, Some((universals, 1)))?;
        state.availability = Availability::Validated;
        Ok(state)
    }

    pub fn new(
        facts: &mut Declarations,
        environment: Option<Arc<Environment>>,
        scope: Option<(u32, u32)>,
    ) -> Result<Self, &'static str> {
        let mut obligations = Obligations::new(scope.map_or(1, |(_, first)| first));
        let mut context = Context::default();
        let mut fold = None;
        let mut environment = environment;
        if let Some((count, first)) = scope {
            let end = first
                .checked_add(count)
                .filter(|end| *end < 0x8000_0000)
                .ok_or("declaration region identity space exhausted")?;
            if first == 0 {
                return Err("declaration proof requires a free region allocator");
            }
            let regions: Vec<_> = (first..end).map(RegionId).collect();
            for (slot, &region) in regions.iter().enumerate() {
                let slot = u16::try_from(slot).map_err(|_| "too many declaration lifetimes")?;
                obligations
                    .equalities
                    .push((region, RegionId::bound(0, slot)?));
            }
            obligations.regions.end = end;
            let mut substitution = substitute::Bound::new(regions);
            if let Some(env) = environment {
                environment = Some(Arc::new(
                    env.substitute_regions(&mut facts.types, &mut substitution)?,
                ));
            }
            context
                .account_work(substitution.steps)
                .map_err(|_| "declaration proof work exhausted")?;
            fold = Some(substitution);
        }
        let (assumptions, construction_work) =
            Assumptions::from_environment(&facts.types, environment.as_deref())?;
        context
            .account_work(construction_work)
            .map_err(|_| "declaration assumption work exhausted")?;
        Ok(Self {
            normalized: std::collections::HashMap::new(),
            assumptions,
            environment,
            context,
            fold,
            obligations,
            sized: Vec::new(),
            retain_sized: false,
            provisional_size: false,
            availability: Availability::Declarations,
            quantified: None,
            terms: None,
            shapes: None,
        })
    }

    pub fn prove(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
        closed: bool,
    ) -> Result<bool, &'static str> {
        self.prove_clause(facts, Clause::Predicate(requirement), closed)
    }

    /// Quantified selection retains a universal proof scope; success still needs
    /// the caller's final directed lifetime proof, just like an ordinary premise.
    pub fn prove_clause(
        &mut self,
        facts: &mut Declarations,
        requirement: Clause,
        closed: bool,
    ) -> Result<bool, &'static str> {
        let requirement = match requirement {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => return self.prove_quantified(facts, id, closed),
        };
        let Some(fold) = &mut self.fold else {
            return self
                .assumptions
                .prove_canonical(&mut facts.types, &mut self.context, requirement)
                .map(|evidence| evidence.is_some())
                .map_err(|_| "invalid or exhausted nominal trait proof");
        };
        let before = fold.steps;
        let requirement = fold.predicate(&mut facts.types, requirement)?;
        self.context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(|_| "declaration proof work exhausted")?;
        self.select_opened(facts, requirement, closed, false)
            .map(|source| source.is_some())
    }

    pub(super) fn prove_equality(
        &mut self,
        facts: &mut Declarations,
        mut predicate: Predicate,
        closed: bool,
    ) -> Result<bool, &'static str> {
        use crate::types::{lifetime_proof::ScopeId, parameters::predicates::Bound};
        let Some(fold) = &mut self.fold else {
            return Err("associated equality requires an explicit declaration proof scope");
        };
        let before = fold.steps;
        predicate = fold.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(|_| "equality proof work exhausted")?;
        let Bound::ProjectionEqual(result) = predicate.bound else {
            return Err("expected an associated equality obligation");
        };
        let environment = if closed {
            None
        } else {
            self.environment.as_ref()
        };
        let mut normalized = [predicate.ty, result];
        for ty in &mut normalized {
            let (value, steps) = crate::sema::projections::normalize(
                facts,
                environment,
                &mut self.obligations,
                ScopeId::ROOT,
                *ty,
                self.availability,
            )?;
            *ty = value;
            self.context
                .account_work(steps)
                .map_err(|_| "equality normalization work exhausted")?;
        }
        match self
            .context
            .equate_normal_forms(&facts.types, normalized[0], normalized[1])
        {
            Ok(()) => Ok(true),
            Err(
                crate::types::infer::Error::Mismatch | crate::types::infer::Error::Normalization,
            ) => Ok(false),
            Err(_) => Err("invalid or exhausted associated equality proof"),
        }
    }

    fn select_opened(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
        closed: bool,
        normalized_inputs: bool,
    ) -> Result<Option<selection::Source>, &'static str> {
        let empty = Assumptions::default();
        let caller = selection::Caller {
            assumptions: if closed { &empty } else { &self.assumptions },
            environment: if closed {
                None
            } else {
                self.environment.as_ref()
            },
            first_region: self.obligations.regions.end,
            retain_sized: self.retain_sized,
        };
        let selected = select_requirement(
            facts,
            &mut self.context,
            requirement,
            caller,
            self.availability,
            normalized_inputs,
        )?;
        self.accept_selection(selected)
    }

    fn accept_selection(
        &mut self,
        selected: selection::Selection,
    ) -> Result<Option<selection::Source>, &'static str> {
        match selected {
            selection::Selection::Proven(mut evidence) => {
                let source = evidence.source;
                self.provisional_size |= evidence.provisional_size;
                self.sized.append(&mut evidence.sized);
                self.obligations.append(evidence)?;
                Ok(Some(source))
            }
            selection::Selection::Absent | selection::Selection::Deferred => Ok(None),
        }
    }

    pub fn finish(mut self) -> Obligations {
        self.obligations
            .equalities
            .append(&mut self.context.region_equalities);
        self.obligations
    }
}

fn select_requirement(
    facts: &mut Declarations,
    context: &mut Context,
    requirement: Predicate,
    caller: selection::Caller<'_>,
    availability: Availability,
    normalized_inputs: bool,
) -> Result<selection::Selection, &'static str> {
    let mut substitution = Instantiation::new(context, &facts.types, &[])
        .map_err(|_| "invalid declaration trait instantiation")?;
    let goal = context
        .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
        .map_err(|_| "invalid declaration trait goal")?;
    let selected = match (normalized_inputs, availability) {
        (true, availability) => {
            selection::select_normalized(facts, context, &goal, caller, availability)
        }
        (false, Availability::Declarations) => {
            selection::select_declaration(facts, context, &goal, caller)
        }
        (false, Availability::Validated) => selection::select(facts, context, &goal, caller),
    };
    selected.map_err(|_| "invalid or exhausted canonical trait selection")
}
