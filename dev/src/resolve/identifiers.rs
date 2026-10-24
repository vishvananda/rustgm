//! Consumers share the actual container expansion and definition-site privacy.
use super::{
    expansion::Report,
    model::{DefId, ItemAdjustment, ItemContext},
    package::ModuleId,
};
use crate::expand::marks::ExpansionId;

impl Report {
    /// Adjust a use identity against the expansion that introduced its actual
    /// container. A captured container name does not determine that expansion.
    ///
    /// # Errors
    /// Rejects unknown definitions, modules or expansion/context provenance.
    pub fn adjust_item(
        &self,
        container: DefId,
        context: ItemContext,
        caller: ModuleId,
    ) -> Result<ItemAdjustment, &'static str> {
        self.resolution
            .definitions
            .get(container.0 as usize)
            .ok_or("unknown hygienic item container")?;
        self.package
            .modules
            .get(caller.0 as usize)
            .ok_or("unknown hygienic caller module")?;
        let introduction = self
            .resolution
            .introductions
            .definitions
            .get(&container)
            .copied()
            .unwrap_or(ExpansionId(0));
        let adjusted = self.package.marks.adjust_item(
            context,
            introduction,
            &self.resolution.introductions.ancestry,
        )?;
        let privacy = if let Some(expansion) = adjusted.definition {
            let index = expansion
                .0
                .checked_sub(1)
                .ok_or("physical source has no macro definition")?;
            let invocation = self
                .invocations
                .get(index as usize)
                .filter(|invocation| invocation.context == expansion)
                .ok_or("unknown hygienic invocation")?;
            self.definitions
                .get(invocation.definition)
                .ok_or("unknown hygienic macro definition")?
                .module
        } else {
            caller
        };
        self.package
            .modules
            .get(privacy.0 as usize)
            .ok_or("unknown hygienic privacy module")?;
        Ok(ItemAdjustment {
            context: adjusted.context,
            privacy,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/hygienic_privacy.rs"]
mod tests;
