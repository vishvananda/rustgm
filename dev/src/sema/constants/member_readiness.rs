//! Suspend a value graph for checked members without repeating body inference.
use super::{Error, Queries, Query, Work};
use crate::sema::{
    dispatch::Dispatcher,
    traits::implementations::{Availability, ConstantRequest},
    values::{Dependency, Instance},
};
use std::collections::HashSet;

impl Queries<'_> {
    pub(super) fn demand_members(
        &mut self,
        instances: Vec<Instance>,
        mut substitution: Option<crate::types::substitute::Bound>,
    ) -> Result<Vec<ConstantRequest>, Error> {
        if instances.is_empty() {
            return Ok(Vec::new());
        }
        let Some((body, ty)) = self.value_scope else {
            return Ok(Vec::new());
        };
        let query = Query::Node(body);
        let first = self
            .typing
            .facts
            .body_regions
            .len()
            .checked_add(1)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| self.error(query, "constant member region overflow"))?;
        let mut dispatch = Dispatcher::declaration(first).map_err(|e| self.error(query, e))?;
        let mut seen = HashSet::new();
        let mut pending = HashSet::new();
        for mut instance in instances {
            self.budget()?;
            if self
                .typing
                .facts
                .associated
                .trait_owner(instance.definition)
                .is_some()
                && !self
                    .typing
                    .facts
                    .generics
                    .implementations
                    .as_ref()
                    .is_some_and(|registry| registry.constant_headers_ready())
            {
                return Err(self.error(
                    query,
                    "constant selection requires complete candidate headers",
                ));
            }
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
        let proof = dispatch.finish();
        if !proof.is_empty() {
            proof
                .retain_body(self.typing.facts, body, body)
                .map_err(|e| self.error(query, e))?;
            self.typing.facts.anonymous_constants.insert(body, ty);
        }
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
                    .constants
                    .get(instance.definition.0 as usize)
                    .copied()
                    .flatten()
                    .ok_or_else(|| self.error(query, "selected constant has no initializer"))?;
                (query, index, Some(instance))
            }
            _ => return Ok(None),
        };
        if self.checked_members.contains(&query) {
            return Ok(None);
        }
        if !self.typing.facts.initializers[index].type_checked() {
            let dependency = if self.typing.pending_body() {
                self.typing.advance_body()?
            } else {
                self.typing.start_initializer(index)?
            };
            if let Some(dependency) = dependency {
                return Ok(Some(super::suspension::Dependency::Nominal(dependency)));
            }
            self.require_complete_patterns(query)?;
        }
        self.checked_members.insert(query);
        let definition = self.typing.facts.initializers[index].definition;
        let instances = self
            .typing
            .facts
            .constant_dependencies
            .get(&definition)
            .into_iter()
            .flatten()
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
