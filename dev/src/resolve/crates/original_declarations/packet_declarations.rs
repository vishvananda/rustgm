//! Genuine source declarations are staged before any role or catalog publication.
mod installed_bindings;
mod scope_rows;
use super::{Error, Imported, Interface, Resolver, ScopeId, Work, error, g};
use crate::{
    resolve::{
        body_origin::{BodyBinding, DeclarationBinding, Input, Label},
        model::{Access, DefId, DefKind, Definition, Namespace},
        package::ModuleId,
    },
    support::crates::{ExternalDef, ExternalKind, generics::source as s},
};

#[derive(Clone, Copy)]
pub(super) struct Reservation<'a> {
    pub scope: ScopeId,
    pub artifact: &'a Interface,
    pub payload: &'a g::Payload,
    pub rows: s::ValidatedView<'a>,
}
pub(super) fn reserve(
    resolver: &mut Resolver<'_>,
    context: Reservation<'_>,
    imported: &mut Imported,
    work: &mut Work,
) -> Result<(), Error> {
    require_namespace(context.artifact)?;
    preflight(context.rows, imported, work)?;
    let definitions_before = resolver.facts.definitions.len();
    let scopes_before = resolver.facts.scopes.len();
    let result = staged(resolver, &context, imported, work);
    if result.is_err() {
        // No installed catalog or new external identities escape a failing stage.
        // The interner and unbound table slots may retain charged cold capacity.
        resolver.facts.definitions.truncate(definitions_before);
        resolver.facts.scopes.truncate(scopes_before);
    }
    result
}
fn require_namespace(artifact: &Interface) -> Result<(), Error> {
    if artifact.public_names().is_some()
        || !artifact.functions().is_empty()
        || !artifact.constants().is_empty()
        || artifact.nominals().is_some()
        || artifact.storage().is_some()
    {
        return Err(error(
            "source function declarations require their complete dedicated namespace",
        ));
    }
    Ok(())
}
fn staged(
    resolver: &mut Resolver<'_>,
    context: &Reservation<'_>,
    imported: &mut Imported,
    work: &mut Work,
) -> Result<(), Error> {
    let Reservation {
        scope,
        artifact,
        rows,
        ..
    } = *context;
    let definitions = reserve_definitions(resolver, scope, artifact, rows, imported, work)?;
    let pending = installed_bindings::prepare(
        resolver,
        scope,
        artifact,
        rows,
        &definitions,
        imported,
        work,
    )?;
    let input = catalog_input(context, definitions, work)?;
    // Binding insertion can fail before catalog installation. The pending owner
    // rolls back those exact fresh keys; Catalog::install itself is transactional.
    pending.install(resolver, imported, input)
}
fn catalog_input(
    context: &Reservation<'_>,
    definitions: Vec<DefId>,
    work: &mut Work,
) -> Result<Input, Error> {
    let rows = context.rows;
    let (declarations, declaration_labels) = declaration_sites(rows, work)?;
    let (bindings, labels) = body_sites(rows, work)?;
    Ok(Input {
        owners: context.payload.owners().clone(),
        edition: rows.policy.edition,
        definitions,
        declarations,
        bindings,
        files: Vec::new(),
        labels,
        declaration_labels,
    })
}
fn reserve_definitions(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    imported: &Imported,
    work: &mut Work,
) -> Result<Vec<DefId>, Error> {
    let (definitions, owners) = declarations(resolver, scope, artifact, rows, imported, work)?;
    let body_scopes = scope_rows::bodies(resolver, rows, &owners, work)?;
    scope_rows::assign(resolver, rows, &definitions, &owners, &body_scopes, work)?;
    Ok(definitions)
}
pub(super) fn unpaid_input(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    imported: &Imported,
    work: &mut Work,
) -> Result<Input, Error> {
    let payload = artifact
        .generic_payload()
        .ok_or_else(|| error("unpaid source payload is absent"))?;
    let descriptor = artifact
        .source_descriptor()
        .ok_or_else(|| error("unpaid source descriptor is absent"))?;
    if descriptor.profile() != s::Profile::Source2024Provided {
        return Err(error(
            "unpaid reservations require their actual Provided descriptor",
        ));
    }
    require_namespace(artifact)?;
    let rows = descriptor.rows();
    preflight(rows, imported, work)?;
    let definitions = reserve_definitions(resolver, scope, artifact, rows, imported, work)?;
    catalog_input(
        &Reservation {
            scope,
            artifact,
            payload,
            rows,
        },
        definitions,
        work,
    )
}
fn declarations(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    imported: &Imported,
    work: &mut Work,
) -> Result<(Vec<DefId>, Vec<ScopeId>), Error> {
    let root = ExternalDef::new(
        artifact.identity().clone(),
        Vec::new(),
        ExternalKind::Module,
    )
    .map_err(error)?;
    let root = imported
        .definition(&root)
        .ok_or_else(|| error("source artifact has no real reserved module"))?;
    let root_index = rows
        .declarations
        .iter()
        .position(|row| row.definition.parent.is_none())
        .ok_or_else(|| error("source root declaration is absent"))?;
    let root_row = &rows.declarations[root_index];
    if root_row.definition.kind != s::DefinitionKind::Module
        || root_row.name.as_deref() != Some(artifact.identity().name())
    {
        return Err(error(
            "source root differs from the actual defining artifact module",
        ));
    }
    let mut definitions = Vec::with_capacity(rows.declarations.len());
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if index == root_index {
            definitions.push(root);
            continue;
        }
        if row.definition.kind == s::DefinitionKind::Module {
            return Err(error(
                "nested source modules require a wider source namespace profile",
            ));
        }
        let spelling = row
            .name
            .as_deref()
            .ok_or_else(|| error("unnamed resolver declarations require a wider source profile"))?;
        let name = resolver.package.names.intern(spelling).map_err(error)?;
        let definition = DefId(
            u32::try_from(resolver.facts.definitions.len())
                .map_err(|_| error("source declaration remap overflow"))?,
        );
        let (kind, namespace) = kind(row.definition.kind);
        resolver.facts.definitions.push(Definition {
            name,
            namespace,
            kind,
            scope,
            node: None,
            access: if row.public {
                Access::Public
            } else {
                Access::ExternalPrivate
            },
            members: None,
        });
        definitions.push(definition);
    }
    let owners = scope_rows::owners(resolver, scope, root_index, rows, work)?;
    Ok((definitions, owners))
}
pub(super) fn kind(kind: s::DefinitionKind) -> (DefKind, Namespace) {
    match kind {
        s::DefinitionKind::Module => (DefKind::Module, Namespace::Type),
        s::DefinitionKind::Trait => (DefKind::Trait, Namespace::Type),
        s::DefinitionKind::SelfType => (DefKind::SelfType, Namespace::Type),
        s::DefinitionKind::Function => (DefKind::Function, Namespace::Value),
        s::DefinitionKind::TypeParameter => (DefKind::Generic, Namespace::Type),
        s::DefinitionKind::RegionParameter => (DefKind::Lifetime, Namespace::Lifetime),
        s::DefinitionKind::ValueParameter => (DefKind::Parameter, Namespace::Value),
        s::DefinitionKind::Local => (DefKind::Local, Namespace::Value),
    }
}
fn preflight(
    rows: s::ValidatedView<'_>,
    imported: &Imported,
    work: &mut Work,
) -> Result<(), Error> {
    work.charge(rows.declarations.len().saturating_add(rows.originals.len()))?;
    let mut scopes = rows.declarations.len();
    let mut sites = 0_usize;
    let mut names = 0_usize;
    for declaration in &rows.declarations {
        work.charge(1)?;
        sites = sites.saturating_add(declaration.sites as usize);
        names = names.saturating_add(declaration.name.as_ref().map_or(0, String::len));
    }
    for body in &rows.originals {
        work.charge(1)?;
        scopes = scopes.saturating_add(body.scopes.len());
        sites = sites.saturating_add(body.sites as usize);
    }
    let declarations = rows.declarations.len();
    let reserved = declarations
        .saturating_mul(4 * (size_of::<Definition>() + size_of::<(ExternalDef, DefId)>()))
        .saturating_add(scopes.saturating_mul(4 * size_of::<crate::resolve::model::Scope>()))
        .saturating_add(names.saturating_mul(4))
        .saturating_add(
            declarations.saturating_mul(4 * size_of::<installed_bindings::PendingBinding>()),
        );
    work.retained = work.retained.saturating_add(reserved);
    let scratch = declarations
        .saturating_mul(
            size_of::<[u32; 8]>() + size_of::<usize>() + size_of::<DeclarationBinding>(),
        )
        .saturating_add(scopes.saturating_mul(size_of::<ScopeId>()))
        .saturating_add(rows.originals.len().saturating_mul(
            size_of::<Vec<ScopeId>>() + size_of::<BodyBinding>() + size_of::<Option<u32>>(),
        ))
        .saturating_add(sites.saturating_mul(4 * size_of::<Option<Label>>()));
    let mut base = imported
        .origins
        .get()
        .map_or(0, crate::resolve::body_origin::Catalog::retained_bytes);
    let mut providers = 0_usize;
    for artifact in imported.dependencies.artifacts() {
        work.charge(1)?;
        if let Some(payload) = artifact.generic_payload() {
            base = base.saturating_add(payload.retained_bytes());
            providers += usize::from(matches!(payload.view(), g::PayloadView::Source(_)));
        }
    }
    if providers != 1 {
        return Err(error(
            "source function profile requires one complete source binding provider",
        ));
    }
    work.peak = work
        .peak
        .max(work.retained.saturating_add(base).saturating_add(scratch));
    if work.retained.saturating_add(base).saturating_add(scratch)
        > crate::support::crates::MAX_BYTES
    {
        return Err(error(
            "source declaration reservations exceed aggregate retained and scratch budget",
        ));
    }
    Ok(())
}
fn declaration_sites(
    rows: s::ValidatedView<'_>,
    work: &mut Work,
) -> Result<(Vec<DeclarationBinding>, Vec<Option<Label>>), Error> {
    let mut bindings = Vec::with_capacity(rows.declarations.len());
    let mut labels = Vec::new();
    for row in &rows.declarations {
        work.charge(row.sites as usize)?;
        let first =
            u32::try_from(labels.len()).map_err(|_| error("source declaration site overflow"))?;
        let end = first
            .checked_add(row.sites)
            .ok_or_else(|| error("source declaration site overflow"))?;
        labels.resize(end as usize, None);
        bindings.push(DeclarationBinding {
            kind: kind(row.definition.kind).0,
            sites: first..end,
        });
    }
    Ok((bindings, labels))
}
fn body_sites(
    rows: s::ValidatedView<'_>,
    work: &mut Work,
) -> Result<(Vec<BodyBinding>, Vec<Option<Label>>), Error> {
    let mut counts = vec![None; rows.body_owners().len()];
    for body in &rows.originals {
        work.charge(1)?;
        let slot = counts
            .get_mut(body.body.0 as usize)
            .ok_or_else(|| error("source original site count has an unknown actual body"))?;
        if slot.replace(body.sites).is_some() {
            return Err(error("source original site count was mapped twice"));
        }
    }
    let mut bindings = Vec::with_capacity(counts.len());
    let mut labels = Vec::new();
    for (body, sites) in counts.into_iter().enumerate() {
        let sites = sites.ok_or_else(|| error("source original site coverage is incomplete"))?;
        work.charge(sites as usize)?;
        let first =
            u32::try_from(labels.len()).map_err(|_| error("source operation site overflow"))?;
        let end = first
            .checked_add(sites)
            .ok_or_else(|| error("source operation site overflow"))?;
        labels.resize(end as usize, None);
        bindings.push(BodyBinding {
            unit: crate::support::crates::owner_catalog::BodyRef(
                u32::try_from(body).map_err(|_| error("source body index overflow"))?,
            ),
            sites: first..end,
        });
    }
    Ok((bindings, labels))
}
