//! Active closure input scopes index only the regions their owner produces.
use crate::support::hash::HashMap;
use crate::{resolve::unit::NodeKey, types::RegionId};

#[derive(Default)]
struct Input {
    regions: Vec<RegionId>,
    scopes: usize,
}

#[derive(Default)]
pub(in crate::sema::declarations) struct Inputs {
    #[expect(
        clippy::box_collection,
        reason = "closure annotation ownership is allocated only on first demand"
    )]
    entries: Option<Box<HashMap<NodeKey, Input>>>,
}

impl Inputs {
    pub(in crate::sema::declarations) fn begin(
        &mut self,
        owner: NodeKey,
    ) -> Result<usize, &'static str> {
        let entries = self.entries.get_or_insert_with(Box::default);
        let input = entries.entry(owner).or_default();
        input.scopes = input
            .scopes
            .checked_add(1)
            .ok_or("closure annotation scope count overflow")?;
        let first = input.regions.len();
        #[cfg(test)]
        super::tests::opened(entries.len(), entries.capacity());
        Ok(first)
    }

    pub(in crate::sema::declarations) fn record(&mut self, owner: NodeKey, region: RegionId) {
        if let Some(input) = self
            .entries
            .as_deref_mut()
            .and_then(|entries| entries.get_mut(&owner))
        {
            #[cfg(test)]
            let capacity = input.regions.capacity();
            input.regions.push(region);
            #[cfg(test)]
            super::tests::produced(input.regions.capacity() - capacity);
        }
    }

    pub(in crate::sema::declarations) fn finish(
        &mut self,
        owner: NodeKey,
        first: usize,
    ) -> Result<Vec<RegionId>, &'static str> {
        let entries = self
            .entries
            .as_deref_mut()
            .ok_or("closure annotation completion has no input scope")?;
        let input = entries
            .get_mut(&owner)
            .ok_or("closure annotation completion has no matching owner")?;
        if first > input.regions.len() || input.scopes == 0 {
            return Err("closure annotation input range changed");
        }
        input.scopes -= 1;
        if input.scopes != 0 {
            return Ok(input.regions[first..].to_vec());
        }
        let mut regions = entries
            .remove(&owner)
            .ok_or("closure annotation owner disappeared")?
            .regions;
        #[cfg(test)]
        super::tests::released(regions.len(), regions.capacity());
        regions.drain(..first);
        Ok(regions)
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/annotation_inputs.rs"]
mod tests;
