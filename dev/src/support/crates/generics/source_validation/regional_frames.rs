//! One binder forest traversal resolves all labelled depth queries in linear work.
#[path = "binder_domains/frame_buckets.rs"]
mod buckets;
#[path = "binder_domains/frame_owners.rs"]
mod kinds;
#[path = "binder_domains/late_slots.rs"]
mod late_slots;
#[cfg(test)]
#[path = "../../../../../tests/invariants/regional_forest.rs"]
mod tests;
use super::{HeaderIndex, HeaderRef, SourceRows, Work};
use crate::support::crates::{
    self,
    generics::{self as g, source as s},
};

pub(super) struct BinderIndex<'a> {
    rows: &'a SourceRows,
    enter: Vec<u32>,
    exit: Vec<u32>,
    targets: Vec<Option<g::BinderRef>>,
    declarations: Vec<Option<g::BinderRef>>,
    pub bytes: usize,
    pub peak: usize,
    pub queries: usize,
    pub max_depth: usize,
}
impl<'a> BinderIndex<'a> {
    pub(super) fn new(headers: &HeaderIndex<'a>, work: &mut Work) -> Result<Self, &'static str> {
        let rows = headers.rows();
        let (bytes, peak) = preflight(headers, work)?;
        let count = rows.binders.len();
        let root = u32::try_from(count).map_err(|_| "source binder root index overflow")?;
        work.charge(
            count
                .saturating_mul(2)
                .saturating_add(rows.regions.len())
                .saturating_add(rows.declarations.len()),
        )?;
        let mut index = Self {
            rows,
            enter: vec![0; count],
            exit: vec![0; count],
            targets: vec![None; rows.regions.len()],
            declarations: vec![None; rows.declarations.len()],
            bytes,
            peak,
            queries: 0,
            max_depth: 0,
        };
        let mut slots = 0_usize;
        for (ordinal, frame) in rows.binders.iter().enumerate() {
            work.charge(frame.count as usize)?;
            slots = slots.saturating_add(frame.count as usize);
            if slots > crates::MAX_COMPONENTS {
                return Err("source binder slots exceed the complete unused-formal budget");
            }
            index.frame(headers, ordinal, frame, work)?;
        }
        let children = buckets::Buckets::new(
            count.saturating_add(1),
            count,
            rows.binders.iter().enumerate().map(|(ordinal, frame)| {
                Ok((
                    frame.parent.map_or(root, |parent| parent.0),
                    u32::try_from(ordinal).map_err(|_| "source binder ordinal overflow")?,
                ))
            }),
            work,
        )?;
        work.charge(rows.regions.len())?;
        let bound = rows
            .regions
            .iter()
            .filter(|region| matches!(region, s::Region::Bound { .. }))
            .count();
        let queries = buckets::Buckets::new(
            count,
            bound,
            rows.regions
                .iter()
                .enumerate()
                .filter_map(|(ordinal, region)| {
                    let s::Region::Bound { binder, .. } = region else {
                        return None;
                    };
                    Some(
                        u32::try_from(ordinal)
                            .map(|ordinal| (binder.0, ordinal))
                            .map_err(|_| "source region query ordinal overflow"),
                    )
                }),
            work,
        )?;
        index.walk(root, &children, &queries, work)?;
        index.complete(headers, work)?;
        late_slots::validate(headers, work)?;
        Ok(index)
    }

    pub(super) fn ancestor(
        &self,
        parent: g::BinderRef,
        child: g::BinderRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(3)?;
        let start = *self
            .enter
            .get(parent.0 as usize)
            .ok_or("source ancestor binder is unknown")?;
        let end = *self
            .exit
            .get(parent.0 as usize)
            .ok_or("source ancestor endpoint is unknown")?;
        let child = *self
            .enter
            .get(child.0 as usize)
            .ok_or("source child binder is unknown")?;
        Ok(start <= child && child < end)
    }
    pub(super) fn target(
        &self,
        region: s::RegionRef,
        work: &mut Work,
    ) -> Result<Option<g::BinderRef>, &'static str> {
        work.charge(1)?;
        self.targets
            .get(region.0 as usize)
            .copied()
            .ok_or("source region target is unknown")
    }
    pub(super) fn declaration(
        &self,
        owner: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut Work,
    ) -> Result<g::BinderRef, &'static str> {
        work.charge(1)?;
        self.declarations
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source declaration has no exact header binder")
    }

    fn walk(
        &mut self,
        root: u32,
        children: &buckets::Buckets,
        queries: &buckets::Buckets,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut tasks = Vec::with_capacity(self.rows.binders.len().saturating_mul(2));
        let mut path = Vec::with_capacity(self.rows.binders.len());
        work.charge(self.rows.binders.len())?;
        tasks.extend(
            children
                .get(root)?
                .iter()
                .rev()
                .map(|&frame| (frame, false)),
        );
        let mut clock = 0_u32;
        while let Some((ordinal, finish)) = tasks.pop() {
            work.charge(1)?;
            if finish {
                if path.pop() != Some(ordinal) {
                    return Err("source binder forest traversal changed parent");
                }
                self.exit[ordinal as usize] = clock;
                continue;
            }
            if self.rows.binders[ordinal as usize]
                .parent
                .map(|parent| parent.0)
                != path.last().copied()
            {
                return Err("source binder forest has inconsistent actual ancestry");
            }
            self.enter[ordinal as usize] = clock;
            clock = clock
                .checked_add(1)
                .ok_or("source binder preorder overflow")?;
            path.push(ordinal);
            self.max_depth = self.max_depth.max(path.len());
            for &region in queries.get(ordinal)? {
                work.charge(1)?;
                self.queries = self.queries.saturating_add(1);
                let s::Region::Bound { depth, slot, .. } = self.rows.regions[region as usize]
                else {
                    return Err("source binder query changed its region kind");
                };
                let position = path
                    .len()
                    .checked_sub(usize::from(depth).saturating_add(1))
                    .ok_or("source region escapes its complete binder ancestry")?;
                let target = path[position];
                if slot >= self.rows.binders[target as usize].count {
                    return Err("source bound region exceeds its exact ancestor binder");
                }
                self.targets[region as usize] = Some(g::BinderRef(target));
            }
            tasks.push((ordinal, true));
            let nested = children.get(ordinal)?;
            work.charge(nested.len())?;
            tasks.extend(nested.iter().rev().map(|&frame| (frame, false)));
        }
        if clock as usize != self.rows.binders.len() {
            return Err("source binder forest has incomplete coverage");
        }
        Ok(())
    }
}

