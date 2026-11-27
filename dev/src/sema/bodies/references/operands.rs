//! Value operands honor concrete pointee expectations before producing a borrow.
use super::super::{Checker, Constructor, Dependency, Error, Kind, NodeId, TermId, Work};
use crate::{abi::metadata::Metadata, syntax::ast::Unary, types::aggregate::MetadataPreparation};

/// An unsized rvalue hint describes the outer borrow's coercion target.
#[derive(Clone, Copy)]
pub(in super::super) enum Expectation {
    None,
    Type(TermId),
    UnsizedRvalue(TermId),
}
impl Expectation {
    pub(in super::super) fn from_type(ty: Option<TermId>) -> Self {
        ty.map_or(Self::None, Self::Type)
    }
    pub(in super::super) fn ty(self) -> Option<TermId> {
        match self {
            Self::None => None,
            Self::Type(ty) | Self::UnsizedRvalue(ty) => Some(ty),
        }
    }
    pub(in super::super) fn hard(self) -> Option<TermId> {
        match self {
            Self::Type(ty) => Some(ty),
            Self::None | Self::UnsizedRvalue(_) => None,
        }
    }
    pub(in super::super) fn enter(self, node: NodeId) -> Work {
        match self {
            Self::Type(ty) => Work::Enter(node, Some(ty)),
            Self::None => Work::Enter(node, None),
            Self::UnsizedRvalue(ty) => Work::EnterUnsizedRvalue(node, ty),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum BorrowOperand {
    Place,
    CoercibleValue,
    OtherValue,
}

impl Checker<'_> {
    pub(in super::super) fn borrow_operand_kind(
        &mut self,
        mut node: NodeId,
    ) -> Result<BorrowOperand, Error> {
        loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                    node = value;
                }
                Kind::Expansion {
                    children,
                    tail: Some(tail),
                } if self.facts.source_bindings.is_some()
                    && self
                        .report
                        .package
                        .unit(self.state.file)
                        .configured
                        .ast
                        .list(children)
                        .is_empty() =>
                {
                    node = tail;
                }
                Kind::Block { .. }
                | Kind::Expansion { .. }
                | Kind::If { .. }
                | Kind::Match { .. }
                | Kind::Loop { .. } => {
                    return Ok(BorrowOperand::CoercibleValue);
                }
                Kind::Name(..)
                | Kind::Path { .. }
                | Kind::Qualified { .. }
                | Kind::Field { .. }
                | Kind::Index { .. }
                | Kind::Unary {
                    op: Unary::Dereference,
                    ..
                } => return Ok(BorrowOperand::Place),
                _ => return Ok(BorrowOperand::OtherValue),
            }
        }
    }
    pub(in super::super) fn prepare_borrow_operand(
        &mut self,
        node: NodeId,
        operand: NodeId,
        expected: TermId,
        coerce_value: bool,
    ) -> Result<Option<Dependency>, Error> {
        let head = self.contextual_head(node, expected)?;
        let wide = match head {
            Some(
                Constructor::Slice
                | Constructor::Dynamic { .. }
                | Constructor::Primitive(crate::types::Primitive::Str),
            ) => true,
            Some(Constructor::Adt(..) | Constructor::AdtArgs(..) | Constructor::Tuple) => {
                let ty = match self.state.infer.resolved(&mut self.facts.types, expected) {
                    Ok(ty) => Some(ty),
                    Err(
                        crate::types::infer::Error::Unresolved
                        | crate::types::infer::Error::Normalization,
                    ) => None,
                    Err(e) => return Err(self.error(node, e)),
                };
                // Provisional nominal sizes cannot decide physical metadata.
                // A sealed structural Sized result is enough for a direct
                // rvalue, whose legacy path never demanded field preparation.
                if let Some(ty) = ty
                    && (coerce_value
                        || !self.facts.types.nominal_sizes_sealed()
                        || self.facts.types.is_sized(ty) != Some(true))
                {
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
        if coerce_value && head.is_some() && !wide {
            self.state.work.push(Work::CoerceElement(operand, expected));
        }
        let expectation = if wide && self.facts.source_bindings.is_some() {
            Expectation::UnsizedRvalue(expected)
        } else {
            Expectation::Type(expected)
        };
        self.state.work.push(expectation.enter(operand));
        Ok(None)
    }
}
