//! Operator syntax reserves canonical identities after all source binding allocation.
use super::{
    Comparison, Context, DefId, Equality, Error, Method, ModuleId, NodeKey, Resolver, Traits,
};
impl Traits {
    pub(in crate::resolve) fn note_equality_operator(&mut self, module: ModuleId, source: NodeKey) {
        let declarations = self.comparisons.get_or_insert_default();
        declarations.pending |= 1 << Comparison::PartialEq as u8;
        declarations.origin.get_or_insert((module, source));
    }
    #[must_use]
    pub fn comparison_latent(&self, kind: Comparison) -> bool {
        self.comparisons
            .as_deref()
            .is_some_and(|registry| registry.latent & (1 << kind as u8) != 0)
    }
    #[must_use]
    pub fn latent_comparison_owner(&self, definition: DefId) -> Option<Equality> {
        let registry = self.comparisons.as_deref()?;
        registry
            .entries
            .iter()
            .flatten()
            .find(|declaration| {
                registry.latent & (1 << declaration.kind as u8) != 0
                    && (declaration.definition == definition
                        || declaration.self_parameter == definition
                        || declaration.argument == Some(definition)
                        || declaration.methods.is_some_and(|members| {
                            members.iter().any(|(_, member)| member == definition)
                        }))
            })
            .copied()
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn reserve_operator_comparisons(&mut self) -> Result<(), Error> {
        let Some(registry) = self.facts.builtin_traits.comparisons.as_deref() else {
            return Ok(());
        };
        if registry.pending & (1 << Comparison::PartialEq as u8) == 0
            || self
                .facts
                .builtin_traits
                .comparison(Comparison::PartialEq)
                .is_some()
        {
            return Ok(());
        }
        let (module, source) = registry.origin.ok_or_else(|| {
            crate::resolve::package::Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "operator comparison has no diagnostic provenance",
            )
        })?;
        let c = Context {
            module,
            file: source.file,
            scope: self.facts.module_scopes[module.0 as usize],
            foreign: None,
        };
        self.prepare_comparison_method(c, source.node, Method::Eq)?;
        self.facts
            .builtin_traits
            .comparisons
            .get_or_insert_default()
            .latent |= 1 << Comparison::PartialEq as u8;
        Ok(())
    }
}
