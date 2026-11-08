//! Call openings participate in comparison identity, never in a paid witness.
use super::{
    DomainIndex, Work,
    nodes::{Arena, Node, NodeId, Region, Rewrite},
    s, uses,
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
                    self.children(domains, pair, (expected, actual), pending, work)?;
                }
            }
        }
        Ok(())
    }
    /// Version thirteen: a local nominal whose spine leads with region
    /// arguments compares each argument; other nominals stay exact nodes.
    fn children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        nodes: (Node, Node),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let (first, second) = match nodes {
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
                if a != b || !self.same_region(domains, (ar, br), pair.opening, work)? {
                    return Err("source signature reference mutability or exact lifetime differs");
                }
                ((ac, bc), None)
            }
            (Node::RawPointer { .. } | Node::Slice(_) | Node::Pattern { .. }, _) => {
                (single_child(domains, nodes)?, None)
            }
            (
                Node::Nominal {
                    declaration: a,
                    arguments: ac,
                },
                Node::Nominal {
                    declaration: b,
                    arguments: bc,
                },
            ) if a == b
                && matches!(
                    self.nodes.get(ac.0 as usize),
                    Some(Node::NominalRegion { .. })
                ) =>
            {
                ((ac, bc), None)
            }
            (
                Node::NominalRegion {
                    region: ar,
                    next: an,
                },
                Node::NominalRegion {
                    region: br,
                    next: bn,
                },
            ) => {
                if !self.same_region(domains, (ar, br), pair.opening, work)? {
                    return Err("source signature nominal region argument differs");
                }
                ((an, bn), None)
            }
            (
                Node::NominalArgument { ty: at, next: an },
                Node::NominalArgument { ty: bt, next: bn },
            ) => ((at, bt), Some((an, bn))),
            (a, b) if a == b => return Ok(()),
            _ => {
                return Err("source signature differs after complete application and late opening");
            }
        };
        for (expected, actual) in second.into_iter().chain([first]) {
            self.push_pair(
                pending,
                Task::Enter(Pair {
                    expected,
                    actual,
                    ..pair
                }),
                work,
            )?;
        }
        Ok(())
    }
    fn same_region(
        &self,
        domains: &DomainIndex<'_, '_>,
        (expected, actual): (Region, Region),
        opening: Opening,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        Ok(matches!(opening, Opening::Relation { .. })
            || opened_region(domains, self, expected, opening, work)?
                == opened_region(domains, self, actual, opening, work)?)
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
        .get(domains.headers.original_index(body)?)
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
/// Version thirteen's raw pointer, slice and pattern nodes compare their own
/// form, then their one child.
fn single_child(
    domains: &DomainIndex<'_, '_>,
    nodes: (Node, Node),
) -> Result<(NodeId, NodeId), &'static str> {
    match nodes {
        (
            Node::RawPointer {
                mutable: a,
                pointee: ac,
            },
            Node::RawPointer {
                mutable: b,
                pointee: bc,
            },
        ) => {
            if a != b {
                return Err("source signature raw-pointer mutability differs");
            }
            Ok((ac, bc))
        }
        (Node::Slice(ac), Node::Slice(bc)) => Ok((ac, bc)),
        (
            Node::Pattern {
                base: ac,
                constraint: a,
            },
            Node::Pattern {
                base: bc,
                constraint: b,
            },
        ) => {
            if !same_constraint(domains, a, b) {
                return Err("source signature pattern constraint differs");
            }
            Ok((ac, bc))
        }
        _ => Err("source signature differs after complete application and late opening"),
    }
}
/// Two pattern constraints agree when both are `!null` or both name the same
/// canonical interval, whichever range rows carry it.
fn same_constraint(
    domains: &DomainIndex<'_, '_>,
    first: s::PatternConstraint,
    second: s::PatternConstraint,
) -> bool {
    let ranges = domains.headers.rows().pattern_ranges();
    match (first, second) {
        (s::PatternConstraint::NonNull, s::PatternConstraint::NonNull) => true,
        (s::PatternConstraint::Range(a), s::PatternConstraint::Range(b)) => {
            let (a, b) = (ranges.get(a.0 as usize), ranges.get(b.0 as usize));
            a.is_some() && a == b
        }
        _ => false,
    }
}
