//! Suspend a value graph for checked members without repeating body inference.
use super::{Error, Queries, Query, Work};
use crate::sema::{
    dispatch::Dispatcher,
    traits::implementations::{Availability, ConstantRequest},
    values::{Dependency, Instance},
};
use crate::support::hash::HashSet;

impl Queries<'_> {
    pub(super) fn demand_members(
        &mut self,
        instances: Vec<Instance>,
        mut substitution: Option<crate::types::substitute::Bound>,
    ) -> Result<Vec<ConstantRequest>, Error> {
        if instances.is_empty() {
            return Ok(Vec::new());
        }
        let Some(scope) = self.value_scope else {
            return Ok(Vec::new());
        };
        let body = scope.body();
        let query = Query::Node(body);
        let first = self
            .typing
            .facts
            .body_regions
            .frontier(body)
            .map_err(|_| self.error(query, "constant member region overflow"))?;
        let mut dispatch = Dispatcher::declaration(first).map_err(|e| self.error(query, e))?;
        let mut seen = HashSet::default();
        let mut pending = HashSet::default();
        for mut instance in instances {
            self.budget()?;
            if let Some(fold) = &mut substitution {
                instance = instance
                    .substitute(&mut self.typing.facts.types, fold)
                    .map_err(|e| self.error(query, e))?;
            }
            if !seen.insert(instance) {
                continue;
            }
            let Some(owner) = dispatch
                .constant_owner(self.typing.facts, instance)
                .map_err(|e| self.error(query, e))?
            else {
                continue;
            };
            if !self
                .typing
                .facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| {
                    registry
                        .available_constant(owner, instance.definition, Availability::Declarations)
                        .is_some()
                })
            {
                pending.insert(ConstantRequest {
                    owner,
                    declaration: instance.definition,
                });
            }
        }
        scope
            .retain(self.typing.facts, body, dispatch.finish())
            .map_err(|e| self.error(query, e))?;
        let mut pending: Vec<_> = pending.into_iter().collect();
        pending.sort_unstable_by_key(|request| (request.owner.0, request.declaration.0));
        Ok(pending)
    }

    pub(super) fn member_readiness(
        &mut self,
        work: &Work,
    ) -> Result<Option<super::suspension::Dependency>, Error> {
        let (query, index, instance) = match *work {
            Work::Enter(query @ (Query::Constant(index) | Query::Static(index))) => {
                (query, index, None)
            }
            Work::Enter(Query::Instance(instance)) | Work::PrepareInstance(instance) => {
                let query = Query::Instance(instance);
                let index = self
                    .typing
                    .facts
                    .initializer_index(instance.definition)
                    .ok_or_else(|| self.error(query, "selected constant has no initializer"))?;
                (query, index, Some(instance))
            }
            _ => return Ok(None),
        };
        if self.checked_members.contains(&query) {
            return Ok(None);
        }
        // Root selection retains any caller-specific proof before entering the
        // selected value. A completed independent closure already proves every
        // transitive member ready; do not scan its edges again for each caller.
        if instance.is_some_and(|instance| {
            self.typing
                .facts
                .dispatch_dependencies
                .contains(self.typing.availability, instance)
        }) {
            return Ok(None);
        }
        if !self.typing.facts.initializers[index].type_checked() {
            let dependency = if self.typing.pending_body() {
                self.typing.advance_body()?
            } else {
                self.typing.start_initializer(index)?
            };
            if let Some(dependency) = dependency {
                return Ok(Some(super::suspension::Dependency::Body(dependency)));
            }
            self.require_complete_patterns(query)?;
        }
        self.checked_members.insert(query);
        let definition = self.typing.facts.initializers[index].definition;
        let dependencies = self
            .typing
            .facts
            .constant_dependencies
            .get(&definition)
            .map_or(&[][..], Vec::as_slice);
        self.typing
            .facts
            .dispatch_dependencies
            .work
            .member_dependencies += dependencies.len();
        let instances = dependencies
            .iter()
            .filter_map(|dependency| match dependency {
                Dependency::Constant(instance) => Some(*instance),
                Dependency::Callable(_) => None,
            })
            .collect();
        let substitution = instance
            .map(|instance| instance.substitution(self.typing.facts))
            .transpose()
            .map_err(|e| self.error(query, e))?;
        let members = self.demand_members(instances, substitution)?;
        Ok((!members.is_empty()).then_some(super::suspension::Dependency::Members(members)))
    }
}
