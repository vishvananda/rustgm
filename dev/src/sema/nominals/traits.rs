//! Canonical declaration goals enter inference only after their outer binder becomes free.
use crate::{
    sema::{
        model::Declarations,
        traits::{assumptions::Assumptions, obligations::Obligations, selection},
    },
    types::{
        RegionId,
        infer::{Context, Instantiation},
        parameters::{environment::Environment, predicates::Predicate},
        substitute,
    },
};
use std::sync::Arc;

pub(super) struct State {
    assumptions: Assumptions,
    environment: Option<Arc<Environment>>,
    context: Context,
    fold: Option<substitute::Bound>,
    obligations: Obligations,
}
impl State {
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
        let mut assumptions = Assumptions::default();
        for predicate in environment.iter().flat_map(|env| env.trait_predicates()) {
            assumptions
                .insert(&facts.types, *predicate)
                .map_err(|_| "invalid declaration trait assumption index")?;
        }
        Ok(Self {
            assumptions,
            environment,
            context,
            fold,
            obligations,
        })
    }

    pub fn prove(
        &mut self,
        facts: &mut Declarations,
        requirement: Predicate,
        closed: bool,
    ) -> Result<bool, &'static str> {
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
        let mut substitution = Instantiation::new(&mut self.context, &facts.types, &[])
            .map_err(|_| "invalid declaration trait instantiation")?;
        let goal = self
            .context
            .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
            .map_err(|_| "invalid declaration trait goal")?;
        let empty = Assumptions::default();
        let caller = selection::Caller {
            assumptions: if closed { &empty } else { &self.assumptions },
            environment: if closed {
                None
            } else {
                self.environment.as_ref()
            },
            first_region: self.obligations.regions.end,
        };
        match selection::select_declaration(facts, &mut self.context, &goal, caller)
            .map_err(|_| "invalid or exhausted declaration trait selection")?
        {
            selection::Selection::Proven(evidence) => {
                self.obligations.append(evidence)?;
                Ok(true)
            }
            selection::Selection::Absent | selection::Selection::Deferred => Ok(false),
        }
    }

    pub fn finish(mut self) -> Obligations {
        self.obligations
            .equalities
            .append(&mut self.context.region_equalities);
        self.obligations
    }
}
