//! Owned unpublished reservations never install an `Interface` or `SourceClient` report.
use super::{Error, Input, Resolver, Work, error, kind, s, scope_rows};
use crate::{
    expand::marks::ItemContext,
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId, Limits},
        expansion::Report,
        index::State,
        model::{Access, Binding, DefId, Definition, Namespace, Origin, ScopeId},
        names::NameId,
    },
    support::{
        crates::{
            MAX_BYTES, MAX_COMPONENTS, generics::Work as QueryWork, owner_catalog::DeclarationRef,
        },
        retained,
    },
};
use std::{ops::Range, sync::Arc};

#[path = "module_remap.rs"]
mod module_remap;

type Key = (ScopeId, NameId, Namespace);
struct PendingBinding {
    key: Key,
    binding: Binding,
}
pub(crate) struct PendingNamespace {
    report: Report,
    snapshot: Arc<s::UnpublishedNamespace>,
    owners: Vec<ScopeId>,
    bindings: Vec<PendingBinding>,
    export_keys: Vec<Key>,
    input: Input,
    definitions: Range<usize>,
    scopes: Range<usize>,
}
pub(crate) struct OwnedNamespaceCatalog {
    report: Report,
    snapshot: Arc<s::UnpublishedNamespace>,
    owners: Vec<ScopeId>,
    export_keys: Vec<Key>,
    catalog: Catalog,
    definitions: Range<usize>,
    scopes: Range<usize>,
}
impl PendingNamespace {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
        )
    )]
    pub(crate) fn prepare(
        mut report: Report,
        snapshot: Arc<s::UnpublishedNamespace>,
        work: &mut Work,
    ) -> Result<Self, Error> {
        work.charge(1)?;
        if !report.names_resolved {
            return Err(error(
                "namespace reservation requires a real resolved report",
            ));
        }
        let rows = snapshot.columns();
        module_remap::preflight(&report, &snapshot, work)?;
        let before_defs = report.resolution.definitions.len();
        let before_scopes = report.resolution.scopes.len();
        let mut state = State::default();
        state.facts = std::mem::take(&mut report.resolution);
        let (definitions, owners) = {
            let mut resolver = state.bind(&mut report.package);
            module_remap::reserve(&mut resolver, &snapshot, work)?
        };
        report.resolution = state.facts;
        let (bindings, export_keys) =
            module_remap::pending_bindings(&mut report, &snapshot, &definitions, work)?;
        let (declarations, declaration_labels) = super::declaration_sites(rows, work)?;
        let (bodies, labels) = super::body_sites(rows, snapshot.owners().rows().len(), work)?;
        let input = Input {
            owners: snapshot.owners().clone(),
            edition: rows.policy.edition,
            definitions,
            declarations,
            bindings: bodies,
            files: Vec::new(),
            labels,
            declaration_labels,
        };
        let definitions = before_defs..report.resolution.definitions.len();
        let scopes = before_scopes..report.resolution.scopes.len();
        Ok(Self {
            report,
            snapshot,
            owners,
            bindings,
            export_keys,
            input,
            definitions,
            scopes,
        })
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
        )
    )]
    pub(crate) fn freeze(mut self, work: &mut Work) -> Result<OwnedNamespaceCatalog, Error> {
        work.charge(1)?;
        let mut catalog = Catalog::new(Limits {
            bytes: MAX_BYTES.saturating_sub(work.retained),
            steps: MAX_COMPONENTS.saturating_sub(work.steps),
            ..Limits::default()
        })
        .map_err(error)?;
        let result = catalog
            .install_charged(&self.report.resolution.definitions, self.input, |peak| {
                work.reserve_raw(peak)
            })
            .and_then(|_| catalog.freeze_owned());
        work.charge(catalog.work())?;
        result.map_err(error)?;
        for pending in self.bindings {
            work.charge(1)?;
            if !self
                .report
                .resolution
                .bindings
                .declare_key(
                    (
                        pending.key.0,
                        pending.key.1,
                        pending.key.2,
                        ItemContext::ROOT,
                    ),
                    pending.binding,
                    false,
                )
                .map_err(error)?
            {
                return Err(error("namespace pending key changed before owned freeze"));
            }
        }
        let result = OwnedNamespaceCatalog {
            report: self.report,
            snapshot: self.snapshot,
            owners: self.owners,
            export_keys: self.export_keys,
            catalog,
            definitions: self.definitions,
            scopes: self.scopes,
        };
        let mut query = QueryWork::default();
        let checked = result.require(&result.snapshot, &mut query);
        work.charge(query.steps)?;
        checked?;
        Ok(result)
    }
}
impl OwnedNamespaceCatalog {
    pub(crate) fn require(
        &self,
        actual: &Arc<s::UnpublishedNamespace>,
        work: &mut QueryWork,
    ) -> Result<(), Error> {
        s::UnpublishedNamespace::require_shared(&self.snapshot, actual, work).map_err(error)?;
        if self.report.resolution.definitions.len() != self.definitions.end
            || self.report.resolution.scopes.len() != self.scopes.end
        {
            return Err(error(
                "namespace catalog changed its actual immutable owner/report",
            ));
        }
        self.root()?;
        Ok(())
    }
    pub(crate) fn root(&self) -> Result<ImportedDeclarationId, Error> {
        self.declaration(self.snapshot.root(), crate::resolve::model::DefKind::Module)
    }
    pub(crate) fn declaration(
        &self,
        row: DeclarationRef,
        expected: crate::resolve::model::DefKind,
    ) -> Result<ImportedDeclarationId, Error> {
        let definition = self
            .catalog
            .definition_at(self.snapshot.owner(), row)
            .map_err(error)?;
        self.check_cell(row, definition)?;
        let id = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(id, definition, expected)
            .map_err(error)?;
        Ok(id)
    }
    pub(crate) fn coordinate(
        &self,
        declaration: ImportedDeclarationId,
    ) -> Result<DeclarationRef, Error> {
        let (owner, row, edition) = self
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != self.snapshot.owner() || edition != self.snapshot.columns().policy.edition {
            return Err(error(
                "namespace declaration changed its actual provider coordinate",
            ));
        }
        Ok(row)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
        )
    )]
    pub(crate) fn lookup(
        &self,
        module: ImportedDeclarationId,
        name: &str,
        namespace: crate::support::crates::Namespace,
        work: &mut QueryWork,
    ) -> Result<Option<ImportedDeclarationId>, Error> {
        self.require(&self.snapshot, work)?;
        let definition = self
            .catalog
            .declaration_identity(module, crate::resolve::model::DefKind::Module)
            .map_err(error)?;
        let row = self.coordinate(module)?;
        self.check_cell(row, definition)?;
        let Some(index) = self
            .snapshot
            .find(row, name, namespace, work)
            .map_err(error)?
        else {
            return Ok(None);
        };
        let entry = &self.snapshot.bindings()[index];
        let key = self.export_keys[index];
        let actual = self
            .report
            .resolution
            .bindings
            .get(key.0, key.1, key.2)
            .filter(|b| b.access == Access::Public && b.origin == Origin::Direct && !b.ambiguous)
            .ok_or_else(|| error("namespace binding lost its actual owned cell"))?;
        if key.0 != self.owners[row.0 as usize]
            || self.report.package.names.get(key.1) != name
            || self
                .catalog
                .definition_at(self.snapshot.owner(), entry.target.declaration)
                .map_err(error)?
                != actual.target
        {
            return Err(error(
                "namespace binding changed its actual target coordinate",
            ));
        }
        self.check_cell(entry.target.declaration, actual.target)?;
        self.catalog
            .declaration(actual.target)
            .map(Some)
            .map_err(error)
    }
    fn check_cell(&self, row: DeclarationRef, definition: DefId) -> Result<(), Error> {
        let raw = self
            .snapshot
            .columns()
            .declarations
            .get(row.0 as usize)
            .ok_or_else(|| error("namespace declaration row is absent"))?;
        let actual = self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| error("namespace remapped definition is absent"))?;
        let expected = kind(raw.definition.kind);
        let header = self.snapshot.header(row).map_err(error)?;
        if !matches!(
            (raw.definition.kind, header),
            (s::DefinitionKind::Trait, Some(s::HeaderRef::Trait(_)))
                | (s::DefinitionKind::Function, Some(s::HeaderRef::Function(_)))
                | (
                    s::DefinitionKind::SelfType,
                    Some(s::HeaderRef::Implementation(_))
                )
                | (
                    s::DefinitionKind::Module
                        | s::DefinitionKind::SelfType
                        | s::DefinitionKind::TypeParameter
                        | s::DefinitionKind::RegionParameter
                        | s::DefinitionKind::ValueParameter
                        | s::DefinitionKind::Local,
                    None
                )
        ) {
            return Err(error(
                "namespace declaration changed its actual completed header kind",
            ));
        }
        if actual.node.is_some()
            || (actual.kind, actual.namespace) != expected
            || raw.name.as_deref() != Some(self.report.package.names.get(actual.name))
            || actual.access
                != if raw.public {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                }
            || raw.local.is_none()
                && raw
                    .definition
                    .parent
                    .is_some_and(|parent| actual.scope != self.owners[parent.0 as usize])
        {
            return Err(error(
                "namespace remapped cell changed its actual kind/name/owner/access",
            ));
        }
        if matches!(
            raw.definition.kind,
            s::DefinitionKind::Module | s::DefinitionKind::Trait | s::DefinitionKind::SelfType
        ) && actual.members != Some(self.owners[row.0 as usize])
        {
            return Err(error("namespace owning member scope changed"));
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../../../../tests/invariants/module_catalog_transactions.rs"]
mod tests;
