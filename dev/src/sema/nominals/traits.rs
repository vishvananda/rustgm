//! Canonical declaration goals enter inference only after their outer binder becomes free.
mod normalization;
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
        parameters::{environment::Environment, predicates::Predicate},
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
    availability: Availability,
}
pub(crate) struct Projected {
    pub source: Option<selection::Source>,
    pub resolution: Option<substitute::Resolution>,
    pub proof: Obligations,
    pub sized: Vec<crate::types::TypeId>,
}
impl State {
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
        let assumptions = Self::assumptions(facts, environment.as_deref())?;
        Ok(Self {
            normalized: std::collections::HashMap::new(),
            assumptions,
            environment,
            context,
            fold,
            obligations,
            sized: Vec::new(),
            retain_sized: false,
            availability: Availability::Declarations,
        })
    }

    fn assumptions(
        facts: &Declarations,
        environment: Option<&Environment>,
    ) -> Result<Assumptions, &'static str> {
        let mut assumptions = Assumptions::default();
        for predicate in environment.iter().flat_map(|env| env.trait_predicates()) {
            assumptions
                .insert(&facts.types, *predicate)
                .map_err(|_| "invalid declaration trait assumption index")?;
        }
        Ok(assumptions)
    }

    /// Both normalizer callers retain earlier request proofs, including debts
    /// erased from the normalized inputs. Deferred alias inputs cannot use this
    /// route until their caller has retained the original formation requirement.
    pub(crate) fn projection(
        facts: &mut Declarations,
        environment: Option<Arc<Environment>>,
        request: substitute::Request,
        first: u32,
        retain_sized: bool,
        availability: Availability,
    ) -> Result<Projected, &'static str> {
        use crate::types::parameters::predicates::Bound;
        let application = request.application;
        let requirement = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        if let Some(source) = selection::opaque_normalized(facts, environment.as_deref(), request)
            .map_err(|_| "invalid or exhausted canonical projection premise")?
        {
            return Ok(Projected {
                source: Some(source),
                resolution: Some(substitute::Resolution::Opaque),
                proof: Obligations::new(first),
                sized: Vec::new(),
            });
        }
        if environment
            .as_ref()
            .is_none_or(|env| !env.requires_region_mapping())
            && !requirement.has_free_regions(&facts.types)?
        {
            let mut state = Self::new(facts, environment, None)?;
            state.obligations = Obligations::new(first);
            state.retain_sized = retain_sized;
            state.availability = availability;
            return state.project_opened(facts, application, requirement);
        }
        let mut fold = substitute::Bound::open_regions(first)?;
        let requirement = fold.predicate(&mut facts.types, requirement)?;
        let environment = environment
            .map(|env| {
                env.substitute_regions(&mut facts.types, &mut fold)
                    .map(Arc::new)
            })
            .transpose()?;
        let mut state = Self::new(facts, environment, None)?;
        state.retain_sized = retain_sized;
        state.availability = availability;
        state
            .context
            .account_work(fold.steps)
            .map_err(|_| "projection proof work exhausted")?;
        state.obligations.regions = first
            ..fold
                .next_region()
                .ok_or("missing projection region interval")?;
        let mut opened: Vec<_> = fold.opened_regions().map(|(a, b)| (b, a)).collect();
        opened.sort_unstable_by_key(|(free, original)| (free.0, original.0));
        state.obligations.equalities.extend(opened);
        state.project_opened(facts, application, requirement)
    }

    fn project_opened(
        mut self,
        facts: &mut Declarations,
        application: crate::types::projection::Application,
        requirement: Predicate,
    ) -> Result<Projected, &'static str> {
        use crate::sema::traits::equalities::{self, Selection};
        let mut source = self.select_opened(facts, requirement, false, true)?;
        let mut resolution = None;
        if matches!(
            source,
            Some(selection::Source::Assumption(_) | selection::Source::ResultBound(_))
        ) {
            if application.arguments != crate::types::GenericArgs::EMPTY {
                return Err("associated families require advanced generic relationships");
            }
            let mut substitution = Instantiation::new(&mut self.context, &facts.types, &[])
                .map_err(|_| "invalid equality assumption instantiation")?;
            let predicate = self
                .context
                .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
                .map_err(|_| "invalid equality assumption inputs")?;
            let inferred = crate::types::infer::projections::Application {
                item: application.item,
                predicate,
                arguments: Vec::new(),
            };
            resolution = Some(
                match equalities::for_projection(
                    facts,
                    &mut self.context,
                    &inferred,
                    self.environment.as_deref(),
                )
                .map_err(|_| "invalid or exhausted associated equality selection")?
                {
                    Selection::Absent => {
                        // A fully concrete application can use its independently
                        // proved impl even when a caller repeats the trait bound.
                        // Explicit generic or lifetime assumptions remain opaque.
                        if requirement.is_concrete(&facts.types)?
                            && let Some(selected @ selection::Source::Implementation { .. }) =
                                self.select_opened(facts, requirement, true, false)?
                        {
                            source = Some(selected);
                            substitute::Resolution::Type(
                                crate::sema::projections::implementation_result(
                                    facts,
                                    application.item,
                                    selected,
                                )?,
                            )
                        } else {
                            substitute::Resolution::Opaque
                        }
                    }
                    Selection::Ambiguous => {
                        return Err("ambiguous associated type equality assumptions");
                    }
                    Selection::Proven(ty) => substitute::Resolution::Type(ty),
                },
            );
        }
        let sized = std::mem::take(&mut self.sized);
        let proof = self.finish();
        if let Some(substitute::Resolution::Type(ty)) = &mut resolution {
            *ty = proof.restoration()?.apply(&mut facts.types, *ty)?;
        }
        Ok(Projected {
            source,
            resolution,
            proof,
            sized,
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
            retain_sized: self.retain_sized,
        };
        let selected = match (normalized_inputs, self.availability) {
            (true, availability) => {
                selection::select_normalized(facts, &mut self.context, &goal, caller, availability)
            }
            (false, Availability::Declarations) => {
                selection::select_declaration(facts, &mut self.context, &goal, caller)
            }
            (false, Availability::Validated) => {
                selection::select(facts, &mut self.context, &goal, caller)
            }
        };
        match selected.map_err(|_| "invalid or exhausted canonical trait selection")? {
            selection::Selection::Proven(mut evidence) => {
                let source = evidence.source;
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
