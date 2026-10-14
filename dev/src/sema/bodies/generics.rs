//! Function item inference consumes elaborated arguments and one checked template.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId, Type, TypeId};
use crate::{
    sema::{generics::SourceArgument, model::BodyRegionOrigin},
    types::{
        arguments::{Argument as Canonical, ConstKind},
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
        let parameters = self.facts.generics.parameters[&owner].clone();
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
        for (index, parameter) in parameters.iter().enumerate() {
            let supplied = supplied
                .get(index)
                .copied()
                .unwrap_or(SourceArgument::Infer);
            let (id, argument) = self.inferred_argument(node, parameter.argument, supplied)?;
            arguments.push(argument);
            bindings.push((id, argument));
        }
        let signature = self
            .infer
            .instantiate(&mut self.facts.types, signature, &bindings)
            .map_err(|e| self.error(node, e))?;
        self.bound_obligations(node, owner, &arguments, &bindings)?;
        let term = self
            .infer
            .function_item(item, signature, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.call_types.generic.push((node, owner, term));
        Ok(term)
    }
    pub(super) fn inferred_argument(
        &mut self,
        node: NodeId,
        formal: Canonical,
        supplied: SourceArgument,
    ) -> Result<(crate::types::parameters::ParameterId, Argument), Error> {
        Ok(match formal {
            Canonical::Type(ty) => {
                let Some(Type::Parameter(id)) = self.facts.types.kind(ty) else {
                    return Err(self.error(node, "missing rigid type parameter"));
                };
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
                (id, Argument::Type(term))
            }
            Canonical::Const(constant) => {
                let Some(ConstKind::Parameter(id)) = self.facts.types.const_kind(constant) else {
                    return Err(self.error(node, "missing rigid const parameter"));
                };
                let ty = self
                    .facts
                    .types
                    .const_type(constant)
                    .ok_or_else(|| self.error(node, "missing const parameter type"))?;
                let term = match supplied {
                    SourceArgument::Const(id) => self.infer.known_const(&self.facts.types, id),
                    SourceArgument::Infer => self.infer.const_variable(&self.facts.types, ty),
                    SourceArgument::Type(_) => {
                        return Err(self.error(node, "const parameter requires a const argument"));
                    }
                }
                .map_err(|e| self.error(node, e))?;
                (id, Argument::Const(term))
            }
            Canonical::Lifetime(_) => {
                return Err(self.error(node, "late lifetime was retained as a type argument"));
            }
        })
    }
    pub(super) fn alias_argument(&mut self, node: NodeId, term: TermId) -> Result<TermId, Error> {
        match self.infer.resolved(&mut self.facts.types, term) {
            Ok(ty) => {
                let ty = self.alias_type(node, ty)?;
                self.known(node, ty)
            }
            Err(infer::Error::Unresolved) => {
                self.prepare_universals(node)?;
                let regions = self.call_types.universal_arguments();
                self.infer
                    .substitute_regions(&mut self.facts.types, term, &regions)
                    .map_err(|e| self.error(node, e))
            }
            Err(e) => Err(self.error(node, e)),
        }
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
                .infer
                .substitute_regions(&mut self.facts.types, child, &arguments)
                .map_err(|e| self.error(node, e))?;
            children.push(self.alias_argument(node, child)?);
        }
        self.infer
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
    pub(super) fn validate_generic_uses(&mut self, solved: &infer::Solved) -> Result<(), Error> {
        for index in 0..self.call_types.generic.len() {
            let (node, owner, term) = self.call_types.generic[index];
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
            let parameters = self.facts.generics.parameters[&owner].clone();
            for parameter in &parameters {
                let argument = arguments
                    .next()
                    .ok_or_else(|| self.error(node, "missing inferred generic argument"))?;
                if !matches!(
                    (parameter.argument, argument),
                    (Canonical::Type(_), Canonical::Type(_))
                        | (Canonical::Const(_), Canonical::Const(_))
                ) {
                    return Err(self.error(node, "inferred generic argument kind mismatch"));
                }
                if let (Canonical::Type(formal), Canonical::Type(actual)) =
                    (parameter.argument, argument)
                    && let Some(Type::Parameter(id)) = self.facts.types.kind(formal)
                    && self
                        .facts
                        .types
                        .parameter(id)
                        .is_some_and(|p| matches!(p.kind, ParameterKind::Type { sized: true }))
                    && !self.body_shape(node, actual)?.sized
                {
                    return Err(self.error(node, "generic type argument must be sized"));
                }
            }
            if arguments.next().is_some() {
                return Err(self.error(node, "unexpected inferred generic argument"));
            }
        }
        for index in 0..self.call_types.obligations.len() {
            let (node, bound, term) = self.call_types.obligations[index];
            let ty = *solved
                .types
                .get(term.0 as usize)
                .ok_or_else(|| self.error(node, "missing inferred bound subject"))?;
            self.prove_bound(node, ty, bound)?;
        }
        Ok(())
    }
}
