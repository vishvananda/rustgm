//! A glob row re-binds a dependency module's externally visible names at its
//! placement, as `pub use core::*` does in `rustc_std_workspace_core`. The
//! owner is found by crate identity through the publication's own dependency
//! list, and the target by the owner's public module identities, never by a
//! name or alias. An explicit binding of the publication shadows a glob name;
//! two globs that give one name different targets make it ambiguous, which is
//! an error only where it is used. Every binding a glob row visits is charged
//! once, when the row is expanded.
use super::{
    Access, Binding, DefId, DefKind, Error, Imported, Namespace, Origin, Resolver, ScopeId,
    bound_macro_rows::SourceModules, error,
};
use crate::{
    resolve::model::Resolution,
    support::{
        crates::{CrateIdentity, Dependencies, ExternalKind},
        hash::{HashMap, HashSet},
        names::NameId,
    },
};

/// Work one crate's glob rows may spend, one unit per row and per binding
/// of its target module.
const MAX_STEPS: usize = crate::support::crates::MAX_COMPONENTS;

/// The public glob imports of this crate whose source is a dependency's
/// module, recorded for its publication, and the consumer's glob work.
#[derive(Default)]
pub(super) struct GlobImports {
    imports: Vec<(u32, DefId)>,
    steps: usize,
}

/// Every name a glob row bound, by placement scope: a later glob row may make
/// it ambiguous, but never replace it, and an explicit binding is never one.
type Expanded = HashSet<(ScopeId, NameId, Namespace)>;

/// The owner and canonical path of one dependency module.
pub(crate) type ModuleIdentity<'a> = (&'a CrateIdentity, Vec<String>);

impl Imported {
    /// Work charged to expanding imported glob rows.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn glob_expansion_steps(&self) -> usize {
        self.cold.as_deref().map_or(0, |cold| cold.globs.steps)
    }
    /// This crate's public glob imports of dependency modules: the import's
    /// index and the module, in import order.
    #[must_use]
    pub(crate) fn module_globs(&self) -> &[(u32, DefId)] {
        self.cold.as_deref().map_or(&[], |cold| &cold.globs.imports)
    }
    fn charge_globs(&mut self, units: usize) -> Result<(), Error> {
        let globs = &mut self.cold.get_or_insert_with(Box::default).globs;
        globs.steps = globs.steps.saturating_add(units);
        if globs.steps > MAX_STEPS {
            return Err(error("glob re-export expansion exceeds its work budget"));
        }
        Ok(())
    }
    /// The owner and canonical path of each module, or `None` for a module
    /// that no dependency artifact defines.
    /// # Errors
    /// Rejects a dependency module that its owner reaches only through a
    /// non-public module, which no consumer can walk.
    pub(crate) fn module_identities(
        &self,
        modules: &[DefId],
    ) -> Result<Vec<Option<ModuleIdentity<'_>>>, &'static str> {
        let mut found = Vec::with_capacity(modules.len());
        for &module in modules {
            found.push(self.source_module(module)?);
        }
        if found.iter().any(Option::is_none) {
            let wanted: HashSet<DefId> = modules
                .iter()
                .zip(&found)
                .filter_map(|(&module, found)| found.is_none().then_some(module))
                .collect();
            let legacy: HashMap<DefId, ModuleIdentity<'_>> = self
                .definitions
                .iter()
                .filter(|&(external, id)| {
                    external.kind() == ExternalKind::Module && wanted.contains(id)
                })
                .map(|(external, &id)| (id, (external.owner(), external.path().to_vec())))
                .collect();
            for (module, found) in modules.iter().zip(&mut found) {
                if found.is_none() {
                    *found = legacy.get(module).cloned();
                }
            }
        }
        Ok(found)
    }
    /// A source artifact's module: its owner and the names of its public
    /// ancestors, from the owner's own declaration rows.
    fn source_module(&self, module: DefId) -> Result<Option<ModuleIdentity<'_>>, &'static str> {
        const PRIVATE: &str = "glob re-export target is not a public module of its owner";
        let Some(catalog) = self.origins.get() else {
            return Ok(None);
        };
        let Some(declaration) = catalog.declaration_if_imported(module)? else {
            return Ok(None);
        };
        let (owner, row, _) = catalog.declaration_coordinate(declaration)?;
        let artifacts = self.dependencies.artifacts();
        let rows = &artifacts
            .binary_search_by(|artifact| artifact.identity().cmp(owner))
            .ok()
            .and_then(|index| artifacts[index].source_descriptor())
            .ok_or("glob re-export target has no imported owner")?
            .rows()
            .declarations;
        let mut path = Vec::new();
        let mut current = row;
        loop {
            let declaration = rows
                .get(current.0 as usize)
                .ok_or("glob re-export target has no owner declaration")?;
            let Some(parent) = declaration.definition.parent else {
                break;
            };
            if !declaration.public
                || path.len() >= crate::support::crates::exported_macros::MAX_PLACEMENT_DEPTH
            {
                return Err(PRIVATE);
            }
            path.push(declaration.name.clone().ok_or(PRIVATE)?);
            current = parent;
        }
        path.reverse();
        Ok(Some((owner, path)))
    }
}

