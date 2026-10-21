//! Rust 2024 temporary scopes, distinct from promotion and inferred loan regions.
mod controls;
mod extend;
mod walk;
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::model::{Declarations, Function, Initializer},
    syntax::ast::{Kind as Syntax, List, NodeId, Unary},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Body,
    Block,
    Statement,
    Tail,
    Condition,
    IfThen,
    Iteration,
    Arm,
    Guard,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifetime {
    Scope(Id),
    Static,
}
#[derive(Clone, Copy, Debug)]
pub struct Scope {
    pub parent: Option<Id>,
    pub source: NodeId,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub scope: Id,
    pub temporary: Lifetime,
    pub extended: bool,
}
/// Lexical storage endpoints. These facts alone do not prove a borrow valid.
pub struct Facts {
    scopes: Vec<Scope>,
    scratch: Scratch,
    /// Scheduled syntax, pattern, and extension visits, bounded before allocation.
    pub work: usize,
}
/// Reuse lookup and traversal capacity across bodies; only scope records survive building.
#[derive(Default)]
pub(crate) struct Scratch {
    boundaries: HashMap<(NodeId, Kind), Id>,
    nodes: HashMap<NodeId, Node>,
    bindings: HashMap<DefId, Id>,
    pending: Vec<Work>,
    pattern_work: Vec<NodeId>,
    extensions: HashMap<NodeId, Lifetime>,
}
impl Facts {
    pub(crate) fn into_scopes(self) -> Vec<Scope> {
        self.scopes
    }
    pub(crate) fn synthetic(source: NodeId) -> Self {
        Self::with_scratch(source, Scratch::default())
    }
    fn with_scratch(source: NodeId, mut scratch: Scratch) -> Self {
        scratch.boundaries.insert((source, Kind::Body), Id(0));
        Self {
            scopes: vec![Scope {
                parent: None,
                source,
                kind: Kind::Body,
            }],
            scratch,
            work: 0,
        }
    }
    pub(crate) fn finish(mut self) -> (Vec<Scope>, Scratch) {
        recycle(&mut self.scratch.boundaries);
        recycle(&mut self.scratch.nodes);
        recycle(&mut self.scratch.bindings);
        self.scratch.pending.clear();
        self.scratch.pattern_work.clear();
        recycle(&mut self.scratch.extensions);
        (self.scopes, self.scratch)
    }
    #[must_use]
    pub fn boundary(&self, source: NodeId, kind: Kind) -> Option<Id> {
        self.scratch.boundaries.get(&(source, kind)).copied()
    }
    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }
    #[must_use]
    pub fn node(&self, node: NodeId) -> Option<Node> {
        self.scratch.nodes.get(&node).copied()
    }
    #[must_use]
    pub fn binding(&self, definition: DefId) -> Option<Id> {
        self.scratch.bindings.get(&definition).copied()
    }
}

fn recycle<K, V>(table: &mut HashMap<K, V>) {
    // HashMap clearing visits capacity. A large body must not make every
    // subsequent small body pay to reset its oversized lookup tables.
    if table.capacity() > 4096 && table.len() < table.capacity() / 8 {
        *table = HashMap::new();
    } else {
        table.clear();
    }
}

/// Compute a function's lexical storage endpoints from selected syntax.
///
/// # Errors
/// Reports absent bodies, missing binding facts, unsupported syntax or exhausted work.
pub fn function(
    report: &Report,
    facts: &Declarations,
    function: &Function,
) -> Result<Facts, Error> {
    function_with_scratch(report, facts, function, Scratch::default())
}
pub(crate) fn function_with_scratch(
    report: &Report,
    facts: &Declarations,
    function: &Function,
    scratch: Scratch,
) -> Result<Facts, Error> {
    let source = function
        .body
        .ok_or_else(|| Error::at(function.node.file, 0, "scope analysis requires a body"))?;
    analyze(report, facts, source, Some(function.parameters), scratch)
}
pub(crate) fn closure_with_scratch(
    report: &Report,
    facts: &Declarations,
    closure: &crate::sema::closures::Body,
    scratch: Scratch,
) -> Result<Facts, Error> {
    analyze_at(
        report,
        facts,
        closure.semantic_source(),
        closure.body,
        Some(closure.parameters),
        scratch,
    )
}
/// Compute a const/static initializer's endpoints, including static extension.
///
/// # Errors
/// Reports missing binding facts, unsupported syntax or exhausted work.
pub fn initializer(
    report: &Report,
    facts: &Declarations,
    initializer: &Initializer,
) -> Result<Facts, Error> {
    initializer_with_scratch(report, facts, initializer, Scratch::default())
}
pub(crate) fn initializer_with_scratch(
    report: &Report,
    facts: &Declarations,
    initializer: &Initializer,
    scratch: Scratch,
) -> Result<Facts, Error> {
    analyze(report, facts, initializer.value, None, scratch)
}

/// Reuse the initializer scope rules for an anonymous constant expression.
pub(crate) fn constant_with_scratch(
    report: &Report,
    facts: &Declarations,
    source: NodeKey,
    scratch: Scratch,
) -> Result<Facts, Error> {
    analyze(report, facts, source, None, scratch)
}

