//! Coverage inhabitation is visibility-sensitive and stops at reference boundaries.
use super::decision::Graph;
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::Access, package::ModuleId},
    sema::{members::VariantId, model::Declarations},
    types::{ItemId, Kind, TypeId},
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/canonical_inhabitation_work.rs"]
mod work_tests;

/// Reuse only with the same finalized declarations/report that owns this checker.
#[derive(Default)]
pub struct Cache {
    values: HashMap<(ModuleId, TypeId), Result<bool, &'static str>>,
    tags: HashMap<(ModuleId, TypeId, bool), Arc<[VariantId]>>,
    /// Inhabitation queries that were not answered by the tables above.
    misses: u64,
    /// Coverage storage between queries; never consulted as a result cache.
    pub(super) scratch: super::Scratch,
    /// Completed matches by structural key, recorded only when every
    /// inhabitation query was already cached, so their work is reproducible.
    matches: HashMap<Vec<u64>, super::Outcome>,
    /// Reusable key construction storage.
    key: Vec<u64>,
    key_work: Vec<crate::sema::patterns::PatternId>,
}
impl Cache {
    /// Take the reusable key buffers; return them with [`Self::restore_key`].
    pub fn take_key(&mut self) -> (Vec<u64>, Vec<crate::sema::patterns::PatternId>) {
        (
            std::mem::take(&mut self.key),
            std::mem::take(&mut self.key_work),
        )
    }
    pub fn restore_key(&mut self, key: Vec<u64>, work: Vec<crate::sema::patterns::PatternId>) {
        self.key = key;
        self.key_work = work;
    }
    pub fn misses(&self) -> u64 {
        self.misses
    }
    pub fn outcome(&self, key: &[u64]) -> Option<&super::Outcome> {
        self.matches.get(key)
    }
    pub fn remember(&mut self, key: Vec<u64>, outcome: super::Outcome) {
        self.matches.insert(key, outcome);
    }
}

