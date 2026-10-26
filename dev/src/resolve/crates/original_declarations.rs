//! Reserve real owner-local declarations, then authenticate the complete remap.
mod locations;
mod packet_declarations;
mod reservations;
use super::{Error, Imported, Resolver};
use crate::{
    resolve::{
        body_origin::{BodyBinding, DeclarationBinding, Input},
        model::{DefId, DefKind, Definition, Namespace, ScopeId},
        package::ModuleId,
    },
    support::crates::{self, Interface, generics as g},
};
pub(crate) use packet_declarations::{OwnedNamespaceCatalog, PendingNamespace};
fn error(message: &'static str) -> Error {
    Error::unlocated(message)
}

#[derive(Default)]
pub(crate) struct Work {
    pub steps: usize,
    pub retained: usize,
    pub peak: usize,
}
impl Work {
    pub(crate) fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > crates::MAX_COMPONENTS {
            return Err(error("imported original reservation work budget exceeded"));
        }
        Ok(())
    }
    pub(crate) fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        self.reserve_raw(bytes).map_err(error)
    }
    pub(crate) fn reserve_raw(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self.retained.saturating_add(bytes);
        self.observe_raw(0, 0)
    }
    fn observe(&mut self, base: usize, scratch: usize) -> Result<(), Error> {
        self.observe_raw(base, scratch).map_err(error)
    }
    fn observe_raw(&mut self, base: usize, scratch: usize) -> Result<(), &'static str> {
        let peak = self.retained.saturating_add(base).saturating_add(scratch);
        self.peak = self.peak.max(peak);
        if peak > crates::MAX_BYTES {
            return Err("imported original reservations exceed combined storage budget");
        }
        Ok(())
    }
    fn prepare(&mut self, rows: &g::Rows, catalog: usize) -> Result<(), Error> {
        self.charge(rows.declarations.len().saturating_add(rows.originals.len()))?;
        let scopes = rows
            .originals
            .iter()
            .try_fold(rows.functions.len(), |n, body| {
                self.charge(1)?;
                Ok::<_, Error>(n.saturating_add(body.scopes.len()))
            })?;
        let declarations = rows.declarations.len();
        self.retained = self.retained.saturating_add(
            declarations
                .saturating_mul(4 * size_of::<Definition>())
                .saturating_add(
                    scopes.saturating_mul(4 * size_of::<crate::resolve::model::Scope>()),
                ),
        );
        let sites = rows
            .declaration_sites
            .iter()
            .fold(0_usize, |n, &sites| n.saturating_add(sites as usize))
            .saturating_add(
                rows.originals
                    .iter()
                    .fold(0_usize, |n, body| n.saturating_add(body.sites as usize)),
            );
        let scratch = sites
            .saturating_mul(4 * size_of::<Option<crate::resolve::body_origin::Label>>())
            .saturating_add(declarations.saturating_mul(
                size_of::<DefId>()
                    + size_of::<Expected>()
                    + size_of::<Option<ScopeId>>()
                    + size_of::<DeclarationBinding>(),
            ))
            .saturating_add(scopes.saturating_mul(size_of::<ScopeId>()))
            .saturating_add(
                rows.originals
                    .len()
                    .saturating_mul(size_of::<Vec<ScopeId>>() + size_of::<BodyBinding>()),
            );
        self.observe(catalog, scratch)
    }
}
pub(super) fn unpaid_input(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    imported: &Imported,
    work: &mut Work,
) -> Result<Input, Error> {
    packet_declarations::unpaid_input(resolver, scope, artifact, imported, work)
}
pub(super) fn named_unpaid(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    imported: &mut Imported,
    names: &g::source::NamespaceColumns,
    work: &mut Work,
) -> Result<(), Error> {
    packet_declarations::named_unpaid(resolver, scope, artifact, imported, names, work)
}
pub(super) fn source_kind(kind: g::source::DefinitionKind) -> (DefKind, Namespace) {
    packet_declarations::kind(kind)
}
#[derive(Clone, Copy)]
struct Expected {
    scope: ScopeId,
    kind: DefKind,
    namespace: Namespace,
    name: crate::resolve::names::NameId,
}
impl Resolver<'_> {
    pub(super) fn import_original_declarations(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        imported: &mut Imported,
    ) -> Result<(), Error> {
        if artifact.generic_payload().is_none() {
            return Ok(());
        }
        let mut work = imported.generic_work.take().unwrap_or_default();
        let result = self.reserve_original_declarations(scope, artifact, imported, &mut work);
        imported.generic_work = Some(work);
        result
    }
    fn reserve_original_declarations(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        imported: &mut Imported,
        work: &mut Work,
    ) -> Result<(), Error> {
        let payload = artifact
            .generic_payload()
            .ok_or_else(|| error("missing generic original payload"))?;
        if let g::PayloadView::Source(source) = payload.view() {
            return packet_declarations::reserve(
                self,
                packet_declarations::Reservation {
                    scope,
                    artifact,
                    payload,
                    rows: source.descriptor().rows(),
                },
                imported,
                work,
            );
        }
        let rows = payload.legacy_rows().map_err(error)?;
        let catalog_bytes = imported
            .origins
            .get()
            .map_or(0, crate::resolve::body_origin::Catalog::retained_bytes);
        work.prepare(rows, catalog_bytes)?;
        let (mut definitions, mut expected, owners) =
            reservations::functions(self, scope, artifact, imported, work)?;
        let mut scopes = Vec::with_capacity(rows.originals.len());
        for original in &rows.originals {
            let owner = owners[original.declaration.0 as usize]
                .ok_or_else(|| error("imported original lacks its exact declaration scope"))?;
            scopes.push(locations::scopes(self, original, owner, work)?);
        }
        let locations = locations::declarations(rows, &scopes, work)?;
        reservations::bindings(
            self,
            rows,
            &owners,
            &locations,
            &mut definitions,
            &mut expected,
            work,
        )?;
        locations::authenticate(self, artifact, imported, &definitions, &expected, work)?;
        let (declarations, declaration_labels) = locations::declaration_sites(rows, work)?;
        let (bindings, labels) = locations::body_sites(rows, work)?;
        let input = Input {
            owners: payload.owners().clone(),
            edition: rows.edition,
            definitions,
            declarations,
            bindings,
            files: Vec::new(),
            labels,
            declaration_labels,
        };
        imported
            .origins
            .ensure()
            .map_err(error)?
            .install(&self.facts.definitions, input)
            .map_err(error)?;
        Ok(())
    }
}
