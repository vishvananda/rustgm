//! Shared typed constant queries solve discriminants and pattern values iteratively.
mod arithmetic;
mod canonical_values;
mod computation;
pub(super) use computation::Computations;
mod constructors;
mod contents;
mod discriminants;
mod drops;
mod execution;
mod parked_values;
pub(super) use discriminants::{
    Discriminant, Pending as Discriminants, Progress as DiscriminantProgress,
    advance as advance_discriminants,
};
mod generics;
pub(super) use generics::{
    Input as ArgumentInput, Progress as ArgumentProgress, advance as advance_argument,
    argument as generic_argument,
};
mod address_values;
pub(super) mod images;
mod instances;
mod layouts;
mod member_readiness;
mod patterns;
mod pending_patterns;
mod places;
pub(super) mod products;
pub(super) mod references;
mod scope_frames;
mod scope_progress;
mod scope_requests;
mod suspension;
pub(super) use suspension::Continuation as ArgumentContinuation;
pub(super) use suspension::Dependency as ArgumentDependency;
pub mod values;
use super::{
    bodies,
    members::VariantId,
    model::{Cast, Declarations, InitializerKind, Operation, Scalar},
};
use crate::{
    lex::token::Punctuation as Op,
    resolve::{
        expansion::Report,
        model::{DefId, Target},
        package::Error,
        unit::{FileId, NodeKey},
    },
    syntax::ast::{BlockFlavor, Kind, Unary},
    types::{Kind as Type, Primitive, TypeId},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Value {
    Scalar(Scalar),
    Function(crate::types::instance::Instance),
    Reference(references::Pointer),
    Address {
        bits: u64,
        metadata: Option<references::Metadata>,
    },
    BytePointer {
        text: super::text::Id,
        metadata: references::Metadata,
    },
    Text(super::text::Id),
    Bytes(super::text::Id),
    Variant(VariantId),
    Product(products::Id),
    Image(images::Id),
}
impl Value {
    const fn metadata(self) -> Option<references::Metadata> {
        match self {
            Self::Reference(pointer) => pointer.metadata,
            Self::Address { metadata, .. } => metadata,
            Self::BytePointer { metadata, .. } => Some(metadata),
            _ => None,
        }
    }
    fn scalar(self) -> Result<Scalar, &'static str> {
        match self {
            Self::Scalar(value) => Ok(value),
            Self::Reference(_)
            | Self::Address { .. }
            | Self::BytePointer { .. }
            | Self::Function(_)
            | Self::Variant(_)
            | Self::Product(_)
            | Self::Image(_)
            | Self::Text(_)
            | Self::Bytes(_) => Err("constant requires a scalar value"),
        }
    }
    fn integer(self) -> Result<u128, &'static str> {
        match self {
            Self::Scalar(Scalar::Integer(value)) => Ok(value),
            _ => Err("discriminant requires an integer value"),
        }
    }
    fn boolean(self) -> Result<bool, &'static str> {
        match self {
            Self::Scalar(Scalar::Boolean(value)) => Ok(value),
            _ => Err("constant condition requires bool"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Query {
    Source(computation::SourceComputation),
    Variant(VariantId),
    Constant(usize),
    Instance(crate::sema::values::Instance),
    Static(usize),
    Place(NodeKey),
    Contents(NodeKey, references::Id),
    Node(NodeKey),
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(Value),
}
#[derive(Clone, Copy)]
enum Expression {
    Execute,
    Value(Value),
    Product(products::Expression),
    Reference(references::Expression),
    Constructor(constructors::Expression),
    Forward(NodeKey),
    Constant(usize),
    Instance(crate::sema::values::Instance),
    ImportedConstant(DefId),
    ForeignStatic(DefId),
    Unary {
        op: Unary,
        operand: NodeKey,
        ty: Primitive,
        literal: bool,
    },
    Binary {
        op: Op,
        left: NodeKey,
        right: NodeKey,
        ty: Primitive,
    },
    Cast {
        kind: Cast,
        operand: NodeKey,
        source: TypeId,
        target: Primitive,
    },
    PointerCast {
        kind: Cast,
        operand: NodeKey,
        source: TypeId,
    },
    If {
        condition: NodeKey,
        then: NodeKey,
        otherwise: NodeKey,
    },
}
enum Work {
    Source(computation::SourceComputation),
    Execute(NodeKey),
    PatternOwner(NodeKey),
    PatternStart(pending_patterns::FrameId),
    PatternNext(pending_patterns::FrameId),
    PatternPublish(pending_patterns::FrameId, Query),
    PatternFinish(pending_patterns::FrameId),
    PrepareNode(NodeKey),
    PrepareInstance(crate::sema::values::Instance),
    Layout(NodeKey, TypeId, bool),
    Alignment(super::model::ReferenceAlignment),
    Reference(places::Work),
    Contents(contents::Work),
    Enter(Query),
    Forward(Query, Query),
    Successor(Query, Query, Primitive),
    Finish(NodeKey),
    Select(NodeKey),
    EnumCast(NodeKey, VariantId, Primitive),
}
struct Queries<'a> {
    report: &'a Report,
    typing: bodies::Checker<'a>,
    discriminants: Option<&'a mut Discriminants>,
    expressions: HashMap<(instances::Context, NodeKey), Expression>,
    instances: instances::Instances,
    states: HashMap<(instances::Context, Query), State>,
    work: instances::Worklist,
    scan: Vec<NodeKey>,
    steps: usize,
    limit: usize,
    source_work: usize,
    preparation_steps: usize,
    pattern_values: HashSet<Value>,
    drop_scratch: Option<Box<drops::Scratch>>,
    layout_scratch: layouts::Scratch,
    pattern_scope: Option<PatternScope>,
    pattern_dispatch: Option<super::dispatch::Dispatcher>,
    value_scope: Option<instances::ValueScope>,
    checked_members: HashSet<Query>,
    pattern_frames: Option<Box<pending_patterns::Frames>>,
}
/// Reusable storage belongs to one report and its append-only declaration facts.
#[derive(Default)]
pub(super) struct Scratch {
    typing: bodies::Scratch,
    source_work: Option<usize>,
    preparation_steps: usize,
    scan: Vec<NodeKey>,
    drops: Option<Box<drops::Scratch>>,
    layouts: layouts::Scratch,
}
impl Scratch {
    pub(in crate::sema) fn take_typing(&mut self) -> bodies::Scratch {
        std::mem::take(&mut self.typing)
    }
    pub(in crate::sema) fn retain_typing(&mut self, typing: bodies::Scratch) {
        self.typing = typing;
    }
}

#[derive(Clone, Copy)]
pub(super) enum PatternValue {
    Definition(DefId),
    Literal(NodeKey),
}
#[derive(Clone, Copy)]
pub(super) struct PatternScope {
    pub body: NodeKey,
    pub owner: Option<DefId>,
    pub binders: u32,
    pub end: usize,
}
/// Complete the ordered original scopes in one value/dependency graph.
pub(super) fn prepared_patterns(
    report: &Report,
    facts: &mut Declarations,
    scopes: Vec<bodies::PreparedScope>,
    scratch: &mut Option<Box<Scratch>>,
) -> Result<(), Error> {
    let mut queries =
        Queries::with_scratch(report, facts, None, *scratch.take().unwrap_or_default());
    for scope in scopes.iter().rev() {
        queries.work.push(Work::PatternOwner(scope.values().body));
    }
    queries.retain_prepared_patterns(scopes)?;
    queries.drain()?;
    *scratch = Some(Box::new(queries.into_scratch()));
    Ok(())
}
impl<'a> Queries<'a> {
    fn new(report: &'a Report, facts: &'a mut Declarations) -> Self {
        Self::with_scratch(report, facts, None, Scratch::default())
    }
    fn with_scratch(
        report: &'a Report,
        facts: &'a mut Declarations,
        discriminants: Option<&'a mut Discriminants>,
        mut scratch: Scratch,
    ) -> Self {
        // Scratch belongs to one immutable report. Resuming a body must not
        // rescan every source unit merely to reconstruct its work budget.
        let source_work = *scratch.source_work.get_or_insert_with(|| {
            scratch.preparation_steps += report.package.units.len();
            report.package.units.iter().fold(0_usize, |n, unit| {
                n.saturating_add(unit.configured.ast.nodes.len())
                    .saturating_add(unit.configured.ast.edges.len())
            })
        });
        let limit = source_work
            .saturating_add(discriminants.as_ref().map_or(0, |d| d.len()))
            .saturating_mul(16);
        Self {
            report,
            typing: bodies::Checker::with_scratch(report, facts, scratch.typing),
            discriminants,
            expressions: HashMap::new(),
            states: HashMap::new(),
            work: instances::Worklist::default(),
            instances: instances::Instances::default(),
            scan: scratch.scan,
            steps: 0,
            limit,
            source_work,
            preparation_steps: scratch.preparation_steps,
            pattern_values: HashSet::new(),
            pattern_frames: None,
            drop_scratch: scratch.drops,
            layout_scratch: scratch.layouts,
            pattern_scope: None,
            pattern_dispatch: None,
            value_scope: None,
            checked_members: HashSet::new(),
        }
    }
    fn into_scratch(self) -> Scratch {
        self.into_storage().0
    }
    fn into_storage(self) -> (Scratch, Option<Box<bodies::State>>) {
        let (typing, body) = self.typing.into_storage();
        (
            Scratch {
                source_work: Some(self.source_work),
                preparation_steps: self.preparation_steps,
                scan: self.scan,
                typing,
                drops: self.drop_scratch,
                layouts: self.layout_scratch,
            },
            body,
        )
    }
}
impl Queries<'_> {
    fn error(&self, query: Query, message: impl Into<String>) -> Error {
        let key = match query {
            Query::Source(source) => match source.identity(self.typing.facts) {
                Ok((_, source)) => source,
                Err(error) => return Error::at(FileId(0), 0, error),
            },
            Query::Variant(id) => {
                let Some(source) = self.typing.facts.members.variants[id.0 as usize]
                    .origin
                    .diagnostic()
                else {
                    return Error::at(FileId(0), 0, message);
                };
                source
            }
            Query::Constant(index) | Query::Static(index) => {
                self.typing.facts.initializers[index].value
            }
            Query::Node(key) | Query::Place(key) | Query::Contents(key, _) => key,
            Query::Instance(instance) => self
                .report
                .resolution
                .definitions
                .get(instance.definition.0 as usize)
                .and_then(|definition| definition.node)
                .unwrap_or(NodeKey {
                    file: FileId(0),
                    node: crate::syntax::ast::NodeId(0),
                }),
        };
        Declarations::error(self.report, key, message)
    }
    fn value(&self, query: Query) -> Result<Value, Error> {
        match self.states.get(&self.state_key(query)) {
            Some(State::Ready(value)) => Ok(*value),
            _ => Err(self.error(query, "missing completed constant dependency")),
        }
    }
    fn require_complete_patterns(&mut self, _query: Query) -> Result<(), Error> {
        self.capture_pending_patterns()
    }
    fn primitive(&self, key: NodeKey, ty: TypeId) -> Result<Primitive, Error> {
        if let Some(Type::Primitive(ty)) = self.typing.facts.types.kind(ty) {
            Ok(ty)
        } else {
            Err(self.error(Query::Node(key), "missing selected constant scalar type"))
        }
    }
    fn budget(&mut self) -> Result<(), Error> {
        self.steps += 1;
        if self.steps > self.limit {
            return Err(Error::at(
                FileId(0),
                0,
                "discriminant query work budget exceeded",
            ));
        }
        Ok(())
    }
    fn drain(&mut self) -> Result<(), Error> {
        if self.drain_until(None)?.is_some() {
            return Err(Error::at(
                FileId(0),
                0,
                "constant execution body is not ready",
            ));
        }
        Ok(())
    }
    fn drain_until(
        &mut self,
        ready: Option<&HashSet<DefId>>,
    ) -> Result<Option<suspension::Dependency>, Error> {
        let result = self.drain_until_inner(ready);
        if result.is_err() {
            self.abort_pattern_frames();
        }
        result
    }
    fn drain_until_inner(
        &mut self,
        ready: Option<&HashSet<DefId>>,
    ) -> Result<Option<suspension::Dependency>, Error> {
        while let Some(work) = self.work.pop() {
            self.resume_pattern_dispatch()?;
            self.reserve_pattern_dispatch()?;
            if let Some(ready) = ready
                && let Some(definition) = self.required_initializer(&work)?
                && !ready.contains(&definition)
            {
                self.work.push(work);
                return Ok(Some(suspension::Dependency::Initializer(definition)));
            }
            if ready.is_some()
                && let Some(dependency) = self.member_readiness(&work)?
            {
                self.work.push(work);
                return Ok(Some(dependency));
            }
            self.budget()?;
            match work {
                Work::PatternOwner(body) => {
                    self.schedule_pattern_owner(body, None)?;
                }
                Work::PatternStart(id) => self.start_pattern_frame(id)?,
                Work::PatternNext(id) => self.next_pattern_request(id)?,
                Work::PatternPublish(id, query) => self.publish_pattern_request(id, query)?,
                Work::PatternFinish(id) => self.finish_pattern_frame(id)?,
                Work::Source(source) => {
                    if let Some(requirement) = self.execute_source(source)? {
                        self.work.push(Work::Source(source));
                        if self.prepare_pattern_requirement(requirement)? {
                            continue;
                        }
                        return Ok(Some(suspension::Dependency::Execution(requirement)));
                    }
                }
                Work::Execute(key) => {
                    if let Some(requirement) = self.execute(key)? {
                        self.work.push(Work::Execute(key));
                        if self.prepare_pattern_requirement(requirement)? {
                            continue;
                        }
                        return Ok(Some(suspension::Dependency::Execution(requirement)));
                    }
                }
                Work::PrepareNode(key) => self.prepare(key)?,
                Work::PrepareInstance(instance) => self.prepare_instance(instance)?,
                Work::Layout(key, ty, finish) => self.layout(key, ty, finish)?,
                Work::Alignment(obligation) => self.check_alignment(obligation)?,
                Work::Enter(query) => self.enter(query)?,
                Work::Forward(query, dependency) => {
                    let value = self.value(dependency)?;
                    if let Query::Instance(instance) = query {
                        instance
                            .publish(self.typing.facts, None, values::Constant(value))
                            .map_err(|e| self.error(query, e))?;
                    }
                    self.complete_query(query, value)?;
                    if let Query::Contents(_, id) = query {
                        self.typing
                            .facts
                            .constant_references
                            .complete(id, value)
                            .map_err(|e| self.error(query, e))?;
                    }
                    if let Query::Constant(index) | Query::Static(index) = query {
                        let definition = self.typing.facts.initializers[index].definition;
                        self.typing.facts.constant_values.insert(definition, value);
                    }
                }
                Work::Successor(query, dependency, tag) => {
                    self.finish_successor(query, dependency, tag)?;
                }
                Work::Reference(work) => self.finish_reference(work)?,
                Work::Contents(work) => self.finish_contents(work)?,
                Work::Finish(key) => self.finish(key)?,
                Work::Select(key) => self.select(key)?,
                Work::EnumCast(key, variant, target) => {
                    let source = self.variant_tag(variant)?;
                    let value = self
                        .value(Query::Variant(variant))?
                        .scalar()
                        .map_err(|e| self.error(Query::Node(key), e))?;
                    let value = arithmetic::cast(Cast::IntegerToInteger, source, target, value)
                        .map_err(|e| self.error(Query::Node(key), e.message()))?;
                    self.states.insert(
                        self.state_key(Query::Node(key)),
                        State::Ready(Value::Scalar(value)),
                    );
                }
            }
        }
        Ok(None)
    }
    fn finish_successor(
        &mut self,
        query: Query,
        dependency: Query,
        tag: Primitive,
    ) -> Result<(), Error> {
        let bits = self
            .value(dependency)?
            .integer()
            .map_err(|e| self.error(query, e))?;
        let bits = arithmetic::successor(tag, bits).ok_or_else(|| {
            self.error(
                query,
                "implicit enum discriminant overflows its representation",
            )
        })?;
        self.complete_query(query, Value::Scalar(Scalar::Integer(bits)))?;
        Ok(())
    }
    fn forward(&mut self, query: Query, dependency: Query) {
        self.work.push(Work::Forward(query, dependency));
        self.work.push(Work::Enter(dependency));
    }
    fn enter(&mut self, query: Query) -> Result<(), Error> {
        if let Query::Source(source) = query {
            let result = source
                .result(self.typing.facts)
                .map_err(|e| self.error(query, e))?;
            source
                .validate(self.typing.facts, result)
                .map_err(|e| self.error(query, e))?;
        }
        match self.states.get(&self.state_key(query)) {
            Some(State::Ready(_)) => return Ok(()),
            Some(State::Active) => {
                return Err(
                    self.error(query, "cycle in enum discriminant or constant dependencies")
                );
            }
            None => {}
        }
        if let Query::Constant(index) = query {
            self.check_static_read(index)?;
        }
        if let Query::Constant(index) | Query::Static(index) = query {
            let definition = self.typing.facts.initializers[index].definition;
            if let Some(&value) = self.typing.facts.constant_values.get(&definition) {
                self.states
                    .insert(self.state_key(query), State::Ready(value));
                return Ok(());
            }
        }
        if let Query::Instance(instance) = query
            && let Some(&value) = self.typing.facts.constant_instances.get(&instance)
        {
            self.states
                .insert(self.state_key(query), State::Ready(value.value.0));
            return Ok(());
        }
        self.states.insert(self.state_key(query), State::Active);
        match query {
            Query::Source(source) => self.work.push(Work::Source(source)),
            Query::Instance(instance) => self.enter_instance(instance)?,
            Query::Variant(variant) => {
                self.work.context = instances::Context::default();
                self.enter_variant(variant)?;
            }
            Query::Constant(index) | Query::Static(index) => {
                self.work.context = instances::Context::default();
                if !self.typing.facts.initializers[index].type_checked() {
                    self.typing.initializer(index)?;
                    self.require_complete_patterns(query)?;
                }
                self.begin_pattern_value(index, self.work.context)?;
                let key = self.typing.facts.initializers[index].value;
                self.forward(query, Query::Node(key));
                self.prepare(key)?;
            }
            Query::Place(key) => self.enter_place(key)?,
            Query::Contents(key, id) => self.enter_contents(key, id)?,
            Query::Node(key) => self.enter_node(key)?,
        }
        Ok(())
    }
    fn enter_node(&mut self, key: NodeKey) -> Result<(), Error> {
        let query = Query::Node(key);
        match self.expression(key)? {
            Expression::Execute => self.work.push(Work::Execute(key)),
            Expression::Reference(expression) => self.enter_reference(key, expression),
            Expression::Value(value) => {
                self.complete_query(query, value)?;
            }
            Expression::Product(expression) => {
                self.work.push(Work::Finish(key));
                expression.dependencies(self.report, key, |child| {
                    self.work.push(Work::Enter(Query::Node(child)));
                });
            }
            Expression::Constructor(expression) => {
                self.work.push(Work::Finish(key));
                expression.dependencies(self.report, key, |child| {
                    self.work.push(Work::Enter(Query::Node(child)));
                });
            }
            Expression::Forward(child) => self.forward(query, Query::Node(child)),
            Expression::Constant(index) => self.forward(query, Query::Constant(index)),
            Expression::ImportedConstant(id) => {
                let value = self
                    .typing
                    .facts
                    .constant_values
                    .get(&id)
                    .copied()
                    .ok_or_else(|| {
                        self.error(query, "imported constant is missing its validated value")
                    })?;
                self.complete_query(query, value)?;
            }
            Expression::ForeignStatic(id) => {
                let value = self.imported_static_contents(query, id)?;
                let ty = self.typing.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(query, "missing imported static type"))?;
                self.check_copy_read(query, ty)?;
                self.complete_query(query, value)?;
            }
            Expression::Instance(instance) => self.forward(query, Query::Instance(instance)),
            Expression::Unary { operand, .. }
            | Expression::Cast { operand, .. }
            | Expression::PointerCast { operand, .. } => {
                self.work.push(Work::Finish(key));
                self.work.push(Work::Enter(Query::Node(operand)));
            }
            Expression::Binary {
                op: Op::AndAnd | Op::OrOr,
                left: condition,
                ..
            }
            | Expression::If { condition, .. } => {
                self.work.push(Work::Select(key));
                self.work.push(Work::Enter(Query::Node(condition)));
            }
            Expression::Binary { left, right, .. } => {
                self.work.push(Work::Finish(key));
                self.work.push(Work::Enter(Query::Node(right)));
                self.work.push(Work::Enter(Query::Node(left)));
            }
        }
        Ok(())
    }
    fn expression(&self, key: NodeKey) -> Result<Expression, Error> {
        self.expressions
            .get(&(self.work.context, key))
            .copied()
            .ok_or_else(|| self.error(Query::Node(key), "missing typed constant expression"))
    }
    fn select(&mut self, key: NodeKey) -> Result<(), Error> {
        let query = Query::Node(key);
        match self.expression(key)? {
            Expression::If {
                condition,
                then,
                otherwise,
            } => {
                let condition = self
                    .value(Query::Node(condition))?
                    .boolean()
                    .map_err(|e| self.error(query, e))?;
                self.forward(query, Query::Node(if condition { then } else { otherwise }));
            }
            Expression::Binary {
                op, left, right, ..
            } => {
                let condition = self
                    .value(Query::Node(left))?
                    .boolean()
                    .map_err(|e| self.error(query, e))?;
                if (op == Op::AndAnd && !condition) || (op == Op::OrOr && condition) {
                    self.states.insert(
                        self.state_key(query),
                        State::Ready(Value::Scalar(Scalar::Boolean(condition))),
                    );
                } else {
                    self.forward(query, Query::Node(right));
                }
            }
            _ => return Err(self.error(query, "invalid lazy constant operation")),
        }
        Ok(())
    }
    fn scalar(&self, key: NodeKey) -> Result<Scalar, Error> {
        self.value(Query::Node(key))?
            .scalar()
            .map_err(|e| self.error(Query::Node(key), e))
    }
    fn finish(&mut self, key: NodeKey) -> Result<(), Error> {
        let query = Query::Node(key);
        if let Expression::PointerCast {
            kind,
            operand,
            source,
        } = self.expression(key)?
        {
            let source = match self.typing.facts.types.kind(source) {
                Some(Type::Primitive(p)) => Some(p),
                _ => None,
            };
            let value = address_values::cast(kind, source, self.value(Query::Node(operand))?)
                .ok_or_else(|| self.error(query, "invalid selected constant pointer cast"))?
                .map_err(|e| self.error(query, e.message()))?;
            return self.complete_query(query, value);
        }
        if let Expression::Product(expression) = self.expression(key)? {
            let value = self.product_value(key, expression)?;
            return self.complete_query(query, value);
        }
        if let Expression::Constructor(expression) = self.expression(key)? {
            let value = self.constructor_value(key, expression)?;
            return self.complete_query(query, value);
        }
        let value = match self.expression(key)? {
            Expression::Unary {
                op,
                operand,
                ty,
                literal,
            } => arithmetic::unary(
                op,
                ty,
                self.scalar(operand)?,
                literal,
                values::Overflow::Checked,
            ),
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => arithmetic::binary(
                op,
                ty,
                self.scalar(left)?,
                self.scalar(right)?,
                values::Overflow::Checked,
            ),
            Expression::Cast {
                kind: Cast::EnumToInteger,
                operand,
                target,
                ..
            } => {
                let Value::Variant(variant) = self.value(Query::Node(operand))? else {
                    return Err(self.error(query, "missing constant enum variant"));
                };
                self.work.push(Work::EnumCast(key, variant, target));
                self.work.push(Work::Enter(Query::Variant(variant)));
                return Ok(());
            }
            Expression::Cast {
                kind,
                operand,
                source,
                target,
            } => arithmetic::cast(
                kind,
                self.primitive(key, source)?,
                target,
                self.scalar(operand)?,
            ),
            _ => return Err(self.error(query, "invalid completed constant operation")),
        }
        .map_err(|e| self.error(query, e.message()))?;
        self.states
            .insert(self.state_key(query), State::Ready(Value::Scalar(value)));
        Ok(())
    }
    fn prepare(&mut self, root: NodeKey) -> Result<(), Error> {
        if self.begin_pending_patterns(root)? {
            return Ok(());
        }
        // Validate all branches, including ones evaluation will skip. Inference
        // has already selected their operations; this is constant eligibility.
        // Push value continuations first: preparation schedules prerequisite work.
        let substitution = self
            .drop_substitution()
            .map_err(|e| self.error(Query::Node(root), e))?;
        let mut drops = drops::Check::new(
            self.typing.facts,
            root,
            self.drop_scratch.take(),
            substitution,
        );
        self.scan.push(root);
        let mut execution = false;
        while let Some(key) = self.scan.pop() {
            self.budget()?;
            if self.expressions.contains_key(&(self.work.context, key)) {
                continue;
            }
            let expression = self.classify(key)?;
            if matches!(
                expression,
                Expression::Reference(references::Expression::Borrow(_))
            ) {
                self.prepare_alignment(key)?;
            }
            if let Some(check) = &mut drops {
                check.expression(self.report, self.typing.facts, key, expression)?;
            }
            match expression {
                Expression::Cast {
                    kind: Cast::EnumToInteger,
                    operand,
                    ..
                } => {
                    // Formation of an actual direct constructor precedes branch
                    // selection; the cast and skipped arithmetic stay unevaluated.
                    if let Some(variant) = self.typing.facts.direct_variant(self.report, operand) {
                        self.work.push(Work::Enter(Query::Variant(variant)));
                    }
                    self.scan.push(operand);
                }
                Expression::Forward(value)
                | Expression::Unary { operand: value, .. }
                | Expression::Cast { operand: value, .. }
                | Expression::PointerCast { operand: value, .. } => self.scan.push(value),
                Expression::Binary { left, right, .. } => {
                    self.scan.push(right);
                    self.scan.push(left);
                }
                Expression::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.scan.push(otherwise);
                    self.scan.push(then);
                    self.scan.push(condition);
                }
                Expression::Instance(instance) if self.pattern_scope.is_some() => {
                    // A pattern's value must be available before specialization,
                    // including parameter uses hidden in skipped dependencies.
                    // Prepare eligibility without evaluating skipped arithmetic.
                    self.work.push(Work::PrepareInstance(instance));
                }
                Expression::Value(_)
                | Expression::Constant(_)
                | Expression::Instance(_)
                | Expression::ImportedConstant(_)
                | Expression::ForeignStatic(_) => {}
                Expression::Execute => execution = true,
                Expression::Reference(expression) => {
                    expression.dependencies(|child| self.scan.push(child));
                }
                Expression::Product(expression) => {
                    expression.dependencies(self.report, key, |child| self.scan.push(child));
                }
                Expression::Constructor(expression) => {
                    expression.dependencies(self.report, key, |child| self.scan.push(child));
                }
            }
            self.expressions
                .insert((self.work.context, key), expression);
        }
        self.drop_scratch = drops.map(drops::Check::finish);
        self.resume_pattern_dispatch()?;
        self.reserve_pattern_dispatch()?;
        if execution {
            self.check_execution(root)?;
            // Execute the same complete checked root whose region and ownership
            // proof was retained, including its surrounding scalar operations.
            self.expressions
                .insert((self.work.context, root), Expression::Execute);
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "One typed expression dispatch validates constant eligibility and retains its selected dependencies"
    )]
    fn classify(&mut self, key: NodeKey) -> Result<Expression, Error> {
        if let Some(value) = super::primitive_constants::selected(self.typing.facts, key)
            .map_err(|e| self.error(Query::Node(key), e))?
        {
            let value = values::Constant::argument(&self.typing.facts.types, value)
                .map_err(|e| self.error(Query::Node(key), e))?
                .ok_or_else(|| self.error(Query::Node(key), "unresolved primitive constant"))?;
            return Ok(Expression::Value(Value::Scalar(
                value
                    .as_scalar()
                    .map_err(|e| self.error(Query::Node(key), e.message()))?,
            )));
        }
        if self
            .typing
            .facts
            .inline_constants
            .as_deref()
            .is_some_and(|registry| registry.has_children(key))
        {
            return Ok(Expression::Execute);
        }
        let query = Query::Node(key);
        let facts = self
            .typing
            .facts
            .node(key)
            .copied()
            .ok_or_else(|| self.error(query, "missing constant facts"))?;
        let child = |node| NodeKey { node, ..key };
        if let Some(value) = facts.scalar {
            return Ok(Expression::Value(Value::Scalar(value)));
        }
        if matches!(Declarations::kind(self.report, key), Kind::Literal(_)) {
            let id = self
                .typing
                .facts
                .text_literals
                .get(&key)
                .copied()
                .ok_or_else(|| self.error(query, "missing selected literal value"))?;
            let string = facts.ty.and_then(|ty| self.typing.facts.types.kind(ty))
                .is_some_and(|ty| matches!(ty, Type::Reference { referent, .. } if referent == TypeId::primitive(Primitive::Str)));
            return Ok(Expression::Value(if string {
                Value::Text(id)
            } else {
                Value::Bytes(id)
            }));
        }
        if let Some(expression) = self.reference_expression(key)? {
            return Ok(Expression::Reference(expression));
        }
        if let Some(expression) = self.product_expression(key)? {
            return Ok(Expression::Product(expression));
        }
        if let Some(Operation::Construct { variant }) = facts.operation {
            let member = self.typing.facts.members.variants[variant.0 as usize];
            if member.start == member.end
                && !matches!(
                    Declarations::kind(self.report, key),
                    Kind::Struct { base: Some(_), .. }
                )
            {
                return Ok(Expression::Value(Value::Variant(variant)));
            }
        }
        if matches!(
            Declarations::kind(self.report, key),
            Kind::Name(_) | Kind::Path { .. } | Kind::Qualified { .. }
        ) && let Some(ty) = facts.ty
            && let Some(Type::FunctionItem { item, .. }) = self.typing.facts.types.kind(ty)
            && facts.reference == Some(Target::Definition(DefId(item.0)))
        {
            let ty = self.specialized_type(key, ty)?;
            let instance = crate::types::instance::Instance::new(&self.typing.facts.types, ty)
                .map_err(|e| self.error(Query::Node(key), e))?;
            return Ok(Expression::Value(Value::Function(instance)));
        }
        if let Some(expression) = self.constructor_expression(key)? {
            return Ok(Expression::Constructor(expression));
        }
        Ok(
            match (Declarations::kind(self.report, key), facts.operation) {
                (Kind::Group(value) | Kind::Attributed { value, .. }, _)
                | (Kind::FieldInit { value, .. }, Some(Operation::FieldInit { .. })) => {
                    Expression::Forward(child(value))
                }
                (
                    Kind::Block {
                        flavor: BlockFlavor::Plain | BlockFlavor::Unsafe,
                        statements,
                        tail: Some(tail),
                    }
                    | Kind::Expansion {
                        children: statements,
                        tail: Some(tail),
                    },
                    _,
                ) => {
                    if self
                        .report
                        .package
                        .unit(key.file)
                        .configured
                        .ast
                        .list(statements)
                        .iter()
                        .any(|n| self.report.package.unit(key.file).configured.active[n.0 as usize])
                    {
                        return if self.typing.facts.constant_executor.is_some() {
                            Ok(Expression::Execute)
                        } else {
                            Err(self.error(query, "constant statements require the general constant evaluation assignment"))
                        };
                    }
                    Expression::Forward(child(tail))
                }
                (Kind::Path { .. } | Kind::Name(_) | Kind::Qualified { .. }, _) => {
                    let Some(Target::Definition(id)) = facts.reference else {
                        return Err(self.error(query, "missing constant path identity"));
                    };
                    if let Some(&instance) = self.typing.facts.constant_uses.get(&key) {
                        return self
                            .selected_constant(key, instance)
                            .map(Expression::Instance);
                    }
                    if let Some(&id) = self.typing.facts.generics.constants.get(&id) {
                        let crate::types::arguments::Argument::Const(id) = self
                            .specialized_argument(
                                key,
                                crate::types::arguments::Argument::Const(id),
                            )?
                        else {
                            return Err(
                                self.error(query, "constant parameter changed argument kind")
                            );
                        };
                        let value = values::Constant::argument(&self.typing.facts.types, id)
                            .map_err(|e| self.error(query, e))?
                            .ok_or_else(|| {
                                self.error(
                                    query,
                                    "constant value retains an unresolved const parameter",
                                )
                            })?;
                        return Ok(Expression::Value(Value::Scalar(
                            value
                                .as_scalar()
                                .map_err(|e| self.error(query, e.message()))?,
                        )));
                    }
                    if self
                        .typing
                        .facts
                        .crates
                        .as_deref()
                        .is_some_and(|linkage| linkage.imported_constant(id))
                    {
                        return Ok(Expression::ImportedConstant(id));
                    }
                    if self
                        .typing
                        .facts
                        .foreign
                        .as_deref()
                        .is_some_and(|r| r.imports_global(id))
                    {
                        return Ok(Expression::ForeignStatic(id));
                    }
                    let index = self.typing.facts.initializer_index(id).ok_or_else(|| {
                        self.error(query, "constant expression cannot read this declaration")
                    })?;
                    Expression::Constant(index)
                }
                (
                    Kind::Unary { operand, .. },
                    Some(Operation::Unary {
                        operator,
                        operand: ty,
                        literal,
                    }),
                ) => Expression::Unary {
                    op: operator,
                    operand: child(operand),
                    ty: self.primitive(key, ty)?,
                    literal,
                },
                (
                    Kind::Binary { left, right, .. },
                    Some(Operation::Binary {
                        operator, left: ty, ..
                    }),
                ) => Expression::Binary {
                    op: operator,
                    left: child(left),
                    right: child(right),
                    ty: self.primitive(key, ty)?,
                },
                (
                    Kind::Cast { value, .. },
                    Some(Operation::Cast {
                        kind: Cast::FunctionPointer,
                        ..
                    }),
                ) => Expression::Forward(child(value)),
                (Kind::Unary { .. }, Some(Operation::PointerAddress { operand })) => {
                    Expression::Forward(child(operand))
                }
                (
                    Kind::Cast { value, .. },
                    Some(Operation::Cast {
                        kind:
                            kind @ (Cast::Pointer
                            | Cast::PointerThin
                            | Cast::PointerToInteger
                            | Cast::IntegerToPointer),
                        source,
                        ..
                    }),
                ) => Expression::PointerCast {
                    kind,
                    operand: child(value),
                    source,
                },
                (
                    Kind::Cast { value, .. },
                    Some(Operation::Cast {
                        kind,
                        source,
                        target,
                    }),
                ) => Expression::Cast {
                    kind,
                    operand: child(value),
                    source,
                    target: self.primitive(key, target)?,
                },
                (
                    Kind::If {
                        condition,
                        then,
                        otherwise: Some(otherwise),
                    },
                    _,
                ) => Expression::If {
                    condition: child(condition),
                    then: child(then),
                    otherwise: child(otherwise),
                },
                _ => {
                    if self.typing.facts.constant_executor.is_some() {
                        return Ok(Expression::Execute);
                    }
                    return Err(self.error(
                        query,
                        "expression requires the general constant evaluation assignment",
                    ));
                }
            },
        )
    }
}
