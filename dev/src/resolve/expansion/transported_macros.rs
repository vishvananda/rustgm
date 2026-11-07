//! Imported `macro_rules!` rows are authenticated once and become ordinary
//! declarative definitions spelled in their own image unit; semiopaque
//! `pub macro` rows become scoped definitions with `macro_rules!` hygiene.
//! Imported builtin rows bind this consumer's own expander of their identity.
//! Every imported row's edition-panic contract is the `edition_panic` member
//! of its re-authenticated allowance list, as a local header derives it.
use super::{
    Body, Compiled, Definition, Engine, Error, ExpansionId, FileId, ModuleId, Scope, Sources,
    SyntaxContextId, Transparency, compile,
};
use crate::{
    expand::builtin::{Builtin, BuiltinKind},
    resolve::{crates::ImportedRow, model::ScopeId},
    support::crates::{
        Interface,
        exported_macros::{BuiltinMacro, Macro, MacroKind},
    },
    syntax::token_tree::Forest,
};

/// Where an imported definition's tokens were written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedDefinition {
    /// The image unit of a declarative row: the spelling source, with the
    /// transported edition. A builtin row has no image.
    pub file: Option<FileId>,
    /// One more than the provider's dependency index; `$crate` carries it.
    pub crate_id: u32,
    /// The provider's imported placement scope, the definition site.
    pub scope: ScopeId,
    /// The definition row whose internal-unstable features its frames carry
    /// and whose `#[unstable]` feature its calls need.
    pub row: crate::support::crates::exported_macros::DefinitionRow,
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
        if let Some(imported) = self.imported {
            return imported.file.map(|file| (file, imported.crate_id));
        }
        self.node.map(|node| (node.file, 0))
    }
}

/// The consumer's own expander of one builtin row's identity. Unknown
/// identities and a changed invocation kind are rejected by name.
fn builtin_role(row: &BuiltinMacro) -> Result<Builtin, String> {
    let role = Builtin::ALL
        .into_iter()
        .find(|role| role.name() == row.identity())
        .ok_or_else(|| {
            format!(
                "imported builtin macro `{}` names unknown expander `{}`",
                row.name(),
                row.identity()
            )
        })?;
    let kind = match role.kind() {
        BuiltinKind::Bang => MacroKind::Bang,
        BuiltinKind::Attribute => MacroKind::Attribute,
        BuiltinKind::Derive => MacroKind::Derive,
    };
    if kind != row.kind() {
        return Err(format!(
            "imported builtin macro `{}` changed the macro kind of expander `{}`",
            row.name(),
            row.identity()
        ));
    }
    if !row.helpers().is_empty() && role.kind() != BuiltinKind::Derive {
        return Err(format!(
            "imported builtin macro `{}` declares helpers for a non-derive expander",
            row.name()
        ));
    }
    Ok(role)
}

/// Whether a sorted, re-authenticated allowance list carries the
/// edition-panic contract.
fn edition_panic(features: &[String]) -> bool {
    let contract = crate::resolve::bindings::KnownFeature::EditionPanic.name();
    features
        .binary_search_by(|feature| feature.as_str().cmp(contract.spelling()))
        .is_ok()
}

/// A row's `#[unstable]` feature must be a canonical feature identifier.
fn authenticate_unstable(feature: Option<&str>) -> Result<(), Error> {
    if let Some(feature) = feature {
        crate::resolve::bindings::FeatureName::new(feature)
            .map_err(|message| Error::at(FileId(0), 0, message))?;
    }
    Ok(())
}