#[derive(Clone, Copy)]
struct Context {
    scope: Id,
    block: Id,
    temporary: Lifetime,
}
#[derive(Clone, Copy)]
enum Work {
    Expression(NodeId, Context),
    Statement(NodeId, Context),
    Condition(NodeId, Context),
    Pattern(NodeId, Id),
    ExtendExpression(NodeId, Lifetime),
    ExtendPlace(NodeId, Lifetime),
}
struct Walker<'a> {
    report: &'a Report,
    types: &'a Declarations,
    source: NodeKey,
    facts: Facts,
}
fn analyze(
    report: &Report,
    types: &Declarations,
    source: NodeKey,
    parameters: Option<List>,
    scratch: Scratch,
) -> Result<Facts, Error> {
    analyze_at(report, types, source, source, parameters, scratch)
}
fn analyze_at(
    report: &Report,
    types: &Declarations,
    source: NodeKey,
    root: NodeKey,
    parameters: Option<List>,
    scratch: Scratch,
) -> Result<Facts, Error> {
    if source.file != root.file {
        return Err(Error::at(
            source.file,
            0,
            "scope owner and executable root belong to different files",
        ));
    }
    let mut walker = Walker {
        report,
        types,
        source,
        facts: Facts::with_scratch(root.node, scratch),
    };
    let context = Context {
        scope: Id(0),
        block: Id(0),
        temporary: Lifetime::Scope(Id(0)),
    };
    walker.push(Work::Expression(root.node, context))?;
    if let Some(parameters) = parameters {
        for index in (parameters.start..parameters.end).rev() {
            let mut parameter = walker.child(index);
            if !walker.active(parameter) {
                continue;
            }
            while let Syntax::Attributed { value, .. } = walker.kind(parameter) {
                parameter = value;
            }
            let Syntax::Parameter { pattern, .. } = walker.kind(parameter) else {
                return Err(walker.error(parameter, "missing parameter pattern"));
            };
            if let Some(receiver) = types.receivers.get(&NodeKey {
                file: source.file,
                node: parameter,
            }) {
                walker.bind_scope(receiver.binding.node, receiver.definition, Id(0))?;
                continue;
            }
            walker.push(Work::Pattern(pattern, Id(0)))?;
        }
    } else {
        walker.push(Work::ExtendExpression(root.node, Lifetime::Static))?;
    }
    while let Some(work) = walker.facts.scratch.pending.pop() {
        walker.visit(work).map_err(|e| report.diagnostic(e))?;
    }
    for (node, lifetime) in &walker.facts.scratch.extensions {
        let Some(info) = walker.facts.scratch.nodes.get_mut(node) else {
            return Err(walker.error(*node, "extended expression lacks scope facts"));
        };
        info.temporary = *lifetime;
        info.extended = true;
    }
    Ok(walker.facts)
}
impl Walker<'_> {
    fn child(&self, index: u32) -> NodeId {
        self.report
            .package
            .unit(self.source.file)
            .configured
            .ast
            .edges[index as usize]
    }
    fn kind(&self, node: NodeId) -> Syntax {
        self.report
            .package
            .unit(self.source.file)
            .configured
            .ast
            .node(node)
            .kind
    }
    fn error(&self, node: NodeId, message: &'static str) -> Error {
        let offset = self
            .report
            .package
            .unit(self.source.file)
            .configured
            .ast
            .node(node)
            .span
            .start;
        Error::at(self.source.file, offset, message)
    }
    fn push(&mut self, work: Work) -> Result<(), Error> {
        let (Work::Expression(node, _)
        | Work::Statement(node, _)
        | Work::Condition(node, _)
        | Work::Pattern(node, _)
        | Work::ExtendExpression(node, _)
        | Work::ExtendPlace(node, _)) = work;
        if !self.active(node) {
            return Ok(());
        }
        if self.facts.work >= 1_000_000 {
            return Err(self.error(self.source.node, "temporary scope work limit exceeded"));
        }
        self.facts.work += 1;
        self.facts.scratch.pending.push(work);
        Ok(())
    }
    fn active(&self, node: NodeId) -> bool {
        self.report.package.unit(self.source.file).configured.active[node.0 as usize]
    }
    fn record(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        let info = Node {
            scope: context.scope,
            temporary: context.temporary,
            extended: false,
        };
        if self.facts.scratch.nodes.insert(node, info).is_some() {
            return Err(self.error(node, "expression has repeated scope ownership"));
        }
        Ok(())
    }
    fn scope(&mut self, node: NodeId, kind: Kind, context: Context) -> Result<Context, Error> {
        let id = Id(u32::try_from(self.facts.scopes.len())
            .map_err(|_| self.error(node, "too many temporary scopes"))?);
        self.facts.scopes.push(Scope {
            parent: Some(context.scope),
            source: node,
            kind,
        });
        if self
            .facts
            .scratch
            .boundaries
            .insert((node, kind), id)
            .is_some()
        {
            return Err(self.error(node, "repeated scope boundary identity"));
        }
        Ok(Context {
            scope: id,
            temporary: Lifetime::Scope(id),
            ..context
        })
    }
    fn expressions(&mut self, list: List, context: Context) -> Result<(), Error> {
        for index in (list.start..list.end).rev() {
            self.push(Work::Expression(self.child(index), context))?;
        }
        Ok(())
    }
    fn visit(&mut self, work: Work) -> Result<(), Error> {
        match work {
            Work::Expression(node, context) => self.expression(node, context),
            Work::Statement(node, context) => self.statement(node, context),
            Work::Condition(node, context) => self.condition(node, context),
            Work::Pattern(node, scope) => self.pattern(node, scope),
            Work::ExtendExpression(node, lifetime) => self.extend_expression(node, lifetime),
            Work::ExtendPlace(node, lifetime) => self.extend_place(node, lifetime),
        }
    }
}
