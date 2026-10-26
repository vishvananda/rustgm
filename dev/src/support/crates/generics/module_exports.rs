//! Unpublished module names share real declaration columns; no descriptor is minted.
use super::{Columns, HeaderIndex, SourceRows, freeze_columns, functions};
use crate::support::{
    crates::{
        self, CrateIdentity, Namespace,
        generics::{Work, source as s},
        owner_catalog::{BodyCatalog, DeclarationRef},
    },
    retained,
};
use std::{cmp::Ordering, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct ExportBinding {
    pub module: DeclarationRef,
    pub name: String,
    pub namespace: Namespace,
    pub target: s::DeclarationAddress,
}
#[derive(Clone, Debug)]
pub(crate) struct NamespaceColumns {
    pub root: DeclarationRef,
    pub modules: Vec<DeclarationRef>,
    pub bindings: Vec<ExportBinding>,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NamespaceWork {
    pub comparisons: usize,
    pub compared_bytes: usize,
    pub swaps: usize,
    pub retained_bytes: usize,
    pub peak_bytes: usize,
}
/// Only a structural observation. It has no Descriptor/Profile/Interface accessor.
pub(crate) struct UnpublishedNamespace {
    owners: BodyCatalog,
    columns: Box<Columns>,
    headers: Box<[Option<s::HeaderRef>]>,
    names: NamespaceColumns,
    work: NamespaceWork,
}
impl UnpublishedNamespace {
    pub(crate) fn require_shared(
        expected: &Arc<Self>,
        actual: &Arc<Self>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if !Arc::ptr_eq(expected, actual) || expected.owner() != actual.owner() {
            return Err("namespace snapshot changed its actual immutable owner");
        }
        Ok(())
    }
    pub(crate) fn stage(
        owner: &CrateIdentity,
        rows: SourceRows,
        mut names: NamespaceColumns,
        work: &mut Work,
    ) -> Result<Arc<Self>, &'static str> {
        work.charge(1)?;
        let mut observed = NamespaceWork::default();
        let retained = names.storage_bytes(work)?;
        let scratch = rows
            .declarations
            .len()
            .saturating_add(names.bindings.len().saturating_mul(2 * size_of::<usize>()));
        let base = retained
            .saturating_add(scratch)
            .saturating_add(size_of::<Self>())
            .saturating_add(2 * size_of::<usize>());
        if base > crates::MAX_BYTES {
            return Err("namespace staging exceeds combined storage budget");
        }
        let indexed = HeaderIndex::new(&rows, base, work)?;
        functions::validate_namespace(&rows, &indexed, work)?;
        names.validate(&rows.declarations, work)?;
        names.sort(work, &mut observed)?;
        let domains = indexed.domains(work)?;
        let inputs = domains.original_inputs(work)?;
        let peak = inputs.peak_bytes();
        drop(inputs);
        drop(domains);
        let indexed = indexed.into_header_rows()?;
        let (owners, columns, headers, budget) =
            freeze_columns(owner, rows, indexed, base, peak, work)?;
        observed.retained_bytes = budget
            .bytes
            .saturating_sub(size_of::<super::ValidatedRows>())
            .saturating_add(size_of::<Self>() + 2 * size_of::<usize>())
            .saturating_add(retained.saturating_sub(size_of::<NamespaceColumns>()));
        observed.peak_bytes = budget
            .peak
            .saturating_add(base)
            .max(observed.retained_bytes);
        Ok(Arc::new(Self {
            owners,
            columns,
            headers,
            names,
            work: observed,
        }))
    }
    pub(crate) fn owner(&self) -> &CrateIdentity {
        self.owners.owner()
    }
    pub(crate) fn owners(&self) -> &BodyCatalog {
        &self.owners
    }
    pub(crate) fn columns(&self) -> &Columns {
        &self.columns
    }
    pub(crate) fn root(&self) -> DeclarationRef {
        self.names.root
    }
    pub(crate) fn bindings(&self) -> &[ExportBinding] {
        &self.names.bindings
    }
    pub(crate) fn modules(&self) -> &[DeclarationRef] {
        &self.names.modules
    }
    pub(crate) const fn work(&self) -> NamespaceWork {
        self.work
    }
    pub(crate) fn header(&self, row: DeclarationRef) -> Result<Option<s::HeaderRef>, &'static str> {
        self.headers
            .get(row.0 as usize)
            .copied()
            .ok_or("namespace header coordinate is unknown")
    }
    pub(crate) fn find(
        &self,
        module: DeclarationRef,
        name: &str,
        namespace: Namespace,
        work: &mut Work,
    ) -> Result<Option<usize>, &'static str> {
        let mut start = 0_usize;
        let mut end = self.names.bindings.len();
        while start < end {
            work.charge(1)?;
            let middle = start + (end - start) / 2;
            let entry = &self.names.bindings[middle];
            match compare(
                entry,
                module,
                name,
                namespace,
                work,
                &mut NamespaceWork::default(),
            )? {
                Ordering::Less => start = middle + 1,
                Ordering::Greater => end = middle,
                Ordering::Equal => return Ok(Some(middle)),
            }
        }
        Ok(None)
    }
}
impl NamespaceColumns {
    fn storage_bytes(&self, work: &mut Work) -> Result<usize, &'static str> {
        if self.modules.len() > crates::MAX_FUNCTIONS
            || self.bindings.len() > crates::MAX_COMPONENTS
        {
            return Err("namespace column cardinality exceeds its bounded profile");
        }
        // Pay for the cold capacity walk before visiting any binding/name.
        work.charge(self.bindings.len())?;
        Ok(size_of::<Self>()
            .saturating_add(retained::vector(&self.modules))
            .saturating_add(retained::vector(&self.bindings))
            .saturating_add(
                self.bindings
                    .iter()
                    .map(|b| b.name.capacity())
                    .fold(0, usize::saturating_add),
            ))
    }
    fn validate(
        &self,
        declarations: &[s::Declaration],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(declarations.len().saturating_add(self.bindings.len()))?;
        if self.modules.len() > crates::MAX_FUNCTIONS
            || self.bindings.len() > crates::MAX_COMPONENTS
        {
            return Err("namespace column cardinality exceeds its bounded profile");
        }
        let mut previous = None;
        let mut modules = vec![false; declarations.len()];
        for &module in &self.modules {
            work.charge(1)?;
            if previous.is_some_and(|p| p >= module.0) {
                return Err("namespace modules are duplicate or noncanonical");
            }
            let row = declarations
                .get(module.0 as usize)
                .filter(|row| row.definition.kind == s::DefinitionKind::Module)
                .ok_or("namespace coordinate is not an actual module")?;
            if row.definition.parent.is_none() && module != self.root {
                return Err("namespace root changed its actual module");
            }
            modules[module.0 as usize] = true;
            previous = Some(module.0);
        }
        if declarations.get(self.root.0 as usize).is_none_or(|row| {
            row.definition.parent.is_some() || row.definition.kind != s::DefinitionKind::Module
        }) || declarations
            .iter()
            .enumerate()
            .any(|(i, row)| (row.definition.kind == s::DefinitionKind::Module) != modules[i])
        {
            return Err("namespace omits part of its actual module forest");
        }
        for entry in &self.bindings {
            work.charge(1 + entry.name.len())?;
            if !modules
                .get(entry.module.0 as usize)
                .copied()
                .unwrap_or(false)
                || entry.target.artifact != s::ArtifactRef(0)
            {
                return Err("namespace binding has an unknown or foreign owner");
            }
            crates::identity::validate_component(&entry.name)?;
            let target = declarations
                .get(entry.target.declaration.0 as usize)
                .filter(|row| row.public)
                .ok_or("namespace export targets a private or absent declaration")?;
            let expected = match target.definition.kind {
                s::DefinitionKind::Module | s::DefinitionKind::Trait => Namespace::Type,
                s::DefinitionKind::Function => Namespace::Value,
                _ => return Err("namespace target requires a wider declaration-kind profile"),
            };
            if expected != entry.namespace {
                return Err("namespace binding changed its actual target namespace");
            }
        }
        Ok(())
    }
    fn sort(&mut self, work: &mut Work, observed: &mut NamespaceWork) -> Result<(), &'static str> {
        let mut swaps = observed.swaps;
        let result = super::binding_order::sort(
            &mut self.bindings,
            work,
            &mut swaps,
            |left, right, work| {
                compare(
                    left,
                    right.module,
                    &right.name,
                    right.namespace,
                    work,
                    observed,
                )
            },
        );
        observed.swaps = swaps;
        result
    }
}
fn compare(
    entry: &ExportBinding,
    module: DeclarationRef,
    name: &str,
    namespace: Namespace,
    work: &mut Work,
    observed: &mut NamespaceWork,
) -> Result<Ordering, &'static str> {
    work.charge(1)?;
    observed.comparisons = observed.comparisons.saturating_add(1);
    let module_order = entry.module.0.cmp(&module.0);
    if module_order != Ordering::Equal {
        return Ok(module_order);
    }
    for (left, right) in entry.name.bytes().zip(name.bytes()) {
        work.charge(2)?;
        observed.compared_bytes = observed.compared_bytes.saturating_add(2);
        if left != right {
            return Ok(left.cmp(&right));
        }
    }
    work.charge(1)?;
    Ok(entry
        .name
        .len()
        .cmp(&name.len())
        .then_with(|| entry.namespace.cmp(&namespace)))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/module_name_snapshots.rs"]
mod tests;
