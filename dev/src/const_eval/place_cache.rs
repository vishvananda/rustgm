//! Cached places share ordinary ancestry without merging enum consumer identities.
use super::{Frame, Machine, memory::Resolved};
use crate::{
    body::ir::{ProjectionId, ProjectionKind},
    types::{Kind, aggregate::Kind as AggregateKind},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Mode {
    Common,
    Value,
    Address,
}
impl Mode {
    pub(super) const fn selected(address: bool) -> Self {
        if address { Self::Address } else { Self::Value }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Key {
    projection: ProjectionId,
    mode: Mode,
}
impl Frame {
    pub(super) fn projected(
        &self,
        projection: ProjectionId,
        mode: Mode,
    ) -> Option<(Resolved, bool)> {
        self.projections
            .get(&Key {
                projection,
                mode: Mode::Common,
            })
            .map(|&place| (place, true))
            .or_else(|| {
                self.projections
                    .get(&Key { projection, mode })
                    .map(|&place| (place, false))
            })
    }
    pub(super) fn remember_projection(
        &mut self,
        projection: ProjectionId,
        mode: Mode,
        common: bool,
        place: Resolved,
    ) {
        self.projections.insert(
            Key {
                projection,
                mode: if common { Mode::Common } else { mode },
            },
            place,
        );
    }
    pub(super) fn forget_projection(&mut self, projection: ProjectionId) {
        for mode in [Mode::Common, Mode::Value, Mode::Address] {
            self.projections.remove(&Key { projection, mode });
        }
    }
}
impl Machine<'_, '_> {
    /// Called only after canonical projection succeeds. Any enum ancestor keeps
    /// its distinction for all descendants; no fresh ancestry walk is needed.
    pub(super) fn common_projection(&self, parent: Resolved, kind: ProjectionKind) -> bool {
        if !matches!(kind, ProjectionKind::Field(_)) {
            return true;
        }
        match self.evaluator.facts().types.kind(parent.ty) {
            Some(Kind::Adt(owner, _)) => self
                .evaluator
                .facts()
                .aggregates
                .get(owner)
                .is_some_and(|definition| definition.kind != AggregateKind::Enum),
            Some(Kind::Tuple(_) | Kind::Closure { .. }) => true,
            _ => false,
        }
    }
}
