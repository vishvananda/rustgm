//! A unique validated header can determine inputs before numeric fallback.
use super::{Checker, Error, NodeId, Obligations, ScopeId, TermId};
use crate::{
    sema::{
        model::Declarations,
        projections::Definition,
        traits::{
            assumptions::Assumptions,
            selection::{self, Caller, Selection, Source},
        },
    },
    types::{
        GenericArgs, TypeId,
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
    result: ResultType,
}
pub(super) enum ResultType {
    Type(TypeId),
    Definition(Box<Pending>),
}
pub(super) struct Pending {
    pub definition: Definition,
    restoration: Bound,
}
impl Pending {
    pub fn result(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<Option<(TypeId, usize)>, &'static str> {
        self.definition
            .result(facts)?
            .map(|ty| {
                let before = self.restoration.steps;
                let ty = self.restoration.apply(&mut facts.types, ty)?;
                Ok((ty, self.restoration.steps.saturating_sub(before)))
            })
            .transpose()
    }
}
pub(super) enum Progress {
    AwaitChange,
    InputsRefined,
    AwaitInputs,
    AwaitResolution,
    Complete,
    Normalize(ResultType),
    Symbolic(TermId),
}
enum Attempt {
    Retry(Option<FamilyInputs>),
    NestedInputs,
    Selected(Selected),
}

// Parent selection can refine inputs before the member's own inputs are known.
// Keep the original normalization query pending; only it proves family formation.
struct FamilyInputs {
    predicate: Predicate,
    arguments: Option<GenericArgs>,
}
impl FamilyInputs {
    fn new(
        facts: &mut Declarations,
        context: &mut Context,
        predicate: Predicate,
        own: &[Argument],
        mut restoration: Bound,
    ) -> Result<Self, String> {
        let arguments = match context.resolved_arguments(&mut facts.types, own) {
            Ok(arguments) => Some(restoration.apply_arguments(&mut facts.types, arguments)?),
            Err(crate::types::infer::Error::Unresolved) => None,
            Err(error) => return Err(inference(error)),
        };
        context.account_work(restoration.steps).map_err(inference)?;
        Ok(Self {
            predicate,
            arguments,
        })
    }

    fn commit(
        self,
        facts: &mut Declarations,
        context: &mut Context,
        root: TermId,
        ambient: &[u32],
    ) -> Result<bool, crate::types::infer::Error> {
        use crate::types::{infer::Error, projection};
        let original = context.projection_application(&facts.types, root)?;
        let known = if self.arguments.is_some() {
            context.resolved(&mut facts.types, root).map(|_| ())
        } else {
            context
                .resolved_predicate(&mut facts.types, &original.predicate)
                .map(|_| ())
        };
        match known {
            Ok(()) => return Ok(false),
            Err(Error::Unresolved) => {}
            Err(error) => return Err(error),
        }
        if let Some(arguments) = self.arguments {
            let Some((trait_application, _)) = self.predicate.bound.trait_parts() else {
                return Err(Error::InvalidIdentity);
            };
            context.equate_projection_inputs_inside(
                &mut facts.types,
                &original,
                projection::Application {
                    item: original.item,
                    subject: self.predicate.ty,
                    trait_application,
                    arguments,
                },
                ambient,
            )?;
        } else {
            context.equate_trait_predicate_inside(
                &mut facts.types,
                &original.predicate,
                self.predicate,
                ambient,
            )?;
        }
        Ok(true)
    }
}

impl Checker<'_> {
    /// A retained written type keeps body-bound slots; inferred call inputs use
    /// free aliases. Select one matching caller namespace for this query only.
    pub(super) fn resolved_projection_environment(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<Option<Arc<Environment>>, Error> {
        if self.facts.types.escaping_binders(ty) == Some(0) {
            self.projection_environment(node)
        } else {
            Ok(self
                .state
                .environment_owner
                .and_then(|owner| self.facts.generics.environments.get(&owner))
                .cloned())
        }
    }
    pub(super) fn projection_subject_known(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<bool, Error> {
        let subject = self
            .state
            .infer
            .child(&self.facts.types, term, 0)
            .map_err(|e| self.error(node, e))?;
        selection::has_known_subject(&self.facts.types, &mut self.state.infer, subject)
            .map_err(|e| self.error(node, e))
    }

    pub(in crate::sema::bodies) fn projection_environment(
        &mut self,
        node: NodeId,
    ) -> Result<Option<Arc<Environment>>, Error> {
        if let Some(environment) = self
            .state
            .projections
            .data
            .as_ref()
            .and_then(|d| d.environment.get())
        {
            return Ok(environment.clone());
        }
        let mut fold = Bound::new(self.state.call_types.universal_arguments());
        let original = self
            .state
            .projections
            .data
            .as_ref()
            .and_then(|data| data.source_environment.as_ref())
            .or(self.state.method_environment.as_ref());
        let environment = original
            .map(|env| {
                if self.state.body_binders == 0 || env.has_only_parameters() {
                    Ok(Arc::clone(env))
                } else {
                    env.substitute_regions(&mut self.facts.types, &mut fold)
                        .map(Arc::new)
                }
            })
            .transpose()
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(fold.steps)
            .map_err(|e| self.error(node, e))?;
        self.state
            .projections
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
        expose: bool,
        pending_parent: bool,
    ) -> Result<Progress, Error> {
        let environment = self.projection_environment(node)?;
        let assumptions =
            caller_assumptions(self.facts, &mut self.state.infer, environment.as_ref())
                .map_err(|e| self.error(node, e))?;
        let caller = Caller {
            assumptions: &assumptions,
            environment: environment.as_ref(),
            first_region: proof.regions.end,
            retain_sized: false,
        };
        if pending_parent
            || selection::requires_resolved_inputs(self.facts, &mut self.state.infer, term, &caller)
                .map_err(|e| self.error(node, e))?
        {
            if expose
                && let Some(result) = self.symbolic_projection(node, scope, term, proof, &caller)?
            {
                return Ok(Progress::Symbolic(result));
            }
            return Ok(if pending_parent {
                Progress::AwaitChange
            } else {
                Progress::AwaitResolution
            });
        }
        let counts = proof
            .scope_plan()
            .counts(scope)
            .map_err(|e| self.error(node, e))?;
        let depth = proof.scope_depth(scope).map_err(|e| self.error(node, e))?;
        let mut opened = Opened::new(
            self.facts,
            &mut self.state.infer,
            term,
            environment.as_ref(),
            depth,
            proof.regions.end,
        )
        .map_err(|e| self.error(node, e))?;
        let selected = opened.select(self.facts, self.availability);
        self.state
            .infer
            .account_work(opened.context.steps)
            .map_err(|e| self.error(node, e))?;
        let selected = match selected.map_err(|e| self.error(node, e))? {
            Attempt::Selected(selected) => selected,
            Attempt::NestedInputs => return Ok(Progress::AwaitInputs),
            Attempt::Retry(inputs) => {
                if let Some(inputs) = inputs
                    && inputs
                        .commit(self.facts, &mut self.state.infer, term, &counts)
                        .map_err(|e| self.error(node, e))?
                {
                    proof
                        .append_at(opened.proof, scope)
                        .map_err(|e| self.error(node, e))?;
                    return Ok(Progress::InputsRefined);
                }
                if expose
                    && let Some(result) =
                        self.symbolic_projection(node, scope, term, proof, &caller)?
                {
                    return Ok(Progress::Symbolic(result));
                }
                return Ok(Progress::AwaitChange);
            }
        };
        let original = self
            .state
            .infer
            .projection_application(&self.facts.types, term)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .equate_trait_predicate_inside(
                &mut self.facts.types,
                &original.predicate,
                selected.predicate,
                &counts,
            )
            .map_err(|e| self.error(node, e))?;
        proof
            .append_at(opened.proof, scope)
            .map_err(|e| self.error(node, e))?;
        if matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Declarations
        ) {
            return Ok(Progress::Normalize(selected.result));
        }
        let ResultType::Type(ty) = selected.result else {
            return Err(self.error(node, "missing completed associated type definition"));
        };
        let ty = self.normalize_body_type(node, proof, scope, ty)?;
        self.state
            .infer
            .apply_normalization(&mut self.facts.types, term, ty)
            .map_err(|e| self.error(node, e))?;
        Ok(Progress::Complete)
    }

    fn symbolic_projection(
        &mut self,
        node: NodeId,
        scope: ScopeId,
        term: TermId,
        proof: &mut Obligations,
        caller: &Caller<'_>,
    ) -> Result<Option<TermId>, Error> {
        let Some((result, selected)) =
            selection::symbolic_result(self.facts, &mut self.state.infer, term, caller)
                .map_err(|e| self.error(node, e))?
        else {
            return Ok(None);
        };
        proof
            .append_at(selected, scope)
            .map_err(|e| self.error(node, e))?;
        Ok(Some(result))
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

    fn select(
        &mut self,
        facts: &mut Declarations,
        availability: crate::sema::traits::implementations::Availability,
    ) -> Result<Attempt, String> {
        if self.pending_inputs(facts).map_err(inference)? {
            return Ok(Attempt::NestedInputs);
        }
        if let Some(selected) = self.metadata_request(facts)? {
            return Ok(Attempt::Selected(selected));
        }
        self.select_member_result(facts, availability)
    }

    fn select_member_result(
        &mut self,
        facts: &mut Declarations,
        availability: crate::sema::traits::implementations::Availability,
    ) -> Result<Attempt, String> {
        let assumptions = caller_assumptions(facts, &mut self.context, self.environment.as_ref())?;
        let caller = Caller {
            assumptions: &assumptions,
            environment: self.environment.as_ref(),
            first_region: self.proof.regions.end,
            retain_sized: false,
        };
        facts.body_work.projection_selections += 1;
        let selected = selection::select_available(
            facts,
            &mut self.context,
            &self.application.predicate,
            caller,
            availability,
        )?;
        let Selection::Proven(evidence) = selected else {
            return Ok(Attempt::Retry(None));
        };
        let source = evidence.source.clone();
        let assumed = if matches!(
            source,
            Source::Assumption(_)
                | Source::ResultBound(_)
                | Source::QuantifiedAssumption { .. }
                | Source::QuantifiedResultBound { .. }
        ) {
            use crate::sema::traits::equalities::{self, Selection};
            match equalities::for_projection(
                facts,
                &mut self.context,
                &self.application,
                self.environment.as_deref(),
                evidence.proof.regions.end,
            )? {
                Selection::Proven(equality) => Some(equality),
                Selection::Absent | Selection::Ambiguous
                    if self.application.arguments.is_empty() =>
                {
                    return Ok(Attempt::Retry(None));
                }
                Selection::Absent | Selection::Ambiguous => None,
            }
        } else {
            None
        };
        self.proof.append(evidence)?;
        self.proof
            .equalities
            .append(&mut self.context.region_equalities);
        let predicate = self
            .context
            .resolved_predicate(&mut facts.types, &self.application.predicate)
            .map_err(inference)?;
        let assumed = assumed
            .map(|equality| equality.append_to(&mut self.proof))
            .transpose()?;
        let mut restoration = self.proof.body_restoration()?;
        let predicate = restoration.predicate(&mut facts.types, predicate)?;
        if !self.application.arguments.is_empty() {
            return FamilyInputs::new(
                facts,
                &mut self.context,
                predicate,
                &self.application.arguments,
                restoration,
            )
            .map(|inputs| Attempt::Retry(Some(inputs)));
        }
        let result = if let Some(ty) = assumed {
            ResultType::Type(restoration.apply(&mut facts.types, ty)?)
        } else {
            let definition = Definition::select(
                facts,
                self.application.item,
                crate::types::GenericArgs::EMPTY,
                &source,
            )?;
            if let Some(ty) = definition.result(facts)? {
                ResultType::Type(restoration.apply(&mut facts.types, ty)?)
            } else {
                self.context
                    .account_work(restoration.steps)
                    .map_err(inference)?;
                return Ok(Attempt::Selected(Selected {
                    predicate,
                    result: ResultType::Definition(Box::new(Pending {
                        definition,
                        restoration,
                    })),
                }));
            }
        };
        self.context
            .account_work(restoration.steps)
            .map_err(inference)?;
        Ok(Attempt::Selected(Selected { predicate, result }))
    }

    /// This fast path supplies only the real request. Cursor owns all source
    /// parent, equality, metadata and result-region proof on the normal path.
    fn metadata_request(&mut self, facts: &mut Declarations) -> Result<Option<Selected>, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let mut matching = false;
        for role in [
            crate::support::bindings::LangItem::MetadataType,
            crate::support::bindings::LangItem::DiscriminantType,
        ] {
            matching |= catalog
                .role(role)?
                .is_some_and(|m| m.definition.0 == self.application.item.0);
        }
        if !matching || !self.application.arguments.is_empty() {
            return Ok(None);
        }
        let predicate = match self
            .context
            .resolved_predicate(&mut facts.types, &self.application.predicate)
        {
            Ok(predicate) => predicate,
            Err(
                crate::types::infer::Error::Unresolved | crate::types::infer::Error::Normalization,
            ) => return Ok(None),
            Err(error) => return Err(inference(error)),
        };
        let mut restoration = self.proof.body_restoration()?;
        let predicate = restoration.predicate(&mut facts.types, predicate)?;
        let (trait_application, _) = predicate
            .bound
            .trait_parts()
            .ok_or("source Metadata request has no real parent")?;
        crate::sema::header_attributes::charge_source_retention(
            facts,
            1,
            size_of::<crate::types::Kind>(),
        )
        .map_err(|error| error.to_string())?;
        let ty = facts.types.intern(crate::types::Kind::Projection(
            crate::types::projection::Application {
                item: self.application.item,
                subject: predicate.ty,
                trait_application,
                arguments: GenericArgs::EMPTY,
            },
        ))?;
        self.context
            .account_work(restoration.steps)
            .map_err(inference)?;
        Ok(Some(Selected {
            predicate,
            result: ResultType::Type(ty),
        }))
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
        roots.extend(
            arguments
                .iter()
                .chain(&self.application.arguments)
                .filter_map(|arg| match arg {
                    Argument::Type(term) => Some(*term),
                    Argument::Const(_) | Argument::Lifetime(_) => None,
                }),
        );
        self.context.account_work(
            arguments
                .len()
                .saturating_add(self.application.arguments.len()),
        )?;
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
    let (assumptions, work) =
        Assumptions::from_environment(&facts.types, environment.map(Arc::as_ref))?;
    context.account_work(work).map_err(inference)?;
    Ok(assumptions)
}

fn inference(error: crate::types::infer::Error) -> String {
    error.to_string()
}

#[cfg(test)]
#[path = "../../../../tests/invariants/inferred_members.rs"]
mod tests;
