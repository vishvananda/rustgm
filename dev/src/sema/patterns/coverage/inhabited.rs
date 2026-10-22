//! Coverage inhabitation is visibility-sensitive and stops at reference boundaries.
use super::decision::Graph;
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::Access, package::ModuleId},
    sema::{members::VariantId, model::Declarations},
    types::{ItemId, Kind, TypeId},
};
use std::sync::Arc;

#[derive(Default)]
pub struct Cache {
    values: HashMap<(ModuleId, TypeId), bool>,
    tags: HashMap<(ModuleId, TypeId, bool), Arc<[VariantId]>>,
}

pub struct Inhabited<'a> {
    pub facts: &'a Declarations,
    report: &'a Report,
    module: ModuleId,
    cache: &'a mut Cache,
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
    pub fn get(&mut self, ty: TypeId, graph: &mut Graph) -> Result<bool, &'static str> {
        if let Some(&value) = self.cache.values.get(&(self.module, ty)) {
            graph.charge()?;
            return Ok(value);
        }
        let mut work = vec![(ty, false)];
        while let Some((ty, finish)) = work.pop() {
            graph.charge()?;
            if self.cache.values.contains_key(&(self.module, ty)) {
                continue;
            }
            let kind = self.facts.types.kind(ty).ok_or("missing coverage type")?;
            if finish {
                let inhabited = match kind {
                    Kind::Tuple(fields) => self
                        .facts
                        .types
                        .list(fields)
                        .ok_or("missing tuple types")?
                        .iter()
                        .all(|t| self.cache.values.get(&(self.module, *t)) == Some(&true)),
                    Kind::Array { element, length } => {
                        length == 0 || self.cache.values.get(&(self.module, element)) == Some(&true)
                    }
                    Kind::Adt(item, _) => self
                        .facts
                        .members
                        .variants(item)
                        .iter()
                        .any(|v| self.variant_ready(*v)),
                    _ => return Err("invalid deferred inhabitation query"),
                };
                self.cache.values.insert((self.module, ty), inhabited);
                continue;
            }
            match kind {
                Kind::Never => {
                    self.cache.values.insert((self.module, ty), false);
                }
                Kind::Tuple(fields) => {
                    work.push((ty, true));
                    for &field in self.facts.types.list(fields).ok_or("missing tuple types")? {
                        work.push((field, false));
                    }
                }
                Kind::Array { element, length } if length != 0 => {
                    work.push((ty, true));
                    work.push((element, false));
                }
                Kind::Adt(item, _) => {
                    if self.facts.aggregates.get(item).is_none() {
                        return Err("missing coverage aggregate");
                    }
                    if self.union(item) || self.facts.members.external_open(item) {
                        self.cache.values.insert((self.module, ty), true);
                        continue;
                    }
                    work.push((ty, true));
                    for &variant in self.facts.members.variants(item) {
                        for field in self
                            .facts
                            .members
                            .fields(variant)
                            .ok_or("missing coverage fields")?
                        {
                            if self.visible(field.access) {
                                work.push((field.ty, false));
                            }
                        }
                    }
                }
                // A reference can point at a not-yet-valid place. In particular,
                // &Void and slices of Void are not empty coverage domains.
                _ => {
                    self.cache.values.insert((self.module, ty), true);
                }
            }
        }
        self.cache
            .values
            .get(&(self.module, ty))
            .copied()
            .ok_or("missing inhabitation result")
    }
    fn variant_ready(&self, id: VariantId) -> bool {
        if self.union(self.facts.members.variants[id.0 as usize].owner) {
            return true;
        }
        self.facts.members.fields(id).is_some_and(|fields| {
            fields.iter().all(|f| {
                !self.visible(f.access)
                    || self.cache.values.get(&(self.module, f.ty)) == Some(&true)
            })
        })
    }
    pub fn variant(&mut self, id: VariantId, graph: &mut Graph) -> Result<bool, &'static str> {
        if self.union(self.facts.members.variants[id.0 as usize].owner) {
            return Ok(true);
        }
        for field in self
            .facts
            .members
            .fields(id)
            .ok_or("missing coverage fields")?
        {
            if self.visible(field.access) && !self.get(field.ty, graph)? {
                return Ok(false);
            }
        }
        Ok(true)
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
        let mut tags = Vec::new();
        for &variant in self.facts.members.variants(item) {
            graph.charge()?;
            if !valid || self.variant(variant, graph)? {
                tags.push(variant);
            }
        }
        let tags: Arc<[VariantId]> = tags.into();
        self.cache.tags.insert(key, Arc::clone(&tags));
        Ok(tags)
    }
}
