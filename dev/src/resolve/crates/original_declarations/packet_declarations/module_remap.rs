//! Preflight and reserve real declaration/scope/binding cells before any catalog publication.
use super::{
    DefId, Definition, Error, Key, Namespace, Origin, PendingBinding, Report, Resolver, ScopeId,
    Work, error, kind, retained, s, scope_rows,
};
use crate::{
    resolve::{
        model::{Access, Binding},
        package::ModuleId,
    },
    support::hash::HashSet,
};
pub(super) fn preflight(
    report: &Report,
    snapshot: &s::UnpublishedNamespace,
    work: &mut Work,
) -> Result<(), Error> {
    let rows = snapshot.columns();
    let count = rows.declarations.len();
    let lexical = rows.originals.iter().try_fold(count, |total, body| {
        work.charge(1)?;
        Ok::<_, Error>(total.saturating_add(body.scopes.len()))
    })?;
    work.reserve(
        snapshot
            .work()
            .retained_bytes
            .saturating_add(size_of::<super::PendingNamespace>())
            .saturating_add(
                snapshot
                    .modules()
                    .len()
                    .saturating_mul(size_of::<ScopeId>()),
            )
            .saturating_add(
                report
                    .resolution
                    .definitions
                    .capacity()
                    .saturating_add(count)
                    .saturating_mul(4 * size_of::<Definition>()),
            )
            .saturating_add(
                report
                    .resolution
                    .scopes
                    .capacity()
                    .saturating_add(lexical)
                    .saturating_mul(4 * size_of::<crate::resolve::model::Scope>()),
            )
            .saturating_add(report.package.names.byte_len().saturating_mul(8))
            .saturating_add(
                report
                    .package
                    .names
                    .len()
                    .saturating_add(count)
                    .saturating_mul(256),
            )
            .saturating_add(
                snapshot
                    .bindings()
                    .len()
                    .saturating_mul(4 * (size_of::<PendingBinding>() + size_of::<Key>())),
            )
            .saturating_add(
                count.saturating_mul(4 * (size_of::<Definition>() + size_of::<ScopeId>())),
            )
            .saturating_add(retained::new_map::<Key, ()>(
                count.saturating_add(snapshot.bindings().len()),
            )),
    )?;
    Ok(())
}

pub(super) fn reserve(
    resolver: &mut Resolver<'_>,
    snapshot: &s::UnpublishedNamespace,
    work: &mut Work,
) -> Result<(Vec<DefId>, Vec<ScopeId>), Error> {
    let rows = snapshot.columns();
    let root_scope = resolver.new_scope(None, ModuleId(0), true)?;
    let mut definitions = Vec::with_capacity(rows.declarations.len());
    for row in &rows.declarations {
        work.charge(1)?;
        let spelling = row
            .name
            .as_deref()
            .ok_or_else(|| error("namespace declaration has no supported spelling"))?;
        work.charge(spelling.len())?;
        work.reserve(4 * spelling.len())?;
        let name = resolver.package.names.intern(spelling).map_err(error)?;
        if resolver.package.names.get(name) != spelling {
            return Err(error("namespace declaration spelling is not canonical"));
        }
        let definition = DefId(
            u32::try_from(resolver.facts.definitions.len())
                .map_err(|_| error("namespace remap overflow"))?,
        );
        let (kind, namespace) = kind(row.definition.kind);
        resolver.facts.definitions.push(Definition {
            name,
            namespace,
            kind,
            scope: root_scope,
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
    let root = snapshot.root().0 as usize;
    resolver.facts.definitions[definitions[root].0 as usize].members = Some(root_scope);
    let owners = scope_rows::namespace_owners(resolver, root_scope, root, rows, work)?;
    let bodies = scope_rows::bodies(
        resolver,
        rows,
        &owners,
        snapshot.owners().rows().len(),
        work,
    )?;
    scope_rows::namespace_assign(resolver, rows, &definitions, &owners, &bodies, work)?;
    Ok((definitions, owners))
}
pub(super) fn pending_bindings(
    report: &mut Report,
    snapshot: &s::UnpublishedNamespace,
    definitions: &[DefId],
    work: &mut Work,
) -> Result<(Vec<PendingBinding>, Vec<Key>), Error> {
    let mut keys = HashSet::default();
    let mut bindings = Vec::with_capacity(snapshot.bindings().len());
    let mut export_keys = Vec::with_capacity(snapshot.bindings().len());
    for entry in snapshot.bindings() {
        work.charge(1 + entry.name.len())?;
        let module = definitions[entry.module.0 as usize];
        let scope = report.resolution.definitions[module.0 as usize]
            .members
            .ok_or_else(|| error("namespace export owner has no real member scope"))?;
        let name = report.package.names.intern(&entry.name).map_err(error)?;
        let namespace = match entry.namespace {
            crate::support::crates::Namespace::Type => Namespace::Type,
            crate::support::crates::Namespace::Value => Namespace::Value,
        };
        let key = (scope, name, namespace);
        if !keys.insert(key)
            || report
                .resolution
                .bindings
                .get(scope, name, namespace)
                .is_some()
        {
            return Err(error(
                "namespace pending batch has a duplicate or existing key",
            ));
        }
        export_keys.push(key);
        bindings.push(PendingBinding {
            key,
            binding: Binding {
                target: definitions[entry.target.declaration.0 as usize],
                access: Access::Public,
                origin: Origin::Direct,
                ambiguous: false,
            },
        });
    }
    Ok((bindings, export_keys))
}
