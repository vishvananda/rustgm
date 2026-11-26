//! An opened call may refer to two identities related by its caller's proof.
use super::{
    DomainIndex, Work,
    nodes::{Arena, Region, Rewrite},
    s, uses,
};
use crate::support::{crates::owner_catalog::BodyRef, hash::HashSet, retained};

type Edge = (BodyRef, Region, Region);

#[derive(Default)]
pub(super) struct Equations {
    indexed: HashSet<BodyRef>,
    edges: HashSet<Edge>,
}
impl Equations {
    pub(super) fn retained_bytes(&self) -> usize {
        self.indexed
            .capacity()
            .saturating_mul(size_of::<BodyRef>())
            .saturating_add(self.edges.capacity().saturating_mul(size_of::<Edge>()))
    }
}
impl Arena {
    pub(super) fn same_call_region(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        body: BodyRef,
        (expected, actual): (Region, Region),
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(3)?;
        if matches!(expected, Region::ObjectBound { .. })
            || matches!(actual, Region::ObjectBound { .. })
        {
            return Ok(false);
        }
        if !self.regional_equalities.indexed.contains(&body) {
            self.index_region_equations(domains, body, work)?;
            self.reserve(retained::new_map::<BodyRef, ()>(1), 0, work)?;
            self.regional_equalities.indexed.insert(body);
        }
        let edges = &self.regional_equalities.edges;
        Ok(edges.contains(&(body, expected, actual)) || edges.contains(&(body, actual, expected)))
    }

    fn index_region_equations(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        body: BodyRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        let input = &rows.body_inputs[domains.headers.original_index(body)?];
        work.charge(
            input
                .obligations
                .len()
                .saturating_add(input.normalizations.len()),
        )?;
        for obligation in &input.obligations {
            if let s::InputKind::Equality(pair) = obligation.kind {
                self.record_region_equations(domains, body, &[pair], work)?;
            }
        }
        for proof in &input.normalizations {
            self.record_region_equations(domains, body, &proof.equalities, work)?;
            if let Some(scoped) = &proof.scoped {
                work.charge(scoped.blocks.len())?;
                for block in &scoped.blocks {
                    if block.scope == 0 {
                        self.record_region_equations(domains, body, &block.equalities, work)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn record_region_equations(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        body: BodyRef,
        pairs: &[s::RegionBound],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        for pair in pairs {
            work.charge(1)?;
            if [pair.longer, pair.shorter]
                .iter()
                .any(|id| matches!(rows.regions[id.0 as usize], s::Region::Parameter { .. }))
            {
                continue;
            }
            let left = uses::region(domains, self, pair.longer, Rewrite::Original, work)?;
            let right = uses::region(domains, self, pair.shorter, Rewrite::Original, work)?;
            if left != right {
                self.reserve(retained::new_map::<Edge, ()>(1), 0, work)?;
                self.regional_equalities.edges.insert((body, left, right));
            }
        }
        Ok(())
    }
}
