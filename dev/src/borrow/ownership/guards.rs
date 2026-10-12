//! Backward liveness of internal guard protections; divergent paths need no final use.
use super::{BodyFacts, Error, PathId, Use, paths::PlaceInfo};
use crate::body::ir::{
    Action, BlockId, Body, GuardId, LocalId, LocalKind, Place, Rvalue, Terminator,
};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
struct Frame {
    start: usize,
    end: usize,
}
enum Loan<'a> {
    Tested(&'a [Vec<PlaceInfo>]),
    View(&'a [PlaceInfo]),
}
struct Proof<'a> {
    body: &'a Body,
    facts: &'a BodyFacts,
    frames: Vec<Frame>,
    loans: Vec<Loan<'a>>,
    remaining: usize,
}
impl<'a> Proof<'a> {
    fn new(body: &'a Body, facts: &'a BodyFacts) -> Result<Self, &'static str> {
        let mut proof = Self {
            body,
            facts,
            frames: Vec::new(),
            loans: Vec::new(),
            remaining: 32_000_000,
        };
        for (index, guard) in body.guards.iter().enumerate() {
            let tested = facts
                .paths
                .guard_places
                .get(guard.tested_start as usize..guard.tested_end as usize)
                .ok_or("invalid tested guard places")?;
            let views = body
                .locals
                .get(guard.views_start as usize..guard.views_end as usize)
                .ok_or("invalid guard view range")?;
            let start = proof.loans.len();
            proof.loans.push(Loan::Tested(tested));
            for (offset, local) in views.iter().enumerate() {
                if !matches!(local.kind, LocalKind::Guard(id) if id.0 as usize == index) {
                    return Err("guard view belongs to a different guard");
                }
                let local = LocalId(
                    guard.views_start
                        + u32::try_from(offset).map_err(|_| "too many guard views")?,
                );
                proof.loans.push(Loan::View(
                    facts
                        .paths
                        .views
                        .get(&local)
                        .ok_or("guard view has no selected source")?,
                ));
            }
            proof.frames.push(Frame {
                start,
                end: proof.loans.len(),
            });
        }
        Ok(proof)
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(cost)
            .ok_or("guard liveness work budget exceeded")?;
        Ok(())
    }
    fn view(&self, local: LocalId) -> Result<Option<usize>, &'static str> {
        let Some(LocalKind::Guard(id)) = self.body.locals.get(local.0 as usize).map(|l| l.kind)
        else {
            return Ok(None);
        };
        let guard = self
            .body
            .guards
            .get(id.0 as usize)
            .ok_or("missing guard view owner")?;
        if !(guard.views_start..guard.views_end).contains(&local.0) {
            return Err("view is outside its guard range");
        }
        Ok(Some(
            self.frames[id.0 as usize].start + 1 + (local.0 - guard.views_start) as usize,
        ))
    }
    fn mark_view(&self, local: LocalId, live: &mut [u64], value: bool) -> Result<(), &'static str> {
        if let Some(bit) = self.view(local)? {
            let mask = 1 << (bit % 64);
            if value {
                live[bit / 64] |= mask;
            } else {
                live[bit / 64] &= !mask;
            }
        }
        Ok(())
    }
    fn read(&self, place: Place, live: &mut [u64]) -> Result<(), &'static str> {
        let info = self.facts.paths.place(place)?;
        if let Some(view) = self.facts.paths.view_root(self.body, info.path) {
            self.mark_view(view, live, true)?;
        }
        Ok(())
    }
    fn group(&mut self, id: GuardId, live: &mut [u64], value: bool) -> Result<(), &'static str> {
        let frame = *self
            .frames
            .get(id.0 as usize)
            .ok_or("missing guard protection group")?;
        self.charge(frame.end - frame.start)?;
        for bit in frame.start..frame.end {
            let mask = 1 << (bit % 64);
            if value {
                live[bit / 64] |= mask;
            } else {
                live[bit / 64] &= !mask;
            }
        }
        Ok(())
    }
    fn transfer(&mut self, action: Action, live: &mut [u64]) -> Result<(), &'static str> {
        self.charge(1)?;
        match action {
            Action::GuardStart(id) => self.group(id, live, false)?,
            Action::GuardUse(id) => self.group(id, live, true)?,
            Action::BindView { view, place } => {
                self.mark_view(view, live, false)?;
                self.read(place, live)?;
            }
            Action::StorageLive(local) | Action::StorageDead(local) => {
                self.mark_view(local, live, false)?;
            }
            Action::StorageEnd { first, last } => {
                for local in self.body.storage_range(first, last) {
                    self.charge(1)?;
                    self.mark_view(local?, live, false)?;
                }
            }
            Action::Evaluate(id) => {
                if let Rvalue::Read(place)
                | Rvalue::Validate(place)
                | Rvalue::IsVariant { place, .. } = self.body.values[id.0 as usize].kind
                {
                    self.read(place, live)?;
                }
            }
            Action::Project(id) => self.read(self.body.projections[id.0 as usize].base, live)?,
            Action::Store { .. } => {}
        }
        Ok(())
    }
    fn conflict(&self, written: PathId, targets: &[PlaceInfo], deep: bool) -> bool {
        let written = &self.facts.paths.entries[written.0 as usize];
        targets.iter().any(|target| {
            if target.empty {
                return false;
            }
            target.path.is_some_and(|path| {
                let path = &self.facts.paths.entries[path.0 as usize];
                let end = if deep { path.end } else { path.start + 1 };
                written.start < end && path.start < written.end
            })
        })
    }
    fn write(&mut self, place: Place, live: &[u64]) -> Result<(), &'static str> {
        let info = self.facts.paths.place(place)?;
        let Some(path) = info.path.filter(|_| !info.empty) else {
            return Ok(());
        };
        self.charge(live.len())?;
        for (word, &bits) in live.iter().enumerate() {
            let mut bits = bits;
            while bits != 0 {
                let bit = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let conflict = match &self.loans[bit] {
                    Loan::Tested(groups) => {
                        let cost = groups.iter().map(Vec::len).sum::<usize>();
                        let conflict = groups
                            .iter()
                            .any(|targets| self.conflict(path, targets, false));
                        self.charge(cost + 1)?;
                        conflict
                    }
                    Loan::View(targets) => {
                        let cost = targets.len();
                        let conflict = self.conflict(path, targets, true);
                        self.charge(cost + 1)?;
                        conflict
                    }
                };
                if conflict {
                    return Err("cannot modify a place protected by a live match guard");
                }
            }
        }
        Ok(())
    }
    fn inspect(&mut self, action: Action, live: &[u64]) -> Result<(), &'static str> {
        match action {
            Action::Store { place, .. } => self.write(place, live)?,
            Action::Evaluate(id) if self.facts.uses[id.0 as usize] == Use::Move => {
                if let Rvalue::Read(place) = self.body.values[id.0 as usize].kind {
                    self.write(place, live)?;
                }
            }
            Action::StorageLive(local) | Action::StorageDead(local) => {
                self.write(Place::Local(local), live)?;
            }
            Action::StorageEnd { first, last } => {
                for local in self.body.storage_range(first, last) {
                    self.charge(1)?;
                    self.write(Place::Local(local?), live)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn successors(terminator: Terminator) -> [Option<BlockId>; 2] {
    match terminator {
        Terminator::Jump(target) => [Some(target), None],
        Terminator::Branch { yes, no, .. }
        | Terminator::FalseEdge {
            real: yes,
            imaginary: no,
        } => [Some(yes), Some(no)],
        _ => [None, None],
    }
}
fn predecessors(body: &Body) -> (Vec<usize>, Vec<usize>) {
    let mut ranges = vec![0; body.blocks.len() + 1];
    for block in &body.blocks {
        for target in successors(block.terminator).into_iter().flatten() {
            ranges[target.0 as usize + 1] += 1;
        }
    }
    for i in 1..ranges.len() {
        ranges[i] += ranges[i - 1];
    }
    let mut offsets = ranges.clone();
    let mut entries = vec![0; ranges[body.blocks.len()]];
    for (id, block) in body.blocks.iter().enumerate() {
        for target in successors(block.terminator).into_iter().flatten() {
            let offset = &mut offsets[target.0 as usize];
            entries[*offset] = id;
            *offset += 1;
        }
    }
    (ranges, entries)
}
pub(super) fn verify(body: &Body, facts: &BodyFacts) -> Result<(), Error> {
    if body.guards.is_empty() {
        return Ok(());
    }
    let error = |message| Error {
        node: body.source,
        message,
    };
    let mut proof = Proof::new(body, facts).map_err(error)?;
    let words = proof.loans.len().div_ceil(64);
    let size = words
        .checked_mul(body.blocks.len())
        .filter(|n| *n <= 8_000_000)
        .ok_or_else(|| error("guard liveness state exceeds 64 MB budget"))?;
    let mut incoming = vec![0_u64; size];
    let mut live = vec![0; words];
    let (ranges, predecessors) = predecessors(body);
    let mut queued = facts.reachable.clone();
    let mut work: VecDeque<_> = (0..body.blocks.len())
        .rev()
        .filter(|i| queued[*i])
        .collect();
    while let Some(index) = work.pop_front() {
        queued[index] = false;
        proof.charge(words * 4).map_err(error)?;
        live.fill(0);
        let block = &body.blocks[index];
        for successor in successors(block.terminator).into_iter().flatten() {
            let start = successor.0 as usize * words;
            for (word, value) in live.iter_mut().zip(&incoming[start..start + words]) {
                *word |= value;
            }
        }
        for statement in body.statements(block).iter().rev() {
            proof.transfer(statement.action, &mut live).map_err(error)?;
        }
        let target = &mut incoming[index * words..(index + 1) * words];
        if target != live {
            target.copy_from_slice(&live);
            for &parent in &predecessors[ranges[index]..ranges[index + 1]] {
                if facts.reachable[parent] && !queued[parent] {
                    queued[parent] = true;
                    work.push_back(parent);
                }
            }
        }
    }
    for (index, block) in body.blocks.iter().enumerate() {
        if !facts.reachable[index] {
            continue;
        }
        proof.charge(words * 3).map_err(error)?;
        live.fill(0);
        for successor in successors(block.terminator).into_iter().flatten() {
            let start = successor.0 as usize * words;
            for (word, value) in live.iter_mut().zip(&incoming[start..start + words]) {
                *word |= value;
            }
        }
        for statement in body.statements(block).iter().rev() {
            let error = |message| Error {
                node: crate::body::ir::NodeKey {
                    node: statement.source,
                    ..body.source
                },
                message,
            };
            proof.inspect(statement.action, &live).map_err(error)?;
            proof.transfer(statement.action, &mut live).map_err(error)?;
        }
    }
    Ok(())
}
