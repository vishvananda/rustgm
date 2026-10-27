//! Iteration IDs are winning-trace preorder positions, so subtree counts are ranks.
use super::{Bindings, HashMap, IterationId, LexError, NodeId, charge, error};
pub(super) struct Index {
    pub(super) ends: Vec<u32>,
    pub(super) positions: HashMap<NodeId, Vec<IterationId>>,
}
impl Index {
    pub(super) fn new(bindings: &Bindings, work: &mut usize) -> Result<Self, LexError> {
        let count = bindings.iterations.len();
        let groups = bindings.groups.len();
        charge(
            work,
            count
                .saturating_mul(3)
                .saturating_add(groups.saturating_mul(3))
                .saturating_add(1),
        )?;
        let mut coordinates = vec![(IterationId(0), NodeId(0)); groups];
        if bindings.repetitions.len() != groups {
            return Err(error(
                0,
                "repetition index changed its actual group ownership",
            ));
        }
        for (&coordinate, &group) in &bindings.repetitions {
            *coordinates
                .get_mut(group)
                .ok_or_else(|| error(0, "repetition group is outside its actual bindings"))? =
                coordinate;
        }
        let mut ends = Vec::with_capacity(count + 1);
        for index in 0..=count {
            ends.push(u32::try_from(index + 1).map_err(|_| error(0, "repetition index overflow"))?);
        }
        // Groups are created before descendant groups; reverse order is topological.
        for (range, &(parent, _)) in bindings.groups.iter().zip(&coordinates).rev() {
            if let Some(last) = bindings
                .iterations
                .get(range.clone())
                .ok_or_else(|| error(0, "repetition range is outside its actual bindings"))?
                .last()
            {
                let end = *ends
                    .get(last.0 as usize)
                    .ok_or_else(|| error(0, "repetition child is outside preorder"))?;
                let owner = ends
                    .get_mut(parent.0 as usize)
                    .ok_or_else(|| error(0, "repetition parent is outside preorder"))?;
                *owner = (*owner).max(end);
            }
        }
        let mut positions: HashMap<NodeId, Vec<IterationId>> = HashMap::default();
        for (range, &(_, node)) in bindings.groups.iter().zip(&coordinates) {
            // Instances of one syntax node cannot nest in themselves. Their groups
            // and their children therefore append in increasing preorder position.
            positions.entry(node).or_default().extend_from_slice(
                bindings
                    .iterations
                    .get(range.clone())
                    .ok_or_else(|| error(0, "repetition position range disappeared"))?,
            );
        }
        Ok(Self { ends, positions })
    }

    pub(super) fn count(
        &self,
        values: &[IterationId],
        target: NodeId,
        work: &mut usize,
    ) -> Result<u32, LexError> {
        let (Some(first), Some(last), Some(positions)) =
            (values.first(), values.last(), self.positions.get(&target))
        else {
            charge(work, 1)?;
            return Ok(0);
        };
        charge(
            work,
            3 + 2 * positions.len().checked_ilog2().unwrap_or(0) as usize,
        )?;
        let end = *self
            .ends
            .get(last.0 as usize)
            .ok_or_else(|| error(0, "count subtree is outside actual preorder"))?;
        let start = positions.partition_point(|id| id.0 < first.0);
        let finish = positions.partition_point(|id| id.0 < end);
        u32::try_from(finish - start).map_err(|_| error(0, "count exceeds u32"))
    }
}
