//! A transported original that tests or downcasts an enum carries no proof of
//! which variant a place holds. Each consumer re-proves it over its own rebuilt
//! CFG: a constructor test that ends its block in a Branch yields a positive
//! fact on the `yes` edge and an exclusion on the `no` edge. Facts flow down the
//! dominator tree formed by single-predecessor real edges; a merge starts with
//! none. Every downcast field use must be proved by a live fact, and every
//! reachable `Unreachable` exit must follow an impossible variant path.
//!
//! Facts on a place die when its root local is stored, re-bound or re-scoped;
//! facts on memory reached through a mutable reference, or on a local that is
//! ever mutably borrowed, also die at every call and every store through a
//! dereference. All work is linear in the body's rows.
use crate::{
    body::ir::{Action, Body, Place, ProjectionKind, Rvalue, Terminator},
    sema::{
        members::{Selection, VariantId},
        model::Declarations,
    },
    support::hash::HashMap,
    types::{ItemId, Kind, aggregate},
};

pub(super) fn check(facts: &mut Declarations, body: &Body) -> Result<(), &'static str> {
    let mut work = 0_usize;
    let result = Prover::new(facts, body, &mut work).and_then(|prover| {
        prover.map_or(Ok(()), |mut prover| {
            let result = prover.run(&mut work);
            work = work.saturating_add(prover.steps);
            result
        })
    });
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Step {
    Dereference,
    Field(u32),
}
/// The identity a fact is keyed and invalidated by.
#[derive(Clone, Copy)]
struct Key {
    /// Canonical place identity; region and access spelling are ignored.
    canon: u32,
    root: Option<u32>,
    /// Reached through a mutable or raw reference.
    mutable: bool,
}
#[derive(Clone, Copy)]
struct Shape {
    key: Key,
    /// Reached through any dereference.
    through: bool,
    /// The nearest downcast in this place's chain: its enum base and variant.
    downcast: Option<(Key, VariantId)>,
}
#[derive(Clone, Copy)]
struct Excluded {
    since: u64,
    count: u32,
}
enum Undo {
    Killed(u32, u64),
    Memory(u64),
    Positive(u32, Option<(VariantId, u64)>),
    Excluded(u32, Option<Excluded>),
    Mark((u32, VariantId), Option<u64>),
}
struct Prover<'a> {
    facts: &'a Declarations,
    body: &'a Body,
    places: Vec<Shape>,
    temporaries: u32,
    escaped: Vec<bool>,
    killed: Vec<u64>,
    memory: u64,
    clock: u64,
    positive: HashMap<u32, (VariantId, u64)>,
    excluded: HashMap<u32, Excluded>,
    marks: HashMap<(u32, VariantId), u64>,
    undo: Vec<Undo>,
    steps: usize,
}
impl<'a> Prover<'a> {
    /// `None` when the body has no constructor test and no downcast field: it
    /// keeps the ordinary transported exits unchanged.
    fn new(
        facts: &'a Declarations,
        body: &'a Body,
        work: &mut usize,
    ) -> Result<Option<Self>, &'static str> {
        *work = work.saturating_add(
            body.locals
                .len()
                .saturating_add(body.values.len())
                .saturating_add(body.projections.len()),
        );
        let temporaries =
            u32::try_from(body.locals.len()).map_err(|_| "source local ordinal overflow")?;
        let mut prover = Self {
            facts,
            body,
            places: Vec::with_capacity(body.projections.len()),
            temporaries,
            escaped: vec![false; body.locals.len()],
            killed: vec![0; body.locals.len()],
            memory: 0,
            clock: 0,
            positive: HashMap::default(),
            excluded: HashMap::default(),
            marks: HashMap::default(),
            undo: Vec::new(),
            steps: 0,
        };
        let mut interned = HashMap::default();
        let mut next = temporaries
            .checked_add(u32::try_from(body.values.len()).map_err(|_| "value overflow")?)
            .ok_or("source canonical place overflow")?;
        let mut tested = body
            .values
            .iter()
            .any(|value| matches!(value.kind, Rvalue::IsVariant { .. }));
        for (index, projection) in body.projections.iter().enumerate() {
            let base = match projection.base {
                Place::Projection(parent) if (parent.0 as usize) < index => {
                    prover.places[parent.0 as usize]
                }
                Place::Projection(_) => return Err("source projection base is not prior"),
                place => prover.shape(place)?,
            };
            let (step, mutable, through, downcast) = match projection.kind {
                ProjectionKind::Dereference { mutable, .. } => (
                    Step::Dereference,
                    base.key.mutable || mutable,
                    true,
                    base.downcast,
                ),
                ProjectionKind::Field(Selection::Nominal(field)) => {
                    let variant = prover.downcast(field);
                    (
                        Step::Field(field.0),
                        base.key.mutable,
                        base.through,
                        variant.map(|variant| (base.key, variant)).or(base.downcast),
                    )
                }
                _ => return Err("source variant facts require reference and field places"),
            };
            tested |= downcast.is_some();
            let canon = *interned.entry((base.key.canon, step)).or_insert_with(|| {
                next = next.saturating_add(1);
                next
            });
            prover.places.push(Shape {
                key: Key {
                    canon,
                    root: base.key.root,
                    mutable,
                },
                through,
                downcast,
            });
        }
        if !tested {
            return Ok(None);
        }
        // Locals whose own storage is ever mutably borrowed: any later call or
        // store through a reference may change them.
        for value in &body.values {
            if let Rvalue::Borrow { place, .. } = value.kind
                && matches!(
                    facts.types.kind(value.ty),
                    Some(Kind::Reference { mutable: true, .. })
                )
                && let shape = prover.shape(place)?
                && !shape.through
                && let Some(root) = shape.key.root
            {
                prover.escaped[root as usize] = true;
            }
        }
        Ok(Some(prover))
    }
    fn shape(&self, place: Place) -> Result<Shape, &'static str> {
        let local = |canon, root| Shape {
            key: Key {
                canon,
                root,
                mutable: false,
            },
            through: false,
            downcast: None,
        };
        match place {
            Place::Local(id) if (id.0 as usize) < self.killed.len() => {
                Some(local(id.0, Some(id.0)))
            }
            Place::Projection(projection) => self.places.get(projection.0 as usize).copied(),
            Place::Temporary(value) => Some(local(self.temporaries.saturating_add(value.0), None)),
            Place::Local(_) | Place::Static(_) => None,
        }
        .ok_or("source variant fact place is outside its body")
    }
    /// The variant a field belongs to when its owner is an enum.
    fn downcast(&self, field: crate::sema::members::FieldId) -> Option<VariantId> {
        let variant = self.facts.members.variant(field)?;
        let owner = self.facts.members.variants.get(variant.0 as usize)?.owner;
        self.facts
            .aggregates
            .get(owner)
            .is_some_and(|shape| shape.kind == aggregate::Kind::Enum)
            .then_some(variant)
    }
    fn variants(&self, variant: VariantId) -> u32 {
        self.facts
            .members
            .variants
            .get(variant.0 as usize)
            .map_or(0, |row| {
                u32::try_from(self.facts.members.variants(ItemId(row.owner.0)).len())
                    .unwrap_or(u32::MAX)
            })
    }
    fn tick(&mut self) -> u64 {
        self.clock = self.clock.saturating_add(1);
        self.clock
    }
    fn live(&self, key: Key, at: u64) -> bool {
        let escaped = key.root.is_some_and(|root| self.escaped[root as usize]);
        key.root.is_none_or(|root| at > self.killed[root as usize])
            && (!(key.mutable || escaped) || at > self.memory)
    }
    fn kill(&mut self, root: u32) {
        let now = self.tick();
        if let Some(slot) = self.killed.get_mut(root as usize) {
            self.undo.push(Undo::Killed(root, *slot));
            *slot = now;
        }
    }
    fn clobber(&mut self) {
        let now = self.tick();
        self.undo.push(Undo::Memory(self.memory));
        self.memory = now;
    }
    /// A live positive fact names `variant`, or every other variant is excluded.
    fn proved(&self, shape: Key, variant: VariantId) -> bool {
        if let Some(&(positive, at)) = self.positive.get(&shape.canon)
            && self.live(shape, at)
        {
            return positive == variant;
        }
        let Some(&set) = self.excluded.get(&shape.canon) else {
            return false;
        };
        self.live(shape, set.since)
            && set.count.saturating_add(1) == self.variants(variant)
            && !self.marked(shape, set, variant)
    }
    fn marked(&self, shape: Key, set: Excluded, variant: VariantId) -> bool {
        self.marks
            .get(&(shape.canon, variant))
            .is_some_and(|&at| at >= set.since && self.live(shape, at))
    }
    /// Record one edge fact; `false` means the edge is an impossible path.
    fn assume(&mut self, shape: Key, variant: VariantId, holds: bool) -> bool {
        let now = self.tick();
        let positive = self
            .positive
            .get(&shape.canon)
            .copied()
            .filter(|&(_, at)| self.live(shape, at));
        let set = self
            .excluded
            .get(&shape.canon)
            .copied()
            .filter(|set| self.live(shape, set.since));
        if holds {
            if positive.is_some_and(|(known, _)| known != variant)
                || set.is_some_and(|set| self.marked(shape, set, variant))
            {
                return false;
            }
            let old = self.positive.insert(shape.canon, (variant, now));
            self.undo.push(Undo::Positive(shape.canon, old));
            return true;
        }
        if positive.is_some_and(|(known, _)| known == variant) {
            return false;
        }
        let mut set = set.unwrap_or(Excluded {
            since: now,
            count: 0,
        });
        if !self.marked(shape, set, variant) {
            let old = self.marks.insert((shape.canon, variant), now);
            self.undo.push(Undo::Mark((shape.canon, variant), old));
            set.count = set.count.saturating_add(1);
        }
        let old = self.excluded.insert(shape.canon, set);
        self.undo.push(Undo::Excluded(shape.canon, old));
        set.count < self.variants(variant)
    }
    fn rewind(&mut self, mark: usize) {
        while self.undo.len() > mark {
            match self.undo.pop() {
                Some(Undo::Killed(root, at)) => self.killed[root as usize] = at,
                Some(Undo::Memory(at)) => self.memory = at,
                Some(Undo::Positive(canon, old)) => restore(&mut self.positive, canon, old),
                Some(Undo::Excluded(canon, old)) => restore(&mut self.excluded, canon, old),
                Some(Undo::Mark(key, old)) => restore(&mut self.marks, key, old),
                None => {}
            }
        }
    }
    fn require(&self, place: Place) -> Result<(), &'static str> {
        let Some((base, variant)) = self.shape(place)?.downcast else {
            return Ok(());
        };
        if !self.proved(base, variant) {
            return Err("source downcast field use is not dominated by its variant test");
        }
        Ok(())
    }
    fn run(&mut self, work: &mut usize) -> Result<(), &'static str> {
        let blocks = &self.body.blocks;
        *work = work.saturating_add(blocks.len().saturating_add(self.body.statements.len()));
        let mut predecessors = vec![0_u32; blocks.len()];
        for block in blocks {
            for target in real_successors(block.terminator) {
                let slot = predecessors
                    .get_mut(target.0 as usize)
                    .ok_or("source CFG edge leaves its body")?;
                *slot = slot.saturating_add(1);
            }
        }
        for root in 0..blocks.len() {
            if root == 0 || predecessors[root] > 1 {
                self.tree(root, &predecessors)?;
            }
        }
        Ok(())
    }
    /// One depth-first walk of a single-predecessor tree; facts recorded in a
    /// subtree are rewound before its sibling is visited.
    fn tree(&mut self, root: usize, predecessors: &[u32]) -> Result<(), &'static str> {
        let mark = self.undo.len();
        // (block, undo mark to restore after this block's subtree)
        let mut stack = vec![(root, mark, None)];
        while let Some((index, restore, edge)) = stack.pop() {
            self.rewind(restore);
            self.steps = self.steps.saturating_add(1);
            if let Some((shape, variant, holds)) = edge
                && !self.assume(shape, variant, holds)
            {
                continue;
            }
            let pending = self.block(index)?;
            // Children see this block's edge fact and effects.
            let after = self.undo.len();
            let block = self.body.blocks[index];
            for target in real_successors(block.terminator) {
                let target = target.0 as usize;
                if target == 0 || predecessors[target] != 1 {
                    continue;
                }
                let fact = match (block.terminator, pending) {
                    (Terminator::Branch { yes, no, .. }, Some((shape, variant))) if yes != no => {
                        Some((shape, variant, yes.0 as usize == target))
                    }
                    _ => None,
                };
                stack.push((target, after, fact));
            }
        }
        self.rewind(mark);
        Ok(())
    }
    /// Check and apply one block's statements; return its branch's live test.
    fn block(&mut self, index: usize) -> Result<Option<(Key, VariantId)>, &'static str> {
        let block = self.body.blocks[index];
        let statements = self
            .body
            .statements
            .get(block.start as usize..block.end as usize)
            .ok_or("source block statements leave their body")?;
        let mut tests = HashMap::default();
        let mut diverges = false;
        for statement in statements {
            self.steps = self.steps.saturating_add(1);
            match statement.action {
                Action::Evaluate(value) => {
                    let row = self
                        .body
                        .values
                        .get(value.0 as usize)
                        .ok_or("source evaluated value is outside its body")?;
                    diverges |= matches!(self.facts.types.kind(row.ty), Some(Kind::Never));
                    match row.kind {
                        Rvalue::Read(place) | Rvalue::Borrow { place, .. } => {
                            self.require(place)?;
                        }
                        Rvalue::IsVariant { place, variant } => {
                            self.require(place)?;
                            let shape = self.shape(place)?.key;
                            tests.insert(value.0, (shape, variant, self.tick()));
                        }
                        Rvalue::Call { .. } | Rvalue::Invoke { .. } => self.clobber(),
                        _ => {}
                    }
                }
                Action::Store { place, .. } => {
                    self.require(place)?;
                    let shape = self.shape(place)?;
                    if let Some(root) = shape.key.root {
                        self.kill(root);
                    }
                    if shape.through {
                        self.clobber();
                    }
                }
                Action::Project(projection) => self.require(Place::Projection(projection))?,
                Action::CheckInitialized(place) | Action::Mention(place) => {
                    self.require(place)?;
                }
                Action::StorageLive(local) | Action::StorageDead(local) => self.kill(local.0),
                Action::StorageEnd { first, last } => {
                    for local in first.0..=last.0 {
                        self.steps = self.steps.saturating_add(1);
                        self.kill(local);
                    }
                }
                Action::BindView { view, .. } => self.kill(view.0),
                _ => return Err("source variant facts cannot follow this statement form"),
            }
        }
        match block.terminator {
            Terminator::Unreachable if !diverges => {
                Err("transported match is not exhaustive over its enum")
            }
            Terminator::Branch { condition, .. } => Ok(tests
                .get(&condition.0)
                .copied()
                .filter(|&(shape, _, at)| self.live(shape, at))
                .map(|(shape, variant, _)| (shape, variant))),
            _ => Ok(None),
        }
    }
}
fn restore<K: std::hash::Hash + Eq, V>(map: &mut HashMap<K, V>, key: K, old: Option<V>) {
    match old {
        Some(value) => {
            map.insert(key, value);
        }
        None => {
            map.remove(&key);
        }
    }
}
/// Real control edges; a `FalseEdge`'s imaginary successor never executes.
fn real_successors(terminator: Terminator) -> impl Iterator<Item = crate::body::ir::BlockId> {
    let (first, second) = match terminator {
        Terminator::Jump(target) | Terminator::FalseEdge { real: target, .. } => {
            (Some(target), None)
        }
        Terminator::Branch { yes, no, .. } => (Some(yes), Some(no)),
        Terminator::Return(_) | Terminator::Unreachable | Terminator::Panic => (None, None),
    };
    first.into_iter().chain(second)
}
