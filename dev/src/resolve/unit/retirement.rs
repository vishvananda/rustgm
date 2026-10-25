//! A cold owner-removal transaction checks the real retirement before flag mutation.
use super::{NodeId, Unit};
use crate::resolve::package::Error;

pub(crate) trait Observer {
    type Output;
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error>;
    fn enter(
        &mut self,
        unit: &Unit,
        node: NodeId,
        edge: Option<(NodeId, usize)>,
    ) -> Result<(), Error>;
    fn leave(&mut self);
    fn finish(self) -> Result<Self::Output, Error>;
}
enum Step {
    Enter(NodeId, Option<(NodeId, usize)>),
    Leave,
}
impl Unit {
    pub(crate) fn remove_function_owner_with<O: Observer>(
        &mut self,
        root: NodeId,
        mut observer: O,
    ) -> Result<O::Output, Error> {
        observer.charge(1, 4 * size_of::<Step>())?;
        let mut work = vec![Step::Enter(root, None)];
        let mut retired = Vec::new();
        let mut journal = crate::support::hash::HashSet::default();
        while let Some(step) = work.pop() {
            let Step::Enter(node, edge) = step else {
                observer.leave();
                continue;
            };
            observer.charge(1, 4 * (size_of::<Step>() + size_of::<NodeId>()))?;
            self.retirement_node(node)?;
            observer.charge(1, 0)?;
            let owner_enabled = self.owner_indices[node.get() as usize]
                .is_some_and(|index| self.configured.owners[index.get() as usize - 1].enabled);
            if self.original.needs_retirement_save(
                node,
                self.configured.active[node.get() as usize],
                owner_enabled,
            ) && !journal.contains(&node)
            {
                observer.charge(
                    1,
                    super::original::Original::retirement_row_bytes()
                        + crate::support::retained::new_map::<NodeId, ()>(1),
                )?;
                journal.insert(node);
            }
            observer.enter(self, node, edge)?;
            retired.push(node);
            work.push(Step::Leave);
            let mark = work.len();
            let mut ordinal = 0;
            self.configured.ast.try_for_each_child(node, &mut |child| {
                observer.charge(1, 4 * size_of::<Step>())?;
                work.push(Step::Enter(child, Some((node, ordinal))));
                ordinal += 1;
                Ok(())
            })?;
            work[mark..].reverse();
        }
        self.original
            .reserve_retirements(journal.len())
            .map_err(Error::unlocated)?;
        // The observer also reserves final receipt/index publication here.
        let prepared = observer.finish()?;
        for node in retired {
            let owner = self.owner_indices[node.get() as usize];
            let enabled =
                owner.is_some_and(|index| self.configured.owners[index.get() as usize - 1].enabled);
            let active = self.configured.active[node.get() as usize];
            if active || enabled {
                self.original
                    .save(node, self.configured.ast.node(node), active, owner, enabled);
            }
            self.configured.active[node.get() as usize] = false;
            if let Some(index) = self.owner_indices[node.get() as usize] {
                self.configured.owners[index.get() as usize - 1].enabled = false;
            }
        }
        Ok(prepared)
    }
    fn retirement_node(&self, node: NodeId) -> Result<(), Error> {
        let index = node.get() as usize;
        if self.configured.ast.nodes.get(index).is_none()
            || self.configured.active.get(index).is_none()
            || self.owner_indices.get(index).is_none_or(|owner| {
                owner.is_some_and(|owner| {
                    self.configured
                        .owners
                        .get(owner.get() as usize - 1)
                        .is_none_or(|owner| owner.node != node)
                })
            })
        {
            return Err(Error::unlocated(
                "Test retirement lost an actual node or owner",
            ));
        }
        Ok(())
    }
}