fn preflight(headers: &HeaderIndex<'_>, work: &mut Work) -> Result<(usize, usize), &'static str> {
    let rows = headers.rows();
    work.charge(1)?;
    let bytes = headers
        .retained_bytes()
        .saturating_add(size_of::<BinderIndex<'_>>())
        .saturating_add(rows.binders.len().saturating_mul(2 * size_of::<u32>()))
        .saturating_add(
            rows.regions
                .len()
                .saturating_mul(size_of::<Option<g::BinderRef>>()),
        )
        .saturating_add(
            rows.declarations
                .len()
                .saturating_mul(size_of::<Option<g::BinderRef>>()),
        );
    // Two CSR bucket sets/cursors, one active path and at most two events per
    // frame coexist during the single traversal. No per-frame ancestry vectors.
    let scratch = (2 * size_of::<buckets::Buckets>()).saturating_add(
        rows.binders
            .len()
            .saturating_add(2)
            .saturating_mul(12 * size_of::<u32>())
            .saturating_add(rows.regions.len().saturating_mul(size_of::<u32>()))
            .saturating_add(rows.region_slots.len())
            .saturating_add(rows.slots.len().saturating_mul(size_of::<u32>()))
            .saturating_add(2 * size_of::<Vec<u32>>()),
    );
    let peak = headers.peak_bytes().max(bytes.saturating_add(scratch));
    if peak > crates::MAX_BYTES || rows.binders.len() > crates::MAX_COMPONENTS {
        return Err("source binder indexes exceed aggregate retained/scratch limits");
    }
    Ok((bytes, peak))
}
