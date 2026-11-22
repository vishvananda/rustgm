//! Selected impl arguments share the ordinary callable inference graph.
use super::{Checker, DefId, Error, NodeId};
use crate::types::{
    Kind, TypeId,
    infer::arguments::Argument,
    parameters::{
        ParameterId,
        clauses::Clause,
        predicates::{Bound as PredicateBound, Predicate},
        uses::Uses,
    },
    substitute::Bound,
};

impl Checker<'_> {
    pub(super) fn bind_method_owner(
        &mut self,
        node: NodeId,
        arguments: &[Argument],
        bindings: &[(ParameterId, Argument)],
        signature: Option<TypeId>,
    ) -> Result<(), Error> {
        if self.bind_collection_prefix(node, bindings)?
            || self.bind_library_owner(node, bindings)?
        {
            return Ok(());
        }
        let selected = self.facts.methods.qualified.get(&self.key(node)).copied();
        let dot = self.state.call_types.method_owners.get(&node).copied();
        let Some(implementation) = selected
            .map(|s| s.implementation)
            .or_else(|| dot.map(|d| d.0))
        else {
            return Ok(());
        };

        let schema = self
            .facts
            .associated
            .schema(implementation)
            .ok_or_else(|| self.error(node, "missing selected impl schema"))?;
        let owner = schema.self_type;
        // A projected implementation header relates through the body's
        // normalizer, as an annotated projection does, never structurally.
        let projected = self.facts.types.contains_projections(owner) == Some(true);
        let captures = schema.captures as usize;
        let mut regions = Vec::with_capacity(captures);
        for argument in arguments.iter().take(captures) {
            let Argument::Lifetime(region) = argument else {
                return Err(self.error(node, "missing selected impl lifetime argument"));
            };
            regions.push(*region);
        }
        if regions.len() != captures {
            return Err(self.error(node, "incomplete impl lifetime arguments"));
        }
        let target = Bound::new(regions)
            .apply(&mut self.facts.types, owner)
            .map_err(|e| self.error(node, e))?;
        let target = self
            .state
            .infer
            .instantiate(&mut self.facts.types, target, bindings)
            .map_err(|e| self.error(node, e))?;
        if projected {
            self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, target)?;
        }
        if let Some((_, prefix)) = dot {
            let mut prefix = self.alias_argument(node, prefix)?;
            if self
                .arguments_constrain_receiver(owner, signature)
                .map_err(|e| self.error(node, e))?
            {
                // Leave receiver lifetimes open for the other arguments to
                // constrain. The actual borrow proves the complete directed
                // receiver relation, including mutable invariance.
                prefix = match self.try_inferred_regions(node, prefix)? {
                    Ok(inferred) => inferred,
                    Err(crate::types::infer::Error::Normalization) => prefix,
                    Err(error) => return Err(self.error(node, error)),
                };
            }
            return self.equate(node, target, prefix);
        }
        let selected = selected.ok_or_else(|| self.error(node, "missing selected method owner"))?;
        let prefix = match selected.kind {
            crate::sema::methods::PrefixKind::Nominal(item)
                if !self
                    .facts
                    .generics
                    .annotation_uses
                    .contains_key(&selected.prefix)
                    && self
                        .facts
                        .node(selected.prefix)
                        .is_none_or(|fact| fact.ty.is_none()) =>
            {
                self.nominal_term(node, selected.prefix.node, item, None)?
            }
            _ => self.annotation(selected.prefix.node)?,
        };
        let prefix = self.alias_argument(node, prefix)?;
        self.equate(node, target, prefix)
    }
    fn arguments_constrain_receiver(
        &self,
        owner: TypeId,
        signature: Option<TypeId>,
    ) -> Result<bool, &'static str> {
        let types = &self.facts.types;
        if types.contains_parameters(owner) != Some(true) {
            return Ok(false);
        }
        let Some(Kind::Function { parameters, .. }) = signature.and_then(|ty| types.kind(ty))
        else {
            return Ok(false);
        };
        let inputs = types.list(parameters).ok_or("missing method inputs")?;
        let mut constraining_inputs = inputs
            .iter()
            .copied()
            .skip(1)
            .filter(|&input| types.contains_parameters(input) == Some(true));
        let Some(first) = constraining_inputs.next() else {
            return Ok(false);
        };
        let mut receiver = Uses::constraining();
        receiver.collect(types, owner)?;
        let mut arguments = Uses::constraining();
        for input in std::iter::once(first).chain(constraining_inputs) {
            arguments.collect(types, input)?;
        }
        Ok(receiver
            .parameters
            .iter()
            .any(|parameter| arguments.parameters.contains(parameter)))
    }
    pub(super) fn trait_arguments_constrain_receiver(
        &self,
        method: DefId,
        item_type: TypeId,
    ) -> Result<bool, &'static str> {
        let facts = &self.facts;
        let types = &facts.types;
        let predicates = facts.generics.declared_predicates(method);
        if predicates.is_empty() {
            return Ok(false);
        }
        let Some(receiver) = facts
            .associated
            .trait_owner(method)
            .and_then(|owner| facts.associated.schema(owner))
        else {
            return Ok(false);
        };
        let receiver = receiver.self_type;
        let Some(Kind::FunctionItem { signature, .. }) = types.kind(item_type) else {
            return Ok(false);
        };
        let Some(Kind::Function { parameters, .. }) = types.kind(signature) else {
            return Ok(false);
        };
        let inputs = types
            .list(parameters)
            .ok_or("missing selected trait method inputs")?;
        let mut arguments = Uses::constraining();
        let mut collected = false;
        for predicate in predicates {
            let Clause::Predicate(Predicate {
                ty,
                bound: PredicateBound::ProjectionEqual(result),
            }) = predicate.value
            else {
                continue;
            };
            let (Some(Kind::Projection(left)), Some(Kind::Projection(right))) =
                (types.kind(ty), types.kind(result))
            else {
                continue;
            };
            let other = if left.subject == receiver {
                right.subject
            } else if right.subject == receiver {
                left.subject
            } else {
                continue;
            };
            let Some(Kind::Parameter(parameter)) = types.kind(other) else {
                continue;
            };
            if !collected {
                for &input in inputs.iter().skip(1) {
                    arguments.collect(types, input)?;
                }
                collected = true;
            }
            if arguments.parameters.contains(&parameter) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn bind_library_owner(
        &mut self,
        node: NodeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<bool, Error> {
        if let Some(prefix) = self.facts.methods.length_prefix(self.key(node)) {
            let length = self
                .facts
                .methods
                .demanded_length()
                .ok_or_else(|| self.error(node, "missing selected slice method"))?;
            let prefix = self.annotation(prefix.node)?;
            let prefix = self.alias_argument(node, prefix)?;
            if !matches!(self.head(node, prefix)?, Some(super::Constructor::Slice)) {
                return Err(self.error(node, "slice method prefix is not a slice"));
            }
            let element = self.child(node, prefix, 0)?;
            let target = bindings
                .iter()
                .find_map(|(id, arg)| {
                    if *id == length.parameter {
                        if let Argument::Type(ty) = arg {
                            Some(*ty)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .ok_or_else(|| self.error(node, "missing inherited slice element argument"))?;
            self.equate(node, element, target)?;
            return Ok(true);
        }
        if let Some((prefix, method)) = self.facts.methods.pointer_prefix(self.key(node)) {
            let prefix = self.annotation(prefix.node)?;
            let prefix = self.alias_argument(node, prefix)?;
            let Some(super::Constructor::Pointer { mutable }) = self.head(node, prefix)? else {
                return Err(self.error(node, "pointer method prefix is not a pointer"));
            };
            if !matches!(self.facts.types.kind(method.receiver), Some(crate::types::Kind::Pointer { mutable: expected, .. }) if mutable == expected)
            {
                return Err(self.error(node, "pointer method prefix has wrong mutability"));
            }
            let element = self.child(node, prefix, 0)?;
            let target = bindings
                .iter()
                .find_map(|(id, arg)| match arg {
                    Argument::Type(ty) if *id == method.parameter => Some(*ty),
                    _ => None,
                })
                .ok_or_else(|| self.error(node, "missing inherited pointer element argument"))?;
            self.equate(node, element, target)?;
            return Ok(true);
        }
        Ok(false)
    }
    fn bind_collection_prefix(
        &mut self,
        node: NodeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<bool, Error> {
        if let Some((prefix, parameter)) = self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .and_then(|r| {
                let prefix = r.prefixes.get(&self.key(node)).copied()?;
                let crate::resolve::model::Target::Definition(method) =
                    self.facts.node(self.key(node))?.reference?
                else {
                    return None;
                };
                let original = r.original(method).filter(|o| o.declaration.kind.method())?;
                let crate::types::Kind::Parameter(parameter) =
                    self.facts.types.kind(original.parameter)?
                else {
                    return None;
                };
                Some((prefix, parameter))
            })
        {
            let annotation = self.annotation(prefix.node)?;
            let prefix = self.alias_argument(node, annotation)?;
            if !matches!(self.head(node, prefix)?, Some(super::Constructor::Slice)) {
                return Err(self.error(node, "slice method prefix is not a slice"));
            }
            let element = self.child(node, prefix, 0)?;
            let target = bindings
                .iter()
                .find_map(|(id, arg)| match arg {
                    Argument::Type(ty) if *id == parameter => Some(*ty),
                    _ => None,
                })
                .ok_or_else(|| self.error(node, "missing inherited slice element argument"))?;
            self.equate(node, element, target)?;
            return Ok(true);
        }
        Ok(false)
    }
}
