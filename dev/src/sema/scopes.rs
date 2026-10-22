//! Rust 2024 temporary scopes, distinct from promotion and inferred loan regions.
mod controls;
mod extend;
mod walk;
use crate::support::hash::HashMap;
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::model::{Declarations, Function, Initializer},
    syntax::ast::{Kind as Syntax, List, NodeId, Unary},
};

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
    index: NodeIndex,
    /// Expression scope facts in recording order, located through `index`.
    nodes: Vec<Node>,
    /// Next scope with the same source node, parallel to the scope records.
    same_source: Vec<u32>,
    /// Spare scope-record capacity; finished bodies receive exact copies.
    scopes: Vec<Scope>,
    bindings: HashMap<DefId, Id>,
    pending: Vec<Work>,
    pattern_work: Vec<NodeId>,
    extensions: HashMap<NodeId, Lifetime>,
}
const NONE: u32 = u32::MAX;
/// Nodes this far below the root use the sparse table instead of the window.
const DENSE_SPAN: u32 = 1 << 20;
/// Per-body slots keyed by syntax node: a recorded expression fact and the
/// first scope whose boundary is that node. Children are allocated before their
/// parents, so a body's nodes are dense below its root; any other node, such as
/// later expansion output, uses the sparse table.
#[derive(Default)]
struct NodeIndex {
    root: u32,
    window: Vec<[u32; 2]>,
    sparse: HashMap<NodeId, [u32; 2]>,
}
impl NodeIndex {
    fn reset(&mut self, root: NodeId) {
        self.root = root.get();
        self.window.clear();
        recycle(&mut self.sparse);
    }
    fn get(&self, node: NodeId) -> [u32; 2] {
        match self.root.checked_sub(node.get()) {
            Some(offset) if offset < DENSE_SPAN => self
                .window
                .get(offset as usize)
                .copied()
                .unwrap_or([NONE; 2]),
            _ => self.sparse.get(&node).copied().unwrap_or([NONE; 2]),
        }
    }
    fn slot(&mut self, node: NodeId) -> &mut [u32; 2] {
        match self.root.checked_sub(node.get()) {
            Some(offset) if offset < DENSE_SPAN => {
                let offset = offset as usize;
                if offset >= self.window.len() {
                    self.window.resize(offset + 1, [NONE; 2]);
                }
                &mut self.window[offset]
            }
            _ => self.sparse.entry(node).or_insert([NONE; 2]),
        }
    }
}
impl Facts {
    pub(crate) fn into_scopes(self) -> Vec<Scope> {
        self.scopes
    }
    pub(crate) fn synthetic(source: NodeId) -> Self {
        Self::with_scratch(source, Scratch::default())
    }
    fn with_scratch(source: NodeId, mut scratch: Scratch) -> Self {
        scratch.index.reset(source);
        scratch.index.slot(source)[1] = 0;
        scratch.same_source.push(NONE);
        let mut scopes = std::mem::take(&mut scratch.scopes);
        scopes.push(Scope {
            parent: None,
            source,
            kind: Kind::Body,
        });
        Self {
            scopes,
            scratch,
            work: 0,
        }
    }
    pub(crate) fn finish(mut self) -> (Vec<Scope>, Scratch) {
        self.scratch.index.reset(NodeId::new(0));
        self.scratch.nodes.clear();
        self.scratch.same_source.clear();
        recycle(&mut self.scratch.bindings);
        self.scratch.pending.clear();
        self.scratch.pattern_work.clear();
        recycle(&mut self.scratch.extensions);
        let scopes = self.scopes.as_slice().to_vec();
        self.scopes.clear();
        self.scratch.scopes = self.scopes;
        (scopes, self.scratch)
    }
    #[must_use]
    pub fn boundary(&self, source: NodeId, kind: Kind) -> Option<Id> {
        let mut id = self.scratch.index.get(source)[1];
        while id != NONE {
            if self.scopes[id as usize].kind == kind {
                return Some(Id(id));
            }
            id = self.scratch.same_source[id as usize];
        }
        None
    }
    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }
    #[must_use]
    pub fn node(&self, node: NodeId) -> Option<Node> {
        let entry = self.scratch.index.get(node)[0];
        self.scratch.nodes.get(entry as usize).copied()
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
        *table = HashMap::default();
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
    types: &'a Declarations,
    source: NodeKey,
    /// The owner file's configured syntax, resolved once per body.
    syntax: &'a crate::expand::crate_syntax::ConfiguredSyntax<crate::literal::storage::StoredValue>,
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
        types,
        source,
        syntax: &report.package.unit(source.file).configured,
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
        let entry = walker.facts.scratch.index.get(*node)[0];
        let Some(info) = walker.facts.scratch.nodes.get_mut(entry as usize) else {
            return Err(walker.error(*node, "extended expression lacks scope facts"));
        };
        info.temporary = *lifetime;
        info.extended = true;
    }
    Ok(walker.facts)
}
impl Walker<'_> {
    fn child(&self, index: u32) -> NodeId {
        self.syntax.ast.edges[index as usize]
    }
    fn kind(&self, node: NodeId) -> Syntax {
        self.syntax.ast.node(node).kind
    }
    fn error(&self, node: NodeId, message: &'static str) -> Error {
        let offset = self.syntax.ast.node(node).span.start;
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
        self.syntax.active[node.get() as usize]
    }
    fn record(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        let info = Node {
            scope: context.scope,
            temporary: context.temporary,
            extended: false,
        };
        let entry = u32::try_from(self.facts.scratch.nodes.len())
            .ok()
            .filter(|&entry| entry != NONE)
            .ok_or_else(|| self.error(node, "too many temporary scope facts"))?;
        let slot = &mut self.facts.scratch.index.slot(node)[0];
        if *slot != NONE {
            return Err(self.error(node, "expression has repeated scope ownership"));
        }
        *slot = entry;
        self.facts.scratch.nodes.push(info);
        Ok(())
    }
    fn scope(&mut self, node: NodeId, kind: Kind, context: Context) -> Result<Context, Error> {
        let id = Id(u32::try_from(self.facts.scopes.len())
            .ok()
            .filter(|&id| id != NONE)
            .ok_or_else(|| self.error(node, "too many temporary scopes"))?);
        if self.facts.boundary(node, kind).is_some() {
            return Err(self.error(node, "repeated scope boundary identity"));
        }
        self.facts.scopes.push(Scope {
            parent: Some(context.scope),
            source: node,
            kind,
        });
        let first = &mut self.facts.scratch.index.slot(node)[1];
        let next = std::mem::replace(first, id.0);
        self.facts.scratch.same_source.push(next);
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
