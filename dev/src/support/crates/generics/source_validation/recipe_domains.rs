//! Shared type/application summaries retain exact parameter, binder and body owners.
#[path = "type_domains/existential_bounds.rs"]
mod existential_bounds;
#[path = "type_domains/node_rows.rs"]
mod node_rows;
#[path = "type_domains/recipe_queries.rs"]
mod queries;
#[path = "type_domains/row_edges.rs"]
mod row_edges;
#[path = "type_domains/collection_sizedness.rs"]
mod sizing;
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
    nominal_depth: Option<u16>,
    body: Option<BodyRef>,
    bound: Option<BoundUse>,
    has_function: bool,
    has_item: bool,
    sizing: sizing::Obligation,
}
pub(super) struct Types {
    summaries: Vec<Summary>,
    type_count: usize,
    function_count: usize,
    declaration_policies: bool,
    pub bytes: usize,
    pub peak: usize,
    pub visits: usize,
    pub cache_hits: usize,
}
struct Builder<'a, 'rows> {
    headers: &'a HeaderIndex<'rows>,
    binders: &'a BinderIndex<'rows>,
    profile: s::Profile,
    result: Types,
    dynamic_markers: existential_bounds::Markers,
    colours: Vec<u8>,
    pending: Vec<(usize, bool)>,
}
impl Types {
    pub(super) fn new(
        headers: &HeaderIndex<'_>,
        binders: &BinderIndex<'_>,
        profile: s::Profile,
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
                profile.declaration_policies(),
                work,
            )?;
        }
        let mut builder = Builder {
            headers,
            binders,
            profile,
            dynamic_markers: existential_bounds::Markers::new(rows, work)?,
            result: Self {
                summaries: vec![Summary::default(); count],
                type_count: rows.types.len(),
                function_count: rows.function_applications.len(),
                declaration_policies: profile.declaration_policies(),
                bytes,
                peak,
                visits: 0,
                cache_hits: 0,
            },
            colours: vec![0; count],
            pending: Vec::new(),
        };
        let mut objects = existential_bounds::Inventory::default();
        let mut opaque_results = 0_usize;
        let mut closure_types = 0_usize;
        let expected_closures = rows.closures().map_or(0, |closures| closures.types.len());
        for root in 0..count {
            match rows.types.get(root) {
                Some(s::Type::Dynamic(object)) => {
                    objects.record(&rows.cold().dynamics, *object, work)?;
                }
                Some(s::Type::OpaqueResult(reference)) => {
                    if reference.0 as usize != opaque_results
                        || opaque_results >= rows.cold().opaques.applications.len()
                    {
                        return Err("source opaque result occurs outside its actual type order");
                    }
                    opaque_results += 1;
                }
                Some(s::Type::Closure(reference)) => {
                    if reference.0 as usize != closure_types || closure_types >= expected_closures {
                        return Err("source closure type occurs outside its actual type order");
                    }
                    closure_types += 1;
                }
                _ => {}
            }
            builder.root(root, work)?;
        }
        objects.finish(&rows.cold().dynamics)?;
        if opaque_results != rows.cold().opaques.applications.len() {
            return Err("source opaque result application has no actual type owner");
        }
        if closure_types != expected_closures {
            return Err("source closure type has no actual type row owner");
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

    /// The caller already checked the root's complete scope. Walk its recipes
    /// only to register the applications that actually contribute to the call.
    pub(super) fn claim_signature_items(
        &mut self,
        rows: &s::SourceRows,
        root: g::TypeRef,
        (seen, applications): (&mut [bool], &mut [bool]),
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if !self.summary(root, work)?.has_item {
            return Ok(());
        }
        let edges = row_edges::Edges::new(rows, self.declaration_policies);
        let mut pending = Vec::new();
        self.push_signature_node(&mut pending, root.0 as usize, work)?;
        while let Some(node) = pending.pop() {
            work.charge(1)?;
            if seen[node] {
                continue;
            }
            seen[node] = true;
            if let Some(application) = node.checked_sub(self.type_count) {
                applications[application] = true;
            }
            edges.visit(node, work, |child, work| {
                if self.summaries[child].has_item && !seen[child] {
                    self.push_signature_node(&mut pending, child, work)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    fn push_signature_node(
        &mut self,
        pending: &mut Vec<usize>,
        node: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let growth = retained::Growth::vector::<usize>(pending.len(), pending.capacity(), 1);
        work.charge(growth.work.saturating_add(1))?;
        let peak = self
            .bytes
            .saturating_add(retained::vector(pending))
            .saturating_add(growth.bytes);
        if peak > crates::MAX_BYTES {
            return Err("source call signature recipes exceed the aggregate storage budget");
        }
        pending
            .try_reserve(1)
            .map_err(|_| "source call signature work stack allocation failed")?;
        self.peak = self.peak.max(peak);
        pending.push(node);
        Ok(())
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
                let edges =
                    row_edges::Edges::new(self.headers.rows(), self.profile.declaration_policies());
                edges.visit(node, work, |child, work| self.push((child, false), work))?;
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
                if owner_a == owner_b {
                    // The latest formal suffices for both scope and ordered defaults.
                    left.parameter = Some(if a.0 > b.0 { a } else { b });
                } else if self.headers.scopes().contains(owner_a, b, work)? {
                    left.parameter = Some(a);
                } else if self.headers.scopes().contains(owner_b, a, work)? {
                    left.parameter = Some(b);
                } else {
                    return Err("source type combines unrelated parameter-owner branches");
                }
            }
            _ => {}
        }
        match (left.nominal_depth, right.nominal_depth) {
            (Some(a), Some(b)) if a != b => {
                return Err("source nominal region depth differs within the same binder");
            }
            (None, right) => left.nominal_depth = right,
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
        left.has_function |= right.has_function;
        left.has_item |= right.has_item;
        left.sizing = left.sizing.union(right.sizing);
        Ok(left)
    }
}
