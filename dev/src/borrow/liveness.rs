//! Backward use liveness for typed locals and values, before loan generation.
use super::regions::points::Point;
use crate::{
    body::ir::{Action, BlockId, Body, LocalId, Place, Rvalue, Terminator, ValueId},
    types::{RegionId, Store, TypeId, outlives::LiveRegions},
};
use std::collections::VecDeque;

const LIMIT: usize = 32_000_000;

pub struct Analysis {
    pub seeds: Vec<(RegionId, Point)>,
    pub points: u32,
    pub reachable: Vec<bool>,
    pub steps: usize,
    locations: Vec<(u32, u32, u32)>,
}
impl Analysis {
    /// Translate a global statement index (or block end) to this CFG's point.
    #[must_use]
    pub fn point_before(&self, block: BlockId, statement: u32) -> Option<Point> {
        let &(start, end, base) = self.locations.get(block.0 as usize)?;
        (start..=end)
            .contains(&statement)
            .then(|| Point(base + statement - start))
    }
}
struct Flow<'a> {
    body: &'a Body,
    locals: Vec<Option<usize>>,
    values: Vec<Option<usize>>,
    projections: Vec<Option<usize>>,
    entities: Vec<TypeId>,
    predecessors: Vec<usize>,
    starts: Vec<usize>,
    result: Analysis,
}
impl<'a> Flow<'a> {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.result.steps = self
            .result
            .steps
            .checked_add(count)
            .filter(|n| *n <= LIMIT)
            .ok_or("value liveness work budget exceeded")?;
        Ok(())
    }
    fn new(body: &'a Body, types: &Store, regions: &mut LiveRegions) -> Result<Self, &'static str> {
        let count = body
            .locals
            .len()
            .saturating_add(body.values.len())
            .saturating_add(body.projections.len());
        if count > 2_000_000 || body.blocks.is_empty() || body.blocks.len() > 1_000_000 {
            return Err("value liveness index budget exceeded");
        }
        let mut flow = Self {
            body,
            locals: Vec::new(),
            values: Vec::new(),
            projections: Vec::new(),
            entities: Vec::new(),
            predecessors: Vec::new(),
            starts: vec![0; body.blocks.len() + 1],
            result: Analysis {
                seeds: Vec::new(),
                points: 0,
                reachable: vec![false; body.blocks.len()],
                steps: 0,
                locations: Vec::new(),
            },
        };
        for local in &body.locals {
            let entity = flow.entity(types, regions, local.ty)?;
            flow.locals.push(entity);
        }
        for value in &body.values {
            let entity = flow.entity(types, regions, value.ty)?;
            flow.values.push(entity);
        }
        for projection in &body.projections {
            flow.charge(1)?;
            flow.projections.push(flow.place(projection.base)?);
        }
        flow.index_cfg()?;
        Ok(flow)
    }
    fn entity(
        &mut self,
        types: &Store,
        regions: &mut LiveRegions,
        ty: TypeId,
    ) -> Result<Option<usize>, &'static str> {
        self.charge(1)?;
        if regions.get(types, ty)?.is_empty() {
            return Ok(None);
        }
        let id = self.entities.len();
        self.entities.push(ty);
        Ok(Some(id))
    }
    fn index_cfg(&mut self) -> Result<(), &'static str> {
        for block in &self.body.blocks {
            self.charge(1)?;
            if block.start > block.end || block.end as usize > self.body.statements.len() {
                return Err("invalid value liveness block range");
            }
            let base = self.result.points;
            self.result.points = base
                .checked_add(block.end - block.start)
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 4_000_000)
                .ok_or("too many value liveness points")?;
            self.result.locations.push((block.start, block.end, base));
            for next in successors(block.terminator).into_iter().flatten() {
                let slot = self
                    .starts
                    .get_mut(next.0 as usize + 1)
                    .ok_or("invalid value liveness successor")?;
                *slot += 1;
            }
        }
        for i in 1..self.starts.len() {
            self.starts[i] += self.starts[i - 1];
        }
        self.predecessors
            .resize(*self.starts.last().ok_or("missing liveness edge count")?, 0);
        let mut next = self.starts.clone();
        for (index, block) in self.body.blocks.iter().enumerate() {
            for child in successors(block.terminator).into_iter().flatten() {
                let at = &mut next[child.0 as usize];
                self.predecessors[*at] = index;
                *at += 1;
            }
        }
        let mut work = vec![0];
        self.result.reachable[0] = true;
        while let Some(index) = work.pop() {
            self.charge(1)?;
            for next in successors(self.body.blocks[index].terminator)
                .into_iter()
                .flatten()
            {
                let index = next.0 as usize;
                if !self.result.reachable[index] {
                    self.result.reachable[index] = true;
                    work.push(index);
                }
            }
        }
        Ok(())
    }
    fn place(&self, place: Place) -> Result<Option<usize>, &'static str> {
        match place {
            Place::Local(id) => self.locals.get(id.0 as usize).copied(),
            Place::Temporary(id) => self.values.get(id.0 as usize).copied(),
            Place::Projection(id) => self.projections.get(id.0 as usize).copied(),
            Place::Static(_) => Some(None),
        }
        .ok_or("invalid value liveness place")
    }
    fn use_value(&self, id: ValueId, live: &mut [u64]) -> Result<(), &'static str> {
        mark(
            *self.values.get(id.0 as usize).ok_or("invalid live value")?,
            live,
            true,
        );
        Ok(())
    }
    fn use_place(&self, place: Place, live: &mut [u64]) -> Result<(), &'static str> {
        mark(self.place(place)?, live, true);
        Ok(())
    }
    fn local(&self, id: LocalId, live: &mut [u64], used: bool) -> Result<(), &'static str> {
        mark(
            *self.locals.get(id.0 as usize).ok_or("invalid live local")?,
            live,
            used,
        );
        Ok(())
    }
    fn value(&mut self, id: ValueId, live: &mut [u64]) -> Result<(), &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("invalid liveness value operation")?;
        mark(self.values[id.0 as usize], live, false);
        match value.kind {
            Rvalue::Read(place)
            | Rvalue::Borrow { place, .. }
            | Rvalue::IsVariant { place, .. }
            | Rvalue::Validate(place) => self.use_place(place, live)?,
            Rvalue::Aggregate { fields, .. }
            | Rvalue::Call {
                arguments: fields, ..
            } => {
                let values = self
                    .body
                    .arguments
                    .get(fields.start as usize..fields.end as usize)
                    .ok_or("invalid liveness value operands")?;
                self.charge(values.len())?;
                for &value in values {
                    self.use_value(value, live)?;
                }
                if let Rvalue::Call { callee, .. } = value.kind {
                    self.use_value(callee, live)?;
                }
            }
            Rvalue::Reify(value)
            | Rvalue::Constrain(value)
            | Rvalue::Repeat { value }
            | Rvalue::Unary { operand: value, .. }
            | Rvalue::Cast { operand: value, .. } => self.use_value(value, live)?,
            Rvalue::Binary { left, right, .. } => {
                self.use_value(left, live)?;
                self.use_value(right, live)?;
            }
            Rvalue::Unit
            | Rvalue::Scalar(_)
            | Rvalue::Text(_)
            | Rvalue::Constant(_)
            | Rvalue::Function(_) => {}
        }
        Ok(())
    }
    fn action(&mut self, action: Action, live: &mut [u64]) -> Result<(), &'static str> {
        self.charge(1)?;
        match action {
            Action::Evaluate(id) => self.value(id, live)?,
            Action::Store { place, value } => {
                if let Place::Local(id) = place {
                    self.local(id, live, false)?;
                } else {
                    self.use_place(place, live)?;
                }
                self.use_value(value, live)?;
            }
            Action::BindView { view, place } => {
                self.local(view, live, false)?;
                self.use_place(place, live)?;
            }
            Action::StorageLive(id) | Action::StorageDead(id) => self.local(id, live, false)?,
            Action::StorageEnd { first, last } => {
                for id in self.body.storage_range(first, last) {
                    self.charge(1)?;
                    self.local(id?, live, false)?;
                }
            }
            Action::Project(id) | Action::Select { projection: id, .. } => {
                self.use_place(Place::Projection(id), live)?;
            }
            Action::GuardUse(id) => {
                let guard = self
                    .body
                    .guards
                    .get(id.0 as usize)
                    .ok_or("invalid live guard")?;
                for local in guard.views_start..guard.views_end {
                    self.charge(1)?;
                    self.local(LocalId(local), live, true)?;
                }
            }
            Action::GuardStart(_) => {}
        }
        Ok(())
    }
    fn exit(
        &mut self,
        block: usize,
        incoming: &[u64],
        live: &mut [u64],
    ) -> Result<(), &'static str> {
        self.charge(live.len().saturating_mul(3).saturating_add(1))?;
        live.fill(0);
        let terminator = self.body.blocks[block].terminator;
        for next in successors(terminator).into_iter().flatten() {
            let start = next.0 as usize * live.len();
            for (word, bits) in live.iter_mut().zip(&incoming[start..]) {
                *word |= bits;
            }
        }
        if let Terminator::Return(value)
        | Terminator::Branch {
            condition: value, ..
        } = terminator
        {
            self.use_value(value, live)?;
        }
        Ok(())
    }
    fn solve(&mut self) -> Result<(Vec<u64>, Vec<u64>), &'static str> {
        let words = self.entities.len().div_ceil(64);
        let count = words
            .checked_mul(self.body.blocks.len())
            .filter(|n| *n <= 8_000_000)
            .ok_or("value liveness states exceed 64 MB budget")?;
        let mut incoming = vec![0; count];
        let mut live = vec![0; words];
        let mut queued = self.result.reachable.clone();
        let mut work: VecDeque<_> = (0..self.body.blocks.len())
            .rev()
            .filter(|i| queued[*i])
            .collect();
        while let Some(block) = work.pop_front() {
            queued[block] = false;
            self.exit(block, &incoming, &mut live)?;
            for statement in self.body.statements(&self.body.blocks[block]).iter().rev() {
                self.action(statement.action, &mut live)?;
            }
            self.charge(words)?;
            let target = &mut incoming[block * words..(block + 1) * words];
            if target != live {
                target.copy_from_slice(&live);
                self.charge(self.starts[block + 1] - self.starts[block])?;
                for &parent in &self.predecessors[self.starts[block]..self.starts[block + 1]] {
                    if self.result.reachable[parent] && !queued[parent] {
                        queued[parent] = true;
                        work.push_back(parent);
                    }
                }
            }
        }
        Ok((incoming, live))
    }
    fn seed_type(
        &mut self,
        types: &Store,
        regions: &mut LiveRegions,
        ty: TypeId,
        point: Point,
    ) -> Result<(), &'static str> {
        let ids = regions.get(types, ty)?;
        self.charge(ids.len())?;
        if self.result.seeds.len().saturating_add(ids.len()) > 2_000_000 {
            return Err("value liveness seed budget exceeded");
        }
        self.result.seeds.extend(ids.iter().map(|id| (*id, point)));
        Ok(())
    }
    fn seed(
        &mut self,
        types: &Store,
        regions: &mut LiveRegions,
        live: &[u64],
        point: Point,
    ) -> Result<(), &'static str> {
        self.charge(live.len())?;
        for (word, &bits) in live.iter().enumerate() {
            let mut bits = bits;
            while bits != 0 {
                let entity = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                self.seed_type(types, regions, self.entities[entity], point)?;
            }
        }
        Ok(())
    }
}
fn mark(entity: Option<usize>, live: &mut [u64], used: bool) {
    if let Some(id) = entity {
        let mask = 1 << (id % 64);
        if used {
            live[id / 64] |= mask;
        } else {
            live[id / 64] &= !mask;
        }
    }
}
fn successors(terminator: Terminator) -> [Option<BlockId>; 2] {
    match terminator {
        Terminator::Jump(next) => [Some(next), None],
        Terminator::Branch { yes, no, .. }
        | Terminator::FalseEdge {
            real: yes,
            imaginary: no,
        } => [Some(yes), Some(no)],
        Terminator::Return(_) | Terminator::Unreachable => [None, None],
    }
}
/// Compute use-point seeds, including imaginary edges, without proving loans or
/// destructor uses. A later loan pass combines these with the proved type graph.
///
/// # Errors
/// Rejects malformed CFG/type identities and bounded-dataflow exhaustion.
pub fn analyze(
    types: &Store,
    regions: &mut LiveRegions,
    body: &Body,
) -> Result<Analysis, &'static str> {
    let mut flow = Flow::new(body, types, regions)?;
    let (incoming, mut live) = flow.solve()?;
    for (block, data) in body.blocks.iter().enumerate() {
        if !flow.result.reachable[block] {
            continue;
        }
        let base = flow.result.locations[block].2;
        flow.exit(block, &incoming, &mut live)?;
        flow.seed(types, regions, &live, Point(base + data.end - data.start))?;
        for at in (data.start..data.end).rev() {
            let action = body.statements[at as usize].action;
            flow.action(action, &mut live)?;
            let point = Point(base + at - data.start);
            flow.seed(types, regions, &live, point)?;
            if let Action::Evaluate(id) = action
                && matches!(body.values[id.0 as usize].kind, Rvalue::Borrow { .. })
            {
                flow.seed_type(types, regions, body.values[id.0 as usize].ty, point)?;
            }
        }
    }
    Ok(flow.result)
}
