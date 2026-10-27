//! Syntax reserves canonical assignment identities; typed operands activate their headers.
use super::{Context, DefId, Error, Hook, Language, ModuleId, NodeKey, Resolver, Traits};
#[cfg(test)]
#[path = "../../../../../tests/invariants/compound_contracts.rs"]
mod tests;
impl Traits {
    pub(in crate::resolve) fn note_assignment_operator(
        &mut self,
        kind: Hook,
        module: ModuleId,
        source: NodeKey,
    ) {
        let registry = self.language.get_or_insert_default();
        registry.pending |= 1 << kind as u8;
        registry.origin.get_or_insert((module, source));
    }
    #[must_use]
    pub fn language_latent(&self, kind: Hook) -> bool {
        self.language
            .as_deref()
            .is_some_and(|registry| registry.latent & (1 << kind as u8) != 0)
    }
    #[must_use]
    pub fn latent_language_owner(&self, definition: DefId) -> Option<Language> {
        let registry = self.language.as_deref()?;
        registry
            .entries
            .values()
            .find(|entry| {
                registry.latent & (1 << entry.kind as u8) != 0
                    && (entry.definition == definition
                        || entry.self_parameter == definition
                        || entry.argument == Some(definition)
                        || entry.method == definition
                        || entry.output.declared() == Some(definition))
            })
            .copied()
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn reserve_assignment_operators(&mut self) -> Result<(), Error> {
        if self.package.uses_source_bindings() {
            return Ok(());
        }
        for kind in Hook::ALL.into_iter().filter(|kind| kind.assignment()) {
            let Some(registry) = self.facts.builtin_traits.language.as_deref() else {
                return Ok(());
            };
            if registry.pending & (1 << kind as u8) == 0 || registry.entries.contains_key(&kind) {
                continue;
            }
            let (module, source) = registry.origin.ok_or_else(|| {
                crate::resolve::package::Error::unlocated("assignment demand has no source origin")
            })?;
            let context = Context {
                module,
                file: source.file,
                scope: self.facts.module_scopes[module.0 as usize],
                foreign: None,
            };
            if self.facts.builtin_traits.library.is_none() {
                self.prepare_external_traits(context, source.node)?;
            }
            self.prepare_language_trait(context, source.node, kind)?;
            self.facts
                .builtin_traits
                .language
                .get_or_insert_default()
                .latent |= 1 << kind as u8;
        }
        Ok(())
    }
}
