//! Scalar body constraints consume resolved identities; fallback occurs once per body.
use super::{
    model::{Adjustment, AdjustmentKind, Declarations, Function, Operation},
    scalar,
};
use crate::{
    lex::token::Punctuation as Op,
    literal::{Value, value::FloatType},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::{FileId, NodeKey},
    },
    syntax::ast::{BlockFlavor, Kind, List, NodeId, TypeFlavor, Unary},
    types::{
        Kind as Type, Primitive, TypeId,
        infer::{Constructor, Context, Domain, TermId},
    },
};
use std::collections::HashSet;

/// Check scalar function bodies using the already elaborated item signatures.
///
/// # Errors
/// Reports type mismatches, unresolved variables and explicitly unsupported body semantics.
pub fn analyze(report: &Report, mut facts: Declarations) -> Result<Declarations, Error> {
    let mut checker = Checker {
        report,
        terms: facts
            .nodes
            .iter()
            .map(|nodes| vec![None; nodes.len()])
            .collect(),
        locals: vec![None; facts.definitions.len()],
        facts: &mut facts,
        file: FileId(0),
        result: TypeId::UNIT,
        infer: Context::default(),
        touched: Vec::new(),
        bindings: Vec::new(),
        work: Vec::new(),
        children: Vec::new(),
        checks: Vec::new(),
        negated: HashSet::new(),
        adjustments: Vec::new(),
        calls: Vec::new(),
    };
    for index in 0..checker.facts.functions.len() {
        let function = checker.facts.functions[index];
        checker
            .function(function)
            .map_err(|error| report.diagnostic(error))?;
    }
    Ok(facts)
}
struct Checker<'a> {
    report: &'a Report,
    facts: &'a mut Declarations,
    file: FileId,
    result: TypeId,
    infer: Context,
    terms: Vec<Vec<Option<TermId>>>,
    locals: Vec<Option<TermId>>,
    touched: Vec<NodeId>,
    bindings: Vec<DefId>,
    work: Vec<(NodeId, bool)>,
    children: Vec<NodeId>,
    checks: Vec<NodeId>,
    negated: HashSet<NodeId>,
    adjustments: Vec<(NodeId, TermId, AdjustmentKind)>,
    calls: Vec<(NodeId, TermId)>,
}
impl Checker<'_> {
    fn key(&self, node: NodeId) -> NodeKey {
        NodeKey {
            file: self.file,
            node,
        }
    }
    fn kind(&self, node: NodeId) -> Kind {
        Declarations::kind(self.report, self.key(node))
    }
    fn error(&self, node: NodeId, message: impl std::fmt::Display) -> Error {
        Declarations::error(self.report, self.key(node), message.to_string())
    }
    fn known(&mut self, node: NodeId, ty: TypeId) -> Result<TermId, Error> {
        self.infer
            .known(&self.facts.types, ty)
            .map_err(|e| self.error(node, e))
    }
    fn variable(&mut self, node: NodeId, domain: Domain) -> Result<TermId, Error> {
        self.infer.variable(domain).map_err(|e| self.error(node, e))
    }
    fn term(&self, node: NodeId) -> Result<TermId, Error> {
        self.terms[self.file.0 as usize][node.0 as usize]
            .ok_or_else(|| self.error(node, "missing body type constraint"))
    }
    fn save(&mut self, node: NodeId, term: TermId) {
        if self.terms[self.file.0 as usize][node.0 as usize]
            .replace(term)
            .is_none()
        {
            self.touched.push(node);
        }
    }
    fn equate(&mut self, node: NodeId, left: TermId, right: TermId) -> Result<(), Error> {
        self.infer
            .equate(&self.facts.types, left, right)
            .map_err(|e| self.error(node, e))
    }
    fn head(&mut self, node: NodeId, term: TermId) -> Result<Option<Constructor>, Error> {
        self.infer
            .head(&self.facts.types, term)
            .map_err(|e| self.error(node, e))
    }
    fn child(&mut self, node: NodeId, term: TermId, index: usize) -> Result<TermId, Error> {
        self.infer
            .child(&self.facts.types, term, index)
            .map_err(|e| self.error(node, e))
    }
    fn active(&self, node: NodeId) -> bool {
        self.report.package.unit(self.file).configured.active[node.0 as usize]
    }
    fn function(&mut self, function: Function) -> Result<(), Error> {
        let Some(body) = function.body else {
            return Err(Declarations::error(
                self.report,
                function.node,
                "function body is required",
            ));
        };
        self.file = body.file;
        let Some(Type::Function { result, .. }) = self.facts.types.kind(function.signature) else {
            return Err(self.error(body.node, "missing function signature"));
        };
        self.result = result;
        let unit = self.report.package.unit(self.file);
        for mut node in unit
            .configured
            .ast
            .list(function.parameters)
            .iter()
            .copied()
        {
            if !self.active(node) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(node) {
                node = value;
            }
            let Kind::Parameter {
                pattern,
                ty: Some(ty),
            } = self.kind(node)
            else {
                return Err(self.error(node, "unsupported parameter pattern"));
            };
            let ty = self
                .facts
                .node(self.key(ty))
                .and_then(|f| f.ty)
                .ok_or_else(|| self.error(node, "missing parameter annotation"))?;
            let term = self.known(node, ty)?;
            self.binding(pattern, term)?;
            self.save(node, term);
        }
        self.work.push((body.node, false));
        let mut steps = 0_usize;
        while let Some((node, finish)) = self.work.pop() {
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(node, "body type work budget exceeded"));
            }
            if !self.active(node) {
                continue;
            }
            if finish {
                let term = self.expression(node)?;
                self.save(node, term);
            } else {
                self.enter(node)?;
            }
        }
        let expected = self.known(body.node, self.result)?;
        self.coerce(body.node, expected)?;
        self.finish(body.node)
    }
    fn finish(&mut self, body: NodeId) -> Result<(), Error> {
        for (_, term, kind) in &self.adjustments {
            if *kind == AdjustmentKind::Never {
                self.infer
                    .default_never(*term)
                    .map_err(|e| self.error(body, e))?;
            }
        }
        let solved = std::mem::take(&mut self.infer)
            .finish(&mut self.facts.types)
            .map_err(|e| self.error(body, e))?;
        // Region obligations must remain attached when reference body operations are added.
        if !solved.region_equalities.is_empty() {
            return Err(self.error(
                body,
                "reference body constraints require lifetime contracts",
            ));
        }
        for node in &self.touched {
            let term = self.term(*node)?;
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid body node"))?
                .ty = Some(solved.types[term.0 as usize]);
        }
        for id in &self.bindings {
            if let Some(term) = self.locals[id.0 as usize].take() {
                self.facts.definitions[id.0 as usize] = Some(solved.types[term.0 as usize]);
            }
        }
        for (node, term, kind) in &self.adjustments {
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid adjustment node"))?
                .adjustment = Some(Adjustment {
                kind: *kind,
                target: solved.types[term.0 as usize],
            });
        }
        for (node, term) in &self.calls {
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid call node"))?
                .operation = Some(Operation::Call {
                signature: solved.types[term.0 as usize],
            });
        }
        self.validate_operations()?;
        self.validate_literals()?;
        for node in self.touched.drain(..) {
            self.terms[self.file.0 as usize][node.0 as usize] = None;
        }
        self.bindings.clear();
        self.checks.clear();
        self.negated.clear();
        self.adjustments.clear();
        self.calls.clear();
        Ok(())
    }
    fn binding(&mut self, mut node: NodeId, term: TermId) -> Result<(), Error> {
        while let Kind::Attributed { value, .. } | Kind::Group(value) = self.kind(node) {
            node = value;
        }
        match self.kind(node) {
            Kind::Binding {
                by_ref: false,
                subpattern: None,
                ..
            } => {
                let id = self
                    .facts
                    .node(self.key(node))
                    .and_then(|f| f.binding)
                    .ok_or_else(|| self.error(node, "missing resolved binding"))?;
                self.locals[id.0 as usize] = Some(term);
                self.bindings.push(id);
            }
            Kind::Wildcard => {}
            _ => return Err(self.error(node, "compound patterns require pattern semantics")),
        }
        self.save(node, term);
        Ok(())
    }
    fn annotation(&mut self, mut node: NodeId) -> Result<TermId, Error> {
        let origin = node;
        loop {
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } => node = value,
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => {
                    node = *self
                        .report
                        .package
                        .unit(self.file)
                        .configured
                        .ast
                        .list(elements)
                        .first()
                        .ok_or_else(|| self.error(node, "missing annotation path"))?;
                }
                _ => break,
            }
        }
        let term = match self.kind(node) {
            Kind::InferType => self.variable(node, Domain::Any)?,
            Kind::Path { .. } | Kind::Name(_) => {
                let Some(Target::Definition(id)) =
                    self.facts.node(self.key(node)).and_then(|f| f.reference)
                else {
                    return Err(self.error(node, "missing annotation resolution"));
                };
                let ty = self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(node, "unsupported annotation declaration"))?;
                self.known(node, ty)?
            }
            Kind::Type {
                flavor: TypeFlavor::Tuple,
                elements,
            } if elements.start == elements.end => self.known(node, TypeId::UNIT)?,
            _ => {
                return Err(
                    self.error(node, "structural local annotations are not implemented yet")
                );
            }
        };
        self.save(origin, term);
        Ok(term)
    }
    fn enter(&mut self, node: NodeId) -> Result<(), Error> {
        self.work.push((node, true));
        self.children.clear();
        match self.kind(node) {
            Kind::Path { .. } | Kind::Name(_) | Kind::Item(_) => {}
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise: None,
            } => {
                let term = if let Some(ty) = ty {
                    self.annotation(ty)?
                } else {
                    self.variable(node, Domain::Any)?
                };
                self.binding(pattern, term)?;
                self.children.extend(value);
            }
            Kind::Let {
                otherwise: Some(_), ..
            } => return Err(self.error(node, "let-else requires pattern semantics")),
            Kind::Cast { value, ty } => {
                self.annotation(ty)?;
                self.children.push(value);
            }
            _ => self
                .report
                .package
                .unit(self.file)
                .configured
                .ast
                .push_children(node, &mut self.children),
        }
        self.work
            .extend(self.children.iter().rev().map(|n| (*n, false)));
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "typed expression constraints keep operand and result relationships in one dispatch"
    )]
    fn expression(&mut self, node: NodeId) -> Result<TermId, Error> {
        match self.kind(node) {
            Kind::Literal(index) => self.literal(node, index),
            Kind::Path { .. } | Kind::Name(_) => {
                let Some(Target::Definition(id)) =
                    self.facts.node(self.key(node)).and_then(|f| f.reference)
                else {
                    return Err(self.error(node, "missing value resolution"));
                };
                if let Some(term) = self.locals[id.0 as usize] {
                    return Ok(term);
                }
                let definition = &self.report.resolution.definitions[id.0 as usize];
                if !matches!(
                    definition.kind,
                    DefKind::Function | DefKind::Const | DefKind::Static
                ) {
                    return Err(self.error(node, "unsupported value declaration"));
                }
                let ty = self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(node, "missing value type"))?;
                self.known(node, ty)
            }
            Kind::Group(value) | Kind::Attributed { value, .. } => self.term(value),
            Kind::Block {
                flavor: BlockFlavor::Plain,
                statements,
                tail,
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                if let Some(tail) = tail {
                    return self.term(tail);
                }
                let unit = self.report.package.unit(self.file);
                let mut diverges = false;
                for statement in unit.configured.ast.list(statements) {
                    if self.active(*statement) {
                        let term = self.term(*statement)?;
                        diverges |= self.head(*statement, term)? == Some(Constructor::Never);
                    }
                }
                self.known(
                    node,
                    if diverges {
                        TypeId::NEVER
                    } else {
                        TypeId::UNIT
                    },
                )
            }
            Kind::Tuple(elements) if elements.start == elements.end => {
                self.known(node, TypeId::UNIT)
            }
            Kind::EmptyStatement | Kind::Item(_) => self.known(node, TypeId::UNIT),
            Kind::Statement(value) => {
                let term = self.term(value)?;
                let never = self.head(node, term)? == Some(Constructor::Never);
                self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
            }
            Kind::Let { pattern, value, .. } => {
                if let Some(value) = value {
                    self.coerce(value, self.term(pattern)?)?;
                }
                self.known(node, TypeId::UNIT)
            }
            Kind::Unary {
                op: Unary::Negate | Unary::Not,
                operand,
            } => {
                if matches!(
                    self.kind(node),
                    Kind::Unary {
                        op: Unary::Negate,
                        ..
                    }
                ) && let Some(literal) = self.literal_operand(operand)
                {
                    self.negated.insert(literal);
                }
                self.checks.push(node);
                self.term(operand)
            }
            Kind::Binary { op, left, right } => self.binary(node, op, left, right),
            Kind::Call { callee, arguments } => self.call(node, callee, arguments),
            Kind::Return(value) => {
                let expected = self.known(node, self.result)?;
                if let Some(value) = value {
                    self.coerce(value, expected)?;
                } else {
                    let unit = self.known(node, TypeId::UNIT)?;
                    self.equate(node, unit, expected)?;
                }
                self.known(node, TypeId::NEVER)
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
                self.coerce(condition, boolean)?;
                let result = if let Some(otherwise) = otherwise {
                    let a = self.term(then)?;
                    let b = self.term(otherwise)?;
                    if self.head(node, a)? == Some(Constructor::Never) {
                        self.coerce(then, b)?;
                        b
                    } else {
                        self.coerce(otherwise, a)?;
                        a
                    }
                } else {
                    let unit = self.known(node, TypeId::UNIT)?;
                    self.coerce(then, unit)?;
                    unit
                };
                Ok(result)
            }
            Kind::Cast { ty, .. } => {
                self.checks.push(node);
                self.term(ty)
            }
            _ => Err(self.error(
                node,
                "body construct requires later semantic implementation",
            )),
        }
    }
    fn literal(&mut self, node: NodeId, index: u32) -> Result<TermId, Error> {
        let value = self
            .report
            .package
            .unit(self.file)
            .literal(index)
            .map_err(|e| self.error(node, e.message))?;
        let primitive = match value {
            Value::Integer(value) => {
                if let Some(suffix) = value.suffix {
                    Primitive::ALL
                        .into_iter()
                        .find(|p| p.name() == suffix.name())
                        .ok_or_else(|| self.error(node, "invalid integer suffix"))?
                } else {
                    return self.variable(node, Domain::Integer);
                }
            }
            Value::Float(value) => match value.suffix {
                Some(FloatType::F32) => Primitive::F32,
                Some(FloatType::F64) => Primitive::F64,
                None => return self.variable(node, Domain::Float),
            },
            Value::Boolean(_) => Primitive::Bool,
            Value::Character(_) => Primitive::Char,
            Value::Byte(_) => Primitive::U8,
            _ => return Err(self.error(node, "text literal body types are not implemented yet")),
        };
        self.known(node, TypeId::primitive(primitive))
    }
    fn coerce(&mut self, source: NodeId, expected: TermId) -> Result<(), Error> {
        let actual = self.term(source)?;
        let actual_head = self.head(source, actual)?;
        let expected_head = self.head(source, expected)?;
        if actual_head == Some(Constructor::Never) {
            self.adjustments
                .push((source, expected, AdjustmentKind::Never));
            return Ok(());
        }
        if matches!(actual_head, Some(Constructor::FunctionItem(_)))
            && matches!(expected_head, Some(Constructor::Function { .. }))
        {
            let signature = self.child(source, actual, 0)?;
            self.equate(source, signature, expected)?;
            self.adjustments
                .push((source, expected, AdjustmentKind::FunctionPointer));
            return Ok(());
        }
        self.equate(source, actual, expected)
    }
    fn binary(
        &mut self,
        node: NodeId,
        op: Op,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let a = self.term(left)?;
        let b = self.term(right)?;
        self.checks.push(node);
        match op {
            Op::AndAnd | Op::OrOr => {
                let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
                self.coerce(left, boolean)?;
                self.coerce(right, boolean)?;
                Ok(boolean)
            }
            Op::ShiftLeft | Op::ShiftRight => {
                let integer = self.variable(node, Domain::Integer)?;
                self.equate(node, b, integer)?;
                Ok(a)
            }
            Op::EqualEqual
            | Op::BangEqual
            | Op::Less
            | Op::Greater
            | Op::LessEqual
            | Op::GreaterEqual => {
                self.equate(node, a, b)?;
                self.known(node, TypeId::primitive(Primitive::Bool))
            }
            Op::Plus
            | Op::Minus
            | Op::Star
            | Op::Slash
            | Op::Percent
            | Op::And
            | Op::Or
            | Op::Caret => {
                self.equate(node, a, b)?;
                Ok(a)
            }
            _ => Err(self.error(
                node,
                "assignment and operator semantics are not implemented yet",
            )),
        }
    }
    fn call(&mut self, node: NodeId, callee: NodeId, arguments: List) -> Result<TermId, Error> {
        let mut term = self.term(callee)?;
        if matches!(self.head(node, term)?, Some(Constructor::FunctionItem(_))) {
            term = self.child(node, term, 0)?;
        }
        let Some(Constructor::Function {
            parameters,
            unsafe_,
            bound_regions,
        }) = self.head(node, term)?
        else {
            return Err(self.error(node, "call requires a known function type"));
        };
        if unsafe_ || bound_regions != 0 {
            return Err(self.error(
                node,
                "unsafe or lifetime-polymorphic calls require later implementation",
            ));
        }
        let unit = self.report.package.unit(self.file);
        let mut count = 0_usize;
        for argument in unit.configured.ast.list(arguments) {
            if !self.active(*argument) {
                continue;
            }
            if count >= parameters as usize {
                return Err(self.error(node, "too many call arguments"));
            }
            let expected = self.child(node, term, count)?;
            self.coerce(*argument, expected)?;
            count += 1;
        }
        if count != parameters as usize {
            return Err(self.error(node, "too few call arguments"));
        }
        self.calls.push((node, term));
        self.child(node, term, count)
    }
    fn ty(&self, node: NodeId) -> Result<TypeId, Error> {
        self.facts
            .node(self.key(node))
            .and_then(|f| f.adjustment.map(|a| a.target).or(f.ty))
            .ok_or_else(|| self.error(node, "missing solved expression type"))
    }
    fn primitive(&self, node: NodeId) -> Result<Primitive, Error> {
        if let Some(Type::Primitive(p)) = self.facts.types.kind(self.ty(node)?) {
            Ok(p)
        } else {
            Err(self.error(node, "operation requires a scalar type"))
        }
    }
    fn literal_operand(&self, mut node: NodeId) -> Option<NodeId> {
        loop {
            match self.kind(node) {
                Kind::Group(value) => node = value,
                Kind::Literal(_) => return Some(node),
                _ => return None,
            }
        }
    }
    fn validate_operations(&mut self) -> Result<(), Error> {
        for node in &self.checks {
            let operation = match self.kind(*node) {
                Kind::Unary { op, operand } => {
                    if !scalar::unary(op, self.primitive(operand)?) {
                        return Err(
                            self.error(*node, "unary operator is not defined for this scalar type")
                        );
                    }
                    Operation::Unary {
                        operator: op,
                        operand: self.ty(operand)?,
                        literal: self.literal_operand(operand).is_some(),
                    }
                }
                Kind::Binary { op, left, right } => {
                    if !scalar::binary(op, self.primitive(left)?, self.primitive(right)?) {
                        return Err(self.error(
                            *node,
                            "binary operator is not defined for these scalar types",
                        ));
                    }
                    Operation::Binary {
                        operator: op,
                        left: self.ty(left)?,
                        right: self.ty(right)?,
                    }
                }
                Kind::Cast { value, ty } => {
                    let kind = scalar::cast(self.primitive(value)?, self.primitive(ty)?)
                        .ok_or_else(|| self.error(*node, "invalid scalar cast"))?;
                    Operation::Cast {
                        kind,
                        source: self.ty(value)?,
                        target: self.ty(ty)?,
                    }
                }
                _ => return Err(self.error(*node, "invalid scalar operation obligation")),
            };
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid operation node"))?
                .operation = Some(operation);
        }
        Ok(())
    }
    fn validate_literals(&mut self) -> Result<(), Error> {
        // Keep only negated literal IDs; no per-function copy of the file's AST.
        for node in &self.touched {
            let Kind::Literal(index) = self.kind(*node) else {
                continue;
            };
            let unit = self.report.package.unit(self.file);
            let value = unit
                .literal(index)
                .map_err(|e| self.error(*node, e.message))?;
            let negated = self.negated.contains(node);
            let scalar = scalar::literal(&value, self.primitive(*node)?, negated)
                .map_err(|e| self.error(*node, e))?;
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid literal node"))?
                .scalar = Some(scalar);
        }
        Ok(())
    }
}
