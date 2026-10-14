//! Scalar body constraints consume resolved identities; fallback occurs once per body.
mod aggregates;
mod alignment;
mod bounds;
mod calls;
mod conditions;
mod divergence;
mod generics;
mod lifetimes;
mod matches;
mod nominals;
mod pattern_constants;
mod patterns;
mod references;
mod text;
use super::{
    declarations,
    model::{AdjustmentKind, Declarations, Function, Operation, RegionEquality},
    scalar,
};
use crate::{
    lex::token::Punctuation as Op,
    literal::{Value, value::FloatType},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::{Error, ModuleId},
        unit::{FileId, NodeKey},
    },
    syntax::ast::{BlockFlavor, Kind, List, NodeId, TypeFlavor, Unary},
    types::{
        Kind as Type, Primitive, TypeId,
        infer::{Constructor, Context, Domain, TermId},
    },
};
use std::collections::HashSet;

/// Check function bodies and remaining initializers using completed declaration facts.
///
/// # Errors
/// Reports type mismatches, unresolved variables and explicitly unsupported body semantics.
pub fn analyze(report: &Report, mut facts: Declarations) -> Result<Declarations, Error> {
    let mut checker = Checker::new(report, &mut facts);
    for index in 0..checker.facts.functions.len() {
        let function = checker.facts.functions[index];
        checker
            .function(function)
            .map_err(|error| report.diagnostic(error))?;
    }
    for index in 0..checker.facts.initializers.len() {
        if !checker.facts.initializers[index].type_checked {
            checker
                .initializer(index)
                .map_err(|e| report.diagnostic(e))?;
        }
    }
    checker
        .complete_constant_patterns()
        .map_err(|e| report.diagnostic(e))?;
    Ok(facts)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    Loop,
    While,
    Block,
}
#[derive(Clone, Copy)]
struct Frame {
    node: NodeId,
    body: NodeId,
    kind: FrameKind,
    label: Option<DefId>,
    result: TermId,
    breaks: Option<usize>,
    expected: Option<TermId>,
    in_condition: bool,
}
struct BreakSite {
    node: NodeId,
    value: Option<NodeId>,
    next: Option<usize>,
}
struct PendingAdjustment {
    node: NodeId,
    actual: TermId,
    expected: TermId,
    kind: AdjustmentKind,
    reset: bool,
}
enum Work {
    Conditional {
        node: NodeId,
        pattern: NodeId,
        value: NodeId,
        expected: Option<TermId>,
    },
    MatchArms {
        node: NodeId,
        expected: Option<TermId>,
    },
    Arm {
        index: u32,
        term: TermId,
        expected: Option<TermId>,
    },
    BindPattern {
        pattern: NodeId,
        term: TermId,
        value: Option<NodeId>,
    },
    Enter(NodeId, Option<TermId>),
    Exit(NodeId),
    CallArguments(NodeId, Option<TermId>),
    AssignmentValue(NodeId),
}
/// Dense, cleared slots can be reused between short declaration-time constants.
/// Successful body checking clears only touched nodes and bindings.
#[derive(Default)]
pub(super) struct Scratch {
    terms: Vec<Vec<Option<TermId>>>,
    diverges: Vec<bool>,
    locals: Vec<Option<TermId>>,
}
pub(super) struct Checker<'a> {
    alignment: alignment::Pending,
    report: &'a Report,
    pub(super) facts: &'a mut Declarations,
    file: FileId,
    body: NodeKey,
    module: ModuleId,
    field_seen: Vec<bool>,
    result: TypeId,
    in_function: bool,
    infer: Context,
    terms: Vec<Vec<Option<TermId>>>,
    diverges: Vec<bool>,
    locals: Vec<Option<TermId>>,
    touched: Vec<NodeId>,
    bindings: Vec<DefId>,
    pattern_work: Vec<patterns::Work>,
    pattern_types: Vec<(super::patterns::PatternId, TermId)>,
    pattern_refutable: bool,
    pending_patterns: Option<Box<pattern_constants::Pending>>,
    pattern_constants: bool,
    coverage_cache: super::patterns::coverage::Cache,
    range_work: Vec<matches::RangeWork>,
    pattern_requirements: Vec<(super::patterns::PatternId, bool)>,
    matches: Vec<u32>,
    work: Vec<Work>,
    children: Vec<NodeId>,
    checks: Vec<NodeId>,
    negated: HashSet<NodeId>,
    adjustments: Vec<PendingAdjustment>,
    reference_adjustments:
        std::collections::HashMap<usize, (super::model::ReferenceConversion, TermId)>,
    reference_expected: HashSet<NodeId>,
    reference_work: Vec<references::CoercionWork>,
    reference_steps: usize,
    break_roots: std::collections::HashMap<NodeId, usize>,
    discarded_adjustments: std::collections::HashMap<usize, (TermId, TermId)>,
    calls: Vec<(NodeId, TermId)>,
    pending_indexes: std::collections::HashMap<NodeId, TermId>,
    call_types: calls::CallTypes,
    parameter_copies: Option<crate::types::copy::Copies>,
    parameter_shapes: Option<crate::types::parameters::shape::Shapes>,
    body_binders: u32,
    annotation_work: Vec<(NodeId, bool)>,
    annotation_checks: Vec<NodeId>,
    annotation_children: HashSet<NodeId>,
    type_children: Vec<TermId>,
    frames: Vec<Frame>,
    pending_label: Option<(NodeId, DefId)>,
    assignments: Vec<(NodeId, Option<DefId>, Option<Op>)>,
    breaks: Vec<BreakSite>,
}
impl<'a> Checker<'a> {
    pub(super) fn new(report: &'a Report, facts: &'a mut Declarations) -> Self {
        Self::with_scratch(report, facts, Scratch::default())
    }
    pub(super) fn with_scratch(
        report: &'a Report,
        facts: &'a mut Declarations,
        mut scratch: Scratch,
    ) -> Self {
        scratch.terms.resize_with(facts.nodes.len(), Vec::new);
        Self {
            alignment: alignment::Pending::default(),
            report,
            terms: scratch.terms,
            diverges: scratch.diverges,
            locals: scratch.locals,
            facts,
            file: FileId(0),
            body: NodeKey {
                file: FileId(0),
                node: NodeId(0),
            },
            module: ModuleId(0),
            field_seen: Vec::new(),
            result: TypeId::UNIT,
            in_function: true,
            infer: Context::default(),
            touched: Vec::new(),
            bindings: Vec::new(),
            pattern_work: Vec::new(),
            pattern_types: Vec::new(),
            pattern_refutable: false,
            pending_patterns: None,
            pattern_constants: false,
            coverage_cache: super::patterns::coverage::Cache::default(),
            range_work: Vec::new(),
            pattern_requirements: Vec::new(),
            matches: Vec::new(),
            work: Vec::new(),
            children: Vec::new(),
            checks: Vec::new(),
            negated: HashSet::new(),
            adjustments: Vec::new(),
            reference_adjustments: std::collections::HashMap::new(),
            reference_expected: HashSet::new(),
            reference_work: Vec::new(),
            reference_steps: 0,
            break_roots: std::collections::HashMap::new(),
            discarded_adjustments: std::collections::HashMap::new(),
            calls: Vec::new(),
            pending_indexes: std::collections::HashMap::new(),
            call_types: calls::CallTypes::default(),
            parameter_copies: None,
            parameter_shapes: None,
            body_binders: 0,
            annotation_work: Vec::new(),
            annotation_checks: Vec::new(),
            annotation_children: HashSet::new(),
            type_children: Vec::new(),
            frames: Vec::new(),
            pending_label: None,
            assignments: Vec::new(),
            breaks: Vec::new(),
        }
    }
    pub(super) fn has_pending_patterns(&self) -> bool {
        self.pending_patterns.is_some()
    }
    /// Consume only after successful checking; failed checks discard their slots.
    pub(super) fn into_scratch(self) -> Scratch {
        Scratch {
            terms: self.terms,
            diverges: self.diverges,
            locals: self.locals,
        }
    }
    pub(super) fn constant(&mut self, key: NodeKey, ty: TypeId, owner: DefId) -> Result<(), Error> {
        let scope = self.report.resolution.definitions[owner.0 as usize].scope;
        self.constant_in_module(
            key,
            ty,
            self.report.resolution.scopes[scope.0 as usize].module,
        )
    }
    pub(super) fn constant_in_module(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: ModuleId,
    ) -> Result<(), Error> {
        self.call_types.clear();
        self.body_binders = 0;
        self.parameter_copies = None;
        self.parameter_shapes = None;
        self.body = key;
        self.prepare_file(key.file);
        self.module = module;
        self.result = ty;
        self.in_function = false;
        self.body(key.node)
    }
    pub(super) fn anonymous_constant(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: ModuleId,
    ) -> Result<(), Error> {
        if let Some(&prior) = self.facts.anonymous_constants.get(&key) {
            return if prior == ty {
                Ok(())
            } else {
                Err(self.error(key.node, "anonymous constant changed its result type"))
            };
        }
        let before = self.facts.lifetime_fact_count();
        self.constant_in_module(key, ty, module)?;
        let annotations = self.facts.constant_annotation_bodies.remove(&key);
        if self.facts.lifetime_fact_count() != before || annotations {
            self.facts.anonymous_constants.insert(key, ty);
        }
        Ok(())
    }
    pub(super) fn initializer(&mut self, index: usize) -> Result<(), Error> {
        let initializer = self.facts.initializers[index];
        self.constant(initializer.value, initializer.ty, initializer.definition)?;
        self.facts.initializers[index].type_checked = true;
        Ok(())
    }
}
impl Checker<'_> {
    fn prepare_file(&mut self, file: FileId) {
        self.locals.resize(self.facts.definitions.len(), None);
        self.file = file;
        self.terms[file.0 as usize].resize(self.facts.nodes[file.0 as usize].len(), None);
        self.diverges
            .resize(self.facts.nodes[file.0 as usize].len(), false);
    }
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
    fn shape_equation(&mut self, node: NodeId, left: TermId, right: TermId) -> Result<(), Error> {
        // Generic holes also occur beneath callable binders. Once an argument
        // solves such a hole, caller universals must remain free there rather
        // than becoming lifetimes owned by the nested callable.
        let (left, right) = if self.body_binders != 0 && !self.call_types.generic.is_empty() {
            (
                self.alias_argument(node, left)?,
                self.alias_argument(node, right)?,
            )
        } else {
            (left, right)
        };
        self.infer
            .equate_shape(&self.facts.types, left, right)
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
        self.call_types.clear();
        self.body_binders = self
            .facts
            .lifetimes
            .functions
            .get(&function.definition)
            .map_or(0, |f| f.total);
        self.in_function = true;
        let Some(body) = function.body else {
            return Err(Declarations::error(
                self.report,
                function.node,
                "function body is required",
            ));
        };
        self.prepare_file(body.file);
        self.body = body;
        self.prepare_bound_environment(body.node, function.definition)?;
        let scope = self.report.resolution.definitions[function.definition.0 as usize].scope;
        self.module = self.report.resolution.scopes[scope.0 as usize].module;
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
            if let Some(receiver) = self
                .facts
                .destructors
                .receivers
                .get(&self.key(node))
                .copied()
            {
                let term = self.known(node, receiver.ty)?;
                self.locals[receiver.definition.0 as usize] = Some(term);
                self.bindings.push(receiver.definition);
                self.save(receiver.binding.node, term);
                self.save(node, term);
                continue;
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
            self.binding(pattern, term, None)?;
            self.save(node, term);
        }
        self.body(body.node)
    }
    fn body(&mut self, body: NodeId) -> Result<(), Error> {
        let expected = self.known(body, self.result)?;
        self.work.push(Work::Enter(body, Some(expected)));
        let mut steps = 0_usize;
        while let Some(work) = self.work.pop() {
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(body, "body type work budget exceeded"));
            }
            match work {
                Work::Conditional {
                    node,
                    pattern,
                    value,
                    expected,
                } => self.conditional_pattern(node, pattern, value, expected)?,
                Work::MatchArms { node, expected } => self.match_arms(node, expected)?,
                Work::Arm {
                    index,
                    term,
                    expected,
                } => self.match_arm(index, term, expected)?,
                Work::BindPattern {
                    pattern,
                    term,
                    value,
                } => {
                    if let Some(value) = value {
                        self.coerce(value, term)?;
                    }
                    self.binding(pattern, term, value)?;
                }
                Work::Enter(node, expected) if self.active(node) => self.enter(node, expected)?,
                Work::Enter(..) => {}
                Work::CallArguments(node, expected) => self.call_arguments(node, expected)?,
                Work::AssignmentValue(node) => {
                    let Kind::Binary { left, right, .. } = self.kind(node) else {
                        return Err(self.error(node, "invalid assignment work"));
                    };
                    let expected = self.term(left)?;
                    self.work.push(Work::Enter(right, Some(expected)));
                }
                Work::Exit(node) => {
                    let term = self.expression(node)?;
                    self.save(node, term);
                    let mut local_exit = false;
                    if self
                        .frames
                        .last()
                        .is_some_and(|f| f.node == node && f.kind == FrameKind::Block)
                    {
                        let frame = self
                            .frames
                            .pop()
                            .ok_or_else(|| self.error(node, "missing block control frame"))?;
                        local_exit = frame.breaks.is_some();
                        let result = self.finish_frame(frame, Some(term))?;
                        self.save(node, result);
                    }
                    self.finish_divergence(node, local_exit)?;
                }
            }
        }
        let expected = self.known(body, self.result)?;
        self.coerce(body, expected)?;
        self.finish(body)
    }
    fn finish(&mut self, body: NodeId) -> Result<(), Error> {
        for adjustment in &self.adjustments {
            if adjustment.kind == AdjustmentKind::Never {
                self.infer
                    .default_never(adjustment.expected)
                    .map_err(|e| self.error(body, e))?;
            }
        }
        let solved = std::mem::take(&mut self.infer)
            .finish(&mut self.facts.types)
            .map_err(|e| self.error(body, e))?;
        self.validate_generic_uses(&solved)?;
        let body_key = self.key(body);
        self.publish_pattern_types(&solved)?;
        self.facts
            .region_equalities
            .extend(
                solved
                    .region_equalities
                    .iter()
                    .map(|(left, right)| RegionEquality {
                        body: body_key,
                        left: *left,
                        right: *right,
                    }),
            );
        self.publish_body_types(body, &solved)?;
        self.publish_index_bounds(&solved)?;
        self.publish_adjustments(&solved, body_key)?;
        for (node, term) in &self.calls {
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid call node"))?
                .operation = Some(Operation::Call {
                signature: solved.types[term.0 as usize],
            });
        }
        for node in &self.annotation_checks {
            crate::types::bound_uses::check_pointer(&self.facts.types, self.ty(*node)?)
                .map_err(|e| self.error(*node, e))?;
        }
        self.annotation_checks.clear();
        self.annotation_children.clear();
        self.validate_operations()?;
        self.validate_assignments()?;
        self.validate_values()?;
        self.publish_reference_alignment()?;
        self.finish_patterns()?;
        for node in self.touched.drain(..) {
            self.terms[self.file.0 as usize][node.0 as usize] = None;
            self.diverges[node.0 as usize] = false;
        }
        self.bindings.clear();
        self.checks.clear();
        self.negated.clear();
        self.adjustments.clear();
        self.reference_adjustments.clear();
        self.reference_expected.clear();
        self.reference_work.clear();
        self.reference_steps = 0;
        self.break_roots.clear();
        self.discarded_adjustments.clear();
        self.calls.clear();
        self.assignments.clear();
        self.breaks.clear();
        Ok(())
    }
    fn publish_body_types(
        &mut self,
        body: NodeId,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        let body_key = self.key(body);
        let mut well_formed = HashSet::new();
        let mut nominal_types = HashSet::new();
        // An implicit dereference need not have an AST node among `touched`.
        // Its selected field receiver still supplies a concrete type query key.
        // Retain discovery order so appended canonical types are deterministic.
        for &receiver in &self.call_types.nominal_order {
            let ty = solved.types[receiver.0 as usize];
            self.facts
                .aggregates
                .prepare(&mut self.facts.types, ty)
                .map_err(|e| self.error(body, e))?;
        }
        for index in 0..self.touched.len() {
            let node = self.touched[index];
            let term = self.term(node)?;
            let key = self.key(node);
            let ty = solved.types[term.0 as usize];
            self.nominal_requirements(node, ty, &mut nominal_types)?;
            if self.facts.generics.has_nominal_requirements {
                self.facts
                    .aggregates
                    .prepare(&mut self.facts.types, ty)
                    .map_err(|e| self.error(node, e))?;
            }
            if !self.body_shape(body, ty)?.well_formed {
                return Err(self.error(node, "body uses an ill-formed type"));
            }
            if (self.facts.types.contains_regions(ty) == Some(true)
                || self.facts.types.contains_parameters(ty) == Some(true))
                && !self.annotation_children.contains(&node)
                && well_formed.insert(ty)
            {
                self.facts.well_formed.push(super::model::WellFormed {
                    body: body_key,
                    source: key,
                    ty,
                });
            }
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid body node"))?
                .ty = Some(ty);
        }
        for index in 0..self.bindings.len() {
            let id = self.bindings[index];
            if let Some(term) = self.locals[id.0 as usize].take() {
                let ty = solved.types[term.0 as usize];
                if !self.body_shape(body, ty)?.sized {
                    return Err(self.error(body, "local bindings must be sized"));
                }
                self.facts.definitions[id.0 as usize] = Some(ty);
            }
        }
        Ok(())
    }
    fn binding(
        &mut self,
        mut node: NodeId,
        term: TermId,
        input: Option<NodeId>,
    ) -> Result<(), Error> {
        if !self.simple_pattern(node) {
            let valid = input
                .map(|input| self.pattern_input_validity(input))
                .transpose()?
                .unwrap_or(true);
            self.pattern(node, term, valid)?;
            let root = self.facts.patterns.roots[&self.key(node)];
            self.alignment.patterns.push((root, input));
            return Ok(());
        }
        while let Kind::Attributed { value, .. } | Kind::Group(value) = self.kind(node) {
            self.save(node, term);
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
    fn annotation(&mut self, root: NodeId) -> Result<TermId, Error> {
        let first = self.touched.len();
        self.annotation_work.push((root, false));
        while let Some((node, finish)) = self.annotation_work.pop() {
            if self.terms[self.file.0 as usize][node.0 as usize].is_some() {
                continue;
            }
            if let Some(ty) = self.facts.node(self.key(node)).and_then(|f| f.ty) {
                let term = self.known(node, ty)?;
                self.save(node, term);
                continue;
            }
            if finish {
                let term = self.annotation_finish(node)?;
                self.save(node, term);
            } else {
                self.annotation_work.push((node, true));
                let unit = self.report.package.unit(self.file);
                match self.kind(node) {
                    Kind::Group(value) | Kind::Attributed { value, .. } => {
                        self.annotation_work.push((value, false));
                    }
                    Kind::Type { flavor, elements } => {
                        if matches!(flavor, TypeFlavor::Ref | TypeFlavor::RefMut) {
                            let referent = unit
                                .configured
                                .ast
                                .list(elements)
                                .last()
                                .copied()
                                .ok_or_else(|| {
                                    self.error(node, "missing reference annotation referent")
                                })?;
                            self.annotation_work.push((referent, false));
                            continue;
                        }
                        let elements = unit.configured.ast.list(elements);
                        let count = elements
                            .len()
                            .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                        self.annotation_work
                            .extend(elements[..count].iter().rev().map(|n| (*n, false)));
                    }
                    Kind::FunctionType(index) => {
                        let function = unit.configured.ast.function_types[index as usize];
                        self.annotation_work
                            .extend(function.result.map(|n| (n, false)));
                        for parameter in unit.configured.ast.list(function.parameters).iter().rev()
                        {
                            if self.active(*parameter) {
                                let ty = declarations::parameter_type(
                                    self.report,
                                    self.key(*parameter),
                                )?;
                                self.annotation_work.push((ty, false));
                            }
                        }
                    }
                    Kind::Path { .. } | Kind::Name(_) | Kind::Applied { .. } => {
                        self.annotation_arguments(node);
                    }
                    Kind::InferType | Kind::NeverType => {}
                    _ => return Err(self.error(node, "unsupported local type annotation")),
                }
            }
        }
        // Only the complete annotation is a well-formedness root. Its children
        // may use a callable's private binder, which is not the body binder.
        self.annotation_children
            .extend(self.touched[first..].iter().copied().filter(|n| *n != root));
        self.term(root)
    }
    fn annotation_arguments(&mut self, node: NodeId) {
        if let Some(use_) = self.facts.generics.annotation_uses.get(&self.key(node)) {
            self.annotation_work
                .extend(use_.arguments.iter().rev().filter_map(|arg| {
                    if let super::generics::AnnotationArgument::Type(key) = arg {
                        Some((key.node, false))
                    } else {
                        None
                    }
                }));
        }
    }
    fn annotation_finish(&mut self, node: NodeId) -> Result<TermId, Error> {
        let constructor = match self.kind(node) {
            Kind::InferType => return self.variable(node, Domain::Any),
            Kind::NeverType => return self.known(node, TypeId::NEVER),
            Kind::Group(value) | Kind::Attributed { value, .. } => return self.term(value),
            Kind::Path { .. } | Kind::Name(_) | Kind::Applied { .. } => {
                if self
                    .facts
                    .generics
                    .annotation_uses
                    .contains_key(&self.key(node))
                {
                    return self.annotation_application(node);
                }
                let Some(Target::Definition(id)) =
                    self.facts.node(self.key(node)).and_then(|f| f.reference)
                else {
                    return Err(self.error(node, "missing annotation resolution"));
                };
                let ty = self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(node, "missing resolved annotation type"))?;
                return self.known(node, ty);
            }
            Kind::Type { flavor, elements } => {
                self.type_children.clear();
                let children = self
                    .report
                    .package
                    .unit(self.file)
                    .configured
                    .ast
                    .list(elements);
                let children = if matches!(flavor, TypeFlavor::Ref | TypeFlavor::RefMut) {
                    children
                        .last()
                        .map(std::slice::from_ref)
                        .ok_or_else(|| self.error(node, "missing inferred reference referent"))?
                } else {
                    children
                };
                for child in children.iter().take(
                    children
                        .len()
                        .saturating_sub(usize::from(flavor == TypeFlavor::Array)),
                ) {
                    self.type_children.push(self.term(*child)?);
                }
                match flavor {
                    TypeFlavor::Path => {
                        return self
                            .type_children
                            .first()
                            .copied()
                            .ok_or_else(|| self.error(node, "missing type path"));
                    }
                    TypeFlavor::Tuple => Constructor::Tuple,
                    TypeFlavor::Slice => Constructor::Slice,
                    TypeFlavor::Array => Constructor::Array(declarations::array_length(
                        self.report,
                        self.key(
                            *children
                                .get(1)
                                .ok_or_else(|| self.error(node, "missing array length"))?,
                        ),
                    )?),
                    TypeFlavor::PointerConst | TypeFlavor::PointerMut => Constructor::Pointer {
                        mutable: flavor == TypeFlavor::PointerMut,
                    },
                    TypeFlavor::Ref | TypeFlavor::RefMut => {
                        let Some(super::model::PartialAnnotation::Reference(region)) =
                            self.facts.partial_annotations.get(&self.key(node))
                        else {
                            return Err(self.error(node, "missing elaborated annotation region"));
                        };
                        Constructor::Reference {
                            mutable: flavor == TypeFlavor::RefMut,
                            region: *region,
                        }
                    }
                }
            }
            Kind::FunctionType(index) => self.annotation_function(node, index)?,
            _ => return Err(self.error(node, "invalid local annotation work")),
        };
        self.infer
            .construct(constructor, &self.type_children)
            .map_err(|e| self.error(node, e))
    }
    fn annotation_function(&mut self, node: NodeId, index: u32) -> Result<Constructor, Error> {
        self.type_children.clear();
        let unit = self.report.package.unit(self.file);
        let function = unit.configured.ast.function_types[index as usize];
        for parameter in unit.configured.ast.list(function.parameters) {
            if self.active(*parameter) {
                let ty = declarations::parameter_type(self.report, self.key(*parameter))?;
                self.type_children.push(self.term(ty)?);
            }
        }
        let parameters = u32::try_from(self.type_children.len())
            .map_err(|_| self.error(node, "too many function parameters"))?;
        let result = if let Some(result) = function.result {
            self.term(result)?
        } else {
            self.known(node, TypeId::UNIT)?
        };
        self.type_children.push(result);
        let Some(super::model::PartialAnnotation::Function { bound_regions }) =
            self.facts.partial_annotations.get(&self.key(node))
        else {
            return Err(self.error(node, "missing elaborated callable binder"));
        };
        self.annotation_checks.push(node);
        Ok(Constructor::Function {
            bound_regions: *bound_regions,
            parameters,
            unsafe_: function.unsafe_,
        })
    }
    #[expect(
        clippy::too_many_lines,
        reason = "syntax dispatch schedules context-sensitive child constraints without recursive traversal"
    )]
    fn enter(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        if let Some(expected) = expected
            && matches!(
                self.head(node, expected)?,
                Some(Constructor::Reference { .. })
            )
        {
            self.reference_expected.insert(node);
        }
        if let Some(frame) = self.frames.last_mut()
            && frame.kind == FrameKind::While
            && frame.body == node
        {
            frame.in_condition = false;
        }
        self.enter_control(node, expected)?;
        self.work.push(Work::Exit(node));
        self.children.clear();
        match self.kind(node) {
            Kind::Match { value, .. } => {
                self.work.push(Work::MatchArms { node, expected });
                self.work.push(Work::Enter(value, None));
                return Ok(());
            }
            Kind::Unary {
                op: Unary::Borrow | Unary::BorrowMut,
                operand,
            } => {
                return self.enter_borrow(node, operand, expected);
            }
            Kind::Tuple(elements) | Kind::Array(elements) => {
                return self.enter_elements(node, elements, expected);
            }
            Kind::Repeat { value, count } => {
                return self.enter_repeat(node, value, count, expected);
            }
            Kind::Struct { path, fields, base } => {
                return self.enter_struct(node, path, fields, base, expected);
            }
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
                if self.simple_pattern(pattern) {
                    self.binding(pattern, term, value)?;
                } else {
                    self.work.push(Work::BindPattern {
                        pattern,
                        term,
                        value,
                    });
                }
                self.work
                    .extend(value.map(|value| Work::Enter(value, Some(term))));
                return Ok(());
            }
            Kind::Let {
                pattern,
                ty,
                value: Some(value),
                otherwise: Some(otherwise),
            } => {
                return self.enter_let_else(node, pattern, ty, value, otherwise);
            }
            Kind::LetCondition { pattern, value } => {
                self.work.push(Work::Conditional {
                    node,
                    pattern,
                    value,
                    expected: None,
                });
                self.work.push(Work::Enter(value, None));
                return Ok(());
            }
            Kind::Cast { value, ty } => {
                self.annotation(ty)?;
                self.children.push(value);
            }
            Kind::FieldInit { value, .. }
            | Kind::Group(value)
            | Kind::Attributed { value, .. }
            | Kind::Label { value, .. } => {
                self.work.push(Work::Enter(value, expected));
                return Ok(());
            }
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.work
                    .extend(tail.map(|value| Work::Enter(value, expected)));
                let unit = self.report.package.unit(self.file);
                self.work.extend(
                    unit.configured
                        .ast
                        .list(statements)
                        .iter()
                        .rev()
                        .map(|n| Work::Enter(*n, None)),
                );
                return Ok(());
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                self.work
                    .extend(otherwise.map(|n| Work::Enter(n, expected)));
                self.work.push(Work::Enter(then, expected));
                self.work.push(Work::Enter(condition, None));
                return Ok(());
            }
            Kind::Return(value) => {
                let result = self.known(node, self.result)?;
                self.work
                    .extend(value.map(|n| Work::Enter(n, Some(result))));
                return Ok(());
            }
            Kind::Binary {
                op: Op::Equal,
                left,
                ..
            } => {
                self.work.push(Work::AssignmentValue(node));
                self.work.push(Work::Enter(left, None));
                return Ok(());
            }
            Kind::Call { callee, .. } => {
                self.work.push(Work::CallArguments(node, expected));
                self.work.push(Work::Enter(callee, None));
                return Ok(());
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
            .extend(self.children.iter().rev().map(|n| Work::Enter(*n, None)));
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "typed expression constraints keep operand and result relationships in one dispatch"
    )]
    fn expression(&mut self, node: NodeId) -> Result<TermId, Error> {
        match self.kind(node) {
            Kind::Match { value, arms } => self.match_result(node, value, arms),
            Kind::Arm { guard, value, .. } => {
                if let Some(guard) = guard {
                    let boolean = self.known(guard, TypeId::primitive(Primitive::Bool))?;
                    self.coerce(guard, boolean)?;
                }
                self.term(value)
            }
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
                    DefKind::Function
                        | DefKind::Const
                        | DefKind::Static
                        | DefKind::Constructor
                        | DefKind::Variant
                ) && !self.facts.generics.constants.contains_key(&id)
                {
                    return Err(self.error(node, "unsupported value declaration"));
                }
                let ty = self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(node, "missing value type"))?;
                self.select_unit_constructor(node, id)?;
                if let Some(&variant) = self.facts.members.constructors.get(&id)
                    && self.facts.generics.parameters.contains_key(&DefId(
                        self.facts.members.variants[variant.0 as usize].owner.0,
                    ))
                {
                    return self.generic_constructor(node, ty, variant);
                }
                if self.facts.generics.parameters.contains_key(&id) {
                    return self.generic_function(node, id, ty);
                }
                let ty = self.function_item(node, id, ty)?;
                self.ordinary_bound_obligations(node, id, ty)?;
                self.known(node, ty)
            }
            Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                self.term(value)
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
                if let Some(tail) = tail {
                    return self.term(tail);
                }
                let unit = self.report.package.unit(self.file);
                let mut diverges = false;
                for statement in unit.configured.ast.list(statements) {
                    if self.active(*statement) {
                        diverges |= self.diverges[statement.0 as usize];
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
            Kind::Tuple(elements) | Kind::Array(elements) => self.finish_elements(node, elements),
            Kind::Repeat { value, .. } => self.finish_repeat(node, value),
            Kind::Struct { base, .. } => self.finish_struct(node, base),
            Kind::FieldInit { value, .. } => self.finish_field_init(node, value),
            Kind::Field { value, name } => self.field(node, value, name),
            Kind::Index { value, index } => self.array_index(node, value, index),
            Kind::EmptyStatement | Kind::Item(_) => self.known(node, TypeId::UNIT),
            Kind::Statement(value) => {
                let term = self.term(value)?;
                let never = self.head(node, term)? == Some(Constructor::Never);
                self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
            }
            Kind::Let {
                pattern,
                value,
                otherwise,
                ..
            } => {
                if let Some(otherwise) = otherwise {
                    let never = self.known(otherwise, TypeId::NEVER)?;
                    self.coerce(otherwise, never)?;
                }
                let mut diverges = false;
                if let Some(value) = value {
                    diverges = self.diverges[value.0 as usize];
                    // Compound patterns were constrained after their initializer
                    // by BindPattern; a literal's node type may now be adjusted.
                    if otherwise.is_none() && self.simple_pattern(pattern) {
                        self.coerce(value, self.term(pattern)?)?;
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
            Kind::Unary {
                op: op @ (Unary::Borrow | Unary::BorrowMut),
                operand,
            } => self.borrow(node, operand, op == Unary::BorrowMut),
            Kind::Unary {
                op: Unary::Dereference,
                operand,
            } => self.dereference(node, operand),
            Kind::LetCondition { .. } => self.known(node, TypeId::primitive(Primitive::Bool)),
            Kind::Binary { op, left, right } => {
                let term = self.binary(node, op, left, right)?;
                if op == Op::AndAnd
                    && !self.facts.patterns.chains.is_empty()
                    && (self.facts.patterns.chains.contains(&self.key(left))
                        || self.facts.patterns.chains.contains(&self.key(right)))
                {
                    self.facts.patterns.chains.insert(self.key(node));
                }
                Ok(term)
            }
            Kind::Call { callee, arguments } => self.call(node, callee, arguments),
            Kind::Loop { body } => self.finish_loop(node, body, None),
            Kind::While { condition, body } => self.finish_loop(node, body, Some(condition)),
            Kind::Break { label, value } => self.transfer(node, label.is_some(), value, false),
            Kind::Continue(label) => self.transfer(node, label.is_some(), None, true),
            Kind::Return(value) => {
                if !self.in_function {
                    return Err(self.error(node, "return is outside a function"));
                }
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
                    self.join(node, then, otherwise)?
                } else {
                    let unit = self.known(node, TypeId::UNIT)?;
                    self.coerce(then, unit)?;
                    unit
                };
                Ok(result)
            }
            Kind::Cast { value, ty } => {
                let source = self.term(value)?;
                let target = self.term(ty)?;
                if matches!(
                    self.head(node, source)?,
                    Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
                ) && matches!(self.head(node, target)?, Some(Constructor::Function { .. }))
                {
                    let signature = self.child(node, source, 0)?;
                    self.callable_shape(node, signature, target)?;
                }
                self.checks.push(node);
                Ok(target)
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
            Value::String(bytes) => return self.text_literal_type(node, &bytes, true),
            Value::ByteString(bytes) => return self.text_literal_type(node, &bytes, false),
            Value::CString(_) => {
                return Err(self.error(node, "C string values require the library type assignment"));
            }
        };
        self.known(node, TypeId::primitive(primitive))
    }
    fn coerce(&mut self, source: NodeId, expected: TermId) -> Result<(), Error> {
        self.coerce_one(source, expected, self.reference_expected.contains(&source))?;
        self.finish_reference_coercions()
    }
    fn record_adjustment(
        &mut self,
        node: NodeId,
        expected: TermId,
        kind: AdjustmentKind,
    ) -> Result<(), Error> {
        self.adjustments.push(PendingAdjustment {
            node,
            actual: self.term(node)?,
            expected,
            kind,
            reset: false,
        });
        Ok(())
    }
    fn coerce_one(
        &mut self,
        source: NodeId,
        expected: TermId,
        identity_reborrow: bool,
    ) -> Result<(), Error> {
        let actual = self.term(source)?;
        let actual_head = self.head(source, actual)?;
        let expected_head = self.head(source, expected)?;
        if actual_head == Some(Constructor::Never) {
            self.record_adjustment(source, expected, AdjustmentKind::Never)?;
            return Ok(());
        }
        if matches!(
            actual_head,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        ) && matches!(expected_head, Some(Constructor::Function { .. }))
        {
            let signature = self.child(source, actual, 0)?;
            self.callable_shape(source, signature, expected)?;
            self.record_adjustment(source, expected, AdjustmentKind::FunctionPointer)?;
            return Ok(());
        }
        if matches!(actual_head, Some(Constructor::Function { .. }))
            && matches!(expected_head, Some(Constructor::Function { .. }))
        {
            self.callable_shape(source, actual, expected)?;
            self.record_adjustment(source, expected, AdjustmentKind::Equality)?;
            return Ok(());
        }
        if matches!(actual_head, Some(Constructor::Reference { .. }))
            && matches!(expected_head, Some(Constructor::Reference { .. }))
            && self.reference_coercion(source, actual, expected, identity_reborrow)?
        {
            return Ok(());
        }
        let inferred = if expected_head.is_none() {
            self.inferred_regions(source, actual)?
        } else {
            actual
        };
        self.callable_shape(source, inferred, expected)?;
        self.record_adjustment(source, expected, AdjustmentKind::Equality)?;
        Ok(())
    }
    fn join(&mut self, node: NodeId, left: NodeId, right: NodeId) -> Result<TermId, Error> {
        let common = self.join_types(node, self.term(left)?, self.term(right)?)?;
        self.coerce(left, common)?;
        self.coerce(right, common)?;
        Ok(common)
    }
    fn join_types(&mut self, node: NodeId, a: TermId, b: TermId) -> Result<TermId, Error> {
        let ah = self.head(node, a)?;
        let bh = self.head(node, b)?;
        if ah == Some(Constructor::Never) {
            return Ok(b);
        }
        if bh == Some(Constructor::Never) {
            return Ok(a);
        }
        if matches!(ah, Some(Constructor::Reference { .. }))
            && matches!(bh, Some(Constructor::Reference { .. }))
        {
            return self.reference_join(node, a, b);
        }
        let a_item = matches!(
            ah,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        );
        let b_item = matches!(
            bh,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        );
        let distinct_items = match (ah, bh) {
            (Some(Constructor::FunctionItem(a, _)), Some(Constructor::FunctionItem(b, _))) => {
                a != b
            }
            _ => {
                a_item
                    && b_item
                    && self
                        .infer
                        .incompatible(&self.facts.types, a, b)
                        .map_err(|e| self.error(node, e))?
            }
        };
        if distinct_items
            || (a_item && matches!(bh, Some(Constructor::Function { .. })))
            || (b_item && matches!(ah, Some(Constructor::Function { .. })))
        {
            let left = if a_item { self.child(node, a, 0)? } else { a };
            let right = if b_item { self.child(node, b, 0)? } else { b };
            self.shape_equation(node, left, right)?;
            return self.inferred_regions(node, left);
        }
        self.shape_equation(node, a, b)?;
        self.inferred_regions(node, a)
    }
    fn binary(
        &mut self,
        node: NodeId,
        op: Op,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        if op == Op::Equal || scalar::compound_operator(op).is_some() {
            return self.assignment(node, op, left, right);
        }
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
    fn call(&mut self, node: NodeId, _callee: NodeId, arguments: List) -> Result<TermId, Error> {
        let term = self
            .call_types
            .active
            .remove(&node)
            .ok_or_else(|| self.error(node, "missing call instantiation"))?;
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
            let exclusive = matches!(
                self.head(node, expected)?,
                Some(Constructor::Reference { mutable: true, .. })
            );
            let mark = self.adjustments.len();
            self.coerce(*argument, expected)?;
            let mut site = *argument;
            while let Kind::Group(value) | Kind::Attributed { value, .. } = self.kind(site) {
                site = value;
            }
            for index in mark..self.adjustments.len() {
                if self.adjustments[index].node == site
                    && !matches!(
                        self.kind(site),
                        Kind::Unary {
                            op: Unary::BorrowMut,
                            ..
                        }
                    )
                    && let Some((adjustment, _)) = self.reference_adjustments.get_mut(&index)
                {
                    adjustment.two_phase = exclusive && adjustment.dereferences == 1;
                }
            }
            count += 1;
        }
        if count != parameters as usize {
            return Err(self.error(node, "too few call arguments"));
        }
        self.calls.push((node, term));
        self.child(node, term, count)
    }
    fn enter_control(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        let label = if self.pending_label.is_some_and(|(target, _)| target == node) {
            self.pending_label.take().map(|(_, label)| label)
        } else {
            None
        };
        let (kind, body) = match self.kind(node) {
            Kind::Loop { body } => (FrameKind::Loop, body),
            Kind::While { body, .. } => (FrameKind::While, body),
            Kind::Block { .. } if label.is_some() => (FrameKind::Block, node),
            Kind::Label { value, .. } => {
                let label = self
                    .facts
                    .node(self.key(node))
                    .and_then(|f| f.binding)
                    .ok_or_else(|| self.error(node, "missing label definition"))?;
                self.pending_label = Some((value, label));
                return Ok(());
            }
            _ => return Ok(()),
        };
        let result = if kind == FrameKind::While {
            self.known(node, TypeId::UNIT)?
        } else {
            self.variable(node, Domain::Any)?
        };
        self.frames.push(Frame {
            node,
            body,
            kind,
            label,
            result,
            breaks: None,
            expected: if kind == FrameKind::While {
                None
            } else {
                expected
            },
            in_condition: kind == FrameKind::While,
        });
        Ok(())
    }
    fn finish_loop(
        &mut self,
        node: NodeId,
        body: NodeId,
        condition: Option<NodeId>,
    ) -> Result<TermId, Error> {
        let frame = self
            .frames
            .pop()
            .ok_or_else(|| self.error(node, "missing loop control frame"))?;
        if frame.node != node {
            return Err(self.error(node, "mismatched loop control frame"));
        }
        let unit = self.known(node, TypeId::UNIT)?;
        self.coerce(body, unit)?;
        if let Some(condition) = condition {
            let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
            self.coerce(condition, boolean)?;
            self.diverges[node.0 as usize] =
                frame.breaks.is_none() && self.diverges[condition.0 as usize];
        }
        self.finish_frame(
            frame,
            if condition.is_some() {
                Some(unit)
            } else {
                None
            },
        )
    }
    fn finish_frame(&mut self, frame: Frame, base: Option<TermId>) -> Result<TermId, Error> {
        if let Some(root) = frame.breaks {
            self.break_roots.insert(frame.node, root);
        }
        let expected = self.frame_expected(frame)?;
        let mut common = if frame.breaks.is_some() {
            expected.or(base)
        } else {
            base
        };
        let mut site = frame.breaks;
        while let Some(index) = site {
            let value = self.breaks[index].value;
            let node = self.breaks[index].node;
            let term = if let Some(value) = value {
                self.term(value)?
            } else {
                self.known(node, TypeId::UNIT)?
            };
            common = Some(if let Some(previous) = common {
                self.join_types(node, previous, term)?
            } else {
                term
            });
            site = self.breaks[index].next;
        }
        let common = if let Some(common) = common {
            common
        } else {
            self.known(frame.node, TypeId::NEVER)?
        };
        self.equate(frame.node, frame.result, common)?;
        let reference = matches!(
            self.head(frame.node, common)?,
            Some(Constructor::Reference { .. })
        );
        let identity_reborrow = self.reference_expected.contains(&frame.node);
        if frame.kind == FrameKind::Block {
            if reference
                && let Kind::Block {
                    tail: Some(tail), ..
                } = self.kind(frame.node)
            {
                if frame.breaks.is_some() {
                    self.reference_expected.insert(tail);
                }
                self.coerce_one(
                    tail,
                    frame.result,
                    identity_reborrow || frame.breaks.is_some(),
                )?;
                self.finish_reference_coercions()?;
            } else {
                self.coerce(frame.node, if reference { frame.result } else { common })?;
            }
        }
        let mut site = frame.breaks;
        while let Some(index) = site {
            if let Some(value) = self.breaks[index].value {
                self.coerce_one(
                    value,
                    if reference { frame.result } else { common },
                    identity_reborrow || self.reference_expected.contains(&value),
                )?;
                self.finish_reference_coercions()?;
            }
            site = self.breaks[index].next;
        }
        Ok(frame.result)
    }
    fn frame_expected(&mut self, frame: Frame) -> Result<Option<TermId>, Error> {
        if let Some(expected) = frame.expected
            && self.head(frame.node, expected)?.is_some()
        {
            return Ok(Some(expected));
        }
        Ok(None)
    }
    fn call_arguments(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        let Kind::Call { callee, arguments } = self.kind(node) else {
            return Err(self.error(node, "invalid call work"));
        };
        let mut term = self.term(callee)?;
        let constructor = matches!(self.head(node, term)?,
        Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _))
            if self.facts.members.constructors.get(&DefId(item.0)).is_some_and(|variant| {
                let owner = self.facts.members.variants[variant.0 as usize].owner;
                self.facts.generics.parameters.contains_key(&DefId(owner.0))
            }));
        if matches!(
            self.head(node, term)?,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        ) {
            term = self.child(node, term, 0)?;
        }
        term = self.call_signature(node, term)?;
        self.call_types.active.insert(node, term);
        let Some(Constructor::Function { parameters, .. }) = self.head(node, term)? else {
            return Err(self.error(node, "call requires a known function type"));
        };
        if constructor && let Some(expected) = expected {
            let result = self.child(node, term, parameters as usize)?;
            self.shape_equation(node, result, expected)?;
        }
        let unit = self.report.package.unit(self.file);
        let count = unit
            .configured
            .ast
            .list(arguments)
            .iter()
            .filter(|n| self.active(**n))
            .count();
        if count != parameters as usize {
            return Err(self.error(node, "call argument count mismatch"));
        }
        let mut index = count;
        for argument in unit.configured.ast.list(arguments).iter().rev() {
            if self.active(*argument) {
                index -= 1;
                let expected = self.child(node, term, index)?;
                self.work.push(Work::Enter(*argument, Some(expected)));
            }
        }
        Ok(())
    }
    fn transfer(
        &mut self,
        node: NodeId,
        labeled: bool,
        value: Option<NodeId>,
        continuing: bool,
    ) -> Result<TermId, Error> {
        let index = if labeled {
            let Some(Target::Definition(label)) =
                self.facts.node(self.key(node)).and_then(|f| f.reference)
            else {
                return Err(self.error(node, "missing control label resolution"));
            };
            self.frames.iter().rposition(|f| f.label == Some(label))
        } else {
            self.frames.len().checked_sub(1)
        }
        .ok_or_else(|| self.error(node, "break or continue is outside a loop"))?;
        let frame = self.frames[index];
        if !labeled && (frame.kind == FrameKind::Block || frame.in_condition) {
            return Err(self.error(
                node,
                "control transfer here requires an explicit loop label",
            ));
        }
        if continuing {
            if frame.kind == FrameKind::Block {
                return Err(self.error(node, "cannot continue a labeled block"));
            }
        } else {
            if value.is_some() && frame.kind == FrameKind::While {
                return Err(self.error(node, "while loops cannot break with a value"));
            }
            let expected = self.frame_expected(frame)?.unwrap_or(frame.result);
            if let Some(value) = value {
                if matches!(
                    self.head(value, expected)?,
                    Some(Constructor::Reference { .. })
                ) {
                    self.reference_expected.insert(value);
                }
                self.coerce(value, expected)?;
            } else {
                let unit = self.known(node, TypeId::UNIT)?;
                self.equate(node, unit, expected)?;
            }
            let site = self.breaks.len();
            self.breaks.push(BreakSite {
                node,
                value,
                next: frame.breaks,
            });
            self.frames[index].breaks = Some(site);
        }
        let key = self.key(node);
        self.facts
            .node_mut(key)
            .ok_or_else(|| Declarations::error(self.report, key, "invalid control node"))?
            .operation = Some(Operation::Transfer {
            target: frame.node,
            continuing,
        });
        self.known(node, TypeId::NEVER)
    }
    fn assignment(
        &mut self,
        node: NodeId,
        op: Op,
        mut left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let a = self.term(left)?;
        let b = self.term(right)?;
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(left) {
            left = inner;
        }
        let target = self.assignment_target(node, left)?;
        let operator = scalar::compound_operator(op);
        if matches!(operator, Some(Op::ShiftLeft | Op::ShiftRight)) {
            let integer = self.variable(node, Domain::Integer)?;
            self.equate(node, b, integer)?;
        } else if operator.is_some() {
            self.equate(node, a, b)?;
        } else {
            self.coerce(right, a)?;
        }
        self.assignments.push((node, target, operator));
        let never = self.head(node, b)? == Some(Constructor::Never);
        self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
    }
    fn validate_assignments(&mut self) -> Result<(), Error> {
        for (node, target, operator) in &self.assignments {
            let Kind::Binary { left, right, .. } = self.kind(*node) else {
                return Err(self.error(*node, "invalid assignment syntax"));
            };
            if let Some(operator) = operator
                && !scalar::binary(*operator, self.primitive(left)?, self.primitive(right)?)
            {
                return Err(self.error(*node, "compound assignment is not defined for these types"));
            }
            let operation = if let Some(target) = target {
                Operation::Assign {
                    target: *target,
                    operator: *operator,
                    left: self.ty(left)?,
                    right: self.ty(right)?,
                }
            } else {
                Operation::AssignPlace {
                    operator: *operator,
                    left: self.ty(left)?,
                    right: self.ty(right)?,
                }
            };
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid assignment node"))?
                .operation = Some(operation);
        }
        Ok(())
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
                    let kind = if matches!(
                        self.facts.types.kind(self.ty(value)?),
                        Some(Type::FunctionItem { .. })
                    ) && matches!(
                        self.facts.types.kind(self.ty(ty)?),
                        Some(Type::Function { .. })
                    ) {
                        super::model::Cast::FunctionPointer
                    } else if let Some(Type::Adt(item, _)) = self.facts.types.kind(self.ty(value)?)
                    {
                        if !self.facts.enum_casts.contains(&item)
                            || self.facts.destructors.has_declared(item)
                            || self.primitive(ty)?.integer().is_none()
                        {
                            return Err(self.error(
                                *node,
                                "only eligible field-less enums can be cast to integers",
                            ));
                        }
                        super::model::Cast::EnumToInteger
                    } else {
                        scalar::cast(self.primitive(value)?, self.primitive(ty)?)
                            .ok_or_else(|| self.error(*node, "invalid scalar cast"))?
                    };
                    Operation::Cast {
                        kind,
                        source: self.ty(value)?,
                        target: self.ty(ty)?,
                    }
                }
                _ => return Err(self.error(*node, "invalid scalar operation obligation")),
            };
            let key = self.key(*node);
            if let Operation::Cast {
                kind: super::model::Cast::FunctionPointer,
                source,
                target,
            } = operation
            {
                self.facts.retain_coercion(super::model::Coercion {
                    body: self.body,
                    source: key,
                    actual: source,
                    expected: target,
                    kind: AdjustmentKind::FunctionPointer,
                    reference: None,
                });
            }
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid operation node"))?
                .operation = Some(operation);
        }
        Ok(())
    }
    fn validate_values(&mut self) -> Result<(), Error> {
        // Keep only negated literal IDs; no per-function copy of the file's AST.
        for index in 0..self.touched.len() {
            let node = self.touched[index];
            if let Kind::Repeat { value, .. } = self.kind(node) {
                self.validate_repeat(node, value)?;
                continue;
            }
            let Kind::Literal(index) = self.kind(node) else {
                continue;
            };
            let unit = self.report.package.unit(self.file);
            let value = unit
                .literal(index)
                .map_err(|e| self.error(node, e.message))?;
            if matches!(value, Value::String(_) | Value::ByteString(_)) {
                continue;
            }
            let negated = self.negated.contains(&node);
            let scalar = scalar::literal(&value, self.primitive(node)?, negated)
                .map_err(|e| self.error(node, e))?;
            let key = self.key(node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid literal node"))?
                .scalar = Some(scalar);
        }
        Ok(())
    }
}
