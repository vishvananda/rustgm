//! Publish only exact fresh keys; an installation failure withdraws the batch.
mod binding_publication;
use super::{
    Access, DefId, Error, ExternalDef, ExternalKind, Imported, Input, Interface, Resolver, ScopeId,
    Work, error, kind, s,
};
use crate::{
    expand::marks::ItemContext,
    resolve::{
        model::{Binding, Namespace, Origin},
        names::NameId,
    },
    support::{hash::HashSet, retained},
};

pub(super) struct PendingBinding {
    pub(super) scope: ScopeId,
    pub(super) name: NameId,
    pub(super) namespace: Namespace,
    pub(super) binding: Binding,
}
pub(super) struct Pending {
    bindings: Vec<PendingBinding>,
    external: Vec<(ExternalDef, DefId)>,
}
impl Pending {
    pub(super) fn install(
        self,
        resolver: &mut Resolver<'_>,
        imported: &mut Imported,
        input: Input,
    ) -> Result<(), Error> {
        self.publish(resolver, imported, |resolver, imported| {
            imported
                .origins
                .ensure()
                .map_err(error)?
                .install(&resolver.facts.definitions, input)
                .map(|_| ())
                .map_err(error)
        })
    }
    pub(super) fn install_charged(
        self,
        resolver: &mut Resolver<'_>,
        imported: &mut Imported,
        input: Input,
        work: &mut Work,
    ) -> Result<(), Error> {
        if imported.origins.get().is_some() {
            return Err(error(
                "named reservation cannot replace an existing catalog",
            ));
        }
        self.publish(resolver, imported, |resolver, imported| {
            use crate::resolve::body_origin::{Catalog, Limits};
            let mut catalog = Catalog::new(Limits {
                bytes: crate::support::crates::MAX_BYTES.saturating_sub(work.retained),
                steps: crate::support::crates::MAX_COMPONENTS.saturating_sub(work.steps),
                ..Limits::default()
            })
            .map_err(error)?;
            let result = catalog
                .install_charged(&resolver.facts.definitions, input, |peak| {
                    work.reserve_raw(peak)
                })
                .and_then(|_| catalog.freeze_owned());
            work.charge(catalog.work())?;
            result.map_err(error)?;
            imported.origins = catalog.into_slot().map_err(error)?;
            Ok(())
        })
    }
    fn publish(
        self,
        resolver: &mut Resolver<'_>,
        imported: &mut Imported,
        install: impl FnOnce(&mut Resolver<'_>, &mut Imported) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut inserted = 0;
        let installed = (|| {
            for pending in &self.bindings {
                if !resolver
                    .facts
                    .bindings
                    .declare_key(
                        (
                            pending.scope,
                            pending.name,
                            pending.namespace,
                            ItemContext::ROOT,
                        ),
                        pending.binding,
                        false,
                    )
                    .map_err(error)?
                {
                    return Err(error(
                        "source namespace changed after its complete preflight",
                    ));
                }
                inserted += 1;
            }
            install(resolver, imported)
        })();
        if installed.is_err() {
            for pending in self.bindings.iter().take(inserted) {
                resolver.facts.bindings.remove_key((
                    pending.scope,
                    pending.name,
                    pending.namespace,
                    ItemContext::ROOT,
                ));
            }
            return installed;
        }
        // All fallible checks and catalog validation completed before these
        // ordinary HashMap moves. Required methods retain declaration identity.
        for (identity, definition) in self.external {
            imported.definitions.insert(identity, definition);
        }
        Ok(())
    }
}
pub(super) fn prepare_namespace(
    resolver: &mut Resolver<'_>,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    imported: &Imported,
    names: &s::NamespaceColumns,
    work: &mut Work,
) -> Result<Pending, Error> {
    let count = names.bindings.len().saturating_add(rows.functions.len());
    work.charge(count.saturating_add(rows.declarations.len()))?;
    work.reserve(
        retained::new_map::<(ScopeId, NameId, Namespace), ()>(count)
            .saturating_add(count.saturating_mul(4 * size_of::<PendingBinding>()))
            .saturating_add(names.bindings.len().saturating_mul(size_of::<(
                ScopeId,
                NameId,
                Namespace,
            )>()))
            .saturating_add(
                rows.functions
                    .len()
                    .saturating_mul(4 * size_of::<(ExternalDef, DefId)>()),
            ),
    )?;
    binding_publication::actual_cells(resolver, rows, definitions, work)?;
    let mut bindings =
        super::namespace_catalog::pending_columns(resolver, names, definitions, work)?;
    work.reserve(
        crate::support::retained::Growth::vector::<PendingBinding>(
            bindings.len(),
            bindings.capacity(),
            rows.functions.len(),
        )
        .bytes,
    )?;
    bindings.reserve(rows.functions.len());
    append_members(resolver, rows, definitions, &mut bindings, work)?;
    let mut external = Vec::with_capacity(rows.functions.len());
    for header in &rows.functions {
        work.charge(1)?;
        if header.parent.is_some_and(|parent| {
            rows.declarations
                .get(parent.0 as usize)
                .is_some_and(|row| row.definition.kind == s::DefinitionKind::SelfType)
        }) {
            // The genuine impl remains in the complete catalog. Its eventual
            // instance uses that canonical owner, never a fabricated public path.
            continue;
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
            return Err(error(
                "named function has a duplicate stable external identity",
            ));
        }
        external.push((identity, definitions[header.declaration.0 as usize]));
    }
    Ok(Pending { bindings, external })
}
fn append_members(
    resolver: &Resolver<'_>,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    bindings: &mut Vec<PendingBinding>,
    work: &mut Work,
) -> Result<(), Error> {
    for header in &rows.functions {
        work.charge(1)?;
        let Some(parent) = header.parent else {
            continue;
        };
        if !matches!(
            rows.declarations
                .get(parent.0 as usize)
                .map(|row| row.definition.kind),
            Some(s::DefinitionKind::Trait | s::DefinitionKind::SelfType)
        ) {
            return Err(error("named member has an unsupported actual owner"));
        }
        let target = definitions[header.declaration.0 as usize];
        let actual = &resolver.facts.definitions[target.0 as usize];
        if resolver
            .facts
            .bindings
            .get(actual.scope, actual.name, actual.namespace)
            .is_some()
        {
            return Err(error("named member has an existing binding"));
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
    Ok(())
}
pub(super) fn prepare(
    resolver: &mut Resolver<'_>,
    root: ScopeId,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    definitions: &[DefId],
    imported: &Imported,
    work: &mut Work,
) -> Result<Pending, Error> {
    if rows.descriptor().profile() == s::Profile::Source2024ModuleContracts {
        return binding_publication::prepare(resolver, artifact, rows, definitions, imported, work);
    }
    let scratch = retained::new_map::<(ScopeId, NameId, Namespace), ()>(rows.declarations.len());
    if work.retained.saturating_add(scratch) > crate::support::crates::MAX_BYTES {
        return Err(error("source binding preflight exceeds aggregate budget"));
    }
    let mut keys = HashSet::default();
    let mut bindings = Vec::new();
    let mut external = Vec::new();
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let actual = resolver
            .facts
            .definitions
            .get(definitions[index].0 as usize)
            .ok_or_else(|| error("source namespace reservation is incomplete"))?;
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
                "source remap changed actual declaration kind/name/namespace/access",
            ));
        }
        if !matches!(
            row.definition.kind,
            s::DefinitionKind::Trait | s::DefinitionKind::Function
        ) {
            continue;
        }
        if row.definition.kind == s::DefinitionKind::Trait && actual.scope != root {
            return Err(error(
                "source trait is outside the supported root namespace",
            ));
        }
        let key = (actual.scope, actual.name, actual.namespace);
        if !keys.insert(key) || resolver.facts.bindings.get(key.0, key.1, key.2).is_some() {
            return Err(error(
                "source namespace has a duplicate or preexisting actual key",
            ));
        }
        bindings.push(PendingBinding {
            scope: key.0,
            name: key.1,
            namespace: key.2,
            binding: Binding {
                target: definitions[index],
                access: actual.access,
                origin: Origin::Direct,
                ambiguous: false,
            },
        });
        if row.definition.kind == s::DefinitionKind::Function {
            let name = row
                .name
                .as_ref()
                .ok_or_else(|| error("source function has no canonical spelling"))?;
            let mut path = Vec::new();
            if actual.scope != root {
                let parent = row
                    .definition
                    .parent
                    .ok_or_else(|| error("source member lost its actual trait parent"))?;
                let parent = &rows.declarations[parent.0 as usize];
                if parent.definition.kind != s::DefinitionKind::Trait {
                    return Err(error(
                        "source implementation methods require a wider body profile",
                    ));
                }
                path.push(
                    parent
                        .name
                        .clone()
                        .ok_or_else(|| error("source trait has no canonical spelling"))?,
                );
            }
            path.push(name.clone());
            work.charge(path.iter().map(String::len).sum::<usize>())?;
            let identity =
                ExternalDef::new(artifact.identity().clone(), path, ExternalKind::Function)
                    .map_err(error)?;
            if imported.definition(&identity).is_some() {
                return Err(error(
                    "source function has a duplicate stable external identity",
                ));
            }
            external.push((identity, definitions[index]));
        }
    }
    Ok(Pending { bindings, external })
}
