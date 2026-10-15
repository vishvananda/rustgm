//! Body-local deferred work waits for independently determined inputs before selection.
mod contracts;
use super::{Checker, Error, NodeId, TermId};
use crate::{
    sema::{alias_requirements::Expansion, annotations::Fields, generics::AnnotationArgument},
    types::{
        RegionId,
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
    Normalize(TermId),
}
struct Query {
    node: NodeId,
    scope: ScopeId,
    task: Task,
}
struct Data {
    scopes: Obligations,
    tasks: HashMap<WatchId, Query>,
    equations: HashMap<EquationId, NodeId>,
    normalizations: HashSet<(ScopeId, TermId)>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            scopes: Obligations::new(1),
            tasks: HashMap::new(),
            equations: HashMap::new(),
            normalizations: HashSet::new(),
        }
    }
}
#[derive(Default)]
pub(super) struct Pending {
    data: Option<Box<Data>>,
}
impl Pending {
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
        self.infer
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
        self.projections.insert(
            watch,
            Query {
                node,
                scope,
                task: Task::Contract(Contract {
                    expansion,
                    arguments,
                    regions,
                }),
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
            .infer
            .normalization_roots(&self.facts.types, term, scope, |parent, count| {
                self.projections
                    .scope(parent, count)
                    .map_err(|_| crate::types::infer::Error::InvalidIdentity)
            })
            .map_err(|e| self.error(node, e))?;
        for (scope, term) in roots {
            if !self
                .projections
                .data
                .get_or_insert_with(Default::default)
                .normalizations
                .insert((scope, term))
            {
                continue;
            }
            let watch = self
                .infer
                .watch_resolution(&[term], &[])
                .map_err(|e| self.error(node, e))?;
            self.projections.insert(
                watch,
                Query {
                    node,
                    scope,
                    task: Task::Normalize(term),
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
            .infer
            .equate_deferred(&self.facts.types, left, right, relation)
            .map_err(|e| self.error(node, e))?
        {
            self.projections
                .data
                .get_or_insert_with(Default::default)
                .equations
                .insert(id, node);
        }
        Ok(())
    }
    pub(super) fn advance_projections(&mut self, body: NodeId) -> Result<bool, Error> {
        if !self.projections.active() {
            return Ok(false);
        }
        let mut progress = false;
        loop {
            while let Some(watch) = self
                .infer
                .ready_resolution(&self.facts.types)
                .map_err(|e| self.error(body, e))?
            {
                let query = self
                    .projections
                    .data
                    .as_mut()
                    .and_then(|d| d.tasks.remove(&watch))
                    .ok_or_else(|| self.error(body, "missing body normalization query"))?;
                self.complete_projection(query)?;
                progress = true;
            }
            let mut resumed = false;
            while let Some(id) = self
                .infer
                .ready_equation()
                .map_err(|e| self.error(body, e))?
            {
                let node = self
                    .projections
                    .data
                    .as_ref()
                    .and_then(|d| d.equations.get(&id))
                    .copied()
                    .ok_or_else(|| self.error(body, "missing suspended body equation"))?;
                let outcome = self
                    .infer
                    .resume_equation(&self.facts.types, id)
                    .map_err(|e| self.error(node, e))?;
                if outcome == Outcome::Complete
                    && let Some(data) = &mut self.projections.data
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
    fn complete_projection(&mut self, query: Query) -> Result<(), Error> {
        let node = query.node;
        self.prepare_universals(node)?;
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| self.error(node, "body projection region space exhausted"))?;
        let plan = self
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
        match query.task {
            Task::Contract(contract) => {
                self.complete_contract(node, scope, contract, &mut proof)?;
            }
            Task::Normalize(term) => {
                let ty = self
                    .infer
                    .resolved(&mut self.facts.types, term)
                    .map_err(|e| self.error(node, e))?;
                let ty = self.normalize_body_type(node, &mut proof, scope, ty)?;
                self.infer
                    .apply_normalization(&mut self.facts.types, term, ty)
                    .map_err(|e| self.error(node, e))?;
            }
        }
        proof
            .retain_body(self.facts, self.body, self.key(node))
            .map_err(|e| self.error(node, e))
    }
    fn normalize_body_type(
        &mut self,
        node: NodeId,
        proof: &mut Obligations,
        scope: ScopeId,
        ty: crate::types::TypeId,
    ) -> Result<crate::types::TypeId, Error> {
        let (ty, steps) = crate::sema::projections::normalize(
            self.report,
            self.facts,
            self.method_environment.as_ref(),
            proof,
            scope,
            ty,
        )
        .map_err(|e| self.error(node, e))?;
        self.infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        Ok(ty)
    }
}
