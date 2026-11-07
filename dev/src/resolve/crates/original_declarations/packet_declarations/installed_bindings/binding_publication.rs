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
    let conversions = rows.descriptor().profile().conversion_roles();
    let count = names
        .bindings
        .len()
        .saturating_add(if conversions { rows.functions.len() } else { 0 })
        .saturating_add(rows.associated_types.len())
        .saturating_add(rows.enums().variants.len());
    work.charge(names.bindings.len().saturating_add(rows.declarations.len()))?;
    work.reserve(
        retained::new_map::<(ScopeId, NameId, Namespace), ()>(count)
            .saturating_add(count.saturating_mul(4 * size_of::<PendingBinding>()))
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
    module_bindings(resolver, names, definitions, &mut keys, &mut bindings, work)?;
    for header in &rows.functions {
        if conversions
            && let Some(binding) = member(
                resolver,
                rows,
                definitions,
                header.declaration,
                header.parent,
                work,
            )?
        {
            if !keys.insert((binding.scope, binding.name, binding.namespace)) {
                return Err(error(
                    "v9 member duplicates an actual pending namespace key",
                ));
            }
            bindings.push(binding);
        }
        if rows.descriptor().profile().feature_predicates() {
            work.charge(1)?;
            if header.parent.is_some_and(|parent| {
                matches!(
                    rows.header(parent),
                    Ok(Some(s::HeaderRef::Implementation(_)))
                )
            }) {
                // The validated impl keeps its real catalog/body identity and
                // derives its private symbol from that same canonical owner.
                continue;
            }
        }
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
    for header in &rows.associated_types {
        let binding = member(
            resolver,
            rows,
            definitions,
            header.declaration,
            Some(header.parent),
            work,
        )?
        .ok_or_else(|| error("associated type lost its actual parent binding"))?;
        if !keys.insert((binding.scope, binding.name, binding.namespace)) {
            return Err(error("associated type duplicates a pending namespace key"));
        }
        bindings.push(binding);
    }
    variant_bindings(resolver, rows, definitions, &mut keys, &mut bindings, work)?;
    Ok(Pending { bindings, external })
}
/// Version thirteen: each variant is named once, in its enum's member scope.
/// A consumer's variant paths, constructor calls and patterns resolve through
/// this exact binding, never by spelling.
fn variant_bindings(
    resolver: &Resolver<'_>,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    keys: &mut HashSet<(ScopeId, NameId, Namespace)>,
    bindings: &mut Vec<PendingBinding>,
    work: &mut Work,
) -> Result<(), Error> {
    for header in &rows.enums().headers {
        work.charge(1)?;
        let owner =
            &resolver.facts.definitions[definitions[header.declaration.0 as usize].0 as usize];
        for variant in header
            .variants
            .slice(&rows.enums().variants)
            .map_err(error)?
        {
            work.charge(1)?;
            let target = definitions[variant.declaration.0 as usize];
            let actual = &resolver.facts.definitions[target.0 as usize];
            let key = (actual.scope, actual.name, actual.namespace);
            if owner.members != Some(actual.scope)
                || (actual.kind, actual.namespace) != kind(s::DefinitionKind::Variant)
                || !keys.insert(key)
                || resolver.facts.bindings.get(key.0, key.1, key.2).is_some()
            {
                return Err(error(
                    "source variant changed its enum member scope or duplicates a binding",
                ));
            }
            bindings.push(PendingBinding {
                scope: actual.scope,
                name: actual.name,
                namespace: actual.namespace,
                binding: Binding {
                    target,
                    access: actual.access,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            });
        }
    }
    Ok(())
}
fn member(
    resolver: &Resolver<'_>,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    parent: Option<crate::support::crates::owner_catalog::DeclarationRef>,
    work: &mut Work,
) -> Result<Option<PendingBinding>, Error> {
    work.charge(1)?;
    let Some(parent) = parent else {
        return Ok(None);
    };
    work.charge(7)?;
    let binding = super::member_binding(resolver, rows, definitions, declaration, Some(parent))?
        .ok_or_else(|| error("v9 actual member lost its parent"))?;
    let owner = &resolver.facts.definitions[definitions[parent.0 as usize].0 as usize];
    let actual = &resolver.facts.definitions[binding.binding.target.0 as usize];
    if owner.members != Some(binding.scope)
        || owner.kind != kind(rows.declarations[parent.0 as usize].definition.kind).0
        || (actual.kind, actual.namespace)
            != kind(rows.declarations[declaration.0 as usize].definition.kind)
    {
        return Err(error(
            "v9 member changed its actual parent scope, kind or namespace",
        ));
    }
    Ok(Some(binding))
}
fn module_bindings(
    resolver: &mut Resolver<'_>,
    names: &s::ModuleNamespace,
    definitions: &[DefId],
    keys: &mut HashSet<(ScopeId, NameId, Namespace)>,
    bindings: &mut Vec<PendingBinding>,
    work: &mut Work,
) -> Result<(), Error> {
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
    Ok(())
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
