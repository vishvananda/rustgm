//! Constant paths infer owner arguments independently of the value's result type.
use super::{Checker, DefId, Error, NodeId, TermId, TypeId};
use crate::{
    sema::{
        generics::SourceArgument,
        values::{Dependency, Instance},
    },
    types::{
        GenericArgs, arguments::Argument as Canonical, infer::arguments::Argument,
        lifetime_proof::ScopeId, parameters::ParameterKind, substitute::Bound,
    },
};

pub(super) struct Pending {
    pub node: NodeId,
    pub owner: DefId,
    pub arguments: Vec<Argument>,
}
impl Checker<'_> {
    pub(in crate::sema) fn constant_instances(&self) -> Result<Vec<Instance>, Error> {
        self.state
            .call_types
            .constants
            .iter()
            .map(|pending| {
                self.facts
                    .constant_uses
                    .get(&self.key(pending.node))
                    .copied()
                    .ok_or_else(|| self.error(pending.node, "missing inferred constant instance"))
            })
            .collect()
    }
    pub(super) fn associated_constant(
        &mut self,
        node: NodeId,
        owner: DefId,
        ty: TypeId,
    ) -> Result<TermId, Error> {
        let schema = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(node, "missing constant owner schema"))?;
        let captures = schema.captures as usize;
        let bounds = schema.bounds.clone();
        let explicit = self
            .facts
            .generics
            .trait_uses
            .get(&self.key(node))
            .and_then(|use_| use_.arguments.as_ref())
            .and_then(|arguments| arguments.lifetimes);
        let regions = self.early_item_arguments(node, explicit, captures)?;
        self.item_bounds(node, &regions, &bounds)?;
        let (_, parameters) = self
            .facts
            .generics
            .callable_parameters(&self.facts.types, owner)
            .map_err(|e| self.error(node, e))?;
        let mut arguments = regions
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>();
        let mut bindings = Vec::with_capacity(parameters.len());
        for id in parameters {
            let argument = self.inferred_parameter(node, id, SourceArgument::Infer)?;
            arguments.push(argument);
            bindings.push((id, argument));
        }
        self.bind_method_owner(node, &arguments, &bindings)?;
        self.bind_trait_arguments(node, &arguments, &bindings)?;
        let ty = Bound::new(regions)
            .apply(&mut self.facts.types, ty)
            .map_err(|e| self.error(node, e))?;
        let projected = self.facts.types.contains_projections(ty) == Some(true);
        let term = self
            .state
            .infer
            .instantiate(&mut self.facts.types, ty, &bindings)
            .map_err(|e| self.error(node, e))?;
        if projected || self.state.projections.active() {
            self.defer_normalization(node, ScopeId::ROOT, term)?;
        }
        self.bound_obligations(node, owner, &arguments, &bindings)?;
        self.state.call_types.constants.push(Pending {
            node,
            owner,
            arguments,
        });
        Ok(term)
    }
    pub(super) fn publish_constant_uses(
        &mut self,
        arguments: Vec<GenericArgs>,
    ) -> Result<(), Error> {
        if arguments.len() != self.state.call_types.constants.len() {
            return Err(self.error(
                self.state.body.node,
                "incomplete constant argument inference",
            ));
        }
        for (index, arguments) in arguments.into_iter().enumerate() {
            let Pending { node, owner, .. } = self.state.call_types.constants[index];
            let (_, parameters) = self
                .facts
                .generics
                .callable_parameters(&self.facts.types, owner)
                .map_err(|e| self.error(node, e))?;
            let actual = self
                .facts
                .types
                .arguments(arguments)
                .ok_or_else(|| self.error(node, "missing constant arguments"))?
                .filter(|a| !matches!(a, Canonical::Lifetime(_)))
                .collect::<Vec<_>>();
            if parameters.len() != actual.len() {
                return Err(self.error(node, "constant owner argument arity mismatch"));
            }
            for (id, argument) in parameters.into_iter().zip(actual) {
                let parameter = self
                    .facts
                    .types
                    .parameter(id)
                    .ok_or_else(|| self.error(node, "missing constant parameter"))?;
                match (parameter.kind, argument) {
                    (ParameterKind::Type { sized }, Canonical::Type(ty)) => {
                        if sized && !self.body_is_sized(node, ty)? {
                            return Err(self.error(node, "constant type argument must be sized"));
                        }
                    }
                    (ParameterKind::Const { .. }, Canonical::Const(_)) => {}
                    _ => return Err(self.error(node, "constant argument kind mismatch")),
                }
            }
            let instance = Instance {
                definition: owner,
                arguments,
            };
            self.facts.constant_uses.insert(self.key(node), instance);
            if !self.state.in_function
                && let Some(owner) = self.state.environment_owner
            {
                self.facts
                    .constant_dependencies
                    .entry(owner)
                    .or_default()
                    .push(Dependency::Constant(instance));
            }
        }
        Ok(())
    }
    pub(super) fn publish_constant_callables(&mut self) -> Result<(), Error> {
        let Some(owner) = self.state.environment_owner.filter(|&owner| {
            !self.state.in_function && self.facts.associated.owner(owner).is_some()
        }) else {
            return Ok(());
        };
        let mut seen = crate::support::hash::HashSet::default();
        for &node in &self.state.touched {
            let ty = self.ty(node)?;
            let Some(crate::types::Kind::FunctionItem { item, .. }) = self.facts.types.kind(ty)
            else {
                continue;
            };
            if self.facts.associated.trait_owner(DefId(item.0)).is_some() && seen.insert(ty) {
                self.facts
                    .constant_dependencies
                    .entry(owner)
                    .or_default()
                    .push(Dependency::Callable(ty));
            }
        }
        Ok(())
    }
}
