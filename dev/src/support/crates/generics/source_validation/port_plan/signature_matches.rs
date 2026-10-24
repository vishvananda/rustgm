//! Call openings participate in comparison identity, never in a paid witness.
use super::{
    DomainIndex, Work,
    nodes::{Arena, Node, NodeId, Region, Rewrite},
    uses,
};
use crate::support::{
    crates::owner_catalog::{BodyRef, DeclarationRef},
    retained,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Opening {
    Header(DeclarationRef),
    /// Shape admission only; all regional coercion remains a fresh proof request.
    Relation {
        body: BodyRef,
    },
    Call {
        body: BodyRef,
        request: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Pair {
    expected: NodeId,
    actual: NodeId,
    opening: Opening,
}
#[derive(Clone, Copy)]
enum Task {
    Enter(Pair),
    Finish(Pair),
}
impl Arena {
    pub(super) fn same(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        expected: NodeId,
        actual: NodeId,
        opening: Opening,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let pair = Pair {
            expected,
            actual,
            opening,
        };
        if expected == actual || self.matched.contains(&pair) {
            self.hits = self.hits.saturating_add(1);
            return Ok(());
        }
        let mut pending = Vec::new();
        let result = self.compare(domains, pair, &mut pending, work);
        self.set_scratch(0, work)?;
        result
    }
    fn compare(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        root: Pair,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.push_pair(pending, Task::Enter(root), work)?;
        while let Some(task) = pending.pop() {
            work.charge(1)?;
            match task {
                Task::Finish(pair) => {
                    self.reserve(retained::new_map::<Pair, ()>(1), 0, work)?;
                    self.matched.insert(pair);
                }
                Task::Enter(pair) => {
                    if pair.expected == pair.actual || self.matched.contains(&pair) {
                        self.hits = self.hits.saturating_add(1);
                        continue;
                    }
                    let expected = *self
                        .nodes
                        .get(pair.expected.0 as usize)
                        .ok_or("source expected type node is absent")?;
                    let actual = *self
                        .nodes
                        .get(pair.actual.0 as usize)
                        .ok_or("source actual type node is absent")?;
                    self.push_pair(pending, Task::Finish(pair), work)?;
                    match (expected, actual) {
                        (
                            Node::Reference {
                                mutable: a,
                                region: ar,
                                referent: ac,
                            },
                            Node::Reference {
                                mutable: b,
                                region: br,
                                referent: bc,
                            },
                        ) => {
                            if a != b
                                || (!matches!(pair.opening, Opening::Relation { .. })
                                    && opened_region(domains, self, ar, pair.opening, work)?
                                        != opened_region(domains, self, br, pair.opening, work)?)
                            {
                                return Err(
                                    "source signature reference mutability or exact lifetime differs",
                                );
                            }
                            self.push_pair(
                                pending,
                                Task::Enter(Pair {
                                    expected: ac,
                                    actual: bc,
                                    ..pair
                                }),
                                work,
                            )?;
                        }
                        (a, b) if a == b => {}
                        _ => {
                            return Err(
                                "source signature differs after complete application and late opening",
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn push_pair(
        &mut self,
        tasks: &mut Vec<Task>,
        task: Task,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if tasks.len() == tasks.capacity() {
            let capacity = tasks.capacity().saturating_mul(2).max(4);
            self.set_scratch(
                size_of::<Vec<Task>>().saturating_add(capacity.saturating_mul(size_of::<Task>())),
                work,
            )?;
            work.charge(tasks.len())?;
            tasks
                .try_reserve_exact(capacity.saturating_sub(tasks.len()))
                .map_err(|_| "source signature comparison work-stack allocation failed")?;
            self.set_scratch(
                size_of::<Vec<Task>>()
                    .saturating_add(tasks.capacity().saturating_mul(size_of::<Task>())),
                work,
            )?;
        }
        tasks.push(task);
        Ok(())
    }
}
fn opened_region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    region: Region,
    context: Opening,
    work: &mut Work,
) -> Result<Region, &'static str> {
    work.charge(1)?;
    let Region::Late { binder, slot } = region else {
        return Ok(region);
    };
    let Opening::Call { body, request } = context else {
        return Err("source callable late template has no actual Call opening");
    };
    let rows = domains.headers.rows();
    let original = rows
        .originals
        .get(body.0 as usize)
        .ok_or("source signature comparison has an unknown caller body")?;
    let request = original
        .calls
        .get(request as usize)
        .ok_or("source signature comparison has an unknown Call request")?;
    let opening = request
        .opening
        .ok_or("source signature comparison omitted the actual late opening")?;
    if opening.binder != binder {
        return Err("source signature comparison uses another target's late opening");
    }
    let region = *opening
        .regions
        .slice(&original.openings)?
        .get(slot as usize)
        .ok_or("source signature comparison omitted a complete late argument")?;
    uses::region(domains, arena, region, Rewrite::Original, work)
}
