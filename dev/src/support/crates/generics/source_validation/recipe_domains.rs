//! Shared type/application summaries retain exact parameter, binder and body owners.
#[path = "type_domains/node_rows.rs"]
mod node_rows;
#[path = "type_domains/recipe_queries.rs"]
mod queries;
use super::{HeaderIndex, HeaderRef, Work, binder_domains::BinderIndex};
use crate::support::{
    crates::{
        self,
        generics::{self as g, source as s},
        owner_catalog::BodyRef,
    },
    retained,
};
pub(super) use queries::Context;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BoundUse {
    frame: g::BinderRef,
    depth: u16,
}
#[derive(Clone, Copy, Default)]
struct Summary {
    parameter: Option<s::SlotRef>,
    body: Option<BodyRef>,
    bound: Option<BoundUse>,
}
pub(super) struct Types {
    summaries: Vec<Summary>,
    type_count: usize,
    function_count: usize,
    pub bytes: usize,
    pub peak: usize,
    pub visits: usize,
    pub cache_hits: usize,
}
struct Builder<'a, 'rows> {
    headers: &'a HeaderIndex<'rows>,
    binders: &'a BinderIndex<'rows>,
    result: Types,
    colours: Vec<u8>,
    pending: Vec<(usize, bool)>,
}
impl Types {
    pub(super) fn new(
        headers: &HeaderIndex<'_>,
        binders: &BinderIndex<'_>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let rows = headers.rows();
        let count = rows
            .types
            .len()
            .saturating_add(rows.function_applications.len())
            .saturating_add(rows.trait_applications.len());
        let bytes = binders
            .bytes
            .saturating_add(size_of::<Self>())
            .saturating_add(count.saturating_mul(size_of::<Summary>()));
        let peak = binders.peak.max(bytes.saturating_add(count));
        work.charge(count.saturating_mul(2))?;
        if count > crates::MAX_COMPONENTS || peak > crates::MAX_BYTES {
            return Err("source type/application indexes exceed their shared budget");
        }
        for index in 0..rows.regions.len() {
            node_rows::region_summary(
                headers,
                binders,
                s::RegionRef(u32::try_from(index).map_err(|_| "source region ordinal overflow")?),
                work,
            )?;
        }
        let mut builder = Builder {
            headers,
            binders,
            result: Self {
                summaries: vec![Summary::default(); count],
                type_count: rows.types.len(),
                function_count: rows.function_applications.len(),
                bytes,
                peak,
                visits: 0,
                cache_hits: 0,
            },
            colours: vec![0; count],
            pending: Vec::new(),
        };
        for root in 0..count {
            builder.root(root, work)?;
        }
        Ok(builder.result)
    }
    fn summary(&self, ty: g::TypeRef, work: &mut Work) -> Result<Summary, &'static str> {
        work.roots = work.roots.saturating_add(1);
        work.charge(1)?;
        if ty.0 as usize >= self.type_count {
            return Err("source type root is unknown");
        }
        self.summaries
            .get(ty.0 as usize)
            .copied()
            .ok_or("source type summary is absent")
    }
}
impl Builder<'_, '_> {
    fn root(&mut self, root: usize, work: &mut Work) -> Result<(), &'static str> {
        self.push((root, false), work)?;
        while let Some((node, finish)) = self.pending.pop() {
            work.charge(1)?;
            match self.colours.get(node) {
                Some(2) => {
                    self.result.cache_hits = self.result.cache_hits.saturating_add(1);
                    continue;
                }
                Some(1) if !finish => return Err("source type/application recipe cycle"),
                None => return Err("source type/application child is unknown"),
                _ => {}
            }
            if finish {
                let summary = self.finish(node, work)?;
                self.result.summaries[node] = summary;
                self.colours[node] = 2;
                self.result.visits = self.result.visits.saturating_add(1);
            } else {
                self.colours[node] = 1;
                self.push((node, true), work)?;
                self.dependencies(node, work)?;
            }
        }
        Ok(())
    }
    fn push(&mut self, task: (usize, bool), work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        if self.pending.len() == self.pending.capacity() {
            let capacity = self
                .pending
                .len()
                .saturating_add(1)
                .max(4)
                .saturating_mul(4);
            let peak = self
                .result
                .bytes
                .saturating_add(retained::vector(&self.colours))
                .saturating_add(capacity.saturating_mul(size_of::<(usize, bool)>()));
            if peak > crates::MAX_BYTES {
                return Err("source type work stack exceeds aggregate retained/scratch budget");
            }
            work.charge(self.pending.len())?;
            self.pending
                .try_reserve(1)
                .map_err(|_| "source type work stack allocation failed")?;
            self.result.peak = self.result.peak.max(peak);
        }
        self.pending.push(task);
        Ok(())
    }
    fn child(&self, ty: g::TypeRef, work: &mut Work) -> Result<Summary, &'static str> {
        work.charge(1)?;
        let index = ty.0 as usize;
        if index >= self.result.type_count || self.colours.get(index) != Some(&2) {
            return Err("source type child was not completed before its parent");
        }
        Ok(self.result.summaries[index])
    }
    fn push_type(&mut self, ty: g::TypeRef, work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        if ty.0 as usize >= self.result.type_count {
            return Err("source type reference crosses its typed table namespace");
        }
        self.push((ty.0 as usize, false), work)
    }
    fn merge(
        &self,
        mut left: Summary,
        right: Summary,
        work: &mut Work,
    ) -> Result<Summary, &'static str> {
        work.charge(1)?;
        match (left.parameter, right.parameter) {
            (None, right) => left.parameter = right,
            (Some(a), Some(b)) if a != b => {
                let rows = self.headers.rows();
                let owner_a = rows.slots[a.0 as usize].owner;
                let owner_b = rows.slots[b.0 as usize].owner;
                if self.headers.scopes().contains(owner_a, b, work)? {
                    left.parameter = Some(a);
                } else if self.headers.scopes().contains(owner_b, a, work)? {
                    left.parameter = Some(b);
                } else {
                    return Err("source type combines unrelated parameter-owner branches");
                }
            }
            _ => {}
        }
        match (left.body, right.body) {
            (Some(a), Some(b)) if a != b => {
                return Err("source type combines foreign original allocations");
            }
            (None, right) => left.body = right,
            _ => {}
        }
        match (left.bound, right.bound) {
            (Some(a), Some(b)) if a.frame != b.frame => {
                return Err("source type combines different current binder frames");
            }
            (Some(a), Some(b)) => {
                left.bound = Some(BoundUse {
                    depth: a.depth.max(b.depth),
                    ..a
                });
            }
            (None, right) => left.bound = right,
            _ => {}
        }
        Ok(left)
    }
}
