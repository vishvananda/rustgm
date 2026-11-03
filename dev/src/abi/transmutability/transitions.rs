//! Lazy deterministic derivatives merge sorted intervals without enumerating byte values.
use super::{
    Budget, Language, LayoutId, Reference,
    automata::{ByteRange, Node},
    budget::push,
};
use crate::support::{hash::HashMap, retained::Growth};
#[derive(Clone, Copy)]
pub(super) struct Edge {
    pub range: ByteRange,
    pub next: LayoutId,
}
#[derive(Default)]
pub(super) struct Edges {
    pub bytes: Vec<Edge>,
    pub references: Vec<(Reference, LayoutId)>,
}
#[derive(Default)]
pub(super) struct Transitions {
    values: HashMap<LayoutId, Edges>,
}
impl Transitions {
    pub fn get(&self, id: LayoutId) -> Result<&Edges, &'static str> {
        self.values.get(&id).ok_or("missing layout derivatives")
    }
    pub fn prepare(
        &mut self,
        language: &mut Language,
        root: LayoutId,
        budget: &mut Budget<'_>,
    ) -> Result<(), &'static str> {
        let mut stack = Vec::new();
        push(&mut stack, (root, false), budget)?;
        while let Some((id, finish)) = stack.pop() {
            budget.charge(1, 0)?;
            if self.values.contains_key(&id) {
                continue;
            }
            let node = language.node(id)?;
            if !finish {
                push(&mut stack, (id, true), budget)?;
                match node {
                    Node::Sequence(a, _) | Node::Repeat(a, _) => {
                        push(&mut stack, (a, false), budget)?;
                    }
                    Node::Choice(a, b) => {
                        push(&mut stack, (a, false), budget)?;
                        push(&mut stack, (b, false), budget)?;
                    }
                    Node::Guard(_) => {
                        return Err("uninterpreted safety guard in layout derivatives");
                    }
                    _ => {}
                }
                continue;
            }
            let edges = self.construct(language, node, budget)?;
            let growth =
                Growth::map::<LayoutId, Edges>(self.values.len(), self.values.capacity(), 1);
            budget.charge(growth.work, growth.bytes)?;
            self.values
                .try_reserve(1)
                .map_err(|_| "derivative cache allocation failed")?;
            self.values.insert(id, edges);
        }
        Ok(())
    }
    fn construct(
        &self,
        language: &mut Language,
        node: Node,
        budget: &mut Budget<'_>,
    ) -> Result<Edges, &'static str> {
        match node {
            Node::Empty(_) | Node::Epsilon => Ok(Edges::default()),
            Node::Byte(range) => {
                budget.charge(1, 4 * size_of::<Edge>())?;
                Ok(Edges {
                    bytes: vec![Edge {
                        range,
                        next: language.epsilon(budget)?,
                    }],
                    references: Vec::new(),
                })
            }
            Node::Reference(value) => {
                budget.charge(1, 4 * size_of::<(Reference, LayoutId)>())?;
                Ok(Edges {
                    bytes: Vec::new(),
                    references: vec![(value, language.epsilon(budget)?)],
                })
            }
            Node::Sequence(a, b) => self.suffix(language, a, b, budget),
            Node::Repeat(a, n) => {
                let rest = language.repeat(a, n - 1, budget)?;
                self.suffix(language, a, rest, budget)
            }
            Node::Choice(a, b) => Self::merge(language, self.get(a)?, self.get(b)?, budget),
            Node::Guard(_) => Err("uninterpreted safety guard in layout derivatives"),
        }
    }
    fn suffix(
        &self,
        language: &mut Language,
        a: LayoutId,
        rest: LayoutId,
        budget: &mut Budget<'_>,
    ) -> Result<Edges, &'static str> {
        let edges = self.get(a)?;
        budget.charge(
            edges.bytes.len() + edges.references.len(),
            4 * (edges.bytes.len() * size_of::<Edge>()
                + edges.references.len() * size_of::<(Reference, LayoutId)>()),
        )?;
        let mut result = Edges {
            bytes: Vec::with_capacity(edges.bytes.len()),
            references: Vec::with_capacity(edges.references.len()),
        };
        for edge in &edges.bytes {
            result.bytes.push(Edge {
                range: edge.range,
                next: language.sequence(edge.next, rest, budget)?,
            });
        }
        for &(value, next) in &edges.references {
            result
                .references
                .push((value, language.sequence(next, rest, budget)?));
        }
        Ok(result)
    }
    fn merge(
        language: &mut Language,
        left: &Edges,
        right: &Edges,
        budget: &mut Budget<'_>,
    ) -> Result<Edges, &'static str> {
        let mut result = Edges::default();
        let size = left.bytes.len() + right.bytes.len();
        budget.charge(
            size.saturating_mul(4),
            size.saturating_mul(16 * size_of::<Edge>()),
        )?;
        let mut points = Vec::with_capacity(2 * size);
        for edge in left.bytes.iter().chain(&right.bytes) {
            points.push(edge.range.start);
            points.push(edge.range.end);
        }
        // Each input is deterministic; the alphabet has at most 258 endpoints.
        points.sort_unstable();
        points.dedup();
        let (mut left_cursor, mut right_cursor) = (0, 0);
        for point in points.windows(2) {
            let (start, end) = (point[0], point[1]);
            while left
                .bytes
                .get(left_cursor)
                .is_some_and(|e| e.range.end <= start)
            {
                left_cursor += 1;
            }
            while right
                .bytes
                .get(right_cursor)
                .is_some_and(|e| e.range.end <= start)
            {
                right_cursor += 1;
            }
            let left_next = left
                .bytes
                .get(left_cursor)
                .filter(|e| e.range.start <= start)
                .map(|e| e.next);
            let right_next = right
                .bytes
                .get(right_cursor)
                .filter(|e| e.range.start <= start)
                .map(|e| e.next);
            let next = match (left_next, right_next) {
                (Some(left_next), Some(right_next)) => {
                    language.either(left_next, right_next, budget)?
                }
                (Some(left_next), None) | (None, Some(left_next)) => left_next,
                _ => continue,
            };
            result.bytes.push(Edge {
                range: ByteRange { start, end },
                next,
            });
        }
        let count = left.references.len() + right.references.len();
        budget.charge(count, 4 * count * size_of::<(Reference, LayoutId)>())?;
        let (mut left_cursor, mut right_cursor) = (0, 0);
        while left_cursor < left.references.len() || right_cursor < right.references.len() {
            match (
                left.references.get(left_cursor),
                right.references.get(right_cursor),
            ) {
                (Some(&(left_next, left_tail)), Some(&(right_next, right_tail)))
                    if left_next == right_next =>
                {
                    result
                        .references
                        .push((left_next, language.either(left_tail, right_tail, budget)?));
                    left_cursor += 1;
                    right_cursor += 1;
                }
                (Some(&(left_next, left_tail)), Some(&(right_next, _)))
                    if left_next < right_next =>
                {
                    result.references.push((left_next, left_tail));
                    left_cursor += 1;
                }
                (Some(_), Some(&(right_next, right_tail))) => {
                    result.references.push((right_next, right_tail));
                    right_cursor += 1;
                }
                (Some(&value), None) => {
                    result.references.push(value);
                    left_cursor += 1;
                }
                (None, Some(&value)) => {
                    result.references.push(value);
                    right_cursor += 1;
                }
                (None, None) => break,
            }
        }
        Ok(result)
    }
}
