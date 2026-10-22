//! Operator syntax reserves canonical identities after all source binding allocation.
use super::{
    Comparison, Context, DefId, Equality, Error, Method, ModuleId, NodeKey, Resolver, Traits,
};
impl Traits {
    pub(in crate::resolve) fn note_comparison_operator(
        &mut self,
        kind: Comparison,
        module: ModuleId,
        source: NodeKey,
    ) {
        let declarations = self.comparisons.get_or_insert_default();
        declarations.pending |= 1 << kind as u8;
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
const RESULTS: [crate::support::library_types::LibraryType; 2] = [
    crate::support::library_types::LibraryType::ComparisonOrdering,
    crate::support::library_types::LibraryType::Option,
];
impl Traits {
    #[must_use]
    pub fn comparison_result_latent(
        &self,
        kind: crate::support::library_types::LibraryType,
    ) -> bool {
        RESULTS
            .iter()
            .position(|result| *result == kind)
            .is_some_and(|index| {
                self.comparisons
                    .as_deref()
                    .is_some_and(|registry| registry.results & (1 << index) != 0)
            })
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn reserve_operator_comparisons(&mut self) -> Result<(), Error> {
        for (kind, method) in [
            (Comparison::PartialEq, Method::Eq),
            (Comparison::PartialOrd, Method::PartialCmp),
        ] {
            let Some(registry) = self.facts.builtin_traits.comparisons.as_deref() else {
                return Ok(());
            };
            if registry.pending & (1 << kind as u8) == 0
                || self.facts.builtin_traits.comparison(kind).is_some()
            {
                continue;
            }
            let (module, source) = registry.origin.ok_or_else(|| {
                crate::resolve::package::Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "operator comparison has no diagnostic provenance",
                )
            })?;
            let traits =
                Comparison::ALL.map(|kind| self.facts.builtin_traits.comparison(kind).is_some());
            let results = RESULTS.map(|kind| self.facts.library_types.get(kind).is_some());
            let c = Context {
                module,
                file: source.file,
                scope: self.facts.module_scopes[module.0 as usize],
                foreign: None,
            };
            self.prepare_comparison_method(c, source.node, method)?;
            let registry = self
                .facts
                .builtin_traits
                .comparisons
                .get_or_insert_default();
            for (index, existed) in traits.into_iter().enumerate() {
                if !existed && registry.entries[index].is_some() {
                    registry.latent |= 1 << index;
                }
            }
            for (index, existed) in results.into_iter().enumerate() {
                if !existed && self.facts.library_types.get(RESULTS[index]).is_some() {
                    registry.results |= 1 << index;
                }
            }
        }
        Ok(())
    }
}
