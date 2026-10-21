//! Capture requirements consume typed places and the actual expression use context.
mod nested;
mod receivers;
use super::{Checker, Domain, Error, Kind, NodeId, TermId, places::Places};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{
        closures::captures::{Mode, PathId, Plan},
        closures::{AccessKind, AdjustedAccess},
        model::{AdjustmentKind, ReferenceConversion},
        scalar,
    },
    support::call_traits::CallTrait,
    syntax::ast::{List, Unary},
    types::{closure::CallKind, infer},
};
pub(super) use nested::children;
use std::collections::{HashMap, HashSet};

pub(super) struct Captures {
    pub plan: Plan,
    pub types: Vec<TermId>,
    pub accesses: HashMap<NodeKey, PathId>,
    pub adjusted_accesses: HashMap<(NodeKey, AccessKind), AdjustedAccess>,
    pub mentions: Vec<PathId>,
    pub nested_accesses: HashMap<(NodeKey, PathId), PathId>,
    pub kind: CallKind,
    pub work: usize,
}
#[derive(Clone, Copy)]
enum Use {
    Value,
    ValueOf(TermId),
    Shared,
    Mutable,
    Mention,
}
struct Work {
    node: NodeId,
    usage: Use,
    dereferences: u32,
    receiver: Option<(NodeId, AccessKind)>,
}
struct Collector<'a> {
    children: HashMap<NodeId, Option<&'a Captures>>,
    nested_accesses: HashMap<(NodeKey, PathId), PathId>,
    places: Places,
    mentions: Vec<PathId>,
    mentioned: HashSet<PathId>,
    pending: Vec<Work>,
    calls: HashMap<NodeId, (Option<CallTrait>, u32)>,
    methods: HashMap<NodeId, (u32, Option<TermId>)>,
    references: HashMap<NodeId, (ReferenceConversion, TermId)>,
    adjusted: HashSet<NodeId>,
    adjusted_accesses: HashMap<(NodeKey, AccessKind), AdjustedAccess>,
    kind: CallKind,
    moving: bool,
    work: usize,
}
impl Checker<'_> {
    pub(super) fn closure_uses(
        &mut self,
        source: NodeId,
        root: NodeId,
        bindings: &[(DefId, TermId)],
        children: HashMap<NodeId, Option<&Captures>>,
    ) -> Result<Captures, Error> {
        let Kind::Closure { moving, .. } = self.kind(source) else {
            return Err(self.error(source, "missing closure capture owner"));
        };
        let mut collector = Collector {
            children,
            nested_accesses: HashMap::new(),
            places: Places::new(bindings),
            mentions: Vec::new(),
            mentioned: HashSet::new(),
            pending: Vec::new(),
            calls: self
                .state
                .call_types
                .invocations
                .iter()
                .map(|(node, pending)| (*node, pending.capture_receiver()))
                .collect(),
            methods: self
                .state
                .call_types
                .methods
                .iter()
                .map(|(node, pending)| (*node, pending.capture_receiver()))
                .collect(),
            references: self
                .state
                .adjustments
                .iter()
                .enumerate()
                .filter_map(|(index, adjustment)| {
                    self.state
                        .reference_adjustments
                        .get(&index)
                        .map(|&reference| (adjustment.node, reference))
                })
                .collect(),
            adjusted: self
                .state
                .adjustments
                .iter()
                .filter(|adjustment| {
                    !matches!(
                        adjustment.kind,
                        AdjustmentKind::Equality
                            | AdjustmentKind::Never
                            | AdjustmentKind::ArrayToSlice
                            | AdjustmentKind::Unsize
                            | AdjustmentKind::Pointer
                            | AdjustmentKind::FunctionPointer
                    )
                })
                .map(|adjustment| adjustment.node)
                .collect(),
            adjusted_accesses: HashMap::new(),
            kind: CallKind::Shared,
            moving,
            work: 0,
        };
        collector.push(root, Use::Value);
        while let Some(work) = collector.pending.pop() {
            collector.work += 1;
            if collector.work > 1_000_000 {
                return Err(self.error(work.node, "closure use work limit exceeded"));
            }
            if self.active(work.node) {
                collector.visit(self, &work)?;
            }
        }
        let work = collector.work + collector.places.work();
        Ok(Captures {
            plan: collector
                .places
                .graph
                .finish()
                .map_err(|e| self.error(source, e))?,
            types: collector.places.types,
            accesses: collector.places.accesses,
            adjusted_accesses: collector.adjusted_accesses,
            mentions: collector.mentions,
            nested_accesses: collector.nested_accesses,
            kind: collector.kind,
            work,
        })
    }
    fn capture_copy(&mut self, node: NodeId, term: TermId) -> Result<bool, Error> {
        match self
            .state
            .infer
            .unresolved_domain(term)
            .map_err(|e| self.error(node, e))?
        {
            Some(Domain::Integer | Domain::Float) => return Ok(true),
            Some(Domain::Any) => {
                return Err(self.error(
                    node,
                    "capture Copy permission requires a determined input type",
                ));
            }
            None => {}
        }
        let ty = self
            .state
            .infer
            .resolved(&mut self.facts.types, term)
            .map_err(|e| {
                self.error(
                    node,
                    match e {
                        infer::Error::Unresolved => {
                            "capture Copy permission requires a determined input type".to_owned()
                        }
                        _ => e.to_string(),
                    },
                )
            })?;
        if self.facts.generics.has_nominal_requirements {
            self.prepare_storage(node, ty)?;
        }
        self.state
            .parameter_copies
            .as_mut()
            .unwrap_or(&mut self.facts.copies)
            .demand(
                &self.facts.types,
                &self.facts.aggregates,
                &self.facts.derives,
                ty,
            )
            .map(|properties| properties.copy)
            .map_err(|e| self.error(node, e))
    }
}
impl Collector<'_> {
    fn push(&mut self, node: NodeId, usage: Use) {
        self.pending.push(Work {
            node,
            usage,
            dereferences: 0,
            receiver: None,
        });
    }
    fn list(&mut self, checker: &Checker<'_>, list: List) {
        for &node in checker
            .report
            .package
            .unit(checker.state.file)
            .configured
            .ast
            .list(list)
            .iter()
            .rev()
        {
            self.push(node, Use::Value);
        }
    }
    fn use_place(&mut self, checker: &mut Checker<'_>, work: &Work) -> Result<bool, Error> {
        let Some(mut place) = self.places.get(checker, work.node)? else {
            return Ok(false);
        };
        let converted;
        let work = if self.adjusted.contains(&work.node) {
            converted = self.reference_use(checker, work)?;
            &converted
        } else {
            work
        };
        for _ in 0..work.dereferences {
            place = self.places.dereference(checker, work.node, place)?;
        }
        if work.dereferences != 0
            && let Some((site, kind)) = work.receiver
        {
            let access = AdjustedAccess {
                path: place.path,
                dereferences: work.dereferences,
            };
            if self
                .adjusted_accesses
                .insert((checker.key(site), kind), access)
                .is_some_and(|prior| prior != access)
            {
                return Err(checker.error(site, "capture receiver selection changed"));
            }
        }
        if matches!(work.usage, Use::Mention) {
            if self.mentioned.insert(place.path) {
                self.mentions.push(place.path);
            }
            return Ok(true);
        }
        let term = match work.usage {
            Use::ValueOf(term) => term,
            _ => self.places.types[place.path.index()],
        };
        let mut mode = if place.raw {
            Mode::Shared
        } else {
            match work.usage {
                Use::Shared => Mode::Shared,
                Use::Mutable => {
                    self.kind = self.kind.max(CallKind::Mutable);
                    Mode::Mutable
                }
                Use::Value | Use::ValueOf(_) if checker.capture_copy(work.node, term)? => {
                    Mode::Shared
                }
                Use::Value | Use::ValueOf(_) => {
                    self.kind = CallKind::Once;
                    Mode::Value
                }
                Use::Mention => return Err(checker.error(work.node, "unexpected capture mention")),
            }
        };
        let path = place.capture_path(self.moving);
        if self.moving {
            mode = Mode::Value;
        }
        self.places
            .graph
            .require(path, mode)
            .map_err(|e| checker.error(work.node, e))?;
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "typed expression-use dispatch keeps capture contexts explicit without recursive traversal"
    )]
    fn visit(&mut self, checker: &mut Checker<'_>, work: &Work) -> Result<(), Error> {
        if self.use_place(checker, work)? {
            return Ok(());
        }
        let node = work.node;
        match checker.kind(node) {
            Kind::Group(value)
            | Kind::Attributed { value, .. }
            | Kind::Label { value, .. }
            | Kind::Statement(value) => self.push(value, work.usage),
            Kind::Name(_)
            | Kind::Path { .. }
            | Kind::Qualified { .. }
            | Kind::Literal(_)
            | Kind::Item(_)
            | Kind::EmptyStatement
            | Kind::Continue(_)
            | Kind::Block {
                flavor: crate::syntax::ast::BlockFlavor::Const,
                ..
            } => {}
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                if let Some(tail) = tail {
                    self.push(tail, work.usage);
                }
                self.list(checker, statements);
            }
            Kind::Unary { op, operand } => self.push(
                operand,
                match op {
                    Unary::Borrow | Unary::RawConst => Use::Shared,
                    Unary::BorrowMut | Unary::RawMut => Use::Mutable,
                    _ => Use::Value,
                },
            ),
            Kind::Binary { op, left, right } => {
                self.push(right, Use::Value);
                self.push(
                    left,
                    if op == crate::lex::token::Punctuation::Equal
                        || scalar::compound_operator(op).is_some()
                    {
                        Use::Mutable
                    } else {
                        Use::Value
                    },
                );
            }
            Kind::Let {
                pattern,
                value,
                otherwise,
                ..
            } => {
                let mut pattern = pattern;
                while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } =
                    checker.kind(pattern)
                {
                    pattern = inner;
                }
                let usage = match checker.kind(pattern) {
                    Kind::Wildcard => Use::Mention,
                    Kind::Binding {
                        by_ref: false,
                        subpattern: None,
                        ..
                    } => Use::Value,
                    _ => {
                        return Err(
                            checker.error(pattern, "compound capture patterns are not implemented")
                        );
                    }
                };
                if let Some(otherwise) = otherwise {
                    self.push(otherwise, Use::Value);
                }
                if let Some(value) = value {
                    self.push(value, usage);
                }
            }
            Kind::Call { callee, arguments } => {
                self.list(checker, arguments);
                let (kind, dereferences) = self.call_receiver(checker, node, callee)?;
                self.pending.push(Work {
                    node: callee,
                    usage: match kind {
                        Some(CallTrait::Fn) => Use::Shared,
                        Some(CallTrait::FnMut) => Use::Mutable,
                        _ => Use::Value,
                    },
                    dereferences,
                    receiver: Some((node, AccessKind::Invocation)),
                });
            }
            Kind::Tuple(elements) | Kind::Array(elements) => self.list(checker, elements),
            Kind::MethodCall {
                receiver,
                arguments,
                ..
            }
            | Kind::GenericMethodCall {
                receiver,
                arguments,
                ..
            } => {
                self.list(checker, arguments);
                self.method(checker, node, receiver)?;
            }
            Kind::Index { value, index } => self.index(checker, work, value, index)?,
            Kind::Field { value, .. } => self.projected(checker, node, value, work.usage)?,
            Kind::Repeat { value, .. }
            | Kind::Cast { value, .. }
            | Kind::FieldInit { value, .. } => self.push(value, Use::Value),
            Kind::Struct { fields, base, .. } => {
                if let Some(base) = base {
                    self.push(base, Use::Value);
                }
                self.list(checker, fields);
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                if let Some(otherwise) = otherwise {
                    self.push(otherwise, Use::Value);
                }
                self.push(then, Use::Value);
                self.push(condition, Use::Value);
            }
            Kind::Loop { body } => self.push(body, Use::Value),
            Kind::While { condition, body } => {
                self.push(body, Use::Value);
                self.push(condition, Use::Value);
            }
            Kind::Return(value) | Kind::Break { value, .. } | Kind::LibraryValue { value, .. } => {
                if let Some(value) = value {
                    self.push(value, Use::Value);
                }
            }
            Kind::Range { start, end, .. } => {
                if let Some(end) = end {
                    self.push(end, Use::Value);
                }
                if let Some(start) = start {
                    self.push(start, Use::Value);
                }
            }
            Kind::Closure { .. } => self.nested(checker, node)?,
            _ => return Err(checker.error(node, "this typed capture use is not implemented")),
        }
        Ok(())
    }
}
