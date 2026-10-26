//! Bounded constant execution consumes selected typed CFGs, never source syntax.
pub(crate) mod code;
mod counted;
mod executed_calls;
pub(crate) mod lease_codes;
mod memory;
mod offsets;
mod use_calls;
pub(crate) mod use_inputs;
pub(crate) use use_calls::Reached as ReachedConstCall;
mod argument_witness;
mod entry_policy;
mod function_calls;
mod operator_calls;
mod place_cache;
mod raw_reads;
mod reached_drop;
mod reached_layout;
mod readiness;
mod results;
mod returns;
mod runtime_checks;
mod selector_sites;
mod static_reads;
mod transfers;
mod transmute_calls;
mod values;

use crate::sema::values::execution::{Progress, Requirement};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::ir::{Action, BlockId, Body, LocalKind, Owner, Terminator, ValueId},
    resolve::{model::DefId, unit::NodeKey},
    sema::values::{Constant, Evaluator, Failure},
};
pub(crate) use reached_drop::Call as SourceNeedsDropCall;
pub(crate) use reached_layout::Call as SourceLayoutCall;
use static_reads::EvaluationContext;

/// Deterministic execution limits are independent of the host stack and time.
#[derive(Clone, Copy)]
pub struct Limits {
    pub steps: usize,
    pub frames: usize,
    pub slots: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            steps: 16_000_000,
            frames: 1024,
            slots: 4_000_000,
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Work {
    pub steps: usize,
    pub calls: usize,
    pub peak_frames: usize,
    pub peak_slots: usize,
}
pub struct Error {
    pub source: Option<crate::body::ir::ProofSite>,
    pub message: &'static str,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ReadOperation {
    frame: usize,
    position: u32,
}
struct Frame {
    instance: usize,
    context: EvaluationContext,
    body: usize,
    block: BlockId,
    next: u32,
    operand_cursor: usize,
    return_to: Option<ValueId>,
    locals: Vec<Option<Constant>>,
    values: Vec<Option<Constant>>,
    temporaries: HashMap<ValueId, Constant>,
    projections: HashMap<place_cache::Key, memory::Resolved>,
    raw_views: HashMap<crate::body::ir::LocalId, memory::RawAccess>,
    union_views: HashMap<crate::body::ir::LocalId, memory::UnionField>,
    allocations: Vec<crate::sema::values::LocationId>,
}
/// All CFGs are immutable and already checked by their defining-body pipeline.
pub struct Machine<'a, 'b> {
    evaluator: Evaluator<'a>,
    bodies: code::CatalogRef<'b>,
    frames: Vec<Frame>,
    memory: memory::Memory,
    limits: Limits,
    work: Work,
    slots: usize,
    active_initializers: HashSet<DefId>,
    active_associated: HashSet<(crate::sema::values::Instance, crate::types::TypeId)>,
    active_inline: HashSet<NodeKey>,
    active_anonymous: HashSet<(
        crate::sema::inline_constants::InlineInstance,
        crate::types::TypeId,
    )>,
    paid_call: Option<Box<use_calls::Continuation>>,
    waiting: Option<Requirement>,
    read_operation: Option<ReadOperation>,
    context: EvaluationContext,
}
pub(crate) struct State {
    paid_call: Option<Box<use_calls::Continuation>>,
    frames: Vec<Frame>,
    memory: memory::Memory,
    work: Work,
    slots: usize,
    active_initializers: HashSet<DefId>,
    active_associated: HashSet<(crate::sema::values::Instance, crate::types::TypeId)>,
    active_inline: HashSet<NodeKey>,
    active_anonymous: HashSet<(
        crate::sema::inline_constants::InlineInstance,
        crate::types::TypeId,
    )>,
}
impl State {
    pub(crate) fn retained_slots(&self) -> usize {
        self.slots
            .saturating_add(self.memory.retained_cells())
            .saturating_add(self.paid_call.as_ref().map_or(0, |_| {
                size_of::<use_calls::Continuation>().div_ceil(size_of::<Constant>())
            }))
    }
}
impl<'a, 'b> Machine<'a, 'b> {
    /// # Errors
    /// Rejects ambiguous body identities; permission is still checked at each call.
    pub fn new(
        evaluator: Evaluator<'a>,
        bodies: impl IntoIterator<Item = &'b Body>,
        limits: Limits,
    ) -> Result<Self, &'static str> {
        let mut catalog = code::Catalog::default();
        for body in bodies {
            catalog.insert(body)?;
        }
        Ok(Self::from_catalog(
            evaluator,
            code::CatalogRef::Owned(catalog),
            limits,
        ))
    }
    pub(crate) fn catalog(
        evaluator: Evaluator<'a>,
        catalog: &'b code::Catalog,
        limits: Limits,
    ) -> Self {
        Self::from_catalog(evaluator, code::CatalogRef::Borrowed(catalog), limits)
    }
    fn from_catalog(
        evaluator: Evaluator<'a>,
        bodies: code::CatalogRef<'b>,
        limits: Limits,
    ) -> Self {
        Self {
            evaluator,
            bodies,
            frames: Vec::new(),
            memory: memory::Memory::default(),
            limits,
            work: Work::default(),
            slots: 0,
            active_initializers: HashSet::default(),
            active_associated: HashSet::default(),
            active_inline: HashSet::default(),
            active_anonymous: HashSet::default(),
            paid_call: None,
            waiting: None,
            read_operation: None,
            context: EvaluationContext::Constant,
        }
    }
    #[must_use]
    pub const fn work(&self) -> Work {
        self.work
    }
    /// Execute one named definition with explicit argument values.
    /// # Errors
    /// Reports forbidden calls, evaluation faults and deterministic resource limits.
    pub fn definition(
        &mut self,
        definition: DefId,
        arguments: &[Constant],
    ) -> Result<Constant, Error> {
        let index = self
            .bodies
            .definitions
            .get(&definition)
            .copied()
            .ok_or(Error {
                source: None,
                message: "missing constant execution body",
            })?;
        if let Owner::Initializer { .. } = self.bodies[index].owner
            && let Some(value) =
                crate::sema::values::Reader::new(self.evaluator.facts()).definition(definition)
        {
            return Ok(value);
        }
        self.check_source_function_root(index)
            .map_err(|failure| self.error(index, failure))?;
        self.push(index, arguments, None)
            .map_err(|failure| self.error(index, failure))?;
        match self.advance()? {
            Progress::Complete(value) => Ok(value),
            Progress::Prepare(_) => {
                Err(self.error(index, Failure::Invalid("missing demanded constant body")))
            }
        }
    }
    pub(crate) fn restore(&mut self, state: State) {
        self.paid_call = state.paid_call;
        self.frames = state.frames;
        self.memory = state.memory;
        self.work = state.work;
        self.slots = state.slots;
        self.active_initializers = state.active_initializers;
        self.active_associated = state.active_associated;
        self.active_inline = state.active_inline;
        self.active_anonymous = state.active_anonymous;
    }
    pub(crate) fn suspend(self) -> (State, crate::sema::values::Scratch) {
        let state = State {
            paid_call: self.paid_call,
            frames: self.frames,
            memory: self.memory,
            work: self.work,
            slots: self.slots,
            active_initializers: self.active_initializers,
            active_associated: self.active_associated,
            active_inline: self.active_inline,
            active_anonymous: self.active_anonymous,
        };
        (state, self.evaluator.into_scratch())
    }
    pub(crate) fn source_catalog(
        evaluator: Evaluator<'a>,
        catalog: &'b code::Catalog,
        inputs: Option<&'b use_inputs::Inputs>,
        limits: Limits,
    ) -> Self {
        let mut machine = Self::catalog(evaluator, catalog, limits);
        if let Some(inputs) = inputs {
            machine.bodies = code::CatalogRef::Source(catalog, inputs);
        }
        machine
    }
    pub(crate) fn begin(&mut self, index: usize, arguments: &[Constant]) -> Result<(), Error> {
        self.check_source_function_root(index)
            .map_err(|failure| self.error(index, failure))?;
        self.push(index, arguments, None)
            .map_err(|failure| self.error(index, failure))
    }
    pub(crate) fn advance(&mut self) -> Result<Progress, Error> {
        self.waiting = None;
        let result = self.run();
        if result.is_err() {
            self.memory.abandon_byte_reads();
        }
        result.map_err(|(index, failure)| self.error(index, failure))
    }
    fn error(&self, index: usize, failure: Failure) -> Error {
        Error {
            source: self.bodies.get(index).map(|body| body.source.root_site()),
            message: failure.message(),
        }
    }
    fn charge(&mut self, amount: usize) -> Result<(), Failure> {
        self.work.steps = self
            .work
            .steps
            .checked_add(amount)
            .ok_or("constant execution work overflow")?;
        if self.work.steps > self.limits.steps {
            return Err(Failure::Fault("constant execution step limit exceeded"));
        }
        Ok(())
    }
    fn push(
        &mut self,
        index: usize,
        arguments: &[Constant],
        return_to: Option<ValueId>,
    ) -> Result<(), Failure> {
        self.bodies.validate_entry(index)?;
        let body = self.bodies[index].clone();
        if !body.const_context {
            return Err(Failure::Fault(
                "non-const body requested during constant execution",
            ));
        }
        if let Owner::Initializer { definition, .. } = body.owner
            && !self.active_initializers.insert(definition)
        {
            return Err(Failure::Fault("cycle in constant initializer values"));
        }
        self.start_source_inline(&body)?;
        if let Owner::AnonymousInstance(instance) = body.owner
            && !self.active_anonymous.insert((instance, body.result))
        {
            return Err(Failure::Fault(
                "cycle in anonymous constant instance values",
            ));
        }
        if let Owner::InitializerInstance { instance, .. } = body.owner
            && !self.active_associated.insert((instance, body.result))
        {
            return Err(Failure::Fault("cycle in selected initializer values"));
        }
        let slots = body
            .locals
            .len()
            .checked_add(body.values.len())
            .ok_or("constant frame size overflow")?;
        if self.frames.len() >= self.limits.frames {
            return Err(Failure::Fault("constant execution frame limit exceeded"));
        }
        self.slots = self
            .slots
            .checked_add(slots)
            .filter(|n| n.saturating_add(self.memory.retained_cells()) <= self.limits.slots)
            .ok_or(Failure::Fault("constant execution storage limit exceeded"))?;
        self.work.peak_slots = self.work.peak_slots.max(
            self.slots
                .saturating_add(self.memory.retained_cells())
                .saturating_add(self.paid_call.as_ref().map_or(0, |_| {
                    size_of::<use_calls::Continuation>().div_ceil(size_of::<Constant>())
                })),
        );
        self.charge(slots)?;
        let first = body
            .blocks
            .first()
            .ok_or("constant body has no entry block")?;
        let mut frame = Frame {
            instance: self.work.calls,
            context: self.frame_context(body.owner),
            body: index,
            block: BlockId(0),
            next: first.start,
            operand_cursor: if body.operands.is_empty() {
                usize::MAX
            } else {
                0
            },
            return_to,
            locals: vec![None; body.locals.len()],
            values: vec![None; body.values.len()],
            temporaries: HashMap::default(),
            projections: HashMap::default(),
            raw_views: HashMap::default(),
            union_views: HashMap::default(),
            allocations: Vec::new(),
        };
        let mut parameters = 0;
        for (local, declaration) in body.locals.iter().enumerate() {
            if let LocalKind::Parameter(ordinal) = declaration.kind {
                let value = arguments
                    .get(ordinal as usize)
                    .copied()
                    .ok_or("constant call has missing argument")?;
                frame.locals[local] =
                    Some(self.allocate(&mut frame.allocations, Some(value), declaration.ty)?);
                parameters += 1;
            }
        }
        if parameters != arguments.len() {
            return Err("constant call argument arity mismatch".into());
        }
        self.work.calls += 1;
        self.work.peak_frames = self.work.peak_frames.max(self.frames.len() + 1);
        self.work.peak_slots = self.work.peak_slots.max(self.slots);
        self.frames.push(frame);
        Ok(())
    }
    fn run(&mut self) -> Result<Progress, (usize, Failure)> {
        loop {
            let Some(frame) = self.frames.pop() else {
                return Err((0, "missing constant execution frame".into()));
            };
            let index = frame.body;
            self.validate_const_resume(&frame)
                .map_err(|e| (index, e.into()))?;
            self.context = frame.context;
            self.read_operation = Some(ReadOperation {
                frame: frame.instance,
                position: frame.next,
            });
            let mut frame = if frame.operand_cursor == usize::MAX {
                frame
            } else {
                let Some(frame) = self.prepare_operands(frame).map_err(|e| (index, e))? else {
                    if let Some(requirement) = self.waiting.take() {
                        return Ok(Progress::Prepare(requirement));
                    }
                    continue;
                };
                frame
            };
            self.charge(1).map_err(|e| (index, e))?;
            let body = self.bodies[index].clone();
            let block = body
                .blocks
                .get(frame.block.0 as usize)
                .ok_or((index, Failure::Invalid("invalid constant execution block")))?;
            if frame.next < block.end {
                let statement = body.statements.get(frame.next as usize).ok_or((
                    index,
                    Failure::Invalid("invalid constant execution statement"),
                ))?;
                frame.next += 1;
                let call = self
                    .statement(&mut frame, statement.action)
                    .map_err(|e| (index, e))?;
                if let Some(requirement) = self.waiting.take() {
                    frame.next -= 1;
                    self.frames.push(frame);
                    if self
                        .initialize_storage(requirement)
                        .map_err(|e| (index, e))?
                    {
                        continue;
                    }
                    return Ok(Progress::Prepare(requirement));
                }
                self.frames.push(frame);
                if let Some((target, arguments, destination)) = call {
                    self.push(target, &arguments, Some(destination))
                        .map_err(|e| (index, e))?;
                }
                continue;
            }
            let successor = match block.terminator {
                Terminator::Jump(target) | Terminator::FalseEdge { real: target, .. } => target,
                Terminator::Branch { condition, yes, no } => {
                    if values::boolean(Self::value(&frame, condition).map_err(|e| (index, e))?)
                        .map_err(|e| (index, e))?
                    {
                        yes
                    } else {
                        no
                    }
                }
                Terminator::Return(value) => {
                    if let Some(result) = self
                        .returned(&frame, &body, value)
                        .map_err(|e| (index, e))?
                    {
                        return Ok(Progress::Complete(result));
                    }
                    continue;
                }
                Terminator::Panic => {
                    return Err((index, Failure::Fault("constant execution reached panic")));
                }
                Terminator::Unreachable => {
                    return Err((
                        index,
                        Failure::Fault("constant execution reached unreachable code"),
                    ));
                }
            };
            frame.block = successor;
            frame.next = body
                .blocks
                .get(successor.0 as usize)
                .ok_or((index, Failure::Invalid("invalid constant successor")))?
                .start;
            self.frames.push(frame);
        }
    }
    fn value(frame: &Frame, id: ValueId) -> Result<Constant, Failure> {
        frame
            .values
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or(Failure::Invalid("constant value used before evaluation"))
    }
    fn statement(
        &mut self,
        frame: &mut Frame,
        action: Action,
    ) -> Result<Option<(usize, Vec<Constant>, ValueId)>, Failure> {
        match action {
            Action::Evaluate(id) => return self.evaluate(frame, id),
            Action::Store { place, value } => {
                let value = Self::value(frame, value)?;
                let Some(location) = self.place(frame, place)? else {
                    return Ok(None);
                };
                if location.raw.is_some() {
                    self.check_raw_write(location)?;
                }
                self.write_place(location, value)?;
            }
            Action::StorageLive(id) => {
                let ty = self.bodies[frame.body].locals[id.0 as usize].ty;
                if !matches!(
                    self.bodies[frame.body].locals[id.0 as usize].kind,
                    LocalKind::Parameter(_)
                ) {
                    frame.locals[id.0 as usize] =
                        Some(self.allocate(&mut frame.allocations, None, ty)?);
                }
            }
            Action::StorageDead(id) => self.end_local(frame, id)?,
            Action::StorageEnd { first, last } => {
                let body = self.bodies[frame.body].clone();
                for id in body.storage_range(first, last) {
                    self.end_local(frame, id?)?;
                }
            }
            Action::Project(id) => {
                frame.forget_projection(id);
                self.place(frame, crate::body::ir::Place::Projection(id))?;
            }
            Action::Select { projection, .. } => {
                frame.forget_projection(projection);
                self.place(frame, crate::body::ir::Place::Projection(projection))?;
            }
            Action::BindView { view, place } => {
                let Some(location) = self.place(frame, place)? else {
                    return Ok(None);
                };
                if let Some(access) = location.raw {
                    frame.raw_views.insert(view, access);
                } else {
                    frame.raw_views.remove(&view);
                }
                if let Some(union) = location.union {
                    frame.union_views.insert(view, union);
                } else {
                    frame.union_views.remove(&view);
                }
                frame.locals[view.0 as usize] = Some(location.address);
            }
            Action::CheckInitialized(_)
            | Action::Mention(_)
            | Action::GuardStart(_)
            | Action::GuardUse(_) => {}
        }
        Ok(None)
    }
}

#[cfg(test)]
pub(crate) fn const_continuation_size() -> usize {
    size_of::<use_calls::Continuation>()
}
