//! Evaluation work and dependency states belong to the selected initializer instance.
#[cfg(test)]
#[path = "../../../tests/invariants/selected_constants.rs"]
mod tests;
use super::{Error, NodeKey, Queries, Query, Work};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{GenericArgs, TypeId, arguments::Argument, substitute::Bound},
};
use std::collections::HashMap;

/// A typed use or selected initializer. Trait declarations still require dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Instance {
    pub definition: DefId,
    pub arguments: GenericArgs,
}
/// Source completion and checked materialization share one full-instance value.
#[derive(Clone, Copy)]
pub(crate) struct Completed {
    /// None retains source-query completion; Some is a materialized CFG result.
    pub(crate) ty: Option<TypeId>,
    pub(crate) value: super::values::Constant,
}
/// Initializer dependencies retain complete identities before owner substitution.
#[derive(Clone, Copy)]
pub(crate) enum Dependency {
    Constant(Instance),
    Callable(TypeId),
}
#[derive(Clone, Copy)]
pub(super) enum ValueScope {
    Anonymous { body: NodeKey, ty: TypeId },
    Initializer { body: NodeKey, index: usize },
}
impl ValueScope {
    pub(super) const fn body(self) -> NodeKey {
        match self {
            Self::Anonymous { body, .. } | Self::Initializer { body, .. } => body,
        }
    }
    pub(super) fn retain(
        self,
        facts: &mut Declarations,
        source: NodeKey,
        proof: crate::types::lifetime_proof::Obligations,
    ) -> Result<(), &'static str> {
        if !proof.is_empty() {
            proof.retain_body(facts, self.body(), source)?;
            if let Self::Anonymous { body, ty } = self {
                facts.anonymous_constants.insert(body, ty);
            }
        }
        Ok(())
    }
}
pub(crate) struct InstanceArguments {
    pub bindings: Vec<(crate::types::parameters::ParameterId, Argument)>,
    pub regions: Vec<crate::types::RegionId>,
}
impl Instance {
    /// Uniquely selected result type; cached values never supply type authority.
    pub(crate) fn result_type(self, facts: &mut Declarations) -> Result<TypeId, &'static str> {
        if !facts.types.arguments_are_concrete(self.arguments)? {
            return Err("selected constant arguments are not concrete");
        }
        let declared = facts
            .definitions
            .get(self.definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("selected constant lacks a declared result")?;
        let mut substitution = self.substitution(facts)?;
        let result = substitution.apply(&mut facts.types, declared)?;
        if facts.types.contains_parameters(result) != Some(false)
            || facts.types.contains_projections(result) != Some(false)
        {
            return Err("selected constant result is not closed and normalized");
        }
        Ok(result)
    }
    pub(crate) fn publish(
        self,
        facts: &mut Declarations,
        expected: Option<TypeId>,
        value: super::values::Constant,
    ) -> Result<(), &'static str> {
        let ty = expected
            .map(|expected| {
                let ty = self.result_type(facts)?;
                if expected != ty {
                    return Err("selected initializer result changed at completion");
                }
                Ok(ty)
            })
            .transpose()?;
        if let Some(previous) = facts.constant_instances.get_mut(&self) {
            if previous.value != value
                || matches!((previous.ty, ty), (Some(old), Some(new)) if old != new)
            {
                return Err("selected initializer value changed after publication");
            }
            if ty.is_some() {
                previous.ty = ty;
            }
        } else {
            let bytes = facts
                .constant_work
                .associated_value_bytes
                .checked_add(crate::support::retained::new_map::<Instance, Completed>(1))
                .filter(|&bytes| bytes <= 64 * 1024 * 1024)
                .ok_or("completed associated value storage exceeds 64 MiB budget")?;
            facts
                .constant_instances
                .insert(self, Completed { ty, value });
            facts.constant_work.associated_value_bytes = bytes;
        }
        Ok(())
    }
    /// Substitute captured arguments without rewriting the shared declaration.
    /// # Errors
    /// Rejects malformed argument identities and exhausted substitution work.
    pub fn substitute(
        self,
        store: &mut crate::types::Store,
        bound: &mut Bound,
    ) -> Result<Self, &'static str> {
        let arguments = store
            .arguments(self.arguments)
            .ok_or("invalid constant arguments")?
            .collect::<Vec<_>>();
        let mut result = Vec::with_capacity(arguments.len());
        for argument in arguments {
            result.push(bound.apply_argument(store, argument)?);
        }
        Ok(Self {
            definition: self.definition,
            arguments: store.intern_arguments(&result)?,
        })
    }
    /// # Errors
    /// Rejects missing or mismatched captured parameter identities.
    pub(crate) fn substitution(self, facts: &Declarations) -> Result<Bound, &'static str> {
        if facts.associated.schema(self.definition).is_none() {
            return if self.arguments == GenericArgs::EMPTY {
                Ok(Bound::new(Vec::new()))
            } else {
                Err("ordinary constant has captured arguments")
            };
        }
        let selected = self.selected_arguments(facts)?;
        Bound::parameters_captured(&facts.types, &selected.bindings, selected.regions)
    }
    pub(crate) fn selected_arguments(
        self,
        facts: &Declarations,
    ) -> Result<InstanceArguments, &'static str> {
        let Some(schema) = facts.associated.schema(self.definition) else {
            return if self.arguments == GenericArgs::EMPTY {
                Ok(InstanceArguments {
                    bindings: Vec::new(),
                    regions: Vec::new(),
                })
            } else {
                Err("ordinary constant has captured arguments")
            };
        };
        let captures = schema.captures as usize;
        let (_, parameters) = facts
            .generics
            .callable_parameters(&facts.types, self.definition)?;
        let arguments = facts
            .types
            .arguments(self.arguments)
            .ok_or("invalid constant arguments")?
            .collect::<Vec<_>>();
        if arguments.len() != captures.saturating_add(parameters.len()) {
            return Err("constant owner argument arity mismatch");
        }
        let regions = arguments[..captures]
            .iter()
            .map(|argument| match argument {
                Argument::Lifetime(region) => Ok(*region),
                _ => Err("constant owner requires lifetime arguments"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bindings = parameters
            .into_iter()
            .zip(arguments[captures..].iter().copied())
            .collect::<Vec<_>>();
        Ok(InstanceArguments { bindings, regions })
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(super) struct Context(pub u32);
pub(super) struct Environment {
    pub instance: Instance,
    substitution: Bound,
}
#[derive(Default)]
pub(super) struct Worklist {
    pub context: Context,
    values: Vec<(Context, Work)>,
}
impl Worklist {
    pub fn push(&mut self, work: Work) {
        self.values.push((self.context, work));
    }
    pub fn extend(&mut self, work: impl Iterator<Item = Work>) {
        self.values.extend(work.map(|work| (self.context, work)));
    }
    pub fn pop(&mut self) -> Option<Work> {
        let (context, work) = self.values.pop()?;
        self.context = context;
        Some(work)
    }
}
#[derive(Default)]
pub(super) struct Instances {
    identities: HashMap<Instance, Context>,
    environments: Vec<Environment>,
}
impl Queries<'_> {
    pub(super) fn execution_context(&self) -> Option<Instance> {
        self.work
            .context
            .0
            .checked_sub(1)
            .map(|index| self.instances.environments[index as usize].instance)
    }
    pub(super) fn drop_substitution(&self) -> Result<Option<Bound>, &'static str> {
        self.work
            .context
            .0
            .checked_sub(1)
            .map(|index| {
                self.instances.environments[index as usize]
                    .instance
                    .substitution(self.typing.facts)
            })
            .transpose()
    }
    pub(super) fn state_key(&self, query: Query) -> (Context, Query) {
        let context = if matches!(
            query,
            Query::Node(_) | Query::Place(_) | Query::Contents(..)
        ) {
            self.work.context
        } else {
            Context::default()
        };
        (context, query)
    }
    pub(super) fn specialized_type(&mut self, key: NodeKey, ty: TypeId) -> Result<TypeId, Error> {
        let result = self.substituted_type(key, ty)?;
        self.complete_type(key, result)
    }
    pub(super) fn substituted_type(&mut self, key: NodeKey, ty: TypeId) -> Result<TypeId, Error> {
        let Some(index) = self.work.context.0.checked_sub(1) else {
            return Ok(ty);
        };
        self.instances.environments[index as usize]
            .substitution
            .apply(&mut self.typing.facts.types, ty)
            .map_err(|e| self.error(Query::Node(key), e))
    }
    pub(super) fn complete_type(&mut self, key: NodeKey, result: TypeId) -> Result<TypeId, Error> {
        if self.work.context == Context::default() {
            return Ok(result);
        }
        if self.typing.facts.types.contains_parameters(result) != Some(false) {
            return Err(self.error(
                Query::Node(key),
                "constant value retains an unresolved parameter",
            ));
        }
        if self.typing.facts.types.contains_projections(result) == Some(true) {
            return Err(self.error(
                Query::Node(key),
                "constant instance projection normalization is not implemented",
            ));
        }
        self.typing
            .facts
            .aggregates
            .prepare(&mut self.typing.facts.types, result)
            .map_err(|e| self.error(Query::Node(key), e))?;
        Ok(result)
    }
    pub(super) fn specialized_argument(
        &mut self,
        key: NodeKey,
        argument: crate::types::arguments::Argument,
    ) -> Result<crate::types::arguments::Argument, Error> {
        let Some(index) = self.work.context.0.checked_sub(1) else {
            return Ok(argument);
        };
        self.instances.environments[index as usize]
            .substitution
            .apply_argument(&mut self.typing.facts.types, argument)
            .map_err(|e| self.error(Query::Node(key), e))
    }
    pub(super) fn selected_constant(
        &mut self,
        key: NodeKey,
        mut instance: Instance,
    ) -> Result<Instance, Error> {
        if let Some(index) = self.work.context.0.checked_sub(1) {
            instance = instance
                .substitute(
                    &mut self.typing.facts.types,
                    &mut self.instances.environments[index as usize].substitution,
                )
                .map_err(|e| self.error(Query::Node(key), e))?;
        }
        if let Some(scope) = self.pattern_scope {
            // Inherent values already retain their owner bounds from inference.
            // With no associated dependencies there is no implementation to select.
            if self
                .typing
                .facts
                .associated
                .trait_owner(instance.definition)
                .is_none()
                && !self
                    .typing
                    .facts
                    .constant_dependencies
                    .contains_key(&instance.definition)
            {
                return Ok(instance);
            }
            if self.pattern_dispatch.is_none() {
                let first = self
                    .typing
                    .facts
                    .body_regions
                    .len()
                    .checked_add(1)
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| {
                        self.error(Query::Node(key), "pattern dispatch region overflow")
                    })?;
                self.pattern_dispatch = Some(
                    crate::sema::dispatch::Dispatcher::pattern(
                        self.typing.facts,
                        scope.binders,
                        first,
                        scope.owner,
                    )
                    .map_err(|e| self.error(Query::Node(key), e))?,
                );
            }
            if let Some(dispatch) = &mut self.pattern_dispatch {
                return dispatch
                    .constant(self.typing.facts, instance)
                    .map_err(|e| self.error(Query::Node(key), e));
            }
        }
        if self
            .typing
            .facts
            .associated
            .trait_owner(instance.definition)
            .is_none()
        {
            return Ok(instance);
        }
        if let Some(scope) = self.value_scope {
            return self.value_instance(key, instance, scope);
        }
        self.typing
            .facts
            .constant_targets
            .get(&instance)
            .copied()
            .ok_or_else(|| {
                self.error(
                    Query::Node(key),
                    "dependent trait constant requires completed value selection",
                )
            })
    }
    fn value_instance(
        &mut self,
        key: NodeKey,
        instance: Instance,
        scope: ValueScope,
    ) -> Result<Instance, Error> {
        let first = self
            .typing
            .facts
            .body_regions
            .len()
            .checked_add(1)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| self.error(Query::Node(key), "constant dispatch region overflow"))?;
        let mut dispatch = match self.typing.availability {
            crate::sema::traits::implementations::Availability::Declarations => {
                crate::sema::dispatch::Dispatcher::declaration(first)
            }
            crate::sema::traits::implementations::Availability::Validated => {
                crate::sema::dispatch::Dispatcher::new(0, first)
            }
        }
        .map_err(|e| self.error(Query::Node(key), e))?;
        let result = dispatch
            .constant(self.typing.facts, instance)
            .map_err(|e| self.error(Query::Node(key), e))?;
        scope
            .retain(self.typing.facts, key, dispatch.finish())
            .map_err(|e| self.error(Query::Node(key), e))?;
        Ok(result)
    }
    pub(super) fn enter_instance(&mut self, instance: Instance) -> Result<(), Error> {
        let context = self.work.context;
        let key = self.instance_source(instance)?;
        let index = self
            .typing
            .facts
            .initializer_index(instance.definition)
            .ok_or_else(|| {
                self.error(
                    Query::Instance(instance),
                    "missing selected pattern value initializer",
                )
            })?;
        self.begin_pattern_value(index, context)?;
        self.forward(Query::Instance(instance), Query::Node(key));
        self.prepare(key)
    }
    pub(super) fn prepare_instance(&mut self, instance: Instance) -> Result<(), Error> {
        let context = self.work.context;
        let source = self.instance_source(instance)?;
        let index = self
            .typing
            .facts
            .initializer_index(instance.definition)
            .ok_or_else(|| self.error(Query::Instance(instance), "missing prepared initializer"))?;
        self.begin_pattern_value(index, context)?;
        self.prepare(source)
    }
    pub(super) fn instance_source(&mut self, instance: Instance) -> Result<NodeKey, Error> {
        let query = Query::Instance(instance);
        // Instantiations may revisit a large template with distinct arguments.
        self.limit = self.limit.max(16_000_000);
        let index = self
            .typing
            .facts
            .initializer_index(instance.definition)
            .ok_or_else(|| self.error(query, "selected constant has no initializer"))?;
        if !matches!(
            self.typing.facts.initializers[index].kind,
            super::InitializerKind::Const
        ) {
            return Err(self.error(query, "constant instance requires a constant initializer"));
        }
        if !self.typing.facts.initializers[index].type_checked() {
            self.typing.initializer(index)?;
            self.require_complete_patterns(query)?;
        }
        let context = if let Some(&context) = self.instances.identities.get(&instance) {
            context
        } else {
            let substitution = instance
                .substitution(self.typing.facts)
                .map_err(|e| self.error(query, e))?;
            let ordinal = self.instances.environments.len();
            if ordinal >= 1_000_000 {
                return Err(self.error(query, "constant instance work budget exceeded"));
            }
            let context = Context(
                u32::try_from(ordinal + 1)
                    .map_err(|_| self.error(query, "constant instance identity overflow"))?,
            );
            self.instances.environments.push(Environment {
                instance,
                substitution,
            });
            self.instances.identities.insert(instance, context);
            context
        };
        self.work.context = context;
        Ok(self.typing.facts.initializers[index].value)
    }
}
