//! User traits and builtin auto markers share implementation indexing without fake declarations.
use crate::types::{ItemId, auto_traits::AutoTrait, infer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) enum Identity {
    User(ItemId),
    Auto(AutoTrait),
}
impl From<ItemId> for Identity {
    fn from(item: ItemId) -> Self {
        Self::User(item)
    }
}
impl Identity {
    pub(super) fn goal(goal: &infer::predicates::Predicate) -> Result<Self, infer::Error> {
        if let Some((item, _)) = goal.trait_arguments() {
            return Ok(Self::User(item));
        }
        match goal.builtin_bound() {
            Some(crate::types::parameters::predicates::Bound::Auto(marker)) => {
                Ok(Self::Auto(marker))
            }
            _ => Err(infer::Error::InvalidIdentity),
        }
    }
}
