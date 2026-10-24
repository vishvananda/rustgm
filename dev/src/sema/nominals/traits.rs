//! Canonical declaration goals enter inference only after their outer binder becomes free.
pub(in crate::sema) mod normal_form;
mod normalization;
mod quantified;
pub(in crate::sema) use quantified::{Goal as QuantifiedGoal, Queries as QuantifiedQueries};
mod opening;
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
use opening::Opening;
use std::sync::Arc;

pub(crate) struct State {
    reusable: [Option<Box<selection::Reuse>>; 2],
    normalized: crate::support::hash::HashMap<crate::types::TypeId, crate::types::TypeId>,
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
    pub(crate) const fn selection_steps(&self) -> usize {
        self.context.steps
    }

    pub(crate) fn instance(
        facts: &mut Declarations,
        universals: u32,
    ) -> Result<Self, &'static str> {
        Self::instance_at(facts, universals, 1)
    }

    /// Keep an independent validated selection after the actual caller's last
    /// allocation. Its returned proof still owns all new IDs and every debt.
    pub(crate) fn instance_at(
        facts: &mut Declarations,
        universals: u32,
        first: u32,
    ) -> Result<Self, &'static str> {
        let mut state = Self::new(facts, None, Some((universals, first)))?;
        state.availability = Availability::Validated;
        Ok(state)
    }

    pub(crate) fn body(
        facts: &mut Declarations,
        environment: Option<Arc<Environment>>,
        universals: u32,
        first: u32,
    ) -> Result<Self, &'static str> {
        let mut state = Self::scoped(facts, environment, Some((universals, first)), Opening::Body)?;
        state.availability = Availability::Validated;
        Ok(state)
    }

    /// A completed absence may classify a move; pending proof cannot do so.
    pub(crate) fn copy_decision(
        &mut self,
        facts: &mut Declarations,
        predicate: Predicate,
    ) -> Result<bool, &'static str> {
        #[cfg(test)]
        let original = predicate;
        let fold = self
            .fold
            .as_mut()
            .ok_or("source Copy requires a scoped proof")?;
        let before = fold.steps;
        let predicate = fold.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(|_| "source Copy proof work exhausted")?;
        let caller = selection::Caller {
            assumptions: &self.assumptions,
            environment: self.environment.as_ref(),
            first_region: self.obligations.regions.end,
            retain_sized: false,
        };
        #[cfg(test)]
        selection::copy_trace::record_checked(|| {
            let clauses = self
                .environment
                .as_ref()
                .map(|environment| environment.trait_clauses().take(65).collect::<Vec<_>>());
            if clauses.as_ref().is_some_and(|clauses| clauses.len() > 64) {
                return None;
            }
            Some(format!(
                "copy-owner original={original:?} folded={predicate:?} environment={:?} validated={} frontier={} regions={:?} steps={} clauses={clauses:?}",
                self.environment.as_ref().map(Arc::as_ptr),
                self.availability == Availability::Validated,
                caller.first_region,
                self.obligations.regions,
                self.context.steps
            ))
        });
        let selected = select_requirement(
            facts,
            &mut self.context,
            predicate,
            caller,
            Availability::Validated,
            false,
            None,
        )?;
        #[cfg(test)]
        selection::copy_trace::record(|| {
            format!(
                "copy-classification goal={predicate:?} subject={:?} deferred={} absent={}",
                facts.types.kind(predicate.ty),
                matches!(selected, selection::Selection::Deferred),
                matches!(selected, selection::Selection::Absent)
            )
        });
        match selected {
            selection::Selection::Deferred => Err("source Copy selection is still pending"),
            selected => self
                .accept_selection(selected)
                .map(|source| source.is_some()),
        }
    }

    pub fn new(
        facts: &mut Declarations,
        environment: Option<Arc<Environment>>,
        scope: Option<(u32, u32)>,
    ) -> Result<Self, &'static str> {
        Self::scoped(facts, environment, scope, Opening::Declaration)
    }

    fn scoped(
        facts: &mut Declarations,
        environment: Option<Arc<Environment>>,
        scope: Option<(u32, u32)>,
        opening: Opening,
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
            let mut substitution = opening.fold(
                environment.as_ref(),
                regions,
                &mut context,
                &mut obligations,
            )?;
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
            reusable: [None, None],
            normalized: crate::support::hash::HashMap::default(),
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
        let reuse = &mut self.reusable[usize::from(closed)];
        if reuse.is_none()
            && facts.types.nominal_sizes_sealed()
            && selection::Reuse::needed(facts, requirement)?
        {
            *reuse = Some(Box::default());
        }
        let selected = select_requirement(
            facts,
            &mut self.context,
            requirement,
            caller,
            self.availability,
            normalized_inputs,
            reuse.as_deref_mut(),
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
            selection::Selection::Refuted(refutation) => {
                refutation.reject()?;
                Ok(None)
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
    reuse: Option<&mut selection::Reuse>,
) -> Result<selection::Selection, &'static str> {
    if let Some(selected) = reuse
        .as_ref()
        .map(|reuse| reuse.selection(facts, requirement, caller.first_region))
        .transpose()
        .map_err(|_| "completed declaration proof has invalid source provenance")?
        .flatten()
    {
        context
            .account_work(1)
            .map_err(|_| "completed declaration proof work exhausted")?;
        return Ok(selected);
    }
    let mut substitution = Instantiation::new(context, &facts.types, &[])
        .map_err(|_| "invalid declaration trait instantiation")?;
    let goal = context
        .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
        .map_err(|_| "invalid declaration trait goal")?;
    let selected = if let Some(reuse) = reuse {
        selection::select_reusing(
            facts,
            context,
            &goal,
            caller,
            (availability, normalized_inputs),
            reuse,
        )
    } else {
        match (normalized_inputs, availability) {
            (true, availability) => {
                selection::select_normalized(facts, context, &goal, caller, availability)
            }
            (false, Availability::Declarations) => {
                selection::select_declaration(facts, context, &goal, caller)
            }
            (false, Availability::Validated) => selection::select(facts, context, &goal, caller),
        }
    };
    selected.map_err(|_| "invalid or exhausted canonical trait selection")
}
