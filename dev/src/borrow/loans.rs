//! Forward loan reachability constrained by inferred use regions.
mod inspection;
pub mod view;
use super::places::{Depth, Id, Index};
use super::{
    liveness,
    ownership::{BodyFacts, Use},
    regions::{
        self,
        graph::Region,
        points::{self, Point},
    },
    storage,
};
use crate::{
    body::{
        ir::{
            Action, BlockId, Body, BorrowKind, NodeKey, Place, ProjectionKind, Rvalue, Terminator,
            ValueId,
        },
        scopes::Lifetime,
        storage::End,
    },
    types::{Kind, Store, outlives::LiveRegions},
};
pub use inspection::Inspection;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Shared,
    Mutable,
    Reserved,
}
struct Loan {
    place: Id,
    region: Region,
    mode: Mode,
    value: ValueId,
}
#[derive(Debug)]
pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(self.message)
    }
}
impl std::error::Error for Error {}
pub struct Analysis {
    pub loans: usize,
    pub steps: usize,
    pub points: u32,
    pub region_words: usize,
}
#[derive(Clone, Copy)]
struct State {
    live: u64,
    active: u64,
}
struct Flow<'a> {
    body: &'a Body,
    owned: &'a BodyFacts,
    storage: &'a storage::Analysis,
    places: Index,
    loans: Vec<Loan>,
    generated: Vec<(usize, usize)>,
    origins: Vec<Option<ValueId>>,
    uses: liveness::Analysis,
    regions: points::Liveness,
    steps: usize,
    snapshots: Vec<inspection::Snapshot>,
}
impl Flow<'_> {
    fn collect(&mut self, types: &Store, contract: &regions::Body) -> Result<(), &'static str> {
        for (index, value) in self.body.values.iter().enumerate() {
            self.charge(1)?;
            let id = ValueId(u32::try_from(index).map_err(|_| "too many loan values")?);
            match value.kind {
                Rvalue::Borrow { place, kind } => {
                    let Some(Kind::Reference {
                        region, mutable, ..
                    }) = types.kind(value.ty)
                    else {
                        return Err("loan has no reference type");
                    };
                    let region = contract
                        .identities
                        .get(region)
                        .ok_or("missing borrow region")?;
                    let mode = if !mutable {
                        Mode::Shared
                    } else if kind == BorrowKind::TwoPhase {
                        Mode::Reserved
                    } else {
                        Mode::Mutable
                    };
                    let start = self.loans.len();
                    for (place, tracked) in self.places.expanded(place)? {
                        if tracked {
                            if self.loans.len() >= 500_000 {
                                return Err("too many reference loans");
                            }
                            self.loans.push(Loan {
                                place,
                                region,
                                mode,
                                value: id,
                            });
                        }
                    }
                    self.generated[index] = (start, self.loans.len());
                    self.origins[index] = Some(id);
                }
                Rvalue::Constrain(value) | Rvalue::Reify(value) => {
                    self.origins[index] = self.origins.get(value.0 as usize).copied().flatten();
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 32_000_000)
            .ok_or("loan dataflow work budget exceeded")?;
        Ok(())
    }
    fn point(&self, block: usize, statement: u32) -> Result<Point, &'static str> {
        self.uses
            .point_before(
                BlockId(u32::try_from(block).map_err(|_| "invalid loan block")?),
                statement,
            )
            .ok_or("missing loan point")
    }
    fn prune(&mut self, state: &mut [State], point: Point) -> Result<(), &'static str> {
        self.charge(state.len())?;
        for (word, state) in state.iter_mut().enumerate() {
            let mut bits = state.live;
            while bits != 0 {
                self.charge(1)?;
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if !self
                    .regions
                    .contains(self.loans[word * 64 + bit].region, point)?
                {
                    state.live &= !(1 << bit);
                    state.active &= !(1 << bit);
                }
            }
        }
        Ok(())
    }
    fn access(
        &mut self,
        state: &[State],
        place: Place,
        mode: Mode,
        depth: Depth,
        own: Option<ValueId>,
    ) -> Result<(), &'static str> {
        self.charge(state.len())?;
        for (word, state) in state.iter().enumerate() {
            let mut bits = state.live;
            while bits != 0 {
                self.charge(1)?;
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let loan = &self.loans[word * 64 + bit];
                if own == Some(loan.value) {
                    continue;
                }
                let exclusive = loan.mode == Mode::Mutable || state.active & (1 << bit) != 0;
                let incompatible = match mode {
                    Mode::Shared => exclusive,
                    Mode::Mutable => true,
                    Mode::Reserved => loan.mode != Mode::Shared,
                };
                if incompatible && self.places.conflicts(loan.place, place, depth)? {
                    return Err("access conflicts with a live reference loan");
                }
            }
        }
        Ok(())
    }
    fn ending(
        &mut self,
        state: &[State],
        end: End,
        target: Option<Point>,
    ) -> Result<(), &'static str> {
        self.charge(state.len())?;
        for (word, state) in state.iter().enumerate() {
            let mut bits = state.live;
            while bits != 0 {
                self.charge(1)?;
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let loan = &self.loans[word * 64 + bit];
                if let Some(point) = target
                    && !self.regions.contains(loan.region, point)?
                {
                    continue;
                }
                let Place::Temporary(value) = self.places.root(loan.place) else {
                    continue;
                };
                let Some(index) = self.storage.temporary(value) else {
                    continue;
                };
                let ended = match end {
                    End::Temporaries { start, end } => {
                        (start as usize..end as usize).contains(&index)
                    }
                    End::Scopes(range) => {
                        let temporary = &self.body.temporaries[index];
                        let Lifetime::Scope(scope) = temporary.lifetime else {
                            continue;
                        };
                        let storage = self
                            .body
                            .storage
                            .as_ref()
                            .ok_or("missing loan storage facts")?;
                        let mut next = Some(range.inner);
                        let mut found = false;
                        while next != range.outer {
                            self.charge(1)?;
                            let id = next.ok_or("invalid loan storage exit")?;
                            found |= id == scope;
                            next = storage.scopes[id.0 as usize].parent;
                        }
                        found
                    }
                };
                if ended
                    && self.places.conflicts(
                        self.loans[word * 64 + bit].place,
                        Place::Temporary(value),
                        Depth::Storage,
                    )?
                {
                    return Err("borrowed temporary does not live long enough");
                }
            }
        }
        Ok(())
    }
    fn inspect(&mut self, state: &[State], action: Action) -> Result<(), &'static str> {
        match action {
            Action::Evaluate(id) => match self.body.values[id.0 as usize].kind {
                Rvalue::Read(place) => self.access(
                    state,
                    place,
                    if self.owned.uses[id.0 as usize] == Use::Move {
                        Mode::Mutable
                    } else {
                        Mode::Shared
                    },
                    Depth::Deep,
                    None,
                )?,
                Rvalue::Borrow { place, .. } => {
                    let (a, b) = self.generated[id.0 as usize];
                    let mode = self
                        .loans
                        .get(a)
                        .filter(|_| a < b)
                        .map_or(Mode::Shared, |l| l.mode);
                    self.access(state, place, mode, Depth::Deep, None)?;
                }
                Rvalue::Validate(place) | Rvalue::TextEquals { place, .. } => {
                    self.access(state, place, Mode::Shared, Depth::Deep, None)?;
                }
                Rvalue::IsVariant { place, .. } | Rvalue::Length(place) => {
                    self.access(state, place, Mode::Shared, Depth::Discriminant, None)?;
                }
                _ => {}
            },
            Action::Store { place, .. } => {
                self.access(state, place, Mode::Mutable, Depth::Shallow, None)?;
            }
            Action::StorageLive(id) | Action::StorageDead(id) => {
                self.access(state, Place::Local(id), Mode::Mutable, Depth::Storage, None)?;
            }
            Action::StorageEnd { first, last } => {
                for id in self.body.storage_range(first, last) {
                    self.charge(1)?;
                    self.access(
                        state,
                        Place::Local(id?),
                        Mode::Mutable,
                        Depth::Storage,
                        None,
                    )?;
                }
            }
            Action::Project(id) | Action::Select { projection: id, .. } => {
                let projection = &self.body.projections[id.0 as usize];
                if matches!(projection.kind, ProjectionKind::Dereference { .. }) {
                    self.access(state, projection.base, Mode::Shared, Depth::Shallow, None)?;
                }
            }
            Action::BindView { .. } | Action::GuardStart(_) | Action::GuardUse(_) => {}
        }
        Ok(())
    }
    fn activation(
        &mut self,
        state: &mut [State],
        id: ValueId,
        inspect: bool,
    ) -> Result<(), &'static str> {
        let Rvalue::Call { arguments, .. } = self.body.values[id.0 as usize].kind else {
            return Ok(());
        };
        for &argument in self.body.arguments(arguments) {
            self.charge(1)?;
            let Some(origin) = self.origins[argument.0 as usize] else {
                continue;
            };
            let Rvalue::Borrow {
                place,
                kind: BorrowKind::TwoPhase,
            } = self.body.values[origin.0 as usize].kind
            else {
                continue;
            };
            if inspect {
                self.access(state, place, Mode::Mutable, Depth::Deep, Some(origin))?;
            }
            let (start, end) = self.generated[origin.0 as usize];
            for loan in start..end {
                self.charge(1)?;
                if state[loan / 64].live & (1 << (loan % 64)) != 0 {
                    state[loan / 64].active |= 1 << (loan % 64);
                }
            }
        }
        Ok(())
    }
    fn transfer<const RECORD: bool>(
        &mut self,
        block: usize,
        state: &mut [State],
        inspect: bool,
    ) -> Result<(), Error> {
        let data = &self.body.blocks[block];
        let mut ends = self.storage.ends(block).peekable();
        for at in data.start..=data.end {
            let node = NodeKey {
                node: if at < data.end {
                    self.body.statements[at as usize].source
                } else {
                    data.source
                },
                ..self.body.source
            };
            let error = |message| Error { node, message };
            self.charge(1).map_err(error)?;
            let point = self.point(block, at).map_err(error)?;
            self.prune(state, point).map_err(error)?;
            while let Some((_, end)) = ends.peek().filter(|(position, _)| *position == at).copied()
            {
                if inspect {
                    self.ending(state, end, None).map_err(error)?;
                }
                ends.next();
            }
            if at == data.end {
                if RECORD && inspect {
                    inspection::record(&mut self.snapshots, point, state).map_err(error)?;
                }
                continue;
            }
            let action = self.body.statements[at as usize].action;
            if let Action::Evaluate(id) = action {
                self.activation(state, id, inspect).map_err(error)?;
            }
            if RECORD && inspect {
                inspection::record(&mut self.snapshots, point, state).map_err(error)?;
            }
            if inspect {
                self.inspect(state, action).map_err(error)?;
            }
            if let Action::Evaluate(id) = action {
                let (start, end) = self.generated[id.0 as usize];
                for loan in start..end {
                    self.charge(1).map_err(error)?;
                    state[loan / 64].live |= 1 << (loan % 64);
                    state[loan / 64].active &= !(1 << (loan % 64));
                }
            }
        }
        Ok(())
    }
    fn solve<const RECORD: bool>(&mut self) -> Result<(), Error> {
        let error = |message| Error {
            node: self.body.source,
            message,
        };
        let words = self.loans.len().div_ceil(64);
        let count = words
            .checked_mul(self.body.blocks.len())
            .filter(|n| *n <= 4_000_000)
            .ok_or_else(|| error("loan block states exceed 64 MB budget"))?;
        let empty = State { live: 0, active: 0 };
        let mut incoming = vec![empty; count];
        let mut state = vec![empty; words];
        let mut reached = vec![false; self.body.blocks.len()];
        let mut queued = reached.clone();
        reached[0] = true;
        queued[0] = true;
        let mut work = VecDeque::from([0]);
        while let Some(block) = work.pop_front() {
            queued[block] = false;
            self.charge(words).map_err(error)?;
            state.copy_from_slice(&incoming[block * words..(block + 1) * words]);
            self.transfer::<false>(block, &mut state, false)?;
            for next in successors(self.body.blocks[block].terminator)
                .into_iter()
                .flatten()
            {
                let next = next.0 as usize;
                self.charge(words).map_err(error)?;
                let mut changed = !reached[next];
                reached[next] = true;
                for (target, source) in incoming[next * words..(next + 1) * words]
                    .iter_mut()
                    .zip(&state)
                {
                    changed |=
                        source.live & !target.live != 0 || source.active & !target.active != 0;
                    target.live |= source.live;
                    target.active |= source.active;
                }
                if changed && !queued[next] {
                    queued[next] = true;
                    work.push_back(next);
                }
            }
        }
        for (block, reachable) in reached.into_iter().enumerate() {
            if !reachable {
                continue;
            }
            self.charge(words).map_err(error)?;
            state.copy_from_slice(&incoming[block * words..(block + 1) * words]);
            self.transfer::<RECORD>(block, &mut state, true)?;
            for target in successors(self.body.blocks[block].terminator)
                .into_iter()
                .flatten()
            {
                for end in self.storage.edge_ends(block, target) {
                    let point = self
                        .point(target.0 as usize, self.body.blocks[target.0 as usize].start)
                        .map_err(error)?;
                    self.ending(&state, end, Some(point)).map_err(error)?;
                }
            }
        }
        Ok(())
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
/// Check source loans using completed type contracts, owned access facts and
/// temporary storage metadata. This analysis does not grant reference lowering
/// permission: alignment and the final combined proof boundary are separate.
///
/// # Errors
/// Reports conflicting accesses, escaping storage or bounded-dataflow exhaustion.
pub fn check(
    types: &Store,
    components: &mut LiveRegions,
    body: &Body,
    contract: &regions::Body,
    owned: &BodyFacts,
    storage: &storage::Analysis,
) -> Result<Analysis, Error> {
    Ok(analyze::<false>(types, components, body, contract, owned, storage)?.analysis)
}
pub(super) struct Report {
    pub analysis: Analysis,
    pub inspection: Option<Inspection>,
}
pub(super) fn analyze<const RECORD: bool>(
    types: &Store,
    components: &mut LiveRegions,
    body: &Body,
    contract: &regions::Body,
    owned: &BodyFacts,
    storage: &storage::Analysis,
) -> Result<Report, Error> {
    let error = |message| Error {
        node: body.source,
        message,
    };
    if contract.source != body.source || owned.uses.len() != body.values.len() {
        return Err(error("loan analysis facts belong to another body"));
    }
    if !body
        .values
        .iter()
        .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
    {
        return Ok(Report {
            analysis: Analysis {
                loans: 0,
                steps: 0,
                points: 0,
                region_words: 0,
            },
            inspection: RECORD.then_some(Inspection {
                source: body.source,
                detail: None,
            }),
        });
    }
    let uses = liveness::analyze(types, components, body).map_err(error)?;
    let mut seeds = Vec::with_capacity(uses.seeds.len());
    for &(region, point) in &uses.seeds {
        seeds.push((
            contract
                .identities
                .get(region)
                .ok_or_else(|| error("missing live region identity"))?,
            point,
        ));
    }
    let external = std::iter::once(Region::STATIC).chain(contract.identities.universals());
    let extents = external.flat_map(|region| (0..uses.points).map(move |p| (region, Point(p))));
    let regions = points::infer(
        &contract.solution,
        uses.points,
        seeds.into_iter().chain(extents),
        32_000_000,
    )
    .map_err(error)?;
    let mut flow = Flow {
        body,
        owned,
        storage,
        places: Index::new(body).map_err(error)?,
        loans: Vec::new(),
        generated: vec![(0, 0); body.values.len()],
        origins: vec![None; body.values.len()],
        uses,
        regions,
        steps: 0,
        snapshots: Vec::new(),
    };
    flow.collect(types, contract).map_err(error)?;
    flow.solve::<RECORD>()?;
    let analysis = Analysis {
        loans: flow.loans.len(),
        steps: flow.steps + flow.places.steps + flow.uses.steps + flow.regions.steps,
        points: flow.uses.points,
        region_words: flow.regions.mask_count(),
    };
    Ok(Report {
        analysis,
        inspection: RECORD.then_some(Inspection {
            source: body.source,
            detail: Some(inspection::Detail {
                places: flow.places,
                loans: flow.loans,
                uses: flow.uses,
                regions: flow.regions,
                snapshots: flow.snapshots,
            }),
        }),
    })
}
