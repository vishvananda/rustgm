//! One body alias and its declaration slot share one fresh scoped identity.
use super::{Action, Bound, RegionId};
use crate::types::relations::BodyAliases;
use std::sync::Arc;

impl Bound {
    pub(crate) fn body_scope(
        arguments: Vec<RegionId>,
        aliases: Arc<BodyAliases>,
    ) -> Result<Self, &'static str> {
        if arguments.len() != aliases.len() {
            return Err("body alias slots do not match the opened declaration scope");
        }
        if arguments.iter().any(|region| {
            *region == RegionId::STATIC
                || region.bound_parts().is_some()
                || aliases.slot(*region).is_some()
        }) {
            return Err("body opening overlaps its retained universal aliases");
        }
        let mut fold = Self::new(arguments);
        fold.steps = aliases.len();
        fold.action = Action::BodyScope(aliases);
        Ok(fold)
    }
}
