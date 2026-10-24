//! Consumers share the actual container expansion and definition-site privacy.
use super::{
    expansion::Report,
    model::{DefId, ItemAdjustment, ItemContext, ScopeId},
    package::ModuleId,
};
use crate::expand::marks::ExpansionId;

impl Report {
    /// Trait discovery visits actual item containers using the method token's
    /// context. Ordinary value/binder scopes do not trigger a definition rewind.
    pub(crate) fn trait_scope_parent(
        &self,
        scope: ScopeId,
        context: ItemContext,
    ) -> Result<(Option<ScopeId>, ItemContext), &'static str> {
        let frame = self
            .resolution
            .scopes
            .get(scope.0 as usize)
            .ok_or("unknown trait lookup scope")?;
        let introductions = &self.resolution.introductions;
        if (frame.parent.is_none() || introductions.item_block(scope))
            && let Some((parent, expansion)) = self.package.marks.item_parent(context)?
            && !introductions.ancestor(expansion, introductions.scope(scope))
        {
            let definition = introductions.definition_scope(expansion)?;
            self.resolution
                .scopes
                .get(definition.0 as usize)
                .ok_or("unknown trait lookup definition scope")?;
            return Ok((Some(definition), parent));
        }
        Ok((frame.parent, context))
    }

    /// Walk normalized item ancestry without exposing expansion storage to consumers.
    pub(crate) fn item_context_parent(
        &self,
        context: ItemContext,
    ) -> Result<Option<ItemContext>, &'static str> {
        self.package
            .marks
            .item_parent(context)
            .map(|parent| parent.map(|(context, _)| context))
    }

    /// Read the same declared-name identity used by the binding producer.
    /// Source-free declarations retain the admitted metadata or builtin profile's
    /// root identity; this does not authorize unsupported foreign macro contexts.
    pub(crate) fn definition_item_context(
        &self,
        definition: DefId,
    ) -> Result<ItemContext, super::package::Error> {
        let declaration = self
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| super::package::Error::unlocated("unknown item definition"))?;
        super::hygiene::definition_context(&self.package, declaration)
    }

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
