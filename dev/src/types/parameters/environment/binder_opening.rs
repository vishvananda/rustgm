//! A body query opens producer-owned aliases together with their declaration slots.
use super::Environment;
use crate::types::{RegionId, relations::BodyAliases, substitute::Bound};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/lifetime_opening.rs"]
mod tests;

impl Environment {
    /// The semantic body producer supplies these aliases in declaration slot order.
    /// This preserves the declaration environment and adds no trait permission.
    pub(crate) fn with_body_aliases(
        mut self,
        aliases: Vec<RegionId>,
    ) -> Result<Self, &'static str> {
        if aliases.is_empty() {
            return Ok(self);
        }
        let aliases = Arc::new(BodyAliases::new(aliases)?);
        let equalities = self.regions.as_deref().cloned().unwrap_or_default();
        self.regions = Some(Arc::new(equalities.with_body_aliases(aliases)));
        self.region_mapping = true;
        Ok(self)
    }

    pub(crate) fn body_aliases(&self) -> Option<&Arc<BodyAliases>> {
        self.regions.as_deref()?.body_aliases()
    }

    pub(crate) fn open_body_scope(&self, arguments: Vec<RegionId>) -> Result<Bound, &'static str> {
        match self.body_aliases() {
            Some(aliases) => Bound::body_scope(arguments, Arc::clone(aliases)),
            None => Ok(Bound::new(arguments)),
        }
    }
}
