//! Closure interfaces transport inputs and retain all enclosing generic arguments.
use super::{
    Argument, Checker, Constructor, Error, NodeId, Refinement, TermId, contextual, universes,
};
use crate::resolve::model::DefId;
use crate::types::{ItemId, Kind, RegionId, arguments::ConstKind, parameters::ParameterKind};

pub(super) struct Inputs<'a> {
    pub terms: &'a [TermId],
    pub self_ty: TermId,
    pub bindings: Vec<(DefId, TermId)>,
}
pub(super) struct Imported {
    pub signature: TermId,
    pub self_ty: TermId,
    pub bindings: Vec<(DefId, TermId)>,
}
impl Checker<'_> {
    pub(super) fn closure_slot_ownership(
        &mut self,
        node: NodeId,
        closure: TermId,
        signature: TermId,
        slots: &[TermId],
        bound_regions: u32,
    ) -> Result<(), Error> {
        if bound_regions != 0 {
            self.state
                .infer
                .define_signature_slots(&self.facts.types, closure, signature, slots)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }

    pub(in crate::sema::bodies) fn nested_projection_environment(
        &mut self,
        node: NodeId,
        lifetimes: &mut universes::Interface,
    ) -> Result<Option<std::sync::Arc<crate::types::parameters::environment::Environment>>, Error>
    {
        if self
            .state
            .method_environment
            .as_ref()
            .is_none_or(|environment| !environment.requires_region_mapping())
        {
            return Ok(None);
        }
        let Some(parent) = self.projection_environment(node)? else {
            return Ok(None);
        };
        let before = lifetimes.opening.steps;
        let count = lifetimes.count().map_err(|e| self.error(node, e))?;
        let environment = parent
            .substitute_regions(&mut self.facts.types, &mut lifetimes.opening)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(lifetimes.opening.steps - before)
            .map_err(|e| self.error(node, e))?;
        if lifetimes.count().map_err(|e| self.error(node, e))? != count {
            return Err(self.error(node, "projection caller introduces an unimported lifetime"));
        }
        Ok(Some(std::sync::Arc::new(environment)))
    }

    pub(super) fn closure_signature_inputs(
        &mut self,
        node: NodeId,
        parameters: crate::syntax::ast::List,
    ) -> Result<(Vec<NodeId>, Vec<TermId>), Error> {
        let nodes: Vec<_> = self
            .report
            .package
            .unit(self.state.file)
            .configured
            .ast
            .list(parameters)
            .iter()
            .copied()
            .filter(|&node| self.active(node))
            .collect();
        let mut terms = Vec::with_capacity(nodes.len() + 1);
        for _ in 0..=nodes.len() {
            terms.push(self.variable(node, super::Domain::Any)?);
        }
        Ok((nodes, terms))
    }

    pub(in crate::sema::bodies) fn closure_pointer_environment(
        &mut self,
        node: NodeId,
        source: TermId,
    ) -> Result<(), Error> {
        if matches!(self.head(node, source)?, Some(Constructor::Closure(..))) {
            let environment = self.child(node, source, 1)?;
            let empty = self.known(node, crate::types::TypeId::UNIT)?;
            self.equate(node, environment, empty)?;
        }
        Ok(())
    }

    pub(super) fn closure_body_interface(
        &mut self,
        node: NodeId,
        inputs: Inputs<'_>,
        parameters: u32,
        lifetimes: &mut universes::Interface,
        mut expected: Option<contextual::Expected>,
    ) -> Result<(Refinement, Imported), Error> {
        let mut roots: Vec<_> = inputs.terms.iter().copied().map(Argument::Type).collect();
        roots.push(Argument::Type(inputs.self_ty));
        roots.extend(
            inputs
                .bindings
                .iter()
                .map(|&(_, term)| Argument::Type(term)),
        );
        let bindings = inputs.bindings;
        if let Some(contextual::Expected {
            signature: contextual::Signature::Inferred(signature),
            ..
        }) = expected
        {
            roots.push(Argument::Type(signature));
        }
        let (mut inputs, mut roots) =
            Refinement::new(&self.facts.types, &mut self.state.infer, &roots)
                .map_err(|e| self.error(node, e))?;
        if let Some(contextual::Expected {
            signature: contextual::Signature::Inferred(signature),
            ..
        }) = &mut expected
        {
            let Some(Argument::Type(imported)) = roots.pop() else {
                return Err(self.error(node, "missing contextual signature transport"));
            };
            *signature = imported;
        }
        let mut terms = roots
            .into_iter()
            .map(|root| match root {
                Argument::Type(term) => Ok(term),
                _ => Err(self.error(node, "missing closure signature transport")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        // The callable retains its own binder while enclosing lifetimes and
        // free locals open together in this body's one lifetime namespace.
        let outer = terms.split_off(parameters as usize + 1);
        let outer = inputs
            .context
            .map_region_roots(&mut self.facts.types, &outer, &mut lifetimes.opening)
            .map_err(|e| self.error(node, e))?;
        let mut outer = outer.into_iter();
        let self_ty = outer
            .next()
            .ok_or_else(|| self.error(node, "missing closure self transport"))?;
        let bindings = bindings
            .into_iter()
            .zip(outer)
            .map(|((definition, _), term)| (definition, term))
            .collect();
        let signature = inputs
            .context
            .construct(
                Constructor::Function {
                    bound_regions: lifetimes.count().map_err(|e| self.error(node, e))?,
                    parameters,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &terms,
            )
            .map_err(|e| self.error(node, e))?;
        if let Some(expected) = expected {
            contextual::constrain(
                &mut self.facts.types,
                &mut inputs.context,
                signature,
                lifetimes,
                expected,
            )
            .map_err(|e| self.error(node, e))?;
        }
        Ok((
            inputs,
            Imported {
                signature,
                self_ty,
                bindings,
            },
        ))
    }

    pub(super) fn closure_arguments(&mut self, node: NodeId) -> Result<Vec<Argument>, Error> {
        let Some(owner) = self.state.environment_owner else {
            return Ok(Vec::new());
        };
        let parameters = if let Some(parameters) = self
            .facts
            .lifetimes
            .callables
            .generic_parameters(ItemId(owner.0))
        {
            parameters.to_vec()
        } else {
            self.facts
                .generics
                .callable_parameters(&self.facts.types, owner)
                .map_err(|e| self.error(node, e))?
                .1
        };
        let mut arguments = Vec::with_capacity(self.state.body_binders as usize + parameters.len());
        for index in 0..self.state.body_binders {
            let index = u16::try_from(index)
                .map_err(|_| self.error(node, "closure lifetime binder exhausted"))?;
            let region = RegionId::bound(0, index).map_err(|e| self.error(node, e))?;
            arguments.push(Argument::Lifetime(self.alias_region(node, region)?));
        }
        for id in parameters {
            let parameter = self
                .facts
                .types
                .parameter(id)
                .ok_or_else(|| self.error(node, "missing closure owner parameter"))?;
            arguments.push(match parameter.kind {
                ParameterKind::Type { .. } => {
                    let ty = self
                        .facts
                        .types
                        .intern(Kind::Parameter(id))
                        .map_err(|e| self.error(node, e))?;
                    Argument::Type(self.known(node, ty)?)
                }
                ParameterKind::Const { .. } => {
                    let constant = self
                        .facts
                        .types
                        .intern_const(ConstKind::Parameter(id))
                        .map_err(|e| self.error(node, e))?;
                    Argument::Const(
                        self.state
                            .infer
                            .known_const(&self.facts.types, constant)
                            .map_err(|e| self.error(node, e))?,
                    )
                }
            });
        }
        Ok(arguments)
    }
}
