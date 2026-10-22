//! Value operands honor concrete pointee expectations before producing a borrow.
use super::super::{Checker, Constructor, Dependency, Error, Kind, NodeId, TermId, Work};
use crate::{abi::metadata::Metadata, types::aggregate::MetadataPreparation};

impl Checker<'_> {
    pub(in super::super) fn value_borrow_operand(
        &mut self,
        mut node: NodeId,
    ) -> Result<bool, Error> {
        loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                    node = value;
                }
                kind => {
                    return Ok(matches!(
                        kind,
                        Kind::Block { .. }
                            | Kind::Expansion { .. }
                            | Kind::If { .. }
                            | Kind::Match { .. }
                            | Kind::Loop { .. }
                    ));
                }
            }
        }
    }
    pub(in super::super) fn prepare_borrow_operand(
        &mut self,
        node: NodeId,
        operand: NodeId,
        expected: TermId,
    ) -> Result<Option<Dependency>, Error> {
        let weak = match self.contextual_head(node, expected)? {
            // An inference hole supplies no pointee conversion yet. Let the
            // final outer coercion discover it, without replaying a value move.
            None
            | Some(
                Constructor::Slice
                | Constructor::Dynamic { .. }
                | Constructor::Primitive(crate::types::Primitive::Str),
            ) => true,
            Some(Constructor::Adt(..) | Constructor::AdtArgs(..) | Constructor::Tuple) => {
                let ty = match self.state.infer.resolved(&mut self.facts.types, expected) {
                    Ok(ty) => Some(ty),
                    Err(crate::types::infer::Error::Unresolved) => None,
                    Err(e) => return Err(self.error(node, e)),
                };
                if let Some(ty) = ty {
                    match self
                        .facts
                        .aggregates
                        .prepare_physical_metadata(&mut self.facts.types, ty)
                        .map_err(|e| self.error(node, e))?
                    {
                        MetadataPreparation::Fields(item) => {
                            return Ok(Some(Dependency {
                                requirement: super::super::readiness::Requirement::Fields(item),
                                source: self.key(node),
                            }));
                        }
                        MetadataPreparation::Unresolved(_) => false,
                        MetadataPreparation::Ready => matches!(
                            self.facts
                                .layouts
                                .prepared_pointer_metadata(
                                    &self.facts.types,
                                    &self.facts.aggregates,
                                    ty
                                )
                                .map_err(|e| self.error(node, e))?,
                            Metadata::Length | Metadata::VTable(_)
                        ),
                    }
                } else {
                    false
                }
            }
            _ => false,
        };
        if !weak {
            self.state.work.push(Work::CoerceElement(operand, expected));
        }
        self.state.work.push(Work::Enter(operand, Some(expected)));
        Ok(None)
    }
}
