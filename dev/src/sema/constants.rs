//! Discriminant queries retain typed scalar facts and solve dependency cycles iteratively.
mod arithmetic;
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

pub(super) struct Discriminant {
    pub variant: VariantId,
    pub value: Option<NodeKey>,
    pub tag: Primitive,
    pub previous: Option<VariantId>,
}
#[derive(Clone, Copy)]
enum Value {
    Scalar(Scalar),
    Variant(VariantId),
}
impl Value {
    fn scalar(self) -> Result<Scalar, &'static str> {
        match self {
            Self::Scalar(value) => Ok(value),
            Self::Variant(_) => Err("constant requires a scalar value"),
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
    Variant(VariantId),
    Constant(usize),
    Node(NodeKey),
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(Value),
}
#[derive(Clone, Copy)]
enum Expression {
    Value(Value),
    Forward(NodeKey),
    Constant(usize),
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
    If {
        condition: NodeKey,
        then: NodeKey,
        otherwise: NodeKey,
    },
}
enum Work {
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
    specs: &'a [Discriminant],
    variants: Vec<Option<usize>>,
    constants: Vec<Option<usize>>,
    expressions: HashMap<NodeKey, Expression>,
    states: HashMap<Query, State>,
    work: Vec<Work>,
    scan: Vec<NodeKey>,
    steps: usize,
    limit: usize,
}

pub(super) fn discriminants(
    report: &Report,
    facts: &mut Declarations,
    specs: &[Discriminant],
) -> Result<(), Error> {
    if specs.is_empty() {
        return Ok(());
    }
    if specs.iter().all(|spec| spec.value.is_none()) {
        let mut previous = 0;
        for spec in specs {
            let member = facts.members.variants[spec.variant.0 as usize];
            let error = |message| Declarations::error(report, member.node, message);
            let value = if spec.previous.is_none() {
                0
            } else {
                arithmetic::successor(spec.tag, previous).ok_or_else(|| {
                    error("implicit enum discriminant overflows its representation")
                })?
            };
            facts
                .aggregates
                .set_discriminant(member.owner, member.ordinal, value)
                .map_err(error)?;
            previous = value;
        }
        return Ok(());
    }
    let mut variants = vec![None; facts.members.variants.len()];
    for (index, spec) in specs.iter().enumerate() {
        variants[spec.variant.0 as usize] = Some(index);
    }
    let mut constants = vec![None; facts.definitions.len()];
    for (index, initializer) in facts.initializers.iter().enumerate() {
        if matches!(
            initializer.kind,
            InitializerKind::Const | InitializerKind::Static { mutable: false }
        ) {
            constants[initializer.definition.0 as usize] = Some(index);
        }
    }
    let limit = report
        .package
        .units
        .iter()
        .fold(specs.len(), |n, unit| {
            n.saturating_add(unit.configured.ast.nodes.len())
                .saturating_add(unit.configured.ast.edges.len())
        })
        .saturating_mul(16);
    let mut queries = Queries {
        report,
        typing: bodies::Checker::new(report, facts),
        specs,
        variants,
        constants,
        expressions: HashMap::new(),
        states: HashMap::new(),
        work: Vec::new(),
        scan: Vec::new(),
        steps: 0,
        limit,
    };
    let mut used = HashSet::new();
    for spec in specs {
        let query = Query::Variant(spec.variant);
        queries.work.push(Work::Enter(query));
        queries.drain()?;
        let value = queries
            .value(query)?
            .integer()
            .map_err(|e| queries.error(query, e))?;
        let member = queries.typing.facts.members.variants[spec.variant.0 as usize];
        if !used.insert((member.owner, value)) {
            return Err(queries.error(query, "duplicate enum discriminant"));
        }
        queries
            .typing
            .facts
            .aggregates
            .set_discriminant(member.owner, member.ordinal, value)
            .map_err(|e| queries.error(query, e))?;
    }
    Ok(())
}
impl Queries<'_> {
    fn error(&self, query: Query, message: impl Into<String>) -> Error {
        let key = match query {
            Query::Variant(id) => self.typing.facts.members.variants[id.0 as usize].node,
            Query::Constant(index) => self.typing.facts.initializers[index].value,
            Query::Node(key) => key,
        };
        Declarations::error(self.report, key, message)
    }
    fn value(&self, query: Query) -> Result<Value, Error> {
        match self.states.get(&query) {
            Some(State::Ready(value)) => Ok(*value),
            _ => Err(self.error(query, "missing completed constant dependency")),
        }
    }
    fn spec(&self, variant: VariantId) -> Result<&Discriminant, Error> {
        self.variants
            .get(variant.0 as usize)
            .copied()
            .flatten()
            .and_then(|i| self.specs.get(i))
            .ok_or_else(|| self.error(Query::Variant(variant), "missing discriminant query"))
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
        while let Some(work) = self.work.pop() {
            self.budget()?;
            match work {
                Work::Enter(query) => self.enter(query)?,
                Work::Forward(query, dependency) => {
                    let value = self.value(dependency)?;
                    self.states.insert(query, State::Ready(value));
                }
                Work::Successor(query, dependency, tag) => {
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
                    self.states
                        .insert(query, State::Ready(Value::Scalar(Scalar::Integer(bits))));
                }
                Work::Finish(key) => self.finish(key)?,
                Work::Select(key) => self.select(key)?,
                Work::EnumCast(key, variant, target) => {
                    let source = self.spec(variant)?.tag;
                    let value = self
                        .value(Query::Variant(variant))?
                        .scalar()
                        .map_err(|e| self.error(Query::Node(key), e))?;
                    let value = arithmetic::cast(Cast::IntegerToInteger, source, target, value)
                        .map_err(|e| self.error(Query::Node(key), e))?;
                    self.states
                        .insert(Query::Node(key), State::Ready(Value::Scalar(value)));
                }
            }
        }
        Ok(())
    }
    fn forward(&mut self, query: Query, dependency: Query) {
        self.work.push(Work::Forward(query, dependency));
        self.work.push(Work::Enter(dependency));
    }
    fn enter(&mut self, query: Query) -> Result<(), Error> {
        match self.states.get(&query) {
            Some(State::Ready(_)) => return Ok(()),
            Some(State::Active) => {
                return Err(
                    self.error(query, "cycle in enum discriminant or constant dependencies")
                );
            }
            None => {}
        }
        self.states.insert(query, State::Active);
        match query {
            Query::Variant(variant) => {
                let spec = self.spec(variant)?;
                if let Some(key) = spec.value {
                    let ty = TypeId::primitive(spec.tag);
                    let owner = DefId(
                        self.typing.facts.members.variants[variant.0 as usize]
                            .owner
                            .0,
                    );
                    self.typing.constant(key, ty, owner)?;
                    self.prepare(key)?;
                    self.forward(query, Query::Node(key));
                } else if let Some(previous) = spec.previous {
                    self.work
                        .push(Work::Successor(query, Query::Variant(previous), spec.tag));
                    self.work.push(Work::Enter(Query::Variant(previous)));
                } else {
                    self.states
                        .insert(query, State::Ready(Value::Scalar(Scalar::Integer(0))));
                }
            }
            Query::Constant(index) => {
                let initializer = self.typing.facts.initializers[index];
                if matches!(initializer.kind, InitializerKind::Static { .. }) {
                    let facts = &mut *self.typing.facts;
                    let properties = facts
                        .copies
                        .demand(
                            &facts.types,
                            &facts.aggregates,
                            &facts.derives,
                            initializer.ty,
                        )
                        .map_err(|e| self.error(query, e))?;
                    if !properties.copy {
                        return Err(self.error(
                            query,
                            "cannot move a non-Copy constant value out of static storage",
                        ));
                    }
                }
                if !self.typing.facts.initializers[index].type_checked {
                    self.typing.initializer(index)?;
                }
                let key = self.typing.facts.initializers[index].value;
                self.prepare(key)?;
                self.forward(query, Query::Node(key));
            }
            Query::Node(key) => match self.expression(key)? {
                Expression::Value(value) => {
                    self.states.insert(query, State::Ready(value));
                }
                Expression::Forward(child) => self.forward(query, Query::Node(child)),
                Expression::Constant(index) => self.forward(query, Query::Constant(index)),
                Expression::Unary { operand, .. } | Expression::Cast { operand, .. } => {
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
            },
        }
        Ok(())
    }
    fn expression(&self, key: NodeKey) -> Result<Expression, Error> {
        self.expressions
            .get(&key)
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
                        query,
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
        let value = match self.expression(key)? {
            Expression::Unary {
                op,
                operand,
                ty,
                literal,
            } => arithmetic::unary(op, ty, self.scalar(operand)?, literal),
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => arithmetic::binary(op, ty, self.scalar(left)?, self.scalar(right)?),
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
        .map_err(|e| self.error(query, e))?;
        self.states
            .insert(query, State::Ready(Value::Scalar(value)));
        Ok(())
    }
    fn prepare(&mut self, root: NodeKey) -> Result<(), Error> {
        // Validate all branches, including ones evaluation will skip. Inference
        // has already selected their operations; this is constant eligibility.
        self.scan.push(root);
        while let Some(key) = self.scan.pop() {
            self.budget()?;
            if self.expressions.contains_key(&key) {
                continue;
            }
            let expression = self.classify(key)?;
            match expression {
                Expression::Forward(value)
                | Expression::Unary { operand: value, .. }
                | Expression::Cast { operand: value, .. } => self.scan.push(value),
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
                Expression::Value(_) | Expression::Constant(_) => {}
            }
            self.expressions.insert(key, expression);
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "One typed expression dispatch validates constant eligibility and retains its selected dependencies"
    )]
    fn classify(&self, key: NodeKey) -> Result<Expression, Error> {
        let query = Query::Node(key);
        let facts = self
            .typing
            .facts
            .node(key)
            .ok_or_else(|| self.error(query, "missing constant facts"))?;
        let child = |node| NodeKey { node, ..key };
        if let Some(value) = facts.scalar {
            return Ok(Expression::Value(Value::Scalar(value)));
        }
        if let Some(Operation::Construct { variant }) = facts.operation {
            let member = self.typing.facts.members.variants[variant.0 as usize];
            if member.start == member.end && self.variants[variant.0 as usize].is_some() {
                return Ok(Expression::Value(Value::Variant(variant)));
            }
        }
        if let Kind::Call { callee, arguments } = Declarations::kind(self.report, key)
            && self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(arguments)
                .is_empty()
            && let Some(Type::FunctionItem { item, .. }) = self
                .typing
                .facts
                .node(child(callee))
                .and_then(|f| f.ty)
                .and_then(|ty| self.typing.facts.types.kind(ty))
            && let Some(variant) = self.typing.facts.members.constructors.get(&DefId(item.0))
            && self.variants[variant.0 as usize].is_some()
            && self
                .typing
                .facts
                .members
                .fields(*variant)
                .is_some_and(<[_]>::is_empty)
        {
            return Ok(Expression::Value(Value::Variant(*variant)));
        }
        Ok(
            match (Declarations::kind(self.report, key), facts.operation) {
                (Kind::Group(value) | Kind::Attributed { value, .. }, _) => {
                    Expression::Forward(child(value))
                }
                (
                    Kind::Block {
                        flavor: BlockFlavor::Plain,
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
                        return Err(self.error(query, "constant statements require the general constant evaluation assignment"));
                    }
                    Expression::Forward(child(tail))
                }
                (Kind::Path { .. } | Kind::Name(_), _) => {
                    let Some(Target::Definition(id)) = facts.reference else {
                        return Err(self.error(query, "missing constant path identity"));
                    };
                    let index = self.constants[id.0 as usize].ok_or_else(|| {
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
                    return Err(self.error(
                        query,
                        "expression requires the general constant evaluation assignment",
                    ));
                }
            },
        )
    }
}
