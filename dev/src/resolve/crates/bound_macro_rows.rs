//! Exported macro rows become public macro definitions at their placement in
//! their provider's imported module tree. Placements resolve through the
//! provider's own module identities, never through a binding name or alias.
//! `$crate` in their expansions names that root through the dependency-indexed
//! crate id, never through a name or alias. A re-export row binds the very
//! definition its owner's row identity names, so every path to one macro
//! reaches one definition.
use super::{
    Access, Binding, DefId, DefKind, Definition, Error, ExternalDef, ExternalKind, Imported,
    Interface, Namespace, Origin, Resolver, ScopeId, error,
};
use crate::support::{
    crates::{generics::source as s, owner_catalog::DeclarationRef},
    hash::HashMap,
    names::NameId,
};

/// Which table of the provider's macro metadata one imported row comes from.
pub(crate) use crate::support::crates::exported_macros::DefinitionRow as ImportedRow;

/// Imported definitions by artifact index and definition row.
pub(super) type MacroDefinitions = HashMap<(usize, ImportedRow), DefId>;

/// One transported row, bound at `definition` in its provider's module tree.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImportedMacro {
    pub definition: DefId,
    /// The provider's dependency index; its `$crate` id is one more.
    pub artifact: usize,
    pub row: ImportedRow,
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

/// Module rows of a source artifact, keyed by parent row and name.
type SourceModules = HashMap<(DeclarationRef, String), DeclarationRef>;

fn source_modules(rows: &[s::Declaration]) -> Result<SourceModules, Error> {
    let mut modules = HashMap::default();
    for (index, row) in rows.iter().enumerate() {
        let (Some(parent), Some(name), s::DefinitionKind::Module) = (
            row.definition.parent,
            row.name.as_ref(),
            row.definition.kind,
        ) else {
            continue;
        };
        let row = DeclarationRef(
            u32::try_from(index).map_err(|_| error("source module ordinal overflow"))?,
        );
        if modules.insert((parent, name.clone()), row).is_some() {
            return Err(error("source artifact repeats one module name in a scope"));
        }
    }
    Ok(modules)
}

impl Resolver<'_> {
    /// The member scope of the provider's public module at `path`, found by
    /// the module's own identity in the artifact's tables.
    fn placement_scope(
        &self,
        root: ScopeId,
        artifact: &Interface,
        imported: &Imported,
        modules: &mut Option<SourceModules>,
        path: &[String],
    ) -> Result<ScopeId, Error> {
        const MISSING: &str = "exported macro placement has no public imported module";
        if path.is_empty() {
            return Ok(root);
        }
        let definition = if let Some(descriptor) = artifact.source_descriptor() {
            let rows = &descriptor.rows().declarations;
            let modules = match modules {
                Some(modules) => modules,
                None => modules.insert(source_modules(rows)?),
            };
            let mut row = DeclarationRef(
                rows.iter()
                    .position(|row| row.definition.parent.is_none())
                    .and_then(|index| u32::try_from(index).ok())
                    .ok_or_else(|| error("source artifact has no root declaration"))?,
            );
            for component in path {
                row = *modules
                    .get(&(row, component.clone()))
                    .filter(|&&child| rows[child.0 as usize].public)
                    .ok_or_else(|| error(MISSING))?;
            }
            imported
                .origins
                .get()
                .ok_or_else(|| error(MISSING))?
                .definition_at(artifact.identity(), row)
                .map_err(error)?
        } else {
            let identity = ExternalDef::new(
                artifact.identity().clone(),
                path.to_vec(),
                ExternalKind::Module,
            )
            .map_err(error)?;
            imported
                .definition(&identity)
                .ok_or_else(|| error(MISSING))?
        };
        let module = &self.facts.definitions[definition.0 as usize];
        if module.kind != DefKind::Module || module.access != Access::Public {
            return Err(error(MISSING));
        }
        module.members.ok_or_else(|| error(MISSING))
    }
    /// Bind every exported macro definition row of one artifact publicly at
    /// its placement, and register its `$crate` root by the artifact's crate
    /// identity.
    pub(super) fn import_macros(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        artifact_index: usize,
        imported: &mut Imported,
        definitions: &mut MacroDefinitions,
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
        let declarative = macros
            .rows()
            .iter()
            .enumerate()
            .map(|(row, entry)| (scope, entry.name(), ImportedRow::Declarative(row)));
        let mut modules = None;
        let mut placed = Vec::with_capacity(macros.builtins().len());
        for (row, entry) in macros.builtins().iter().enumerate() {
            let scope =
                self.placement_scope(scope, artifact, imported, &mut modules, entry.placement())?;
            placed.push((scope, entry.name(), ImportedRow::Builtin(row)));
        }
        for (scope, name, row) in declarative.chain(placed) {
            let spelling = self.package.names.intern(name).map_err(error)?;
            let definition = DefId(
                u32::try_from(self.facts.definitions.len())
                    .map_err(|_| error("too many imported definitions"))?,
            );
            self.facts.definitions.push(Definition {
                name: spelling,
                namespace: Namespace::Macro,
                kind: DefKind::Macro,
                scope,
                node: None,
                access: Access::Public,
                members: None,
            });
            self.bind_imported_macro(scope, spelling, definition)?;
            definitions.insert((artifact_index, row), definition);
            imported.macros.push(ImportedMacro {
                definition,
                artifact: artifact_index,
                row,
            });
        }
        Ok(())
    }
    /// Bind every re-export row of one artifact at its placement to the
    /// definition its target identity names: the owner crate is this artifact
    /// or one of its own dependencies, found by crate identity, and the target
    /// is that owner's definition row at the transported placement and name.
    /// No binding name, alias or consumer path is consulted.
    pub(super) fn import_macro_reexports(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        artifact_index: usize,
        imported: &Imported,
        definitions: &MacroDefinitions,
    ) -> Result<(), Error> {
        let Some(macros) = artifact
            .macros()
            .filter(|macros| !macros.reexports().is_empty())
        else {
            return Ok(());
        };
        let artifacts = imported.dependencies.artifacts();
        let mut modules = None;
        for row in macros.reexports() {
            let owner = match row.owner().checked_sub(1) {
                None => artifact_index,
                Some(ordinal) => {
                    let identity = artifact
                        .dependencies()
                        .get(ordinal as usize)
                        .ok_or_else(|| error("exported macro re-export names an unknown owner"))?;
                    artifacts
                        .binary_search_by(|candidate| candidate.identity().cmp(identity))
                        .map_err(|_| error("macro re-export owner has no imported artifact"))?
                }
            };
            let target = artifacts[owner]
                .macros()
                .and_then(|owner| owner.definition(row.target_placement(), row.target_name()))
                .and_then(|target| definitions.get(&(owner, target)).copied())
                .ok_or_else(|| error("macro re-export names no imported definition row"))?;
            let placed =
                self.placement_scope(scope, artifact, imported, &mut modules, row.placement())?;
            let name = self.package.names.intern(row.name()).map_err(error)?;
            self.bind_imported_macro(placed, name, target)?;
        }
        Ok(())
    }
    fn bind_imported_macro(
        &mut self,
        scope: ScopeId,
        name: NameId,
        definition: DefId,
    ) -> Result<(), Error> {
        let previous = self
            .facts
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
        if previous.is_some() {
            return Err(error("duplicate imported macro binding in one scope"));
        }
        Ok(())
    }
}
