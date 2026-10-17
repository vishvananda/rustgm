//! Body-local deferred work waits for independently determined inputs before selection.
mod contracts;
mod inputs;
mod selection;
use super::{Checker, Error, NodeId, TermId};
use crate::{
    sema::{alias_requirements::Expansion, annotations::Fields, generics::AnnotationArgument},
    types::{
        RegionId, TypeId,
        infer::{
            arguments::Argument,
            deferred::{EquationId, Outcome, Relation},
            dependencies::WatchId,
        },
        lifetime_proof::{Obligations, ScopeId},
        parameters::{ParameterId, uses::Uses},
    },
};
use std::collections::{HashMap, HashSet};

struct Contract {
    expansion: Expansion,
    arguments: Vec<(ParameterId, Argument)>,
    regions: Vec<RegionId>,
}
enum Task {
    Contract(Contract),
    Normalize { input: TermId, output: TermId },
    Symbolic { input: TermId, output: TermId },
}
struct Query {
    node: NodeId,
    scope: ScopeId,
    task: Task,
    input_normalizations: bool,
}
struct Data {
    scopes: Obligations,
    environment: std::cell::OnceCell<
        Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    >,
    tasks: HashMap<WatchId, Query>,
    equations: HashMap<EquationId, NodeId>,
    normalizations: HashSet<(ScopeId, TermId)>,
    normalization_waiters: HashMap<TermId, HashSet<WatchId>>,
    input_scopes: HashMap<(ScopeId, TermId), ScopeId>,
    normal_forms: HashMap<TypeId, TypeId>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            scopes: Obligations::new(1),
            environment: std::cell::OnceCell::new(),
            tasks: HashMap::new(),
            equations: HashMap::new(),
            normalizations: HashSet::new(),
            normalization_waiters: HashMap::new(),
            input_scopes: HashMap::new(),
            normal_forms: HashMap::new(),
        }
    }
}
#[derive(Default)]
pub(super) struct Pending {
    data: Option<Box<Data>>,
}
impl Pending {
    /// Finishing the worklist does not remove the caller aliases introduced by
    /// projected field/call instantiation from subsequent shape equations.
    pub fn has_context(&self) -> bool {
        self.active()
            || self
                .data
                .as_ref()
                .is_some_and(|data| !data.normalizations.is_empty())
    }
    pub fn active(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| !data.tasks.is_empty() || !data.equations.is_empty())
    }
    pub fn scope(&mut self, parent: ScopeId, count: u32) -> Result<ScopeId, &'static str> {
        self.data
            .get_or_insert_with(Default::default)
            .scopes
            .scope(parent, count)
    }
    fn insert(&mut self, watch: WatchId, query: Query) {
        self.data
            .get_or_insert_with(Default::default)
            .tasks
            .insert(watch, query);
    }
}
impl Checker<'_> {
    pub(super) fn known_projection(&mut self, node: NodeId, ty: TypeId) -> Result<TermId, Error> {
        let ty = self.normalize_known_projection(node, ty)?;
        let term = self
            .state
            .infer
            .known(&self.facts.types, ty)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .apply_normalization(&mut self.facts.types, term, ty)
            .map_err(|e| self.error(node, e))?;
        Ok(term)
    }
    pub(super) fn normalize_known_projection(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        // A canonical declaration type already has determined inputs. Keep its
        // written body lifetimes while proving its complete nested binder tree;
        // replacing the whole type with caller aliases would change its return contract.
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| self.error(node, "body projection region space exhausted"))?;
        let mut proof = Obligations::new(first);
        // Call arguments already use free aliases for caller universals. Their
        // environment must use the same representation, without duplicate
        // original/aliased assumptions. Written body types keep their binders.
        let environment = if self.facts.types.escaping_binders(ty) == Some(0) {
            self.projection_environment(node)?
        } else {
            self.state
                .environment_owner
                .and_then(|owner| self.facts.generics.environments.get(&owner))
                .cloned()
        };
        let (ty, steps) = crate::sema::projections::normalize(
            self.facts,
            environment.as_ref(),
            &mut proof,
            ScopeId::ROOT,
            ty,
            self.availability,
        )
        .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))?;
        Ok(ty)
    }

    pub(super) fn defer_contract(
        &mut self,
        node: NodeId,
        scope: ScopeId,
        expansion: Expansion,
        supplied: &[AnnotationArgument],
        fields: &Fields,
    ) -> Result<(), Error> {
        let mut uses = Uses::default();
        for (_, ty) in expansion
            .proof
            .type_roots()
            .chain(expansion.normalizations.iter().copied())
            .chain(expansion.sized.iter().copied())
        {
            uses.collect(&self.facts.types, ty)
                .map_err(|e| self.error(node, e))?;
        }
        self.state
            .infer
            .account_work(uses.steps)
            .map_err(|e| self.error(node, e))?;
        let mut ids = uses
            .parameters
            .into_iter()
            .chain(uses.constants)
            .collect::<Vec<_>>();
        ids.sort_unstable_by_key(|id| id.0);
        let mut types = Vec::new();
        let mut constants = Vec::new();
        let mut arguments = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(argument) = fields.parameters.binding(id) {
                match argument {
                    Argument::Type(term) => types.push(term),
                    Argument::Const(term) => constants.push(term),
                    Argument::Lifetime(_) => {
                        return Err(self.error(node, "invalid inferred proof parameter"));
                    }
                }
                arguments.push((id, argument));
            }
        }
        let watch = self
            .state
            .infer
            .watch_resolution(&types, &constants)
            .map_err(|e| self.error(node, e))?;
        let regions = supplied
            .iter()
            .filter_map(|arg| match arg {
                AnnotationArgument::Lifetime(region) => Some(*region),
                _ => None,
            })
            .collect();
        self.state.projections.insert(
            watch,
            Query {
                node,
                scope,
                task: Task::Contract(Contract {
                    expansion,
                    arguments,
                    regions,
                }),
                input_normalizations: false,
            },
        );
        Ok(())
    }
    pub(super) fn defer_normalization(
        &mut self,
        node: NodeId,
        scope: ScopeId,
        term: TermId,
    ) -> Result<(), Error> {
        let roots = self
            .state
            .infer
            .normalization_roots(&self.facts.types, term, scope, |parent, count| {
                self.state
                    .projections
                    .scope(parent, count)
                    .map_err(|_| crate::types::infer::Error::InvalidIdentity)
            })
            .map_err(|e| self.error(node, e))?;
        for (scope, term) in roots {
            if !self
                .state
                .projections
                .data
                .get_or_insert_with(Default::default)
                .normalizations
                .insert((scope, term))
            {
                continue;
            }
            let input = self
                .state
                .infer
                .retain_projection(&mut self.facts.types, term)
                .map_err(|e| self.error(node, e))?;
            let watch = self
                .state
                .infer
                .watch_changes(&[input], &[])
                .map_err(|e| self.error(node, e))?;
            self.state.projections.insert(
                watch,
                Query {
                    node,
                    scope,
                    task: Task::Normalize {
                        input,
                        output: term,
                    },
                    input_normalizations: false,
                },
            );
        }
        Ok(())
    }
    pub(super) fn deferred_equation(
        &mut self,
        node: NodeId,
        left: TermId,
        right: TermId,
        relation: Relation,
    ) -> Result<(), Error> {
        if let Outcome::Pending(id) = self
            .state
            .infer
            .equate_deferred(&self.facts.types, left, right, relation)
            .map_err(|e| self.error(node, e))?
        {
            self.state
                .projections
                .data
                .get_or_insert_with(Default::default)
                .equations
                .insert(id, node);
        }
        Ok(())
    }
    pub(super) fn advance_projections(&mut self, body: NodeId) -> Result<bool, Error> {
        if !self.state.projections.active() {
            return Ok(false);
        }
        let mut progress = false;
        loop {
            while let Some(watch) = self
                .state
                .infer
                .ready_resolution(&self.facts.types)
                .map_err(|e| self.error(body, e))?
            {
                let query = self
                    .state
                    .projections
                    .data
                    .as_mut()
                    .and_then(|d| d.tasks.remove(&watch))
                    .ok_or_else(|| self.error(body, "missing body normalization query"))?;
                let resolved = self
                    .state
                    .infer
                    .watch_resolved(watch)
                    .map_err(|e| self.error(body, e))?;
                let (query, advanced) = self.complete_projection(query, resolved, watch)?;
                progress |= advanced;
                if let Some(query) = query {
                    self.state.projections.insert(watch, query);
                } else {
                    self.state
                        .infer
                        .cancel_watch(watch)
                        .map_err(|e| self.error(body, e))?;
                    progress = true;
                }
            }
            let mut resumed = false;
            while let Some(id) = self
                .state
                .infer
                .ready_equation()
                .map_err(|e| self.error(body, e))?
            {
                let node = self
                    .state
                    .projections
                    .data
                    .as_ref()
                    .and_then(|d| d.equations.get(&id))
                    .copied()
                    .ok_or_else(|| self.error(body, "missing suspended body equation"))?;
                let outcome = self
                    .state
                    .infer
                    .resume_equation(&self.facts.types, id)
                    .map_err(|e| self.error(node, e))?;
                if outcome == Outcome::Complete
                    && let Some(data) = &mut self.state.projections.data
                {
                    data.equations.remove(&id);
                }
                resumed = true;
            }
            progress |= resumed;
            if !resumed {
                return Ok(progress);
            }
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one progress dispatch keeps the retained input, shared output and exact scope proof together across suspended and published states"
    )]
    fn complete_projection(
        &mut self,
        mut query: Query,
        resolved: bool,
        watch: WatchId,
    ) -> Result<(Option<Query>, bool), Error> {
        let node = query.node;
        let input = match query.task {
            Task::Normalize { output, .. } | Task::Symbolic { output, .. } => Some(output),
            Task::Contract(_) => None,
        };
        let resolved = match self.projection_readiness(&mut query, resolved, watch)? {
            inputs::Readiness::Waiting => return Ok((Some(query), false)),
            inputs::Readiness::Partial => false,
            inputs::Readiness::Resolved => true,
        };
        self.prepare_universals(node)?;
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| self.error(node, "body projection region space exhausted"))?;
        let plan = self
            .state
            .projections
            .data
            .as_ref()
            .ok_or_else(|| self.error(node, "missing source proof scopes"))?
            .scopes
            .scope_plan();
        let mut proof = Obligations::new(first);
        let scope = plan
            .enter(query.scope, &mut proof)
            .map_err(|e| self.error(node, e))?;
        let published = match query.task {
            Task::Symbolic { input, output } => Some((input, output)),
            Task::Normalize { .. } | Task::Contract(_) => None,
        };
        match query.task {
            Task::Contract(contract) => {
                self.complete_contract(node, scope, contract, &mut proof)?;
            }
            Task::Normalize { input: term, .. } | Task::Symbolic { input: term, .. }
                if !resolved =>
            {
                let expose = matches!(query.task, Task::Normalize { .. });
                match self.select_projection(node, scope, term, &mut proof, expose)? {
                    selection::Progress::Complete => {
                        if let Task::Normalize { output, .. } = query.task {
                            let ty = self
                                .state
                                .infer
                                .resolved(&mut self.facts.types, term)
                                .map_err(|e| self.error(node, e))?;
                            self.state
                                .infer
                                .apply_normalization(&mut self.facts.types, output, ty)
                                .map_err(|e| self.error(node, e))?;
                        }
                    }
                    selection::Progress::Symbolic(result) => {
                        let Task::Normalize { input, output } = query.task else {
                            return Err(self.error(node, "symbolic result was already published"));
                        };
                        self.state
                            .infer
                            .publish_projection(&self.facts.types, output, result)
                            .map_err(|e| self.error(node, e))?;
                        query.task = Task::Symbolic { input, output };
                        proof
                            .retain_body(self.facts, self.state.body, self.key(node))
                            .map_err(|e| self.error(node, e))?;
                        self.defer_normalization(node, query.scope, result)?;
                        self.published_normalization(node, output)?;
                        return Ok((Some(query), true));
                    }
                    selection::Progress::AwaitInputs => {
                        self.defer_input_normalizations(&mut query, term, watch)?;
                        return Ok((Some(query), false));
                    }
                    selection::Progress::AwaitChange => return Ok((Some(query), false)),
                    selection::Progress::AwaitResolution => {
                        self.state
                            .infer
                            .await_resolution(watch)
                            .map_err(|e| self.error(node, e))?;
                        return Ok((Some(query), false));
                    }
                }
            }
            Task::Normalize { input: term, .. } | Task::Symbolic { input: term, .. } => {
                let ty = self
                    .state
                    .infer
                    .resolved(&mut self.facts.types, term)
                    .map_err(|e| self.error(node, e))?;
                let ty = self.normalize_body_type(node, &mut proof, scope, ty)?;
                let target = match query.task {
                    Task::Normalize { output, .. } => output,
                    _ => term,
                };
                self.state
                    .infer
                    .apply_normalization(&mut self.facts.types, target, ty)
                    .map_err(|e| self.error(node, e))?;
            }
        }
        if let Some((input, output)) = published {
            self.deferred_equation(node, input, output, Relation::Equality)?;
        }
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))?;
        if published.is_none()
            && let Some(input) = input
        {
            self.published_normalization(node, input)?;
        }
        Ok((None, true))
    }
    fn normalize_body_type(
        &mut self,
        node: NodeId,
        proof: &mut Obligations,
        scope: ScopeId,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        if self.facts.types.contains_projections(ty) == Some(false) {
            return Ok(ty);
        }
        // The body owns one immutable environment. A root-scope result with no
        // lifetime evidence can be reused without copying or freshening proof.
        let cacheable = scope == ScopeId::ROOT
            && proof.is_empty()
            && self.facts.types.has_free_regions(ty, 0) == Some(false);
        if cacheable
            && let Some(&result) = self
                .state
                .projections
                .data
                .as_ref()
                .and_then(|data| data.normal_forms.get(&ty))
        {
            return Ok(result);
        }
        let environment = self.projection_environment(node)?;
        let (result, steps) = crate::sema::projections::normalize(
            self.facts,
            environment.as_ref(),
            proof,
            scope,
            ty,
            self.availability,
        )
        .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        if cacheable && proof.is_empty() {
            self.state
                .projections
                .data
                .get_or_insert_with(Default::default)
                .normal_forms
                .insert(ty, result);
        }
        Ok(result)
    }
}
