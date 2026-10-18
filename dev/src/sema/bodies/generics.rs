//! Function item inference consumes elaborated arguments and one checked template.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId, Type, TypeId};
use crate::{
    sema::{generics::SourceArgument, model::BodyRegionOrigin, traits::selection::Source},
    types::{
        arguments::Argument as Canonical,
        infer::{self, arguments::Argument},
        parameters::ParameterKind,
    },
};

impl Checker<'_> {
    pub(super) fn generic_function(
        &mut self,
        node: NodeId,
        owner: DefId,
        ty: TypeId,
    ) -> Result<TermId, Error> {
        let supplied = self
            .facts
            .generics
            .item_uses
            .get(&self.key(node))
            .ok_or_else(|| self.error(node, "generic function arguments were not elaborated"))?
            .arguments
            .clone();
        let ty = self.function_item(node, owner, ty)?;
        let Some(Type::FunctionItem {
            item,
            signature,
            arguments,
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(node, "missing generic callable template"));
        };
        let parameters = self.callable_parameters(node, owner)?;
        let mut arguments = self
            .facts
            .types
            .region_args(arguments)
            .ok_or_else(|| self.error(node, "invalid early lifetime arguments"))?
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>();
        let mut bindings = Vec::with_capacity(parameters.len());
        let mut own = 0;
        for id in parameters {
            let parameter = self
                .facts
                .types
                .parameter(id)
                .ok_or_else(|| self.error(node, "missing callable parameter"))?;
            let supplied = if parameter.owner == item {
                let argument = supplied.get(own).copied().unwrap_or(SourceArgument::Infer);
                own += 1;
                argument
            } else {
                SourceArgument::Infer
            };
            let argument = self.inferred_parameter(node, id, supplied)?;
            arguments.push(argument);
            bindings.push((id, argument));
        }
        self.bind_method_owner(node, &arguments, &bindings)?;
        self.bind_trait_arguments(node, &arguments, &bindings)?;
        let projected_signature = self.facts.types.contains_projections(signature) == Some(true);
        let mut substitution =
            infer::Instantiation::new(&mut self.state.infer, &self.facts.types, &bindings)
                .map_err(|e| self.error(node, e))?;
        let signature = self
            .state
            .infer
            .instantiate_with(&mut self.facts.types, signature, &mut substitution)
            .map_err(|e| self.error(node, e))?;
        if projected_signature
            || substitution.shifted_projections()
            || self.state.projections.active()
        {
            self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, signature)?;
        }
        self.bound_obligations(node, owner, &arguments, &bindings)?;
        let term = self
            .state
            .infer
            .function_item(item, signature, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.state.call_types.generic.push((node, owner, term));
        Ok(term)
    }
    fn callable_parameters(
        &self,
        node: NodeId,
        owner: DefId,
    ) -> Result<Vec<crate::types::parameters::ParameterId>, Error> {
        if let Some(parameters) = self
            .facts
            .lifetimes
            .callables
            .generic_parameters(crate::types::ItemId(owner.0))
        {
            return Ok(parameters.to_vec());
        }
        if self.facts.generics.has_parameters(owner) {
            return Err(self.error(node, "missing callable generic parameter order"));
        }
        Ok(Vec::new())
    }
    pub(super) fn inferred_argument(
        &mut self,
        node: NodeId,
        formal: Canonical,
        supplied: SourceArgument,
    ) -> Result<(crate::types::parameters::ParameterId, Argument), Error> {
        let id = crate::sema::generics::parameter_id(&self.facts.types, formal)
            .map_err(|e| self.error(node, e))?;
        Ok((id, self.inferred_parameter(node, id, supplied)?))
    }
    pub(super) fn inferred_parameter(
        &mut self,
        node: NodeId,
        id: crate::types::parameters::ParameterId,
        supplied: SourceArgument,
    ) -> Result<Argument, Error> {
        let parameter = self
            .facts
            .types
            .parameter(id)
            .ok_or_else(|| self.error(node, "missing generic parameter"))?;
        Ok(match parameter.kind {
            ParameterKind::Type { .. } => {
                let term = match supplied {
                    SourceArgument::Type(key) => {
                        let term = self.annotation(key.node)?;
                        self.alias_argument(node, term)?
                    }
                    SourceArgument::Infer => self.variable(node, infer::Domain::Any)?,
                    SourceArgument::Const(_) => {
                        return Err(self.error(node, "type parameter requires a type argument"));
                    }
                };
                Argument::Type(term)
            }
            ParameterKind::Const { ty } => {
                let term = match supplied {
                    SourceArgument::Const(id) => {
                        self.state.infer.known_const(&self.facts.types, id)
                    }
                    SourceArgument::Infer => self.state.infer.const_variable(&self.facts.types, ty),
                    SourceArgument::Type(_) => {
                        return Err(self.error(node, "const parameter requires a const argument"));
                    }
                }
                .map_err(|e| self.error(node, e))?;
                Argument::Const(term)
            }
        })
    }
    pub(super) fn alias_argument(&mut self, node: NodeId, term: TermId) -> Result<TermId, Error> {
        let transformed = match self.state.infer.resolved(&mut self.facts.types, term) {
            Ok(ty) => {
                let ty = self.alias_type(node, ty)?;
                self.known(node, ty)
            }
            Err(infer::Error::Unresolved) => {
                self.prepare_universals(node)?;
                let regions = self.state.call_types.universal_arguments();
                self.state
                    .infer
                    .substitute_regions(&mut self.facts.types, term, &regions)
                    .map_err(|e| self.error(node, e))
            }
            Err(e) => Err(self.error(node, e)),
        }?;
        if self.state.projections.active() && transformed != term {
            self.defer_normalization(
                node,
                crate::types::lifetime_proof::ScopeId::ROOT,
                transformed,
            )?;
        }
        Ok(transformed)
    }
    pub(super) fn instantiate_call_term(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<TermId, Error> {
        let Some(Constructor::Function {
            bound_regions,
            parameters,
            unsafe_,
        }) = self.head(node, term)?
        else {
            return Err(self.error(node, "missing inferred callable signature"));
        };
        let mut arguments = Vec::with_capacity(bound_regions as usize);
        for _ in 0..bound_regions {
            arguments.push(self.body_region(node, BodyRegionOrigin::CallArgument)?);
        }
        let mut children = Vec::with_capacity(parameters as usize + 1);
        for index in 0..=parameters {
            let child = self.child(node, term, index as usize)?;
            let child = self
                .state
                .infer
                .substitute_regions(&mut self.facts.types, child, &arguments)
                .map_err(|e| self.error(node, e))?;
            children.push(self.alias_argument(node, child)?);
        }
        self.state
            .infer
            .construct(
                Constructor::Function {
                    bound_regions: 0,
                    parameters,
                    unsafe_,
                },
                &children,
            )
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn validate_generic_uses(
        &mut self,
        solved: &infer::Solved,
        predicates: &[crate::types::parameters::predicates::Predicate],
        evidence: &std::collections::HashMap<usize, Source>,
    ) -> Result<(), Error> {
        for index in 0..self.state.call_types.generic.len() {
            let (node, owner, term) = self.state.call_types.generic[index];
            let ty = *solved
                .types
                .get(term.0 as usize)
                .ok_or_else(|| self.error(node, "missing inferred item type"))?;
            let Some(Type::FunctionItem { arguments, .. }) = self.facts.types.kind(ty) else {
                return Err(self.error(node, "missing inferred item arguments"));
            };
            let arguments = self
                .facts
                .types
                .arguments(arguments)
                .ok_or_else(|| self.error(node, "invalid inferred arguments"))?
                .filter(|a| !matches!(a, Canonical::Lifetime(_)))
                .collect::<Vec<_>>();
            let mut arguments = arguments.into_iter();
            let parameters = self.callable_parameters(node, owner)?;
            for id in parameters {
                let parameter = self
                    .facts
                    .types
                    .parameter(id)
                    .ok_or_else(|| self.error(node, "missing inferred parameter"))?;
                let argument = arguments
                    .next()
                    .ok_or_else(|| self.error(node, "missing inferred generic argument"))?;
                if !matches!(
                    (parameter.kind, argument),
                    (ParameterKind::Type { .. }, Canonical::Type(_))
                        | (ParameterKind::Const { .. }, Canonical::Const(_))
                ) {
                    return Err(self.error(node, "inferred generic argument kind mismatch"));
                }
                if let (ParameterKind::Type { sized: true }, Canonical::Type(actual)) =
                    (parameter.kind, argument)
                    && !self.body_shape(node, actual)?.sized
                {
                    return Err(self.error(node, "generic type argument must be sized"));
                }
            }
            if arguments.next().is_some() {
                return Err(self.error(node, "unexpected inferred generic argument"));
            }
        }
        for (index, predicate) in predicates.iter().enumerate() {
            let node = self.state.call_types.obligations[index].0;
            let predicate = match evidence.get(&index) {
                Some(Source::Assumption(actual) | Source::ResultBound(actual)) => *actual,
                Some(Source::Implementation { .. }) => continue,
                Some(Source::QuantifiedAssumption {
                    clause,
                    subject,
                    application,
                }) => {
                    if !self
                        .trait_assumptions(node)?
                        .is_some_and(|assumptions| assumptions.contains_quantified(*clause))
                    {
                        return Err(self.error(
                            node,
                            "quantified evidence is outside this caller environment",
                        ));
                    }
                    crate::types::parameters::predicates::Predicate {
                        ty: *subject,
                        bound: crate::types::parameters::predicates::Bound::Trait(*application),
                    }
                }
                Some(Source::Equality) => {
                    return Err(self.error(node, "equality evidence cannot replace a trait goal"));
                }
                Some(Source::Builtin) => {
                    return Err(self.error(node, "builtin evidence cannot replace a trait goal"));
                }
                None => *predicate,
            };
            self.prove_bound(node, predicate.ty, predicate.bound)?;
        }
        Ok(())
    }
}
