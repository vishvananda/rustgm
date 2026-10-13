//! Iterative construction visits executable syntax once and consumes PA9 selections.
mod aggregates;
mod conditions;
mod decisions;
mod drops;
mod guards;
mod matches;
mod pattern_tests;
mod patterns;
mod references;
mod retries;
mod storage;
use super::ir::{
    Action, Block, BlockId, Body, Local, LocalId, LocalKind, Owner, Place, Program, Rvalue,
    ScalarId, Statement, Terminator, Value, ValueId, ValueList,
};
use super::{scopes, storage::Range as ScopeRange};
use crate::{
    lex::token::Punctuation as Op,
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::NodeKey,
    },
    sema::model::{AdjustmentKind, Declarations, NodeFacts, Operation},
    syntax::ast::{BlockFlavor, Kind, List, NodeId},
    types::{Kind as Type, TypeId},
};
use std::collections::HashMap;

/// Construct function and initializer CFGs from the completed type-checking result.
///
/// # Errors
/// Diagnoses missing typed facts, exhausted identities and unsupported body forms.
pub fn build(report: &Report, facts: &Declarations) -> Result<Program, Error> {
    let mut drop_query = (!facts.destructors.implementations().is_empty()).then(|| {
        crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        )
    });
    let mut scratch = Scratch {
        scopes: scopes::Scratch::default(),
        unadjusted: Vec::new(),
        values: facts.nodes.iter().map(|n| vec![None; n.len()]).collect(),
        definitions: vec![None; facts.definitions.len()],
        touched: Vec::new(),
        bindings: Vec::new(),
        work: Vec::new(),
        live: Vec::new(),
        controls: HashMap::new(),
        reach: Vec::new(),
        places: Vec::new(),
        field_values: Vec::new(),
        pattern_work: Vec::new(),
        match_frames: Vec::new(),
        guard_frames: Vec::new(),
        match_patterns: Vec::new(),
        conditional_live: Vec::new(),
    };
    let mut program = Program {
        drops: None,
        bodies: Vec::new(),
        region_equalities: facts.region_equalities.clone(),
        coercions: facts.coercions.clone(),
        outlives: facts.outlives.clone(),
        body_regions: facts.body_regions.clone(),
    };
    for function in &facts.functions {
        let Some(source) = function.body else {
            continue;
        };
        let Some(Type::Function { result, .. }) = facts.types.kind(function.signature) else {
            return Err(Error::at(source.file, 0, "missing function signature"));
        };
        let body = Body::new(
            Owner::Function {
                definition: function.definition,
                signature: function.signature,
            },
            source,
            result,
        );
        let scope_plan = scopes::function_with_scratch(
            report,
            facts,
            function,
            std::mem::take(&mut scratch.scopes),
        )?;
        let builder = Builder::new(
            report,
            facts,
            &mut scratch,
            body,
            scope_plan,
            drop_query.as_mut(),
        );
        program.bodies.push(
            builder
                .run(Some(function.parameters))
                .map_err(|e| report.diagnostic(e))?,
        );
    }
    for initializer in &facts.initializers {
        let body = Body::new(
            Owner::Initializer {
                definition: initializer.definition,
                kind: initializer.kind,
            },
            initializer.value,
            initializer.ty,
        );
        let scope_plan = scopes::initializer_with_scratch(
            report,
            facts,
            initializer,
            std::mem::take(&mut scratch.scopes),
        )?;
        let builder = Builder::new(
            report,
            facts,
            &mut scratch,
            body,
            scope_plan,
            drop_query.as_mut(),
        );
        program
            .bodies
            .push(builder.run(None).map_err(|e| report.diagnostic(e))?);
    }
    aggregates::constructors(report, facts, &mut program, drop_query.as_mut())?;
    drops::finish(report, facts, &mut program, drop_query.as_mut())?;
    Ok(program)
}
impl Body {
    fn new(owner: Owner, source: NodeKey, result: TypeId) -> Self {
        Self {
            owned: None,
            storage: None,
            promotions: None,
            temporaries: Vec::new(),
            guards: Vec::new(),
            guard_places: Vec::new(),
            projections: Vec::new(),
            owner,
            source,
            result,
            locals: Vec::new(),
            values: Vec::new(),
            scalars: Vec::new(),
            arguments: Vec::new(),
            statements: Vec::new(),
            blocks: Vec::new(),
        }
    }
}
#[derive(Clone, Copy)]
struct Control {
    exit: BlockId,
    repeat: Option<BlockId>,
    merge: Option<LocalId>,
    live_base: usize,
    scope: scopes::Id,
}
#[derive(Default)]
struct Reach {
    incoming: bool,
    opened: bool,
}
#[derive(Clone, Copy)]
enum Work {
    CloseScope {
        node: NodeId,
        scopes: ScopeRange,
    },
    Conditional(conditions::Work),
    MatchStart(NodeId),
    MatchNext(usize),
    GuardFinish(usize),
    BindPattern {
        node: NodeId,
        pattern: crate::sema::patterns::PatternId,
        has_value: bool,
    },
    Place(NodeId),
    FinishPlace(NodeId),
    TemporaryPlace(NodeId),
    ReadPlace(NodeId),
    DiscardPlace(NodeId),
    Enter(NodeId),
    EnterUnadjusted(NodeId),
    Reborrow(NodeId),
    Finish(NodeId),
    EndBlock {
        node: NodeId,
        tail: Option<NodeId>,
        base: Option<usize>,
    },
    Switch(BlockId),
    Branch {
        node: NodeId,
        condition: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
        failure_exit: Option<ScopeRange>,
    },
    Arm {
        node: Option<NodeId>,
        source: NodeId,
        merge: Option<LocalId>,
        target: BlockId,
        base: usize,
        scope_exit: Option<ScopeRange>,
    },
    Join {
        node: NodeId,
        target: BlockId,
        merge: Option<LocalId>,
        control: bool,
    },
    Repeat {
        node: NodeId,
        target: BlockId,
        scope_exit: ScopeRange,
    },
}
struct Scratch {
    unadjusted: Vec<NodeId>,
    scopes: scopes::Scratch,
    conditional_live: Vec<LocalId>,
    match_frames: Vec<matches::Frame>,
    guard_frames: Vec<Option<guards::Frame>>,
    match_patterns: Vec<crate::sema::patterns::PatternId>,
    pattern_work: Vec<patterns::Work>,
    places: Vec<Option<Place>>,
    field_values: Vec<Option<ValueId>>,
    values: Vec<Vec<Option<ValueId>>>,
    definitions: Vec<Option<LocalId>>,
    touched: Vec<NodeId>,
    bindings: Vec<DefId>,
    work: Vec<Work>,
    live: Vec<LocalId>,
    controls: HashMap<NodeId, Control>,
    reach: Vec<Reach>,
}
struct Builder<'a> {
    drop_query: Option<&'a mut crate::types::destruction::NeedsDrop>,
    report: &'a Report,
    facts: &'a Declarations,
    scratch: &'a mut Scratch,
    body: Body,
    current: Option<BlockId>,
    scope_plan: scopes::Facts,
    scope_exits: Vec<super::storage::Exit>,
}
impl<'a> Builder<'a> {
    fn new(
        report: &'a Report,
        facts: &'a Declarations,
        scratch: &'a mut Scratch,
        body: Body,
        scope_plan: scopes::Facts,
        drop_query: Option<&'a mut crate::types::destruction::NeedsDrop>,
    ) -> Self {
        Self {
            drop_query,
            report,
            facts,
            scratch,
            body,
            current: None,
            scope_plan,
            scope_exits: Vec::new(),
        }
    }
    fn error(&self, node: NodeId, message: impl Into<String>) -> Error {
        Error::at(
            self.body.source.file,
            self.report
                .package
                .unit(self.body.source.file)
                .configured
                .ast
                .node(node)
                .span
                .start,
            message,
        )
    }
    fn index(&self, node: NodeId, n: usize) -> Result<u32, Error> {
        u32::try_from(n).map_err(|_| self.error(node, "body identity space exhausted"))
    }
    fn kind(&self, node: NodeId) -> Kind {
        self.report
            .package
            .unit(self.body.source.file)
            .configured
            .ast
            .node(node)
            .kind
    }
    fn active(&self, node: NodeId) -> bool {
        self.report
            .package
            .unit(self.body.source.file)
            .configured
            .active[node.0 as usize]
    }
    fn fact(&self, node: NodeId) -> Result<&NodeFacts, Error> {
        self.facts
            .node(NodeKey {
                file: self.body.source.file,
                node,
            })
            .ok_or_else(|| self.error(node, "missing node facts"))
    }
    fn ty(&self, node: NodeId) -> Result<TypeId, Error> {
        self.fact(node)?
            .ty
            .ok_or_else(|| self.error(node, "missing solved type"))
    }
    fn value(&self, node: NodeId) -> Result<ValueId, Error> {
        self.scratch.values[self.body.source.file.0 as usize][node.0 as usize]
            .ok_or_else(|| self.error(node, "missing evaluated value"))
    }
    fn save(&mut self, node: NodeId, mut value: ValueId) -> Result<(), Error> {
        if self.scratch.unadjusted.last() != Some(&node)
            && let Some(adjustment) = self.fact(node)?.adjustment
        {
            match adjustment.kind {
                AdjustmentKind::FunctionPointer => {
                    if self.body.values[value.0 as usize].ty != adjustment.target {
                        value = self.evaluate(node, adjustment.target, Rvalue::Reify(value))?;
                    }
                }
                AdjustmentKind::ArrayToSlice | AdjustmentKind::Equality => {
                    if self.body.values[value.0 as usize].ty != adjustment.target {
                        value = self.evaluate(node, adjustment.target, Rvalue::Constrain(value))?;
                    }
                }
                AdjustmentKind::Reborrow => {
                    if self.body.values[value.0 as usize].ty != adjustment.target {
                        return Err(self.error(node, "missing selected reference adjustment"));
                    }
                }
                // A labeled block's fallthrough can diverge while its break join yields a value.
                AdjustmentKind::Never
                    if self.current.is_some()
                        && self.body.values[value.0 as usize].ty != adjustment.target =>
                {
                    return Err(self.error(node, "reachable never adjustment"));
                }
                AdjustmentKind::Never => {}
            }
        }
        let slot = &mut self.scratch.values[self.body.source.file.0 as usize][node.0 as usize];
        if slot.is_none() {
            self.scratch.touched.push(node);
        }
        *slot = Some(value);
        self.forward_owned(node, value);
        Ok(())
    }
    fn statement(&mut self, node: NodeId, action: Action) -> Result<(), Error> {
        let block = self
            .current
            .ok_or_else(|| self.error(node, "statement without an open block"))?;
        let end = self.index(node, self.body.statements.len() + 1)?;
        self.body.statements.push(Statement {
            source: node,
            action,
        });
        self.body.blocks[block.0 as usize].end = end;
        Ok(())
    }
    fn evaluate(&mut self, node: NodeId, ty: TypeId, kind: Rvalue) -> Result<ValueId, Error> {
        let value = ValueId(self.index(node, self.body.values.len())?);
        self.body.values.push(Value { ty, kind });
        self.statement(node, Action::Evaluate(value))?;
        self.create_owned(node, value)?;
        Ok(value)
    }
    fn unit(&mut self, node: NodeId) -> Result<ValueId, Error> {
        self.evaluate(node, TypeId::UNIT, Rvalue::Unit)
    }
    fn new_block(&mut self, node: NodeId) -> Result<BlockId, Error> {
        let id = BlockId(self.index(node, self.body.blocks.len())?);
        self.body.blocks.push(Block {
            start: 0,
            end: 0,
            source: node,
            terminator: Terminator::Unreachable,
        });
        self.scratch.reach.push(Reach::default());
        Ok(id)
    }
    fn switch(&mut self, node: NodeId, block: BlockId) -> Result<(), Error> {
        if self.current.is_some() || self.scratch.reach[block.0 as usize].opened {
            return Err(self.error(node, "block construction revisited an open block"));
        }
        self.scratch.reach[block.0 as usize].opened = true;
        let start = self.index(node, self.body.statements.len())?;
        let target = &mut self.body.blocks[block.0 as usize];
        target.start = start;
        target.end = start;
        self.current = self.scratch.reach[block.0 as usize]
            .incoming
            .then_some(block);
        Ok(())
    }
    fn terminate(&mut self, node: NodeId, terminator: Terminator) {
        if let Some(block) = self.current.take() {
            match terminator {
                Terminator::Jump(target) => self.scratch.reach[target.0 as usize].incoming = true,
                Terminator::Branch { yes, no, .. }
                | Terminator::FalseEdge {
                    real: yes,
                    imaginary: no,
                } => {
                    self.scratch.reach[yes.0 as usize].incoming = true;
                    self.scratch.reach[no.0 as usize].incoming = true;
                }
                Terminator::Return(_) | Terminator::Unreachable => {}
            }
            let block = &mut self.body.blocks[block.0 as usize];
            block.terminator = terminator;
            block.source = node;
        }
    }
    fn local(
        &mut self,
        node: NodeId,
        ty: TypeId,
        kind: LocalKind,
        binding: Option<DefId>,
        mutable: bool,
    ) -> Result<LocalId, Error> {
        let local = LocalId(self.index(node, self.body.locals.len())?);
        self.body.locals.push(Local {
            scope: binding
                .and_then(|id| self.scope_plan.binding(id))
                .or_else(|| self.scope_plan.node(node).map(|n| n.scope))
                .unwrap_or(scopes::Id(0)),
            storage_parent: self.scratch.live.last().copied(),
            ty,
            kind,
            definition: binding,
            mutable,
            source: node,
        });
        if let Some(definition) = binding {
            self.scratch.definitions[definition.0 as usize] = Some(local);
            self.scratch.bindings.push(definition);
        }
        self.scratch.live.push(local);
        self.statement(node, Action::StorageLive(local))?;
        Ok(local)
    }
    fn merge(&mut self, node: NodeId) -> Result<Option<LocalId>, Error> {
        let ty = self.ty(node)?;
        if ty == TypeId::UNIT || ty == TypeId::NEVER {
            return Ok(None);
        }
        self.local(node, ty, LocalKind::Merge, None, true).map(Some)
    }
    fn cleanup(&mut self, node: NodeId, base: usize) -> Result<(), Error> {
        if self.current.is_some() && base < self.scratch.live.len() {
            let first = self.scratch.live[base];
            let last = self.scratch.live[self.scratch.live.len() - 1];
            self.statement(node, Action::end_storage(first, last))?;
        }
        Ok(())
    }
    fn store(&mut self, node: NodeId, place: Place, value: ValueId) -> Result<(), Error> {
        let ty = match place {
            Place::Local(id) => Some(self.body.locals[id.0 as usize].ty),
            Place::Static(id) => self.facts.definitions[id.0 as usize],
            Place::Temporary(id) => Some(self.body.values[id.0 as usize].ty),
            Place::Projection(id) => Some(self.body.projections[id.0 as usize].ty),
        };
        if ty != Some(self.body.values[value.0 as usize].ty) {
            return Err(self.error(node, "store disagrees with solved type"));
        }
        self.statement(node, Action::Store { place, value })
    }
    fn binding(
        &mut self,
        mut node: NodeId,
        ty: TypeId,
        kind: LocalKind,
    ) -> Result<Option<LocalId>, Error> {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(node) {
            node = inner;
        }
        match self.kind(node) {
            Kind::Binding { mutable, .. } => {
                let definition = self
                    .fact(node)?
                    .binding
                    .ok_or_else(|| self.error(node, "missing binding identity"))?;
                self.local(node, ty, kind, Some(definition), mutable)
                    .map(Some)
            }
            Kind::Wildcard if matches!(kind, LocalKind::Parameter(_)) => {
                self.local(node, ty, kind, None, false).map(Some)
            }
            Kind::Wildcard => Ok(None),
            _ => Err(self.error(node, "unsupported body binding pattern")),
        }
    }
    fn parameters(&mut self, list: List) -> Result<(), Error> {
        let mut index = 0;
        let nodes = self
            .report
            .package
            .unit(self.body.source.file)
            .configured
            .ast
            .list(list);
        for &node in nodes {
            if !self.active(node) {
                continue;
            }
            let mut parameter = node;
            while let Kind::Attributed { value, .. } = self.kind(parameter) {
                parameter = value;
            }
            let Kind::Parameter { pattern, .. } = self.kind(parameter) else {
                return Err(self.error(parameter, "missing parameter syntax"));
            };
            if let Some(receiver) = self.facts.destructors.receivers.get(&NodeKey {
                file: self.body.source.file,
                node: parameter,
            }) {
                self.local(
                    receiver.binding.node,
                    receiver.ty,
                    LocalKind::Parameter(index),
                    Some(receiver.definition),
                    receiver.mutable,
                )?;
                index += 1;
                continue;
            }
            if let Some(id) = self
                .facts
                .patterns
                .roots
                .get(&NodeKey {
                    file: self.body.source.file,
                    node: pattern,
                })
                .copied()
            {
                let local = self.local(
                    pattern,
                    self.ty(parameter)?,
                    LocalKind::Parameter(index),
                    None,
                    false,
                )?;
                self.bind_pattern(id, Some(Place::Local(local)))?;
            } else {
                self.binding(pattern, self.ty(parameter)?, LocalKind::Parameter(index))?;
            }
            index += 1;
        }
        Ok(())
    }
    fn run(mut self, parameters: Option<List>) -> Result<Body, Error> {
        let root = self.body.source.node;
        let entry = self.new_block(root)?;
        self.scratch.reach[entry.0 as usize].incoming = true;
        self.switch(root, entry)?;
        if let Some(parameters) = parameters {
            self.parameters(parameters)?;
        }
        self.scratch.work.push(Work::Enter(root));
        while let Some(work) = self.scratch.work.pop() {
            self.step(work)?;
        }
        if self.current.is_some() {
            let value = self.value(root)?;
            if self.body.values[value.0 as usize].ty != self.body.result {
                return Err(self.error(root, "return disagrees with solved result"));
            }
            self.return_value(root, value, self.boundary(root, scopes::Kind::Body)?)?;
        }
        for node in self.scratch.touched.drain(..) {
            self.scratch.values[self.body.source.file.0 as usize][node.0 as usize] = None;
        }
        for definition in self.scratch.bindings.drain(..) {
            self.scratch.definitions[definition.0 as usize] = None;
        }
        self.scratch.live.clear();
        self.scratch.controls.clear();
        self.scratch.reach.clear();
        self.scratch.match_frames.clear();
        self.scratch.guard_frames.clear();
        self.scratch.match_patterns.clear();
        if !self.scratch.conditional_live.is_empty() {
            return Err(self.error(root, "unbalanced conditional storage continuations"));
        }
        if !self.scratch.places.is_empty() {
            return Err(self.error(root, "unbalanced place construction stack"));
        }
        if !self.scratch.unadjusted.is_empty() {
            return Err(self.error(root, "unbalanced reference adjustment continuations"));
        }
        let scope_work = self.scope_plan.work;
        let (scopes, scratch) = self.scope_plan.finish();
        self.scratch.scopes = scratch;
        self.body.storage = Some(super::storage::Facts {
            scope_work,
            scopes,
            exits: self.scope_exits,
        });
        Ok(self.body)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "flat worklist dispatch keeps storage and control-flow continuations explicit within the file audit limit"
    )]
    fn step(&mut self, work: Work) -> Result<(), Error> {
        match work {
            Work::CloseScope { node, scopes } => self.exit_scopes(node, scopes)?,
            Work::Conditional(work) => self.conditional_step(work)?,
            Work::MatchStart(node) => self.start_match(node)?,
            Work::MatchNext(frame) => self.next_match_arm(frame)?,
            Work::GuardFinish(frame) => self.finish_guard(frame)?,
            Work::BindPattern {
                node,
                pattern,
                has_value,
            } => {
                let place = if has_value {
                    self.pop_place(node)?
                } else {
                    None
                };
                if self.current.is_some() {
                    self.bind_pattern(pattern, place)?;
                    let value = self.unit(node)?;
                    self.save(node, value)?;
                }
            }
            Work::Place(node) => self.enter_place(node)?,
            Work::FinishPlace(node) => self.finish_place(node)?,
            Work::TemporaryPlace(node) => {
                let place = if let Some(block) = self.current {
                    let value = self.value(node)?;
                    self.body.temporaries.push(super::ir::Temporary {
                        lifetime: self
                            .scope_plan
                            .node(node)
                            .ok_or_else(|| self.error(node, "missing temporary scope"))?
                            .temporary,
                        value,
                        source: node,
                        block,
                        before: self.index(node, self.body.statements.len())?,
                        exits_before: self.index(node, self.scope_exits.len())?,
                    });
                    Some(Place::Temporary(value))
                } else {
                    None
                };
                self.scratch.places.push(place);
            }
            Work::ReadPlace(node) => self.read_place(node)?,
            Work::DiscardPlace(node) => {
                self.pop_place(node)?;
            }
            Work::Reborrow(node) => self.finish_reborrow(node)?,
            Work::EnterUnadjusted(node) if self.current.is_some() && self.active(node) => {
                self.enter(node)?;
            }
            Work::Enter(node) if self.current.is_some() && self.active(node) => {
                if let Some(scopes) = self.expression_exit(node) {
                    self.scratch.work.push(Work::CloseScope { node, scopes });
                }
                if self
                    .fact(node)?
                    .adjustment
                    .is_some_and(|a| a.kind == AdjustmentKind::Reborrow)
                {
                    self.scratch.unadjusted.push(node);
                    self.scratch.work.push(Work::Reborrow(node));
                    self.scratch.work.push(Work::Place(node));
                } else {
                    self.enter(node)?;
                }
            }
            Work::Finish(node) if self.current.is_some() => self.finish(node)?,
            Work::Finish(node) => self.discard_unreachable_place(node)?,
            Work::EnterUnadjusted(_) | Work::Enter(_) => {}
            Work::EndBlock { node, tail, base } => self.finish_scope(node, tail, base)?,
            Work::Switch(target) => {
                self.switch(self.body.blocks[target.0 as usize].source, target)?;
            }
            Work::Branch {
                node,
                condition,
                yes,
                no,
                base,
                failure_exit,
            } => {
                if self.current.is_some() {
                    self.cleanup(node, base)?;
                    if let Some(scopes) = failure_exit {
                        self.exit_on_edge(node, no, scopes)?;
                    }
                    self.terminate(
                        node,
                        Terminator::Branch {
                            condition: self.value(condition)?,
                            yes,
                            no,
                        },
                    );
                }
                self.scratch.live.truncate(base);
            }
            Work::Arm {
                node,
                source,
                merge,
                target,
                base,
                scope_exit,
            } => {
                if self.current.is_some() {
                    if let Some(local) = merge {
                        let value = if let Some(node) = node {
                            self.value(node)?
                        } else {
                            self.unit(source)?
                        };
                        self.store(source, Place::Local(local), value)?;
                    }
                    if let Some(scopes) = scope_exit {
                        self.exit_scopes(source, scopes)?;
                    }
                    self.cleanup(source, base)?;
                    self.terminate(source, Terminator::Jump(target));
                }
                self.scratch.live.truncate(base);
            }
            Work::Join {
                node,
                target,
                merge,
                control,
            } => {
                if control {
                    self.scratch.controls.remove(&node);
                }
                self.switch(node, target)?;
                if self.current.is_some() {
                    let value = if let Some(local) = merge {
                        self.evaluate(node, self.ty(node)?, Rvalue::Read(Place::Local(local)))?
                    } else {
                        self.unit(node)?
                    };
                    self.save(node, value)?;
                }
            }
            Work::Repeat {
                node,
                target,
                scope_exit,
            } => {
                self.exit_scopes(node, scope_exit)?;
                self.terminate(node, Terminator::Jump(target));
            }
        }
        Ok(())
    }
    fn children(&mut self, list: List) {
        let children = self
            .report
            .package
            .unit(self.body.source.file)
            .configured
            .ast
            .list(list);
        self.scratch
            .work
            .extend(children.iter().rev().copied().map(Work::Enter));
    }
    fn finish_scope(
        &mut self,
        node: NodeId,
        tail: Option<NodeId>,
        base: Option<usize>,
    ) -> Result<(), Error> {
        if self.current.is_some() {
            let value = if let Some(tail) = tail {
                self.value(tail)?
            } else {
                self.unit(node)?
            };
            self.save(node, value)?;
            if let Some(base) = base {
                self.exit_scopes(node, self.boundary_range(node, scopes::Kind::Block)?)?;
                self.cleanup(node, base)?;
            }
        }
        if let Some(base) = base {
            self.scratch.live.truncate(base);
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "cohesive syntax dispatch exposes evaluation order and work-stack scheduling"
    )]
    fn enter(&mut self, node: NodeId) -> Result<(), Error> {
        if let Kind::Let {
            value: Some(value),
            otherwise: Some(_),
            ..
        } = self.kind(node)
        {
            self.scratch
                .work
                .push(Work::Conditional(conditions::Work::Else {
                    node,
                    temporaries: self.index(node, self.body.temporaries.len())?,
                }));
            self.scratch.work.push(Work::Place(value));
            return Ok(());
        }
        if let Kind::Let { pattern, value, .. } = self.kind(node)
            && let Some(pattern) = self.pattern_root(pattern)
        {
            self.scratch.work.push(Work::BindPattern {
                node,
                pattern,
                has_value: value.is_some(),
            });
            if let Some(value) = value {
                self.scratch.work.push(Work::Place(value));
            }
            return Ok(());
        }
        match self.kind(node) {
            Kind::Match { value, .. } => {
                self.scratch.work.push(Work::MatchStart(node));
                self.scratch.work.push(Work::Place(value));
            }
            Kind::Block {
                flavor: BlockFlavor::Plain,
                statements,
                tail,
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.scratch.work.push(Work::EndBlock {
                    node,
                    tail,
                    // A statement expansion adds statements to its enclosing
                    // scope; only a source block ends their bindings' storage.
                    base: matches!(self.kind(node), Kind::Block { .. })
                        .then_some(self.scratch.live.len()),
                });
                if let Some(tail) = tail {
                    self.scratch.work.push(Work::Enter(tail));
                }
                self.children(statements);
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } if self.conditional_chain(condition) => {
                self.conditional_choice(node, condition, then, otherwise)?;
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => self.choice(node, condition, Some(then), otherwise)?,
            Kind::Binary {
                op: Op::AndAnd,
                left,
                right,
            } => self.choice(node, left, Some(right), Some(left))?,
            Kind::Binary {
                op: Op::OrOr,
                left,
                right,
            } => self.choice(node, left, Some(left), Some(right))?,
            Kind::Loop { body } => self.loop_body(node, body, None)?,
            Kind::While { condition, body } if self.conditional_chain(condition) => {
                self.conditional_loop(node, condition, body)?;
            }
            Kind::While { condition, body } => self.loop_body(node, body, Some(condition))?,
            Kind::Label { value, .. } if matches!(self.kind(value), Kind::Block { .. }) => {
                let merge = self.merge(value)?;
                let exit = self.new_block(value)?;
                self.scratch.controls.insert(
                    value,
                    Control {
                        exit,
                        repeat: None,
                        merge,
                        live_base: self.scratch.live.len(),
                        scope: self.node_scope(value)?,
                    },
                );
                self.scratch.work.push(Work::Finish(node));
                self.scratch.work.push(Work::Join {
                    node: value,
                    target: exit,
                    merge,
                    control: true,
                });
                self.scratch.work.push(Work::Arm {
                    node: Some(value),
                    source: value,
                    merge,
                    target: exit,
                    base: self.scratch.live.len(),
                    scope_exit: None,
                });
                self.scratch.work.push(Work::Enter(value));
            }
            kind => {
                if self.projected_syntax(node) {
                    self.scratch.work.push(Work::ReadPlace(node));
                    self.scratch.work.push(Work::Place(node));
                    return Ok(());
                }
                self.scratch.work.push(Work::Finish(node));
                match kind {
                    Kind::Let {
                        pattern,
                        value: Some(value),
                        ..
                    } if self.discarded_local(pattern, value)? => {
                        if self.projected_syntax(value) {
                            self.scratch.work.push(Work::DiscardPlace(value));
                            self.scratch.work.push(Work::Place(value));
                        }
                    }
                    Kind::Let { value, .. } | Kind::Return(value) | Kind::Break { value, .. } => {
                        if let Some(value) = value {
                            self.scratch.work.push(Work::Enter(value));
                        }
                    }
                    Kind::Unary { operand, .. }
                        if self.fact(node)?.operation == Some(Operation::Borrow) =>
                    {
                        self.scratch.work.push(Work::Place(operand));
                    }
                    Kind::Unary { operand, .. }
                    | Kind::Group(operand)
                    | Kind::Statement(operand)
                    | Kind::Attributed { value: operand, .. }
                    | Kind::Label { value: operand, .. }
                    | Kind::Cast { value: operand, .. } => {
                        self.scratch.work.push(Work::Enter(operand));
                    }
                    Kind::Binary { left, right, .. } => {
                        if matches!(
                            self.fact(node)?.operation,
                            Some(Operation::AssignPlace { .. })
                        ) {
                            self.scratch.work.push(Work::Place(left));
                            self.scratch.work.push(Work::Enter(right));
                            return Ok(());
                        }
                        self.scratch.work.push(Work::Enter(right));
                        if !matches!(self.fact(node)?.operation, Some(Operation::Assign { .. })) {
                            self.scratch.work.push(Work::Enter(left));
                        }
                    }
                    Kind::Call { callee, arguments } => {
                        self.children(arguments);
                        self.scratch.work.push(Work::Enter(callee));
                    }
                    Kind::Tuple(elements) | Kind::Array(elements) => self.children(elements),
                    Kind::Repeat { value, .. } | Kind::FieldInit { value, .. } => {
                        self.scratch.work.push(Work::Enter(value));
                    }
                    Kind::Struct { fields, base, .. } => {
                        if let Some(base) = base {
                            self.scratch.work.push(Work::Place(base));
                        }
                        self.children(fields);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    fn discarded_local(&self, mut pattern: NodeId, mut value: NodeId) -> Result<bool, Error> {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(pattern) {
            pattern = inner;
        }
        if !matches!(self.kind(pattern), Kind::Wildcard) {
            return Ok(false);
        }
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(value) {
            value = inner;
        }
        if self.projected_syntax(value) {
            return Ok(true);
        }
        if !matches!(self.kind(value), Kind::Path { .. } | Kind::Name(_)) {
            return Ok(false);
        }
        let Some(Target::Definition(definition)) = self.fact(value)?.reference else {
            return Ok(false);
        };
        // A wildcard does not read/move a place. Calls and block values still evaluate.
        Ok(matches!(
            self.report.resolution.definitions[definition.0 as usize].kind,
            DefKind::Local | DefKind::Parameter
        ))
    }
    fn choice(
        &mut self,
        node: NodeId,
        condition: NodeId,
        yes_value: Option<NodeId>,
        no_value: Option<NodeId>,
    ) -> Result<(), Error> {
        let branch_exit = if matches!(self.kind(node), Kind::If { .. }) {
            Some(self.boundary_range(node, scopes::Kind::IfThen)?)
        } else {
            None
        };
        let merge = self.merge(node)?;
        let yes = self.new_block(node)?;
        let no = self.new_block(node)?;
        let join = self.new_block(node)?;
        self.scratch.work.push(Work::Join {
            node,
            target: join,
            merge,
            control: false,
        });
        for (target, value) in [(no, no_value), (yes, yes_value)] {
            self.scratch.work.push(Work::Arm {
                node: value,
                source: node,
                merge,
                target: join,
                base: self.scratch.live.len(),
                scope_exit: if target == yes { branch_exit } else { None },
            });
            if let Some(value) = value
                && value != condition
            {
                self.scratch.work.push(Work::Enter(value));
            }
            self.scratch.work.push(Work::Switch(target));
        }
        self.scratch.work.push(Work::Branch {
            node,
            condition,
            yes,
            no,
            base: self.scratch.live.len(),
            failure_exit: branch_exit,
        });
        self.scratch.work.push(Work::Enter(condition));
        Ok(())
    }
    fn loop_body(
        &mut self,
        node: NodeId,
        body: NodeId,
        condition: Option<NodeId>,
    ) -> Result<(), Error> {
        let scope_exit = self.boundary_range(node, scopes::Kind::Iteration)?;
        let merge = self.merge(node)?;
        let header = self.new_block(node)?;
        let exit = self.new_block(node)?;
        self.scratch.controls.insert(
            node,
            Control {
                exit,
                repeat: Some(header),
                merge,
                live_base: self.scratch.live.len(),
                scope: self.node_scope(node)?,
            },
        );
        self.terminate(node, Terminator::Jump(header));
        self.scratch.work.push(Work::Join {
            node,
            target: exit,
            merge,
            control: true,
        });
        self.scratch.work.push(Work::Repeat {
            node,
            target: header,
            scope_exit,
        });
        self.scratch.work.push(Work::Enter(body));
        if let Some(condition) = condition {
            let block = self.new_block(body)?;
            self.scratch.work.push(Work::Switch(block));
            self.scratch.work.push(Work::Branch {
                node,
                condition,
                yes: block,
                no: exit,
                base: self.scratch.live.len(),
                failure_exit: Some(scope_exit),
            });
            self.scratch.work.push(Work::Enter(condition));
        }
        self.scratch.work.push(Work::Switch(header));
        Ok(())
    }
    fn local_place(&self, node: NodeId, definition: DefId) -> Result<Place, Error> {
        self.scratch.definitions[definition.0 as usize]
            .map(Place::Local)
            .ok_or_else(|| self.error(node, "missing local place"))
    }
    fn path(&mut self, node: NodeId) -> Result<Rvalue, Error> {
        if let Some(Operation::Construct { variant }) = self.fact(node)?.operation {
            let start = self.index(node, self.body.arguments.len())?;
            return Ok(Rvalue::Aggregate {
                variant: Some(variant),
                fields: ValueList { start, end: start },
            });
        }
        let Some(Target::Definition(definition)) = self.fact(node)?.reference else {
            return Err(self.error(node, "missing resolved value identity"));
        };
        Ok(
            match self.report.resolution.definitions[definition.0 as usize].kind {
                DefKind::Local | DefKind::Parameter => {
                    Rvalue::Read(self.local_place(node, definition)?)
                }
                DefKind::Function | DefKind::Constructor | DefKind::Variant => {
                    Rvalue::Function(definition)
                }
                DefKind::Const => Rvalue::Constant(definition),
                DefKind::Static => Rvalue::Read(Place::Static(definition)),
                _ => return Err(self.error(node, "unsupported resolved body value")),
            },
        )
    }
    fn literal_value(&mut self, node: NodeId, ty: TypeId) -> Result<ValueId, Error> {
        if matches!(
            self.facts.types.kind(ty),
            Some(crate::types::Kind::Reference { .. })
        ) {
            let key = NodeKey {
                node,
                ..self.body.source
            };
            let text = self
                .facts
                .text_literals
                .get(&key)
                .copied()
                .ok_or_else(|| self.error(node, "missing selected text literal"))?;
            return self.evaluate(node, ty, Rvalue::Text(text));
        }
        let scalar = self
            .fact(node)?
            .scalar
            .ok_or_else(|| self.error(node, "missing scalar value"))?;
        let id = ScalarId(self.index(node, self.body.scalars.len())?);
        self.body.scalars.push(scalar);
        self.evaluate(node, ty, Rvalue::Scalar(id))
    }
    fn finish(&mut self, node: NodeId) -> Result<(), Error> {
        let ty = self.ty(node)?;
        let value = match self.kind(node) {
            Kind::Tuple(elements) if elements.start != elements.end => {
                self.aggregate_elements(node, elements)?
            }
            Kind::Array(elements) => self.aggregate_elements(node, elements)?,
            Kind::Repeat { value, .. } => self.repeat(node, value)?,
            Kind::Struct { fields, base, .. } => self.construct(node, fields, base)?,
            Kind::FieldInit { value, .. } => self.value(value)?,
            Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                self.value(value)?
            }
            Kind::Literal(_) => self.literal_value(node, ty)?,
            Kind::Path { .. } | Kind::Name(_) => {
                let value = self.path(node)?;
                self.evaluate(node, ty, value)?
            }
            Kind::EmptyStatement | Kind::Item(_) | Kind::Statement(_) => self.unit(node)?,
            Kind::Tuple(list) if list.start == list.end => self.unit(node)?,
            Kind::Let {
                mut pattern, value, ..
            } => {
                while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } =
                    self.kind(pattern)
                {
                    pattern = inner;
                }
                let local = self.binding(pattern, self.ty(pattern)?, LocalKind::Binding)?;
                if let (Some(local), Some(value)) = (local, value) {
                    self.store(node, Place::Local(local), self.value(value)?)?;
                }
                self.unit(node)?
            }
            Kind::Unary { .. } if self.fact(node)?.operation == Some(Operation::Borrow) => {
                self.finish_borrow(node)?
            }
            Kind::Unary { operand, .. } => {
                let Some(Operation::Unary {
                    operator, literal, ..
                }) = self.fact(node)?.operation
                else {
                    return Err(self.error(node, "missing unary selection"));
                };
                self.evaluate(
                    node,
                    ty,
                    Rvalue::Unary {
                        operator,
                        operand: self.value(operand)?,
                        literal,
                    },
                )?
            }
            Kind::Binary { left, right, .. } => self.binary(node, left, right)?,
            Kind::Cast { value, .. } => {
                let Some(Operation::Cast { kind, .. }) = self.fact(node)?.operation else {
                    return Err(self.error(node, "missing cast selection"));
                };
                self.evaluate(
                    node,
                    ty,
                    Rvalue::Cast {
                        kind,
                        operand: self.value(value)?,
                    },
                )?
            }
            Kind::Call { callee, arguments } => self.call(node, callee, arguments)?,
            Kind::Return(value) => {
                let value = if let Some(value) = value {
                    self.value(value)?
                } else {
                    self.unit(node)?
                };
                self.return_value(node, value, self.node_scope(node)?)?;
                return Ok(());
            }
            Kind::Break { value, .. } => {
                self.transfer(node, value)?;
                return Ok(());
            }
            Kind::Continue(_) => {
                self.transfer(node, None)?;
                return Ok(());
            }
            _ => return Err(self.error(node, "unsupported typed body syntax")),
        };
        if ty == TypeId::NEVER {
            self.terminate(node, Terminator::Unreachable);
        }
        self.save(node, value)?;
        Ok(())
    }
    fn binary(&mut self, node: NodeId, left: NodeId, right: NodeId) -> Result<ValueId, Error> {
        let right = self.value(right)?;
        match self.fact(node)?.operation {
            Some(Operation::AssignPlace {
                operator, left: ty, ..
            }) => {
                let place = self
                    .pop_place(node)?
                    .ok_or_else(|| self.error(node, "missing assignment place"))?;
                let value = if let Some(operator) = operator {
                    let left = self.evaluate(node, ty, Rvalue::Read(place))?;
                    self.evaluate(
                        node,
                        ty,
                        Rvalue::Binary {
                            operator,
                            left,
                            right,
                        },
                    )?
                } else {
                    right
                };
                self.store(node, place, value)?;
                self.unit(node)
            }
            Some(Operation::Binary { operator, .. }) => self.evaluate(
                node,
                self.ty(node)?,
                Rvalue::Binary {
                    operator,
                    left: self.value(left)?,
                    right,
                },
            ),
            Some(Operation::Assign {
                target,
                operator,
                left: ty,
                ..
            }) => {
                let place = self.local_place(node, target)?;
                let value = if let Some(operator) = operator {
                    // Primitive compound assignment evaluates the RHS before reading the place.
                    let left = self.evaluate(node, ty, Rvalue::Read(place))?;
                    self.evaluate(
                        node,
                        ty,
                        Rvalue::Binary {
                            operator,
                            left,
                            right,
                        },
                    )?
                } else {
                    right
                };
                self.store(node, place, value)?;
                self.unit(node)
            }
            _ => Err(self.error(node, "missing binary selection")),
        }
    }
    fn call(&mut self, node: NodeId, callee: NodeId, arguments: List) -> Result<ValueId, Error> {
        let Some(Operation::Call { signature }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing call signature"));
        };
        let start = self.index(node, self.body.arguments.len())?;
        for &argument in self
            .report
            .package
            .unit(self.body.source.file)
            .configured
            .ast
            .list(arguments)
        {
            if self.active(argument) {
                self.body.arguments.push(self.value(argument)?);
            }
        }
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Call {
                signature,
                callee: self.value(callee)?,
                arguments: ValueList { start, end },
            },
        )
    }
    fn transfer(&mut self, node: NodeId, value: Option<NodeId>) -> Result<(), Error> {
        let Some(Operation::Transfer { target, continuing }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing control-transfer target"));
        };
        let control = *self
            .scratch
            .controls
            .get(&target)
            .ok_or_else(|| self.error(node, "inactive control-transfer target"))?;
        let target = if continuing {
            control
                .repeat
                .ok_or_else(|| self.error(node, "continue without loop header"))?
        } else {
            if let Some(local) = control.merge {
                let value = if let Some(value) = value {
                    self.value(value)?
                } else {
                    self.unit(node)?
                };
                self.store(node, Place::Local(local), value)?;
            }
            control.exit
        };
        self.exit_scopes(
            node,
            ScopeRange {
                inner: self.node_scope(node)?,
                outer: Some(control.scope),
            },
        )?;
        self.cleanup(node, control.live_base)?;
        self.terminate(node, Terminator::Jump(target));
        Ok(())
    }
}