pub struct Inhabited<'a> {
    pub facts: &'a Declarations,
    report: &'a Report,
    module: ModuleId,
    pub(super) cache: &'a mut Cache,
}
impl<'a> Inhabited<'a> {
    pub fn new(
        facts: &'a Declarations,
        report: &'a Report,
        module: ModuleId,
        cache: &'a mut Cache,
    ) -> Self {
        Self {
            facts,
            report,
            module,
            cache,
        }
    }
    pub fn visible(&self, access: Access) -> bool {
        match access {
            Access::Public => true,
            Access::ExternalPrivate => false,
            Access::Within(owner) => self.report.package.is_descendant(self.module, owner),
        }
    }
    fn cached(&self, ty: TypeId) -> Result<Option<bool>, &'static str> {
        self.cache.values.get(&(self.module, ty)).copied().transpose()
    }
    pub fn get(&mut self, ty: TypeId, graph: &mut Graph) -> Result<bool, &'static str> {
        graph.charge()?;
        if let Some(&value) = self.cache.values.get(&(self.module, ty)) { return value }
        if self.cache.values.len() >= 1_000_000 { return Err("inhabitation cache storage budget exceeded") }
        self.cache.misses += 1;
        let result = self.compute(ty, graph);
        if let Err(error) = result { self.remember(ty, Err(error))?; }
        result
    }
    fn remember(&mut self, ty: TypeId, value: Result<bool, &'static str>) -> Result<(), &'static str> {
        if self.cache.values.len() >= 1_000_000 && !self.cache.values.contains_key(&(self.module, ty)) {
            return Err("inhabitation cache storage budget exceeded");
        }
        self.cache.values.insert((self.module, ty), value);
        Ok(())
    }
    fn compute(&mut self, root: TypeId, graph: &mut Graph) -> Result<bool, &'static str> {
        let mut work = vec![(root, false)];
        while let Some((ty, finish)) = work.pop() {
            graph.charge()?;
            if self.cached(ty)?.is_some() { continue }
            let kind = self.facts.types.kind(ty).ok_or("missing coverage type")?;
            if finish {
                let inhabited = self.completed(ty, kind)?;
                self.remember(ty, Ok(inhabited))?;
                continue;
            }
            match kind {
                Kind::Never => { self.remember(ty, Ok(false))?; }
                Kind::Tuple(fields) => {
                    work.push((ty, true));
                    for &field in self.facts.types.list(fields).ok_or("missing tuple types")? { work.push((field, false)); }
                }
                Kind::Array { element, length } if length != 0 => {
                    work.push((ty, true)); work.push((element, false));
                }
                Kind::Adt(item, _) => {
                    if self.facts.aggregates.get(item).is_none() { return Err("missing coverage aggregate") }
                    if self.union(item) || self.facts.members.external_open(item) {
                        self.remember(ty, Ok(true))?; continue;
                    }
                    work.push((ty, true));
                    for &variant in self.facts.members.variants(item) {
                        let (fields, canonical) = self.fields(ty, variant)?;
                        for (field, &ty) in fields.iter().zip(canonical) {
                            if self.visible(field.access) { work.push((ty, false)); }
                        }
                    }
                }
                // References can point at places that do not yet contain valid values.
                _ => { self.remember(ty, Ok(true))?; }
            }
        }
        self.cached(root)?.ok_or("missing inhabitation result")
    }
    fn completed(&self, ty: TypeId, kind: Kind) -> Result<bool, &'static str> {
        match kind {
            Kind::Tuple(fields) => {
                let mut inhabited = true;
                for &field in self.facts.types.list(fields).ok_or("missing tuple types")? {
                    inhabited &= self.cached(field)?.ok_or("missing tuple inhabitation")?;
                }
                Ok(inhabited)
            }
            Kind::Array { element, length } => Ok(length == 0 || self.cached(element)?.ok_or("missing element inhabitation")?),
            Kind::Adt(item, _) => {
                let mut inhabited = false;
                for &variant in self.facts.members.variants(item) { inhabited |= self.variant_ready(ty, variant)?; }
                Ok(inhabited)
            }
            _ => Err("invalid deferred inhabitation query"),
        }
    }
    fn fields(&self, ty: TypeId, id: VariantId) -> Result<(&'a [crate::sema::members::Field], &'a [TypeId]), &'static str> {
        let facts: &'a Declarations = self.facts;
        let variant = facts.members.variants.get(id.0 as usize).ok_or("invalid inhabitation variant identity")?;
        let shape = facts.aggregates.shape(&facts.types, ty)?;
        let variants = facts.members.variants(variant.owner);
        if shape.definition.item != variant.owner || variants.len() != shape.variants().len()
            || variants.get(variant.ordinal as usize) != Some(&id) { return Err("inhabitation variant owner or ordinal changed") }
        let fields = facts.members.fields(id).ok_or("missing inhabitation member fields")?;
        let canonical = shape.variant(variant.ordinal as usize).ok_or("missing canonical inhabitation variant")?;
        let canonical = facts.types.list(canonical.fields).ok_or("missing canonical inhabitation fields")?;
        if fields.len() != canonical.len() { return Err("canonical inhabitation field count changed") }
        Ok((fields, canonical))
    }
    fn variant_ready(&self, ty: TypeId, id: VariantId) -> Result<bool, &'static str> {
        let (fields, canonical) = self.fields(ty, id)?;
        let mut inhabited = true;
        for (field, &ty) in fields.iter().zip(canonical) {
            if self.visible(field.access) { inhabited &= self.cached(ty)?.ok_or("missing field inhabitation")?; }
        }
        Ok(inhabited)
    }
    fn variant(&mut self, ty: TypeId, id: VariantId, graph: &mut Graph) -> Result<bool, &'static str> {
        let owner = self.facts.members.variants.get(id.0 as usize).ok_or("invalid inhabitation variant")?.owner;
        if self.union(owner) { return Ok(true) }
        let (fields, canonical) = self.fields(ty, id)?;
        let mut inhabited = true;
        for (field, &ty) in fields.iter().zip(canonical) {
            if self.visible(field.access) { inhabited &= self.get(ty, graph)?; }
        }
        Ok(inhabited)
    }
    fn union(&self, item: ItemId) -> bool {
        self.facts
            .aggregates
            .get(item)
            .is_some_and(|d| d.kind == crate::types::aggregate::Kind::Union)
    }
    pub fn tags(
        &mut self,
        ty: TypeId,
        item: ItemId,
        valid: bool,
        graph: &mut Graph,
    ) -> Result<Arc<[VariantId]>, &'static str> {
        let key = (self.module, ty, valid);
        if let Some(tags) = self.cache.tags.get(&key) {
            return Ok(Arc::clone(tags));
        }
        self.cache.misses += 1;
        let mut tags = Vec::new();
        for &variant in self.facts.members.variants(item) {
            graph.charge()?;
            if !valid || self.variant(ty, variant, graph)? {
                tags.push(variant);
            }
        }
        let tags: Arc<[VariantId]> = tags.into();
        self.cache.tags.insert(key, Arc::clone(&tags));
        Ok(tags)
    }
}
