//! Exported `macro_rules!` rows become public macro definitions in their
//! provider's imported root. `$crate` in their expansions names that root
//! through the dependency-indexed crate id, never through a name or alias.
use super::{
    Access, Binding, DefId, DefKind, Definition, Error, ExternalDef, ExternalKind, Imported,
    Interface, Namespace, Origin, Resolver, ScopeId, error,
};
use crate::support::names::NameId;

/// One transported row, bound at `definition` in its provider's root.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImportedMacro {
    pub definition: DefId,
    /// The provider's dependency index; its `$crate` id is one more.
    pub artifact: usize,
    pub row: usize,
}

/// The unspellable path root that `$crate` from the provider with this crate
/// id appends. No identifier contains `$`, so source cannot name it.
pub(crate) fn root_spelling(crate_id: u32) -> String {
    format!("$crate:{crate_id}")
}

impl Imported {
    pub(crate) fn macros(&self) -> &[ImportedMacro] {
        &self.macros
    }
    /// The imported root that a provider's `$crate` names.
    pub(crate) fn macro_root(&self, name: NameId) -> Option<DefId> {
        self.macro_roots.get(&name).copied()
    }
}

impl Resolver<'_> {
    /// Bind every exported macro of one artifact publicly in its root scope,
    /// and register its `$crate` root by the artifact's crate identity.
    pub(super) fn import_macros(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        artifact_index: usize,
        imported: &mut Imported,
    ) -> Result<(), Error> {
        let Some(macros) = artifact.macros() else {
            return Ok(());
        };
        let identity = ExternalDef::new(
            artifact.identity().clone(),
            Vec::new(),
            ExternalKind::Module,
        )
        .map_err(error)?;
        let root = imported
            .definitions
            .get(&identity)
            .copied()
            .ok_or_else(|| error("missing imported crate root"))?;
        let crate_id = u32::try_from(artifact_index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or_else(|| error("too many macro providers"))?;
        let root_name = self
            .package
            .names
            .intern(&root_spelling(crate_id))
            .map_err(error)?;
        imported.macro_roots.insert(root_name, root);
        for (row, entry) in macros.rows().iter().enumerate() {
            let name = self.package.names.intern(entry.name()).map_err(error)?;
            let definition = DefId(
                u32::try_from(self.facts.definitions.len())
                    .map_err(|_| error("too many imported definitions"))?,
            );
            self.facts.definitions.push(Definition {
                name,
                namespace: Namespace::Macro,
                kind: DefKind::Macro,
                scope,
                node: None,
                access: Access::Public,
                members: None,
            });
            self.facts
                .bindings
                .insert(
                    scope,
                    name,
                    Namespace::Macro,
                    Binding {
                        target: definition,
                        access: Access::Public,
                        origin: Origin::Direct,
                        ambiguous: false,
                    },
                )
                .map_err(error)?;
            imported.macros.push(ImportedMacro {
                definition,
                artifact: artifact_index,
                row,
            });
        }
        Ok(())
    }
}
