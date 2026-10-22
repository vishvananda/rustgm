//! Use liveness and destructor liveness share the CFG but retain distinct components.
use super::{
    Analysis, Flow as Uses, components, mark,
    population::{Population, State},
    successors,
};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{Action, BlockId, Place, Terminator},
        storage::Location,
    },
    borrow::{
        active::Live,
        destruction::{Effect, Plan, schedule::Schedule},
        ownership::{PathId, Paths, initialization::Facts},
        regions::points::Point,
    },
    types::{Store, aggregate::Definitions, destruction::regions::Regions, outlives::LiveRegions},
};
use std::collections::VecDeque;

struct Flow<'a> {
    uses: Uses<'a>,
    plan: Schedule<'a>,
    paths: &'a Paths,
    components: components::Components,
    drop_entities: HashMap<PathId, usize>,
    intervals: Vec<(u32, usize)>,
}
impl<'a> Flow<'a> {
    fn new(
        types: &Store,
        definitions: &Definitions,
        regions: &mut LiveRegions,
        drops: &mut Regions,
        facts: &'a Facts<'_>,
        plan: Schedule<'a>,
    ) -> Result<Self, &'static str> {
        let mut uses = Uses::new(facts.body(), types, regions)?;
        let operands = super::operands::Requirements::new(
            uses.body,
            types,
            regions,
            &uses.values,
            uses.entities.len(),
        )?;
        uses.charge(operands.steps)?;
        let timeline = crate::borrow::destruction::timeline::Index::new(facts.body(), plan.events)?;
        uses.result.points = timeline.count;
        uses.result.ordered = Some(timeline);
        let mut components = components::Builder::default();
        for (entity, &ty) in uses.entities.iter().enumerate() {
            let key = operands
                .id(entity)
                .map_or(components::Key::Value(ty), components::Key::Operand);
            let set = if let Some(set) = components.get(key) {
                set
            } else {
                let parts = if let Some(parts) = operands.get(entity)? {
                    parts
                } else {
                    regions.symbolic(types, ty)?
                };
                let set = components.intern(parts)?;
                components.remember(key, set)?;
                set
            };
            components.entity(set)?;
        }
        let mut flow = Self {
            uses,
            plan,
            paths: &facts.ownership.paths,
            components: components::Components::default(),
            drop_entities: HashMap::default(),
            intervals: Vec::new(),
        };
        for cleanup in plan.cleanups() {
            flow.uses.charge(1)?;
            let (Some(ty), Some(path)) = (cleanup.live_type, cleanup.candidate.path) else {
                continue;
            };
            if flow.drop_entities.contains_key(&path) {
                continue;
            }
            let key = components::Key::Destructor(ty);
            let set = if let Some(set) = components.get(key) {
                set
            } else {
                let set = components.intern(drops.symbolic(types, definitions, ty)?)?;
                components.remember(key, set)?;
                set
            };
            if set == 0 {
                continue;
            }
            components.entity(set)?;
            let entry = flow
                .paths
                .entries
                .get(path.0 as usize)
                .ok_or("invalid destructor liveness path")?;
            let entity = flow.uses.entities.len();
            flow.uses.entities.push(ty);
            flow.drop_entities.insert(path, entity);
            flow.intervals.push((entry.start, entity));
        }
        flow.uses.charge(
            flow.intervals
                .len()
                .saturating_mul(flow.intervals.len().max(1).ilog2() as usize + 1),
        )?;
        flow.intervals.sort_unstable();
        flow.uses.charge(components.steps)?;
        flow.components = components.finish();
        Ok(flow)
    }
    fn kill(&mut self, start: u32, end: u32, live: &mut impl State) -> Result<(), &'static str> {
        if self.intervals.is_empty() {
            return Ok(());
        }
        self.uses
            .charge(2 * (self.intervals.len().ilog2() as usize + 1))?;
        let first = self.intervals.partition_point(|&(bit, _)| bit < start);
        let last = self.intervals.partition_point(|&(bit, _)| bit < end);
        self.uses.charge(last - first)?;
        for &(_, entity) in &self.intervals[first..last] {
            mark(Some(entity), live, false)?;
        }
        Ok(())
    }
    fn define(&mut self, path: PathId, live: &mut impl State) -> Result<(), &'static str> {
        let path = &self.paths.entries[path.0 as usize];
        self.kill(path.start, path.end, live)
    }
    fn statement(&mut self, action: Action, live: &mut impl State) -> Result<(), &'static str> {
        match action {
            Action::Evaluate(value) => {
                if let Some(path) = self.paths.temporary_path(value) {
                    self.define(path, live)?;
                }
            }
            Action::Store { place, .. } => match place {
                Place::Local(id) => self.define(PathId(id.0), live)?,
                // Whole local definitions kill old Drop uses. A projected store
                // preserves the containing aggregate's drop-check obligation.
                Place::Temporary(id) => {
                    if let Some(path) = self.paths.temporary_path(id) {
                        self.define(path, live)?;
                    }
                }
                _ => {}
            },
            Action::StorageLive(id) | Action::StorageDead(id) => self.define(PathId(id.0), live)?,
            Action::StorageEnd { first, last } => {
                for id in self.uses.body.storage_range(first, last) {
                    self.define(PathId(id?.0), live)?;
                }
            }
            _ => {}
        }
        self.uses.action(action, live)
    }
    fn transfer(&mut self, index: usize, live: &mut impl State) -> Result<(), &'static str> {
        self.uses.charge(1)?;
        let event = &self.plan.events[index];
        match event.effect {
            Effect::Statement => {
                let Location::At { before, .. } = event.location else {
                    return Err("cleanup statement is on an edge");
                };
                let action = self
                    .uses
                    .body
                    .statements
                    .get(before as usize)
                    .ok_or("invalid cleanup statement")?
                    .action;
                self.statement(action, live)?;
            }
            Effect::Cleanup(id) => {
                let cleanup = self
                    .plan
                    .cleanup(id as usize)
                    .ok_or("invalid cleanup liveness identity")?;
                if cleanup.live_type.is_some() {
                    if let Some(path) = cleanup.candidate.path {
                        mark(self.drop_entities.get(&path).copied(), live, true)?;
                    } else {
                        // Indirect destruction also needs the full reference used
                        // to access its destination, including its well-formedness.
                        self.uses.use_place(cleanup.candidate.place, live)?;
                    }
                }
            }
            Effect::Deinitialize { start, end } => self.kill(start, end, live)?,
            _ => {}
        }
        Ok(())
    }
    fn exit(
        &mut self,
        block: usize,
        incoming: &[u64],
        live: &mut Live,
        edge: &mut Live,
    ) -> Result<(), &'static str> {
        self.uses.charge(live.len().saturating_mul(2) + 1)?;
        live.clear();
        for next in successors(self.uses.body.blocks[block].terminator)
            .into_iter()
            .flatten()
        {
            self.uses.charge(live.len() * 6 + 1)?;
            edge.copy_from(
                &incoming[next.0 as usize * live.len()..(next.0 as usize + 1) * live.len()],
            );
            let id = BlockId(u32::try_from(block).map_err(|_| "invalid drop liveness block")?);
            if let Some(range) = self
                .uses
                .result
                .ordered
                .as_ref()
                .ok_or("missing cleanup timeline")?
                .edges
                .get(&(id, next))
                .cloned()
            {
                for index in range.rev() {
                    self.transfer(index, edge)?;
                }
            }
            live.union(edge.words());
        }
        if let Terminator::Return(value)
        | Terminator::Branch {
            condition: value, ..
        } = self.uses.body.blocks[block].terminator
        {
            self.uses.use_value(value, live)?;
        }
        Ok(())
    }
    fn solve(&mut self) -> Result<Vec<u64>, &'static str> {
        let words = self.uses.entities.len().div_ceil(64);
        let count = words
            .checked_mul(self.uses.body.blocks.len())
            .filter(|n| *n <= 8_000_000)
            .ok_or("drop liveness states exceed 64 MB budget")?;
        let mut incoming = vec![0; count];
        let mut live = Live::new(words);
        let mut edge = live.clone();
        let mut queued = self.uses.result.reachable.clone();
        let mut work: VecDeque<_> = (0..queued.len()).rev().filter(|i| queued[*i]).collect();
        while let Some(block) = work.pop_front() {
            queued[block] = false;
            self.exit(block, &incoming, &mut live, &mut edge)?;
            for index in self
                .uses
                .result
                .ordered
                .as_ref()
                .ok_or("missing cleanup timeline")?
                .blocks[block]
                .clone()
                .rev()
            {
                self.transfer(index, &mut live)?;
            }
            self.uses.charge(words)?;
            let target = &mut incoming[block * words..(block + 1) * words];
            if target != live.words() {
                target.copy_from_slice(live.words());
                self.uses
                    .charge(self.uses.starts[block + 1] - self.uses.starts[block])?;
                for &parent in
                    &self.uses.predecessors[self.uses.starts[block]..self.uses.starts[block + 1]]
                {
                    if self.uses.result.reachable[parent] && !queued[parent] {
                        queued[parent] = true;
                        work.push_back(parent);
                    }
                }
            }
        }
        Ok(incoming)
    }
    fn record(
        &mut self,
        index: usize,
        population: &mut Population<'_>,
    ) -> Result<(), &'static str> {
        self.transfer(index, population)?;
        let point = Point(u32::try_from(index).map_err(|_| "too many drop points")?);
        let event = &self.plan.events[index];
        let borrow = if let Effect::Statement = event.effect
            && let Location::At { before, .. } = event.location
        {
            self.uses
                .borrow_entity(self.uses.body.statements[before as usize].action)
        } else {
            None
        };
        population.seed(&mut self.uses.result, point, borrow)?;
        self.uses.charge(population.take_steps())
    }
}
/// Include destructor components and ordered storage endpoints without granting
/// a completed loan proof. The selected plan must belong to these initialization facts.
/// # Errors
/// Rejects malformed typed plans, region queries and bounded liveness exhaustion.
pub fn analyze(
    types: &Store,
    definitions: &Definitions,
    regions: &mut LiveRegions,
    drops: &mut Regions,
    facts: &Facts<'_>,
    plan: &Plan,
) -> Result<Analysis, &'static str> {
    analyze_schedule(
        types,
        definitions,
        regions,
        drops,
        facts,
        Schedule::concrete(plan),
    )
}
pub(crate) fn analyze_schedule(
    types: &Store,
    definitions: &Definitions,
    regions: &mut LiveRegions,
    drops: &mut Regions,
    facts: &Facts<'_>,
    plan: Schedule<'_>,
) -> Result<Analysis, &'static str> {
    let mut flow = Flow::new(types, definitions, regions, drops, facts, plan)?;
    let incoming = flow.solve()?;
    let components = std::mem::take(&mut flow.components);
    let mut population = Population::new(&components);
    let words = flow.uses.entities.len().div_ceil(64);
    let mut live = Live::new(words);
    let mut edge = live.clone();
    for block in 0..flow.uses.body.blocks.len() {
        if !flow.uses.result.reachable[block] {
            continue;
        }
        flow.exit(block, &incoming, &mut live, &mut edge)?;
        population.reset(&live)?;
        flow.uses.charge(population.take_steps())?;
        for index in flow
            .uses
            .result
            .ordered
            .as_ref()
            .ok_or("missing cleanup timeline")?
            .blocks[block]
            .clone()
            .rev()
        {
            flow.record(index, &mut population)?;
        }
        let id = BlockId(u32::try_from(block).map_err(|_| "invalid drop block")?);
        for target in successors(flow.uses.body.blocks[block].terminator)
            .into_iter()
            .flatten()
        {
            if let Some(range) = flow
                .uses
                .result
                .ordered
                .as_ref()
                .ok_or("missing cleanup timeline")?
                .edges
                .get(&(id, target))
                .cloned()
            {
                flow.uses.charge(words.saturating_mul(3))?;
                edge.copy_from(
                    &incoming[target.0 as usize * words..(target.0 as usize + 1) * words],
                );
                population.reset(&edge)?;
                flow.uses.charge(population.take_steps())?;
                for index in range.rev() {
                    flow.record(index, &mut population)?;
                }
            }
        }
    }
    Ok(flow.uses.result)
}
