//! Canonical v2 coordinates install one real target per declaration and alias key.
use super::{
    Access, Binding, DefId, Error, ExternalDef, ExternalKind, Imported, Interface, NameId,
    Namespace, Origin, Pending, PendingBinding, Resolver, ScopeId, Work, error, kind, s,
};
use crate::support::{hash::HashSet, retained};
pub(super) fn prepare(
    resolver: &mut Resolver<'_>,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    imported: &Imported,
    work: &mut Work,
) -> Result<Pending, Error> {
    let modules = rows.module_contracts().map_err(error)?;
    let names = &modules.rows().namespace;
    work.charge(names.bindings.len().saturating_add(rows.declarations.len()))?;
    work.reserve(
        retained::new_map::<(ScopeId, NameId, Namespace), ()>(names.bindings.len())
            .saturating_add(
                names
                    .bindings
                    .len()
                    .saturating_mul(4 * size_of::<PendingBinding>()),
            )
            .saturating_add(
                rows.functions
                    .len()
                    .saturating_mul(4 * size_of::<(ExternalDef, DefId)>()),
            ),
    )?;
    actual_cells(resolver, rows, definitions, work)?;
    let mut keys = HashSet::default();
    let mut bindings = Vec::new();
    let mut external = Vec::new();
    for entry in &names.bindings {
        work.charge(1 + entry.name.len())?;
        let module = resolver
            .facts
            .definitions
            .get(definitions[entry.module.0 as usize].0 as usize)
            .ok_or_else(|| error("v2 module declaration is absent"))?;
        let scope = module
            .members
            .ok_or_else(|| error("v2 actual module has no member scope"))?;
        work.reserve(4 * (entry.name.len() + 64))?;
        let name = resolver.package.names.intern(&entry.name).map_err(error)?;
        if resolver.package.names.get(name) != entry.name {
            return Err(error("v2 namespace spelling is not canonical"));
        }
        let namespace = match entry.namespace {
            crate::support::crates::Namespace::Type => Namespace::Type,
            crate::support::crates::Namespace::Value => Namespace::Value,
        };
        let key = (scope, name, namespace);
        if !keys.insert(key)
            || resolver
                .facts
                .bindings
                .get(scope, name, namespace)
                .is_some()
        {
            return Err(error("v2 namespace has duplicate/preexisting keys"));
        }
        bindings.push(PendingBinding {
            scope,
            name,
            namespace,
            binding: Binding {
                target: definitions[entry.target.declaration.0 as usize],
                access: if matches!(entry.visibility, s::ModuleVisibility::Public) {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                },
                origin: Origin::Direct,
                ambiguous: false,
            },
        });
    }
    for header in &rows.functions {
        let path = s::callable_path(
            &rows.declarations,
            names.root,
            header.declaration,
            |count| work.charge(count),
            error,
        )?;
        work.reserve(path.storage_bound())?;
        let path = path.materialize().map_err(error)?;
        let identity = ExternalDef::new(artifact.identity().clone(), path, ExternalKind::Function)
            .map_err(error)?;
        if imported.definition(&identity).is_some() {
            return Err(error("v2 function has duplicate stable external identity"));
        }
        external.push((identity, definitions[header.declaration.0 as usize]));
    }
    Ok(Pending { bindings, external })
}
pub(super) fn actual_cells(
    resolver: &Resolver<'_>,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    work: &mut Work,
) -> Result<(), Error> {
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let actual = resolver
            .facts
            .definitions
            .get(definitions[index].0 as usize)
            .ok_or_else(|| error("v2 declaration reservation is absent"))?;
        if actual.node.is_some()
            || (actual.kind, actual.namespace) != kind(row.definition.kind)
            || row.name.as_deref() != Some(resolver.package.names.get(actual.name))
            || actual.access
                != if row.public {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                }
        {
            return Err(error(
                "v2 remap changed its actual declaration kind/name/access",
            ));
        }
    }
    Ok(())
}
