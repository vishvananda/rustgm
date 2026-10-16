//! A unique validated header can determine inputs before numeric fallback.
use super::{Checker, Error, NodeId, Obligations, ScopeId, TermId};
use crate::{
    resolve::expansion::Report,
    sema::{
        model::Declarations,
        projections::selected_result,
        traits::{
            assumptions::Assumptions,
            selection::{self, Caller, Selection, Source},
        },
    },
    types::{
        TypeId,
        infer::{Constructor, Context, arguments::Argument, projections::Application},
        parameters::{environment::Environment, predicates::Predicate},
        substitute::Bound,
    },
};
use std::sync::Arc;

struct Opened {
    context: Context,
    application: Application,
    environment: Option<Arc<Environment>>,
    proof: Obligations,
}
struct Selected {
    predicate: Predicate,
    ty: TypeId,
}
pub(super) enum Progress {
    AwaitChange,
    AwaitResolution,
    Complete,
}
enum Attempt {
    Retry,
    Selected(Selected),
}

impl Checker<'_> {
    pub(super) fn projection_subject_known(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<bool, Error> {
        let subject = self
            .infer
            .child(&self.facts.types, term, 0)
            .map_err(|e| self.error(node, e))?;
        selection::has_known_subject(&self.facts.types, &mut self.infer, subject)
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn projection_environment(
        &mut self,
        node: NodeId,
    ) -> Result<Option<Arc<Environment>>, Error> {
        if let Some(environment) = self
            .projections
            .data
            .as_ref()
            .and_then(|d| d.environment.get())
        {
            return Ok(environment.clone());
        }
        let mut fold = Bound::new(self.call_types.universal_arguments());
        let environment = self
            .method_environment
            .as_ref()
            .map(|env| {
                if self.body_binders == 0 || env.has_only_parameters() {
                    Ok(Arc::clone(env))
                } else {
                    env.substitute_regions(&mut self.facts.types, &mut fold)
                        .map(Arc::new)
                }
            })
            .transpose()
            .map_err(|e| self.error(node, e))?;
        self.infer
            .account_work(fold.steps)
            .map_err(|e| self.error(node, e))?;
        self.projections
            .data
            .get_or_insert_with(Default::default)
            .environment
            .set(environment.clone())
            .map_err(|_| self.error(node, "body projection environment was initialized twice"))?;
        Ok(environment)
    }

    pub(super) fn select_projection(
        &mut self,
        node: NodeId,
        scope: ScopeId,
        term: TermId,
        proof: &mut Obligations,
    ) -> Result<Progress, Error> {
        let environment = self.projection_environment(node)?;
        let assumptions = caller_assumptions(self.facts, &mut self.infer, environment.as_ref())
            .map_err(|e| self.error(node, e))?;
        let caller = Caller {
            assumptions: &assumptions,
            environment: environment.as_ref(),
            first_region: proof.regions.end,
            retain_sized: false,
        };
        if selection::requires_resolved_inputs(self.facts, &mut self.infer, term, &caller)
            .map_err(|e| self.error(node, e))?
        {
            return Ok(Progress::AwaitResolution);
        }
        let counts = proof
            .scope_plan()
            .counts(scope)
            .map_err(|e| self.error(node, e))?;
        let depth = proof.scope_depth(scope).map_err(|e| self.error(node, e))?;
        let mut opened = Opened::new(
            self.facts,
            &mut self.infer,
            term,
            environment.as_ref(),
            depth,
            proof.regions.end,
        )
        .map_err(|e| self.error(node, e))?;
        let selected = opened.select(self.report, self.facts);
        self.infer
            .account_work(opened.context.steps)
            .map_err(|e| self.error(node, e))?;
        let selected = match selected.map_err(|e| self.error(node, e))? {
            Attempt::Selected(selected) => selected,
            Attempt::Retry => return Ok(Progress::AwaitChange),
        };
        let original = self
            .infer
            .projection_application(&self.facts.types, term)
            .map_err(|e| self.error(node, e))?;
        self.infer
            .equate_trait_predicate_inside(
                &self.facts.types,
                &original.predicate,
                selected.predicate,
                &counts,
            )
            .map_err(|e| self.error(node, e))?;
        proof
            .append_at(opened.proof, scope)
            .map_err(|e| self.error(node, e))?;
        let ty = self.normalize_body_type(node, proof, scope, selected.ty)?;
        self.infer
            .apply_normalization(&mut self.facts.types, term, ty)
            .map_err(|e| self.error(node, e))?;
        Ok(Progress::Complete)
    }
}

impl Opened {
    fn new(
        facts: &mut Declarations,
        source: &mut Context,
        term: TermId,
        environment: Option<&Arc<Environment>>,
        depth: u16,
        first: u32,
    ) -> Result<Self, String> {
        let mut context = Context::default();
        let copied = context
            .import(&facts.types, source, term)
            .map_err(inference)?;
        let mut fold = Bound::open_regions(first)?;
        let copied = context
            .map_regions(&mut facts.types, copied, &mut fold)
            .map_err(inference)?;
        let application = context
            .projection_application(&facts.types, copied)
            .map_err(inference)?;
        let mut shift = Bound::shift(depth);
        let before = fold.steps;
        let environment = environment
            .map(|env| {
                let shifted = env.substitute_regions(&mut facts.types, &mut shift)?;
                shifted
                    .substitute_regions(&mut facts.types, &mut fold)
                    .map(Arc::new)
            })
            .transpose()?;
        context
            .account_work(
                shift
                    .steps
                    .saturating_add(fold.steps.saturating_sub(before)),
            )
            .map_err(inference)?;
        let mut proof = Obligations::new(first);
        proof.regions.end = fold
            .next_region()
            .ok_or("missing projection region interval")?;
        proof.equalities.extend(
            fold.opened_regions()
                .map(|(original, free)| (free, original)),
        );
        proof
            .equalities
            .sort_unstable_by_key(|(free, original)| (free.0, original.0));
        Ok(Self {
            context,
            application,
            environment,
            proof,
        })
    }

    fn select(&mut self, report: &Report, facts: &mut Declarations) -> Result<Attempt, String> {
        if !self.application.arguments.is_empty()
            || self.pending_inputs(facts).map_err(inference)?
        {
            return Ok(Attempt::Retry);
        }
        let assumptions = caller_assumptions(facts, &mut self.context, self.environment.as_ref())?;
        let caller = Caller {
            assumptions: &assumptions,
            environment: self.environment.as_ref(),
            first_region: self.proof.regions.end,
            retain_sized: false,
        };
        let selected = selection::select(
            facts,
            &mut self.context,
            &self.application.predicate,
            caller,
        )?;
        let Selection::Proven(evidence) = selected else {
            return Ok(Attempt::Retry);
        };
        let source = evidence.source;
        if !matches!(source, Source::Implementation { .. }) {
            return Ok(Attempt::Retry);
        }
        self.proof.append(evidence)?;
        self.proof
            .equalities
            .append(&mut self.context.region_equalities);
        let predicate = self
            .context
            .resolved_predicate(&mut facts.types, &self.application.predicate)
            .map_err(inference)?;
        let mut restoration = self.proof.restoration()?;
        let predicate = restoration.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(restoration.steps)
            .map_err(inference)?;
        let ty = selected_result(report, facts, self.application.item, source, &self.proof)?;
        Ok(Attempt::Selected(Selected { predicate, ty }))
    }

    // Nested input projections need their own normalization before selection can
    // inspect headers. Fully determined inputs retain the canonical normalizer.
    fn pending_inputs(&mut self, facts: &Declarations) -> Result<bool, crate::types::infer::Error> {
        let mut roots = vec![self.application.predicate.subject()];
        let (_, arguments) = self
            .application
            .predicate
            .trait_arguments()
            .ok_or(crate::types::infer::Error::InvalidIdentity)?;
        roots.extend(arguments.iter().filter_map(|arg| match arg {
            Argument::Type(term) => Some(*term),
            Argument::Const(_) | Argument::Lifetime(_) => None,
        }));
        self.context.account_work(arguments.len())?;
        let inputs = self.context.construct(Constructor::Tuple, &roots)?;
        Ok(!self
            .context
            .normalization_roots(&facts.types, inputs, ScopeId::ROOT, |parent, _| Ok(parent))?
            .is_empty())
    }
}

fn caller_assumptions(
    facts: &Declarations,
    context: &mut Context,
    environment: Option<&Arc<Environment>>,
) -> Result<Assumptions, String> {
    let mut assumptions = Assumptions::default();
    for predicate in environment
        .into_iter()
        .flat_map(|env| env.trait_predicates())
    {
        assumptions
            .insert(&facts.types, *predicate)
            .map_err(inference)?;
        context.account_work(1).map_err(inference)?;
    }
    Ok(assumptions)
}

fn inference(error: crate::types::infer::Error) -> String {
    error.to_string()
}
