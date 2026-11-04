//! Owned unsized arguments must remain places; compound temporaries need Sized.
use super::{Checker, Error, Kind, NodeId, TypeId};
impl Checker<'_> {
    pub(super) fn require_sized_operand(&mut self, node: NodeId, ty: TypeId) -> Result<(), Error> {
        if matches!(
            self.kind(node),
            Kind::Block { .. }
                | Kind::If { .. }
                | Kind::Match { .. }
                | Kind::Loop { .. }
                | Kind::Call { .. }
                | Kind::MethodCall { .. }
                | Kind::GenericMethodCall { .. }
                | Kind::Binary { .. }
                | Kind::Cast { .. }
                | Kind::TryBlock { .. }
        ) && self.facts.types.is_sized(ty) != Some(true)
            && !self.body_is_sized(node, ty)?
        {
            return Err(self.error(
                node,
                "an unsized operand must be a place, not a compound temporary",
            ));
        }
        Ok(())
    }
}
