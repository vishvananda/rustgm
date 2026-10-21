//! Initializer owners determine global-memory eligibility independently of raw access.
use super::{Failure, Machine};
use crate::{body::ir::Owner, sema::model::InitializerKind, types::TypeId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EvaluationContext {
    Constant,
    Static,
}
impl EvaluationContext {
    pub(super) fn for_owner(owner: Owner, parent: Self) -> Self {
        match owner {
            Owner::Initializer {
                kind: InitializerKind::Static { .. },
                ..
            } => Self::Static,
            Owner::Initializer { .. }
            | Owner::InitializerInstance { .. }
            | Owner::AnonymousConstant
            | Owner::AnonymousInstance(_) => Self::Constant,
            Owner::Function { .. } | Owner::Closure { .. } | Owner::ClosureAdapter { .. } => parent,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadPurpose {
    Value,
    ReferenceValidation,
}
impl Machine<'_, '_> {
    pub(super) fn frame_context(&self, owner: Owner) -> EvaluationContext {
        EvaluationContext::for_owner(
            owner,
            self.frames
                .last()
                .map_or(EvaluationContext::Constant, |frame| frame.context),
        )
    }
    pub(super) fn check_static_value_read(
        &mut self,
        root: TypeId,
        leaf: TypeId,
        purpose: ReadPurpose,
    ) -> Result<(), Failure> {
        if purpose == ReadPurpose::ReferenceValidation
            || self.context == EvaluationContext::Static
            || !self.evaluator.facts().aggregates.has_shared_mutation()
        {
            return Ok(());
        }
        // Even an ordinary subfield belongs to the interior-mutable allocation.
        // A zero-byte read does not access that allocation's mutable contents.
        if self.footprint(leaf)?.size == 0 {
            return Ok(());
        }
        self.charge(1)?;
        if self.memory.ordinary_static_types.contains(&root) {
            return Ok(());
        }
        let (answer, work) = {
            let facts = self.evaluator.facts();
            let mut query =
                crate::types::interior_mutability::Query::new(&facts.types, &facts.aggregates);
            let answer = query.contains(root)?;
            (answer, query.work().steps)
        };
        self.charge(work)?;
        match answer {
            crate::types::interior_mutability::Answer::Ordinary => {
                // Four logical slots cover the retained hash entry's capacity.
                self.reserve_cells(4)?;
                self.memory.ordinary_static_types.insert(root);
                Ok(())
            }
            crate::types::interior_mutability::Answer::Interior => {
                Err("constant execution cannot read interior-mutable static storage".into())
            }
            crate::types::interior_mutability::Answer::Unresolved => {
                Err("constant static storage mutation facts are unfinished".into())
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/static_read_work.rs"]
mod tests;
