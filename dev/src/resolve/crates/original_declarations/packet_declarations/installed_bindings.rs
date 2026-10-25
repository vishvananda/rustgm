//! Publish only exact fresh keys; an installation failure withdraws the batch.
mod foreign_bindings;
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
    scope: ScopeId,
    name: NameId,
    namespace: Namespace,
    binding: Binding,
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
            imported
                .origins
                .ensure()
                .map_err(error)?
                .install(&resolver.facts.definitions, input)
                .map_err(error)?;
            Ok(())
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
        return foreign_bindings::prepare(resolver, artifact, rows, definitions, imported, work);
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