/// Lex one transported image, rebuild its rows and require them to be equal.
fn authenticate(row: &Macro) -> Result<Forest, String> {
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
    /// Install every imported row before any call is attempted. Each declarative
    /// row is lexed once, compiled in its transported edition and must keep its
    /// rule count; matching and transcription reuse that compiled forest. Each
    /// builtin row binds this consumer's own expander of its identity.
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
            let scope = self
                .paths
                .state
                .facts
                .definitions
                .get(entry.definition.0 as usize)
                .map(|definition| definition.scope)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing imported macro placement"))?;
            let crate_id = u32::try_from(entry.artifact)
                .ok()
                .and_then(|index| index.checked_add(1))
                .ok_or_else(|| Error::at(FileId(0), 0, "too many macro providers"))?;
            let imported = ImportedDefinition {
                file: None,
                crate_id,
                scope,
                row: entry.row,
            };
            let (definition, kind) = match entry.row {
                ImportedRow::Declarative(_) | ImportedRow::Scoped(_) => (
                    self.import_declarative(artifact, entry.row, imported)?,
                    BuiltinKind::Bang,
                ),
                ImportedRow::Builtin(row) => {
                    let row = artifact
                        .macros()
                        .and_then(|macros| macros.builtins().get(row))
                        .ok_or_else(|| Error::at(FileId(0), 0, "missing imported builtin row"))?;
                    let role =
                        builtin_role(row).map_err(|message| Error::at(FileId(0), 0, message))?;
                    self.authenticate_internal_unstable(FileId(0), row.internal_unstable())?;
                    authenticate_unstable(row.unstable())?;
                    let package = &mut self.loader.package;
                    let name = package
                        .names
                        .intern(row.name())
                        .map_err(|e| Error::at(FileId(0), 0, e))?;
                    let edition = package.units[0].edition();
                    (
                        Definition {
                            node: None,
                            module: ModuleId(0),
                            name,
                            body: Body::Builtin(role),
                            context: SyntaxContextId::ROOT,
                            introduction: ExpansionId(0),
                            preceding_scope: Scope::ROOT,
                            local_inner_macros: false,
                            edition,
                            edition_panic: edition_panic(row.internal_unstable()),
                            header: None,
                            imported: Some(imported),
                        },
                        role.kind(),
                    )
                }
            };
            let index = self.definitions.len();
            self.definitions.push(definition);
            self.paths.imported_macro(entry.definition, index, kind);
        }
        Ok(())
    }
    /// Re-check every transported feature spelling once and charge each one,
    /// as the consumer's own header admission charges its declared features.
    fn authenticate_internal_unstable(
        &self,
        file: FileId,
        features: &[String],
    ) -> Result<(), Error> {
        let error = |message| Error::at(file, 0, message);
        let mut bytes = 0usize;
        for feature in features {
            crate::resolve::bindings::FeatureName::new(feature).map_err(error)?;
            bytes = bytes.saturating_add(feature.len());
        }
        if let Some(context) = self.loader.package.macro_context().map_err(error)? {
            context
                .contracts()
                .charge(features.len(), bytes)
                .map_err(error)?;
        }
        Ok(())
    }
    fn import_declarative(
        &mut self,
        artifact: &Interface,
        index: ImportedRow,
        imported: ImportedDefinition,
    ) -> Result<Definition, Error> {
        let macros = artifact
            .macros()
            .ok_or_else(|| Error::at(FileId(0), 0, "missing imported macro row"))?;
        let row = macros
            .image_row(index)
            .ok_or_else(|| Error::at(FileId(0), 0, "missing imported macro row"))?;
        let placement = match index {
            ImportedRow::Scoped(scoped) => macros
                .scoped()
                .get(scoped)
                .map(|row| row.placement().join("::") + "::")
                .unwrap_or_default(),
            _ => String::new(),
        };
        let scoped = matches!(index, ImportedRow::Scoped(_));
        let forest = authenticate(row).map_err(|message| Error::at(FileId(0), 0, message))?;
        let identity = artifact.identity();
        let label = format!(
            "<macro {}@{}::{placement}{}>",
            identity.name(),
            identity.disambiguator(),
            row.name()
        );
        let file = self
            .loader
            .macro_image(label.into(), row.image(), row.edition())?;
        let names = &mut self.loader.package.names;
        let compiled = if scoped {
            compile::declarative_in_edition(row.image(), None, &forest, names, row.edition())
        } else {
            compile::in_edition(row.image(), &forest, names, row.edition())
        }
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        if u32::try_from(compiled.rules.len()).ok() != Some(row.rules()) {
            return Err(Error::at(file, 0, "imported macro rule count changed"));
        }
        self.authenticate_internal_unstable(file, row.internal_unstable())?;
        authenticate_unstable(row.unstable())?;
        let name = self
            .loader
            .package
            .names
            .intern(row.name())
            .map_err(|e| Error::at(file, 0, e))?;
        let body = if scoped {
            // An imported scoped body pays for its cold allocation, as a
            // local scoped declaration does.
            if let Some(context) = self
                .loader
                .package
                .macro_context()
                .map_err(|message| Error::at(file, 0, message))?
            {
                context
                    .contracts()
                    .charge(1, size_of::<Compiled>())
                    .map_err(|message| Error::at(file, 0, message))?;
            }
            Body::Scoped {
                compiled: Box::new(Compiled::new(compiled)),
                mode: Transparency::SemiOpaque,
            }
        } else {
            Body::Declarative(Compiled::new(compiled))
        };
        Ok(Definition {
            node: None,
            module: ModuleId(0),
            name,
            body,
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(0),
            preceding_scope: Scope::ROOT,
            local_inner_macros: row.local_inner_macros(),
            edition: row.edition(),
            edition_panic: edition_panic(row.internal_unstable()),
            header: None,
            imported: Some(ImportedDefinition {
                file: Some(file),
                ..imported
            }),
        })
    }
}
