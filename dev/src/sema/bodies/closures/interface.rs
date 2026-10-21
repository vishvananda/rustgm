//! Closure interfaces transport inputs and retain all enclosing generic arguments.
use super::{
    Argument, Checker, Constructor, Error, NodeId, Refinement, TermId, contextual, universes,
};
use crate::types::{ItemId, Kind, RegionId, arguments::ConstKind, parameters::ParameterKind};
impl Checker<'_> {
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
        terms: &[TermId],
        parameters: u32,
        lifetimes: &mut universes::Interface,
        expected: Option<contextual::Expected>,
    ) -> Result<(Refinement, TermId), Error> {
        let (mut inputs, roots) = Refinement::new(
            &self.facts.types,
            &mut self.state.infer,
            &terms
                .iter()
                .copied()
                .map(Argument::Type)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| self.error(node, e))?;
        let terms = roots
            .into_iter()
            .map(|root| match root {
                Argument::Type(term) => Ok(term),
                _ => Err(self.error(node, "missing closure signature transport")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let signature = inputs
            .context
            .construct(
                Constructor::Function {
                    bound_regions: lifetimes.count().map_err(|e| self.error(node, e))?,
                    parameters,
                    unsafe_: false,
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
        Ok((inputs, signature))
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
