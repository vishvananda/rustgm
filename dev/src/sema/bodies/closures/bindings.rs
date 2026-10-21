//! Lexical imports share inference holes without publishing another body's bindings.
use super::{Checker, Error, NodeId, TermId};
use crate::resolve::model::DefId;

pub(super) struct Constructed {
    pub identity: crate::types::closure::ClosureId,
    pub self_ty: TermId,
    pub bindings: Vec<(DefId, TermId)>,
}
impl Checker<'_> {
    pub(super) fn closure_identity(
        &mut self,
        node: NodeId,
        signature: TermId,
    ) -> Result<Constructed, Error> {
        let identity = self
            .facts
            .types
            .reserve_closure()
            .map_err(|e| self.error(node, e))?;
        let bindings = self.closure_bindings(node)?;
        let environment = if bindings.is_empty() {
            self.known(node, crate::types::TypeId::UNIT)?
        } else {
            self.variable(node, super::Domain::Any)?
        };
        let arguments = self.closure_arguments(node)?;
        let self_ty = self
            .state
            .infer
            .closure(identity, signature, environment, &arguments)
            .map_err(|e| self.error(node, e))?;
        Ok(Constructed {
            identity,
            self_ty,
            bindings,
        })
    }
    pub(super) fn closure_bindings(&mut self, node: NodeId) -> Result<Vec<(DefId, TermId)>, Error> {
        let scope = self
            .report
            .resolution
            .node_scopes
            .get(&(self.state.module, node))
            .copied()
            .ok_or_else(|| self.error(node, "missing closure lexical scope"))?;
        let source = self.key(node);
        let imports = self
            .facts
            .closures
            .lexical_bindings(&self.report.resolution, scope)
            .map_err(|message| {
                crate::sema::model::Declarations::error(self.report, source, message)
            })?
            .to_vec();
        let mut regions = crate::types::infer::RegionMapping::default();
        imports
            .into_iter()
            .map(|definition| {
                let term = self.state.locals[definition.0 as usize].ok_or_else(|| {
                    self.error(node, "closure import is outside its defining body")
                })?;
                // Enclosing bound slots belong to the parent's body, whereas
                // the closure signature introduces its own public binder.
                // Refresh all lexical roots through the same cached mapping.
                let term = if self.state.body_binders == 0 {
                    term
                } else {
                    self.alias_inferred_mapped(node, term, &mut regions)?
                };
                Ok((definition, term))
            })
            .collect()
    }
}