/// Retain each public glob import whose final source is a module that no
/// local declaration defines. The publication decides which dependency owns
/// it; a glob of a local module or of any enum keeps its ordinary bindings.
pub(in crate::resolve) fn record_module_globs(
    resolution: &mut Resolution,
    sources: impl Iterator<Item = (usize, ScopeId)>,
) -> Result<(), Error> {
    if resolution.crates.is_none() {
        return Ok(());
    }
    let mut wanted: HashMap<ScopeId, Vec<u32>> = HashMap::default();
    for (index, scope) in sources {
        let import = &resolution.imports[index];
        let module = resolution.scopes[scope.0 as usize].module;
        if !import.glob
            || import.access != Access::Public
            || resolution.module_scopes.get(module.0 as usize) == Some(&scope)
        {
            continue;
        }
        let index = u32::try_from(index).map_err(|_| error("glob import ordinal overflow"))?;
        wanted.entry(scope).or_default().push(index);
    }
    if wanted.is_empty() {
        return Ok(());
    }
    let mut imports = Vec::new();
    for (id, definition) in resolution.definitions.iter().enumerate() {
        if definition.node.is_some() || definition.kind != DefKind::Module {
            continue;
        }
        let Some(indices) = definition.members.and_then(|scope| wanted.get(&scope)) else {
            continue;
        };
        let module = DefId(u32::try_from(id).map_err(|_| error("too many definitions"))?);
        imports.extend(indices.iter().map(|&index| (index, module)));
    }
    if imports.is_empty() {
        return Ok(());
    }
    imports.sort_unstable_by_key(|&(index, _)| index);
    if let Some(crates) = resolution.crates.as_deref_mut() {
        crates.cold.get_or_insert_with(Box::default).globs.imports = imports;
    }
    Ok(())
}

impl Resolver<'_> {
    /// Expand every artifact's glob rows, each artifact after the artifacts
    /// its rows name, so that a glob of a glob-filled module sees its names.
    pub(super) fn expand_glob_reexports(
        &mut self,
        dependencies: &Dependencies,
        roots: &[ScopeId],
        imported: &mut Imported,
    ) -> Result<(), Error> {
        let artifacts = dependencies.artifacts();
        let mut pending = Vec::new();
        for (index, artifact) in artifacts.iter().enumerate() {
            let Some(macros) = artifact
                .macros()
                .filter(|macros| !macros.globs().is_empty())
            else {
                continue;
            };
            let mut owners = Vec::with_capacity(macros.globs().len());
            for row in macros.globs() {
                let identity = row
                    .owner()
                    .checked_sub(1)
                    .and_then(|ordinal| artifact.dependencies().get(ordinal as usize))
                    .ok_or_else(|| error("exported glob re-export names an unknown owner"))?;
                owners.push(
                    artifacts
                        .binary_search_by(|candidate| candidate.identity().cmp(identity))
                        .map_err(|_| error("glob re-export owner has no imported artifact"))?,
                );
            }
            pending.push((index, owners));
        }
        let mut done: HashSet<usize> = (0..artifacts.len())
            .filter(|index| pending.iter().all(|(artifact, _)| artifact != index))
            .collect();
        let mut expanded = Expanded::default();
        let mut modules = HashMap::<usize, Option<SourceModules>>::default();
        while !pending.is_empty() {
            let Some(position) = pending
                .iter()
                .position(|(_, owners)| owners.iter().all(|owner| done.contains(owner)))
            else {
                return Err(error("glob re-export owners form a cycle"));
            };
            let (artifact, owners) = pending.swap_remove(position);
            let interface = &artifacts[artifact];
            let rows = interface.macros().map_or(&[][..], |macros| macros.globs());
            for (row, owner) in rows.iter().zip(owners) {
                const TARGET: &str = "glob re-export target is not a public module of its owner";
                const PLACED: &str = "glob re-export placement has no public imported module";
                let target = self.placement_scope(
                    roots[owner],
                    &artifacts[owner],
                    imported,
                    modules.entry(owner).or_default(),
                    row.target(),
                    TARGET,
                )?;
                let placed = self.placement_scope(
                    roots[artifact],
                    interface,
                    imported,
                    modules.entry(artifact).or_default(),
                    row.placement(),
                    PLACED,
                )?;
                self.expand_glob(placed, target, imported, &mut expanded)?;
            }
            done.insert(artifact);
        }
        Ok(())
    }
    /// Bind each public name of `target` at `placed`. A name the placement
    /// binds itself shadows the glob's; one a glob already gave another
    /// target becomes ambiguous.
    fn expand_glob(
        &mut self,
        placed: ScopeId,
        target: ScopeId,
        imported: &mut Imported,
        expanded: &mut Expanded,
    ) -> Result<(), Error> {
        let mut visited = 1usize;
        let mut names = Vec::new();
        for (name, namespace, binding) in self.facts.bindings.scope(target) {
            visited = visited.saturating_add(1);
            if binding.access == Access::Public && Namespace::IMPORTABLE.contains(&namespace) {
                names.push((name, namespace, *binding));
            }
        }
        imported.charge_globs(visited)?;
        for (name, namespace, binding) in names {
            let key = (placed, name, namespace);
            let replacement = match self.facts.bindings.get(placed, name, namespace) {
                None => Binding {
                    target: binding.target,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: binding.ambiguous,
                },
                Some(existing)
                    if expanded.contains(&key)
                        && existing.target != binding.target
                        && !existing.ambiguous =>
                {
                    Binding {
                        ambiguous: true,
                        ..*existing
                    }
                }
                Some(_) => continue,
            };
            self.facts
                .bindings
                .insert(placed, name, namespace, replacement)
                .map_err(error)?;
            expanded.insert(key);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/shim_expansion.rs"]
mod tests;
