//! Imported `macro_rules!` rows are authenticated once and become ordinary
//! declarative definitions spelled in their own image unit.
use super::{
    Body, Compiled, Definition, Engine, Error, ExpansionId, FileId, ModuleId, Scope, Sources,
    SyntaxContextId, compile,
};
use crate::{resolve::model::ScopeId, syntax::token_tree::Forest};

/// Where an imported definition's tokens were written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedDefinition {
    /// The image unit: the spelling source, with the transported edition.
    pub file: FileId,
    /// One more than the provider's dependency index; `$crate` carries it.
    pub crate_id: u32,
    /// The provider's imported root scope, the definition site.
    pub scope: ScopeId,
}

impl Definition {
    /// The scope where this definition's tokens were written.
    pub(super) fn site_scope(&self, facts: &crate::resolve::model::Resolution) -> Option<ScopeId> {
        if let Some(imported) = self.imported {
            return Some(imported.scope);
        }
        match self.node {
            Some(node) => facts.node_scopes.get(&(self.module, node.node)).copied(),
            None => facts.module_scopes.get(self.module.0 as usize).copied(),
        }
    }
    /// The file holding this definition's spelling, and the crate id of its `$crate`.
    pub(super) fn spelling(&self) -> Option<(FileId, u32)> {
        self.imported
            .map(|imported| (imported.file, imported.crate_id))
            .or_else(|| self.node.map(|node| (node.file, 0)))
    }
}

/// Lex one transported image, rebuild its rows and require them to be equal.
fn authenticate(row: &crate::support::crates::exported_macros::Macro) -> Result<Forest, String> {
    let forest = Forest::parse(row.image()).map_err(|error| error.message.to_owned())?;
    let (image, rows) = crate::resolve::crates::macro_image(row.image(), &forest)?;
    if image != row.image() || rows != row.rows() {
        return Err(format!(
            "imported macro `{}` does not rebuild its canonical rows",
            row.name()
        ));
    }
    Ok(forest)
}

impl<S: Sources> Engine<'_, S> {
    /// Apply `#[macro_use]` on an external crate item to the macro-use prelude.
    pub(super) fn extern_macro_use(
        &mut self,
        visit: super::Visit,
        file: FileId,
        item: crate::syntax::item::Item,
    ) -> Result<(), Error> {
        if !matches!(item.kind, crate::syntax::item::ItemKind::ExternCrate { .. })
            || !self.flags(file, visit.owner)?.macro_use
        {
            return Ok(());
        }
        let context = crate::resolve::index::Context {
            module: visit.module,
            file,
            scope: visit.path_scope,
            foreign: visit.foreign,
        };
        self.paths
            .macro_use(&mut self.loader.package, context, visit.node, item)
    }
    /// Install every imported row before any call is attempted. Each one is
    /// lexed once, compiled in its transported edition and must keep its rule
    /// count; matching and transcription reuse that compiled forest.
    pub(super) fn import_macros(&mut self) -> Result<(), Error> {
        let Some(crates) = self.paths.state.facts.crates.as_deref() else {
            return Ok(());
        };
        let rows = crates.macros().to_vec();
        let dependencies = crates.shared_dependencies();
        for entry in rows {
            let artifact = dependencies
                .artifacts()
                .get(entry.artifact)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing macro provider"))?;
            let row = artifact
                .macros()
                .and_then(|macros| macros.rows().get(entry.row))
                .ok_or_else(|| Error::at(FileId(0), 0, "missing imported macro row"))?;
            let forest = authenticate(row).map_err(|message| Error::at(FileId(0), 0, message))?;
            let identity = artifact.identity();
            let label = format!(
                "<macro {}@{}::{}>",
                identity.name(),
                identity.disambiguator(),
                row.name()
            );
            let file = self
                .loader
                .macro_image(label.into(), row.image(), row.edition())?;
            let package = &mut self.loader.package;
            let compiled =
                compile::in_edition(row.image(), &forest, &mut package.names, row.edition())
                    .map_err(|e| Error::at(file, e.offset, e.message))?;
            if u32::try_from(compiled.rules.len()).ok() != Some(row.rules()) {
                return Err(Error::at(file, 0, "imported macro rule count changed"));
            }
            let name = package
                .names
                .intern(row.name())
                .map_err(|e| Error::at(file, 0, e))?;
            let scope = self
                .paths
                .state
                .facts
                .definitions
                .get(entry.definition.0 as usize)
                .map(|definition| definition.scope)
                .ok_or_else(|| Error::at(file, 0, "missing imported macro root"))?;
            let crate_id = u32::try_from(entry.artifact)
                .ok()
                .and_then(|index| index.checked_add(1))
                .ok_or_else(|| Error::at(file, 0, "too many macro providers"))?;
            let index = self.definitions.len();
            self.definitions.push(Definition {
                node: None,
                module: ModuleId(0),
                name,
                body: Body::Declarative(Compiled::new(compiled)),
                context: SyntaxContextId::ROOT,
                introduction: ExpansionId(0),
                preceding_scope: Scope::ROOT,
                local_inner_macros: row.local_inner_macros(),
                edition: row.edition(),
                edition_panic: false,
                header: None,
                imported: Some(ImportedDefinition {
                    file,
                    crate_id,
                    scope,
                }),
            });
            self.paths.imported_macro(entry.definition, index);
        }
        Ok(())
    }
}
