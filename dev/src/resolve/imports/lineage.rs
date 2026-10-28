//! Actual import selections retain their namespace/context and explicit terminals.
use super::{Binding, BindingKey, DefKind, Error, Origin, Selection, Work};
use crate::{
    resolve::{
        model::{Access, Resolution, ScopeId},
        unit::NodeKey,
    },
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
};

type Edge = (BindingKey, u32);
pub(super) struct Source {
    pub key: Option<BindingKey>,
    pub binding: Binding,
}
#[derive(Default)]
pub(super) struct Lineage {
    edges: HashMap<Edge, Source>,
}
fn imported(origin: Origin) -> Option<u32> {
    match origin {
        Origin::Direct => None,
        Origin::Import(index) | Origin::Glob(index) => Some(index),
    }
}
impl Work {
    pub(in crate::resolve) fn counterpart(
        &mut self,
        owner: NodeKey,
        key: BindingKey,
        index: u32,
    ) -> Result<(BindingKey, Binding), Error> {
        self.lineage_budget.charge(owner, 1, 0)?;
        let source = self
            .lineage
            .as_deref()
            .and_then(|lineage| lineage.edges.get(&(key, index)))
            .ok_or_else(|| {
                Error::at(
                    owner.file,
                    0,
                    "hidden glob counterpart selection is required",
                )
            })?;
        let key = source.key.ok_or_else(|| {
            Error::at(
                owner.file,
                0,
                "glob counterpart cannot be a textual terminal",
            )
        })?;
        Ok((key, source.binding))
    }
    pub(super) fn record_source(
        &mut self,
        owner: NodeKey,
        key: BindingKey,
        index: usize,
        binding: Binding,
        selected: Option<Selection>,
    ) -> Result<(), Error> {
        let index = u32::try_from(index)
            .map_err(|_| Error::at(owner.file, 0, "import lineage index overflow"))?;
        self.lineage_budget.charge(owner, 1, 0)?;
        let source = if let Some(selected) = selected {
            if selected.binding != binding || selected.key.2 != key.2 {
                return Err(Error::at(
                    owner.file,
                    0,
                    "import lineage lost its actual selection",
                ));
            }
            Source {
                key: Some(selected.key),
                binding,
            }
        } else if binding.origin == Origin::Direct {
            // Textual macro and root-module selections are explicit terminals.
            Source { key: None, binding }
        } else {
            return Err(Error::at(
                owner.file,
                0,
                "imported source selection is required",
            ));
        };
        if self.lineage.is_none() {
            self.lineage_budget.charge(owner, 1, size_of::<Lineage>())?;
            self.lineage = Some(Box::default());
        }
        let Some(lineage) = self.lineage.as_mut() else {
            return Err(Error::at(owner.file, 0, "import lineage payer disappeared"));
        };
        let edge = (key, index);
        let growth = Growth::map::<Edge, Source>(
            lineage.edges.len(),
            lineage.edges.capacity(),
            usize::from(!lineage.edges.contains_key(&edge)),
        );
        self.lineage_budget
            .charge(owner, 1 + growth.work, growth.bytes)?;
        lineage.edges.insert(edge, source);
        Ok(())
    }
    pub(super) fn remove_source(
        &mut self,
        owner: NodeKey,
        key: BindingKey,
        origin: Origin,
    ) -> Result<(), Error> {
        self.lineage_budget.charge(owner, 1, 0)?;
        if let Some(index) = imported(origin)
            && let Some(lineage) = self.lineage.as_mut()
        {
            lineage.edges.remove(&(key, index));
        }
        Ok(())
    }
    /// Each public scope and imported binding chain is visited at most once.
    pub(in crate::resolve) fn exported_globs(
        &mut self,
        facts: &Resolution,
        owner: NodeKey,
    ) -> Result<HashSet<u32>, Error> {
        let root = facts
            .module_scopes
            .first()
            .copied()
            .ok_or_else(|| Error::at(owner.file, 0, "public import root is required"))?;
        let mut scopes = HashSet::default();
        let mut pending = Vec::new();
        let mut visited = HashSet::default();
        let mut exported = HashSet::default();
        self.enqueue_public(owner, root, &mut scopes, &mut pending)?;
        while let Some(scope) = pending.pop() {
            for (name, ns, binding) in facts.bindings.scope(scope) {
                self.lineage_budget.charge(owner, 1, 0)?;
                if binding.access != Access::Public || binding.ambiguous {
                    continue;
                }
                let declaration = &facts.definitions[binding.target.0 as usize];
                if declaration.kind == DefKind::Module
                    && let Some(members) = declaration.members
                {
                    self.enqueue_public(owner, members, &mut scopes, &mut pending)?;
                }
                self.export_chain(
                    facts,
                    owner,
                    (scope, name, ns, crate::expand::marks::ItemContext::ROOT),
                    *binding,
                    &mut visited,
                    &mut exported,
                )?;
            }
        }
        Ok(exported)
    }
    fn enqueue_public(
        &mut self,
        owner: NodeKey,
        scope: ScopeId,
        seen: &mut HashSet<ScopeId>,
        pending: &mut Vec<ScopeId>,
    ) -> Result<(), Error> {
        self.lineage_budget.charge(owner, 1, 0)?;
        if seen.contains(&scope) {
            return Ok(());
        }
        let table = Growth::map::<ScopeId, ()>(seen.len(), seen.capacity(), 1);
        let queue = Growth::vector::<ScopeId>(pending.len(), pending.capacity(), 1);
        self.lineage_budget.charge(
            owner,
            1 + table.work + queue.work,
            table.bytes + queue.bytes,
        )?;
        seen.insert(scope);
        pending.push(scope);
        Ok(())
    }
    fn export_chain(
        &mut self,
        facts: &Resolution,
        owner: NodeKey,
        mut key: BindingKey,
        mut binding: Binding,
        visited: &mut HashSet<Edge>,
        exported: &mut HashSet<u32>,
    ) -> Result<(), Error> {
        while let Some(index) = imported(binding.origin) {
            self.lineage_budget.charge(owner, 1, 0)?;
            if binding.access != Access::Public
                || binding.ambiguous
                || visited.contains(&(key, index))
            {
                return Ok(());
            }
            let growth = Growth::map::<Edge, ()>(visited.len(), visited.capacity(), 1);
            self.lineage_budget
                .charge(owner, 1 + growth.work, growth.bytes)?;
            visited.insert((key, index));
            if matches!(binding.origin, Origin::Glob(_)) && !exported.contains(&index) {
                let growth = Growth::map::<u32, ()>(exported.len(), exported.capacity(), 1);
                self.lineage_budget
                    .charge(owner, 1 + growth.work, growth.bytes)?;
                exported.insert(index);
            }
            let source = self
                .lineage
                .as_ref()
                .and_then(|lineage| lineage.edges.get(&(key, index)))
                .ok_or_else(|| {
                    Error::at(owner.file, 0, "exported import source lineage is required")
                })?;
            if source.binding.target != binding.target {
                return Err(Error::at(
                    owner.file,
                    0,
                    "exported import target lineage changed",
                ));
            }
            if let Some(source_key) = source.key {
                if facts.bindings.get_key(source_key) != Some(&source.binding) {
                    return Err(Error::at(
                        owner.file,
                        0,
                        "completed import source selection changed",
                    ));
                }
                key = source_key;
            } else if source.binding.origin != Origin::Direct {
                return Err(Error::at(
                    owner.file,
                    0,
                    "imported source cannot be an unproved terminal",
                ));
            }
            binding = source.binding;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/import_lineage.rs"]
mod tests;
