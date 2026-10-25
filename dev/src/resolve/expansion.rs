//! Crate expansion visits new typed syntax and retains module-loading and capture state.
use super::{
    index::Context,
    load::{InputFile, Loader, Sources},
    model::{Resolution, ScopeId},
    package::{Error, ModuleId, Package},
    unit::{FileId, NodeKey, editions},
};
use crate::{
    expand::{
        builtin::{self, Builtin},
        config::Config,
        crate_syntax,
        macros::{
            compile,
            input::Input,
            matcher::{Compiled, Workspace},
            prepare, transcribe,
        },
        marks::{ExpansionId, SyntaxContextId, Transparency},
        registry::Registry,
        stream::{Environment, Location, Origin, Source, Stream},
        textual::{Lookup, Observer, Scope, Scopes},
    },
    support::{edition::Edition, names::NameId},
    syntax::{
        ast::{Kind, MacroContext, NodeId},
        edition_context::WithEditions,
        expression::fragment,
        item::{ItemKind, Visibility},
        token_tree::Span,
    },
};
use std::collections::VecDeque;
mod application_kind;
pub(crate) mod attribute_effects;
pub(super) mod attributes;
mod bindings;
mod cfg_gates;
mod decorated_calls;
mod diagnostics;
mod eager;
pub(crate) mod removed_functions;
use application_kind::CallGrammar;
mod format_values;
mod location;
mod panic_dispatch;
mod permission_frames;
mod permission_session;
pub use permission_session::MacroAllowanceSession;
mod choice_dispatch;
mod paths;
mod resources;
#[cfg(test)]
#[path = "../../tests/invariants/role_declarations.rs"]
mod role_declarations;
mod transcription;
mod unavailable;

pub enum Body {
    Declarative(Compiled),
    Builtin(Builtin),
    /// A prepared item-scoped macro keeps its declared literal transparency.
    Scoped {
        compiled: Box<Compiled>,
        mode: Transparency,
    },
}
#[derive(Clone, Copy)]
enum MacroTarget {
    Declarative(usize),
    SourceAttribute(usize),
    Builtin(Builtin),
    SourceDerive(usize),
}
impl MacroTarget {
    fn kind(self) -> builtin::BuiltinKind {
        match self {
            Self::Declarative(_) => builtin::BuiltinKind::Bang,
            Self::SourceAttribute(_) => builtin::BuiltinKind::Attribute,
            Self::SourceDerive(_) => builtin::BuiltinKind::Derive,
            Self::Builtin(role) => role.kind(),
        }
    }
}
pub struct Definition {
    pub node: Option<NodeKey>,
    pub module: ModuleId,
    pub name: NameId,
    pub body: Body,
    pub context: SyntaxContextId,
    pub introduction: ExpansionId,
    pub preceding_scope: Scope,
    pub local_inner_macros: bool,
    pub edition: Edition,
    pub edition_panic: bool,
    pub(super) header: Option<std::rc::Rc<super::bindings::panic_contracts::HeaderData>>,
}
pub struct Invocation {
    pub context: ExpansionId,
    pub node: NodeKey,
    pub original: Span,
    pub definition: usize,
    pub rule: Option<usize>,
    pub generated: Span,
    pub parser_calls: usize,
    pub parent: ExpansionId,
}
pub struct Report {
    pub package: Package,
    pub definitions: Vec<Definition>,
    pub invocations: Vec<Invocation>,
    pub captures: Registry,
    pub textual_scopes: Scopes,
    pub resolution: Resolution,
    /// The import worklist at its fixpoint over the scan's declarations, and
    /// the declarations the scan left to resolution in scan order.
    pub(crate) imports: Option<super::imports::Work>,
    pub(crate) pending: Vec<super::index::Pending>,
    pub(crate) names_resolved: bool,
    pub(crate) attribute_effects: Option<Box<attribute_effects::Effects>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Visit {
    module: ModuleId,
    node: NodeId,
    owner: Option<NodeId>,
    in_block: bool,
    depth: usize,
    introduction: ExpansionId,
    path_scope: ScopeId,
    eager: Option<usize>,
    include: Option<FileId>,
    foreign: Option<NodeKey>,
}
/// Worklist entries stay small: siblings share one stored traversal frame.
enum Work {
    /// Visit `node` with the traversal state stored in `frames[frame]`.
    Visit {
        node: NodeId,
        frame: u32,
    },
    /// Release frames above this mark; every visit sharing them has finished.
    Frames(u32),
    Restore(Scope),
    Finish {
        output: Option<Scope>,
        restore: Scope,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Unresolved,
    Waiting,
    InProgress,
    Complete,
    Removed,
    Failed,
}
impl State {
    fn finished(self) -> bool {
        matches!(self, Self::Complete | Self::Removed)
    }
}
struct Call {
    visit: Visit,
    file: FileId,
    capture: u32,
    grammar: CallGrammar,
    absolute: bool,
    path: super::paths::Saved,
    scope: Scope,
    output: Option<Scope>,
    state: State,
    queued: bool,
    selected: Option<usize>,
    error: Option<Error>,
    offset: u32,
    eager: Option<eager::Arguments>,
    notified: bool,
}
impl Call {
    fn diagnostic(&self, message: &str) -> Error {
        let mut error = Error::at(self.file, self.offset, message);
        error.introduction = self.visit.introduction;
        error
    }
}
struct Engine<'a, S> {
    loader: Loader<'a, S>,
    config: &'a Config,
    definitions: Vec<Definition>,
    invocations: Vec<Invocation>,
    captures: Registry,
    scopes: Scopes,
    head: Scope,
    calls: Vec<Call>,
    ready: VecDeque<usize>,
    paths: paths::Paths,
    workspace: Workspace,
    work: Vec<Work>,
    frames: Vec<Visit>,
    children: Vec<NodeId>,
    recursion_limit: usize,
    builtin_definitions: Vec<Option<usize>>,
    cfg_memo: Option<Box<permission_session::Memo>>,
    attribute_effects: Option<Box<attribute_effects::Effects>>,
}
/// Expand textual macro invocations, including generated syntax and modules.
/// PA8's path-resolution and hygiene integration builds on this retained state.
///
/// # Errors
/// Reports source/loading errors, invalid macro definitions, unresolved calls,
/// malformed generated syntax and bounded expansion recursion/work failures.
pub fn analyze(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
) -> Result<Report, Error> {
    analyze_with_crate_name(root, sources, config, None)
}

/// Expand a crate with an explicit driver-supplied crate name when present.
/// # Errors
/// Reports expansion failures and invalid/conflicting crate names.
pub fn analyze_with_crate_name(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
    crate_name: Option<&str>,
) -> Result<Report, Error> {
    analyze_with_crates(root, sources, config, crate_name, None)
}

/// Expand with a fully validated external graph before resolving any source path.
/// # Errors
/// Reports the ordinary expansion errors and invalid external binding installation.
pub fn analyze_with_crates(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
    crate_name: Option<&str>,
    dependencies: Option<std::sync::Arc<crate::support::crates::Dependencies>>,
) -> Result<Report, Error> {
    let context = super::bindings::Context {
        crate_name: crate_name.map(str::to_owned),
        dependencies,
        ..Default::default()
    };
    analyze_with_context(root, sources, config, &context)
}
/// Source ownership is fixed before expansion and survives ordinary resolution.
/// # Errors
/// Reports ordinary expansion errors and invalid source-provider admission.
pub fn analyze_with_context(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
    context: &super::bindings::Context,
) -> Result<Report, Error> {
    let mut engine = Engine::create(root, sources, config, context)?;
    let result = engine.run();
    result.map_err(|error| engine.diagnostic(error))?;
    super::bindings::validate_features(&engine.loader.package)
        .map_err(|error| engine.diagnostic(error))?;
    if let Some(producer) = engine
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.as_mut())
    {
        producer.finish();
    }
    let (resolution, imports, pending) = engine.paths.finish();
    let report = Report {
        package: engine.loader.package,
        definitions: engine.definitions,
        invocations: engine.invocations,
        captures: engine.captures,
        textual_scopes: engine.scopes,
        resolution,
        imports: Some(imports),
        pending,
        names_resolved: false,
        attribute_effects: engine.attribute_effects,
    };
    super::bindings::validate_crate_options(&report).map_err(|error| report.diagnostic(error))?;
    cfg_gates::validate_attributes(&report)?;
    Ok(report)
}
impl<'a, S: Sources> Engine<'a, S> {
    /// Shared actual creation/initial-root producer; tests borrow this same
    /// engine after a failed use without reconstructing its worklist or owner.
    fn create(
        root: InputFile,
        sources: &'a mut S,
        config: &'a Config,
        context: &super::bindings::Context,
    ) -> Result<Self, Error> {
        let mut loader = Loader::with_context(root, sources, config, context)?;
        if !loader.package.uses_source_bindings() {
            loader.package.crate_name =
                attributes::crate_name(&loader.package, context.crate_name.as_deref())?;
        }
        loader.isolate_expansions();
        let root = loader.package.modules[0].root;
        let recursion_limit = attributes::recursion_limit(&loader.package)?;
        let paths = paths::Paths::new(&mut loader.package, context.dependencies.clone())?;
        let mut engine = Self {
            loader,
            config,
            definitions: Vec::new(),
            invocations: Vec::new(),
            captures: Registry::default(),
            scopes: Scopes::default(),
            head: Scope::ROOT,
            calls: Vec::new(),
            ready: VecDeque::new(),
            paths,
            workspace: Workspace::default(),
            work: Vec::new(),
            frames: Vec::new(),
            children: Vec::new(),
            recursion_limit,
            builtin_definitions: vec![None; Builtin::BOOTSTRAP.len()],
            attribute_effects: None,
            cfg_memo: None,
        };
        engine
            .push_visit(Visit {
                module: ModuleId(0),
                node: root,
                owner: None,
                in_block: false,
                depth: 0,
                introduction: ExpansionId(0),
                path_scope: ScopeId(0),
                eager: None,
                include: None,
                foreign: None,
            })
            .map_err(|error| engine.diagnostic(error))?;
        Ok(engine)
    }
    fn run(&mut self) -> Result<(), Error> {
        self.scan()?;
        while let Some(index) = self.ready.pop_front() {
            self.calls[index].queued = false;
            self.attempt(index)?;
        }
        if let Some(index) = self
            .calls
            .iter()
            .position(|call| call.error.is_some())
            .or_else(|| {
                self.calls
                    .iter()
                    .position(|call| !call.state.finished() && call.eager.is_none())
            })
            .or_else(|| self.calls.iter().position(|call| !call.state.finished()))
        {
            if let Some(error) = self.calls[index].error.take() {
                return Err(error);
            }
            let message = if matches!(self.calls[index].grammar, CallGrammar::Attribute) {
                "unresolved attribute macro is not in path scope"
            } else {
                "unresolved macro is not in textual scope or path scope"
            };
            return Err(self.calls[index].diagnostic(message));
        }
        if let Some(error) = self.paths.take_error() {
            return Err(error);
        }
        Ok(())
    }
    fn wake(&mut self, index: usize) {
        if !self.calls[index].queued {
            self.calls[index].queued = true;
            self.ready.push_back(index);
        }
    }
    /// Queue a traversal root in its own frame.
    fn push_visit(&mut self, visit: Visit) -> Result<(), Error> {
        self.push_visit_with_tail(visit, None)
    }
    fn push_visit_with_tail(
        &mut self,
        visit: Visit,
        tail: Option<super::lint_policy::producer::Tail>,
    ) -> Result<(), Error> {
        let frame = self.frame_mark()?;
        self.lint_set_frame(frame, visit, tail)?;
        self.work.push(Work::Frames(frame));
        self.frames.push(visit);
        self.work.push(Work::Visit {
            node: visit.node,
            frame,
        });
        Ok(())
    }
    fn frame_mark(&self) -> Result<u32, Error> {
        u32::try_from(self.frames.len())
            .map_err(|_| Error::at(FileId(0), 0, "expansion traversal frames exceed 32 bits"))
    }
    fn scan(&mut self) -> Result<(), Error> {
        while let Some(work) = self.work.pop() {
            match work {
                Work::Visit { node, frame } => {
                    let visit = Visit {
                        node,
                        ..self.frames[frame as usize]
                    };
                    self.visit(visit, frame).map_err(|mut error| {
                        if error.introduction.0 == 0 {
                            error.introduction = visit.introduction;
                        }
                        error
                    })?;
                }
                Work::Frames(mark) => {
                    self.frames.truncate(mark as usize);
                    if let Some(producer) = self
                        .attribute_effects
                        .as_deref_mut()
                        .and_then(|effects| effects.policy.as_mut())
                    {
                        producer.truncate(mark);
                    }
                }
                Work::Restore(head) => self.head = head,
                Work::Finish { output, restore } => {
                    if let Some(output) = output {
                        for index in self
                            .scopes
                            .complete(output, self.head)
                            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?
                        {
                            match index {
                                Observer::Invocation(call) => self.wake(call),
                                Observer::Import(import) => self.paths.wake_textual(import),
                            }
                        }
                    }
                    self.head = restore;
                }
            }
        }
        for call in self.paths.drive(
            &mut self.loader.package,
            &mut self.scopes,
            bindings::Bindings {
                definitions: &self.definitions,
            },
        )? {
            self.wake(call);
        }
        Ok(())
    }
    fn macro_path(
        &mut self,
        file: FileId,
        path: NodeId,
    ) -> Result<(bool, super::paths::Buffer), Error> {
        let mut names = super::paths::Buffer::new(Vec::new());
        let absolute = self.loader.package.append_path(file, path, &mut names)?;
        if !absolute && names.len() == 1 {
            let offset =
                super::hygiene::name_offset(&self.loader.package, NodeKey { file, node: path });
            let context = self.context(file, offset)?;
            let local_inner = self
                .loader
                .package
                .marks
                .outer(context)
                .map_err(|message| Error::at(file, offset, message))?
                .and_then(|expansion| expansion.0.checked_sub(1))
                .and_then(|index| self.invocations.get(index as usize))
                .is_some_and(|call| self.definitions[call.definition].local_inner_macros);
            if local_inner {
                let root = self
                    .loader
                    .package
                    .names
                    .intern("crate")
                    .map_err(|e| Error::at(file, offset, e))?;
                names.prepend_root(root);
            }
        }
        Ok((absolute, names))
    }
    fn register(
        &mut self,
        visit: Visit,
        file: FileId,
        path: NodeId,
        capture: u32,
        grammar: MacroContext,
        frame: u32,
    ) -> Result<(), Error> {
        let (absolute, path_names) = self.macro_path(file, path)?;
        let offset = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(visit.node)
            .span
            .start;
        let scope = self.head;
        let output = if matches!(grammar, MacroContext::Items(_) | MacroContext::Statements) {
            let output = self
                .scopes
                .output()
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            self.head = output;
            Some(output)
        } else {
            None
        };
        let index = self.calls.len();
        if let Some(parent) = visit.eager {
            self.calls[parent]
                .eager
                .as_mut()
                .ok_or_else(|| Error::at(file, offset, "missing eager expansion parent"))?
                .pending += 1;
        }
        self.ready.push_back(index);
        self.calls.push(Call {
            visit,
            file,
            capture,
            grammar: CallGrammar::Bang(grammar),
            absolute,
            path: path_names.save(),
            scope,
            output,
            state: State::Unresolved,
            queued: true,
            selected: None,
            error: None,
            offset,
            eager: None,
            notified: false,
        });
        self.lint_save_call(index, frame)?;
        Ok(())
    }
    fn attempt(&mut self, index: usize) -> Result<(), Error> {
        let found = match self.lookup_call(index) {
            Ok(found) => {
                self.calls[index].error = None;
                found
            }
            Err(mut error) => {
                if error.introduction.0 == 0 {
                    error.introduction = self.calls[index].visit.introduction;
                }
                self.calls[index].error = Some(error);
                None
            }
        };
        if let Some(definition) = found {
            let call = &self.calls[index];
            self.require_call_definition(call, definition)?;
        }
        if let Some(previous) = self.calls[index].selected {
            if found == Some(previous) {
                if self.calls[index].state == State::Removed {
                    return Ok(());
                }
                if let Some(arguments) = self.calls[index].eager.as_ref() {
                    if arguments.pending == 0 {
                        self.finish_eager(index, previous)?;
                    }
                } else {
                    self.completed(index)?;
                }
            } else {
                self.calls[index].state = State::Failed;
                if self.calls[index].error.is_none() {
                    self.calls[index].error =
                        Some(self.calls[index].diagnostic("macro path changed after expansion"));
                }
            }
            return Ok(());
        }
        let Some(definition) = found else {
            self.calls[index].state = State::Waiting;
            return Ok(());
        };
        if matches!(
            self.definitions[definition].body,
            Body::Builtin(Builtin::Test)
        ) {
            return self.remove_test_function(index, definition);
        }
        self.calls[index].state = State::InProgress;
        if matches!(self.definitions[definition].body, Body::Builtin(builtin) if builtin.eager()) {
            return self.begin_eager(index, definition);
        }
        let call = &self.calls[index];
        let (visit, file, capture, grammar, scope, output, parent) = (
            call.visit,
            call.file,
            call.capture,
            call.grammar
                .bang()
                .map_err(|message| call.diagnostic(message))?,
            call.scope,
            call.output,
            call.visit.introduction,
        );
        let introduction = match self.invoke(visit, file, capture, grammar, definition, parent) {
            Ok(context) => context,
            Err(error) => return Err(self.failed_invocation(index, definition, error)),
        };
        self.calls[index].selected = Some(definition);
        self.work.push(Work::Finish {
            output,
            restore: self.head,
        });
        self.head = scope;
        let tail = self.lint_call_tail(index)?;
        self.push_visit_with_tail(
            Visit {
                depth: visit.depth + 1,
                introduction,
                ..visit
            },
            tail,
        )?;
        self.scan()?;
        self.completed(index)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one typed-node visit coordinates declaration, module and expansion scope transitions"
    )]
    fn visit(&mut self, mut visit: Visit, frame: u32) -> Result<(), Error> {
        let module = &self.loader.package.modules[visit.module.0 as usize];
        let (file, module_root) = (module.file, module.root);
        let unit = self.loader.package.unit(file);
        if !unit.configured.active[visit.node.get() as usize] {
            return Ok(());
        }
        let kind = unit.configured.ast.node(visit.node).kind;
        if matches!(kind, Kind::Attributed { .. }) && self.attribute_call(visit, file, frame)? {
            return Ok(());
        }
        let tail = if matches!(kind, Kind::Attributed { .. }) {
            self.lint_owner(visit, frame)?
        } else {
            self.lint_frame(
                frame,
                NodeKey {
                    file,
                    node: visit.node,
                },
            )?
        };
        let mut item_scope = None;
        if let Kind::Item(index) = kind {
            let item = self.loader.package.unit(file).configured.ast.items[index as usize];
            if let ItemKind::MacroRules { capture } = item.kind {
                return self.define(visit, file, index, capture);
            }
            if matches!(item.kind, ItemKind::DeclarativeMacro { .. }) {
                return self.define_scoped(visit, file, item);
            }
            if let ItemKind::Use(tree) = item.kind {
                return self.paths.import(
                    &mut self.loader.package,
                    Context {
                        module: visit.module,
                        file,
                        scope: visit.path_scope,
                        foreign: visit.foreign,
                    },
                    tree,
                    item.visibility,
                    (self.head, visit.introduction),
                );
            }
            if matches!(item.kind, ItemKind::Module { .. }) && visit.node != module_root {
                return self.module(visit, file, item.visibility, tail);
            }
            if !matches!(item.kind, ItemKind::Module { .. }) {
                item_scope = Some(self.paths.item(
                    &mut self.loader.package,
                    Context {
                        module: visit.module,
                        file,
                        scope: visit.path_scope,
                        foreign: visit.foreign,
                    },
                    visit.node,
                    item,
                    visit.introduction,
                )?);
            }
        }
        if let Kind::Item(index) = kind {
            let item = self.loader.package.unit(file).configured.ast.items[index as usize];
            visit.foreign = Context {
                module: visit.module,
                file,
                scope: visit.path_scope,
                foreign: visit.foreign,
            }
            .foreign_children(visit.node, item.kind);
        }
        if let Kind::Macro {
            path,
            capture,
            context,
        } = kind
        {
            if visit.depth >= self.recursion_limit {
                return Err(Error::at(
                    file,
                    self.loader
                        .package
                        .unit(file)
                        .configured
                        .ast
                        .node(visit.node)
                        .span
                        .start,
                    format!("macro expansion recursion exceeds {}", self.recursion_limit),
                ));
            }
            if self.invocations.len() >= 1_000_000 {
                return Err(Error::at(file, 0, "crate expansion work budget exceeded"));
            }
            return self.register(visit, file, path, capture, context, frame);
        }
        // The alternative of an `if` stays outside the scope of its condition.
        let mut outside = None;
        let context = Context {
            module: visit.module,
            file,
            scope: visit.path_scope,
            foreign: visit.foreign,
        };
        if let Some(scope) = item_scope {
            if scope != visit.path_scope {
                self.work.push(Work::Restore(self.head));
                visit.path_scope = scope;
            }
        } else if matches!(kind, Kind::Block { .. }) {
            self.work.push(Work::Restore(self.head));
            visit.path_scope = self.paths.block(
                &mut self.loader.package,
                visit.path_scope,
                visit.module,
                visit.node,
                visit.introduction,
            )?;
        } else if let Kind::Generic(index) = kind {
            self.paths.generic(context, visit.node, index);
        } else if matches!(
            kind,
            Kind::If { .. }
                | Kind::Closure { .. }
                | Kind::Arm { .. }
                | Kind::For { .. }
                | Kind::While { .. }
                | Kind::Label { .. }
                | Kind::Bound { .. }
                | Kind::Where { .. }
                | Kind::FunctionType(_)
        ) {
            if let Kind::If { otherwise, .. } = kind {
                outside = otherwise.map(|node| (node, visit.path_scope));
            }
            visit.path_scope = self.paths.lexical(
                &mut self.loader.package,
                context,
                visit.node,
                visit.introduction,
            )?;
        }
        self.children.clear();
        self.loader
            .package
            .unit(file)
            .configured
            .ast
            .push_children(visit.node, &mut self.children);
        // Leaves have no declarations, expansions or descendants to visit.
        let ast = &self.loader.package.unit(file).configured.ast;
        self.children
            .retain(|node| !super::index::is_inert(ast, *node));
        if self.children.is_empty() {
            return Ok(());
        }
        let child = Visit {
            owner: matches!(kind, Kind::Attributed { .. }).then_some(visit.node),
            in_block: visit.in_block || matches!(kind, Kind::Block { .. }),
            ..visit
        };
        if let Some((otherwise, path_scope)) = outside
            && self.children.last() == Some(&otherwise)
        {
            // Visited last, after the condition's scope frame is released.
            self.children.pop();
            let shared = self.share_frame(
                frame,
                Visit {
                    path_scope,
                    ..child
                },
                tail,
            )?;
            self.work.push(Work::Visit {
                node: otherwise,
                frame: shared,
            });
        }
        let shared = self.share_frame(frame, child, tail)?;
        self.work
            .extend(self.children.iter().rev().map(|node| Work::Visit {
                node: *node,
                frame: shared,
            }));
        Ok(())
    }
    /// The frame for children visited with `child`: the parent's own frame
    /// when they inherit it unchanged, otherwise a new one released after them.
    fn share_frame(
        &mut self,
        frame: u32,
        child: Visit,
        tail: Option<super::lint_policy::producer::Tail>,
    ) -> Result<u32, Error> {
        let parent = self.frames[frame as usize];
        if (Visit {
            node: parent.node,
            ..child
        }) == parent
            && self.lint_frame(frame, self.visit_key(child))? == tail
        {
            return Ok(frame);
        }
        let mark = self.frame_mark()?;
        self.work.push(Work::Frames(mark));
        self.lint_set_frame(mark, child, tail)?;
        self.frames.push(child);
        Ok(mark)
    }
    fn module(
        &mut self,
        visit: Visit,
        file: FileId,
        visibility: Visibility,
        tail: Option<super::lint_policy::producer::Tail>,
    ) -> Result<(), Error> {
        let child = self.loader.load_module(
            visit.module,
            visit.node,
            visit.owner,
            visit.in_block,
            visit.include,
        )?;
        let path_scope = self.paths.module(
            &mut self.loader.package,
            Context {
                module: visit.module,
                file,
                scope: visit.path_scope,
                foreign: visit.foreign,
            },
            child,
            visit.node,
            visibility,
            visit.introduction,
        )?;
        if !self.flags(file, visit.owner)?.macro_use {
            self.work.push(Work::Restore(self.head));
        }
        self.push_visit_with_tail(
            Visit {
                module: child,
                node: self.loader.package.modules[child.0 as usize].root,
                owner: None,
                in_block: false,
                include: None,
                foreign: None,
                path_scope,
                ..visit
            },
            tail,
        )
    }
    fn define(
        &mut self,
        visit: Visit,
        file: FileId,
        index: u32,
        capture: u32,
    ) -> Result<(), Error> {
        let flags = self.flags(file, visit.owner)?;
        let contract = self.definition_contract(visit, file)?;
        let builtin = contract
            .as_ref()
            .and_then(|contract| contract.declaration().builtin);
        let context = self.context(
            file,
            self.loader
                .package
                .unit(file)
                .configured
                .ast
                .node(visit.node)
                .span
                .start,
        )?;
        let package = &mut self.loader.package;
        let unit = &package.units[file.0 as usize];
        let item = unit.configured.ast.items[index as usize];
        let span = item
            .name
            .ok_or_else(|| Error::at(file, 0, "macro definition has no name"))?;
        let spelling = unit
            .spelling(span)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let name = package
            .names
            .intern(spelling)
            .map_err(|e| Error::at(file, span.start, e))?;
        let declaration = unit.configured.ast.node(visit.node).span.start;
        let edition = editions::at(
            Location {
                file,
                span: Span {
                    start: declaration,
                    end: declaration + 1,
                },
            },
            |id| package.units.get(id.0 as usize),
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        let definition = compile::in_edition(
            unit.source.text(),
            &unit.configured.ast.captures[capture as usize],
            &mut package.names,
            edition,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        let index = self.definitions.len();
        let name_context = package.item_context(file, span.start)?;
        self.definitions.push(Definition {
            node: Some(NodeKey {
                file,
                node: visit.node,
            }),
            module: visit.module,
            name,
            body: builtin.map_or_else(
                || Body::Declarative(Compiled::new(definition)),
                Body::Builtin,
            ),
            context,
            introduction: visit.introduction,
            preceding_scope: self.head,
            local_inner_macros: flags.local_inner_macros,
            edition,
            edition_panic: contract.as_ref().is_some_and(|contract| {
                contract
                    .declaration()
                    .allows(super::bindings::KnownFeature::EditionPanic)
            }),
            header: contract,
        });
        self.head = self
            .scopes
            .define_context(self.head, name, name_context, index)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let context = Context {
            module: visit.module,
            file,
            scope: visit.path_scope,
            foreign: visit.foreign,
        };
        self.paths.definition(
            &mut self.loader.package,
            context,
            visit.node,
            name,
            flags.exported,
            (index, visit.introduction),
        )?;
        self.paths
            .macro_visibility(context, visit.node, item.visibility);
        Ok(())
    }
    fn lookup_call(&mut self, index: usize) -> Result<Option<usize>, Error> {
        let call = &self.calls[index];
        if matches!(call.grammar, CallGrammar::Attribute) {
            decorated_calls::validate(&self.loader.package, &self.paths.state.facts, call)?;
        }
        if call.absolute {
            return Err(call.diagnostic("external macro paths require external metadata"));
        }
        if matches!(call.grammar, CallGrammar::Bang(_)) && call.path.len() == 1 {
            let name = call.path[0];
            match (bindings::Bindings {
                definitions: &self.definitions,
            })
            .textual(
                &mut self.scopes,
                &self.paths.state.facts.introductions.ancestry,
                call.scope,
                (
                    name,
                    call.path
                        .view()
                        .context(0)
                        .map_err(|e| call.diagnostic(e))?,
                ),
                call.visit.introduction,
            )
            .map_err(|e| self.calls[index].diagnostic(e.message))?
            {
                Lookup::Found(definition) => {
                    self.paths.validate_textual(
                        &mut self.loader.package,
                        call,
                        index,
                        definition,
                    )?;
                    return Ok(Some(definition));
                }
                Lookup::Pending(scope) => {
                    self.scopes.watch(scope, Observer::Invocation(index));
                    return Ok(None);
                }
                Lookup::Missing => {}
            }
        }
        let call = &self.calls[index];
        let target = self.paths.lookup(&mut self.loader.package, call, index)?;
        match target {
            Some(
                MacroTarget::Declarative(index)
                | MacroTarget::SourceAttribute(index)
                | MacroTarget::SourceDerive(index),
            ) => Ok(Some(index)),
            Some(MacroTarget::Builtin(builtin)) => self.builtin_definition(builtin).map(Some),
            None => Ok(None),
        }
    }
    fn builtin_definition(&mut self, builtin: Builtin) -> Result<usize, Error> {
        let slot = builtin.bootstrap_slot().ok_or_else(|| {
            Error::at(
                FileId(0),
                0,
                "source-only builtin has no synthetic bootstrap slot",
            )
        })?;
        if let Some(index) = self.builtin_definitions[slot] {
            return Ok(index);
        }
        let name = self
            .loader
            .package
            .names
            .intern(builtin.name())
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        let index = self.definitions.len();
        self.definitions.push(Definition {
            node: None,
            module: ModuleId(0),
            name,
            body: Body::Builtin(builtin),
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(0),
            preceding_scope: Scope::ROOT,
            header: None,
            local_inner_macros: false,
            edition: self.loader.package.units[0].edition(),
            edition_panic: false,
        });
        self.builtin_definitions[slot] = Some(index);
        Ok(index)
    }
    fn context(&self, file: FileId, offset: u32) -> Result<SyntaxContextId, Error> {
        self.loader.package.syntax_context(file, offset)
    }
    fn input(&mut self, file: FileId, capture: u32, docs: bool) -> Result<Input, Error> {
        let unit = &mut self.loader.package.units[file.0 as usize];
        let forest = &unit.configured.ast.captures[capture as usize];
        if docs {
            let range = forest
                .children(0)
                .ok_or_else(|| Error::at(file, 0, "macro invocation requires delimiters"))?;
            return Input::with_docs(forest, range)
                .map_err(|e| Error::at(file, e.offset, e.message));
        }
        let prepared = prepare::if_needed(unit.source.text(), forest)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let prepared = if let Some(prepared) = prepared {
            let base = unit
                .source
                .append(&prepared.source)
                .map_err(|e| Error::at(file, 0, e))?;
            let mapped = prepared
                .forest
                .map_spans(&mut |span| {
                    Ok(Span {
                        start: span.start + base,
                        end: span.end + base,
                    })
                })
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            unit.origins
                .extend(prepared.origins.into_iter().map(|origin| Origin {
                    generated: Span {
                        start: origin.generated.start + base,
                        end: origin.generated.end + base,
                    },
                    original: Location {
                        file,
                        span: origin.original,
                    },
                    context: None,
                    introduction: ExpansionId(0),
                    joint: origin.joint,
                }));
            Some(mapped)
        } else {
            None
        };
        let forest = prepared
            .as_ref()
            .unwrap_or(&unit.configured.ast.captures[capture as usize]);
        let range = forest
            .children(0)
            .ok_or_else(|| Error::at(file, 0, "macro invocation requires delimiters"))?;
        Input::from_forest(forest, range).map_err(|e| Error::at(file, e.offset, e.message))
    }
    fn invoke_builtin(
        &mut self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        input: &Input,
        builtin: Builtin,
    ) -> Result<ExpansionId, Error> {
        let file = self.loader.package.modules[visit.module.0 as usize].file;
        let parent = visit.introduction;
        let original = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(visit.node)
            .span;
        let generated = self.generate_builtin(
            visit.module,
            Location {
                file,
                span: original,
            },
            grammar,
            input,
            builtin,
        )?;
        let generated = self.install_builtin(
            file,
            visit.node,
            original,
            context,
            Transparency::SemiOpaque,
            generated,
        )?;
        self.paths
            .invocation(context, parent, &self.definitions[definition])
            .map_err(|message| Error::at(file, original.start, message))?;
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file,
                node: visit.node,
            },
            original,
            definition,
            rule: None,
            generated,
            parser_calls: 0,
            parent,
        });
        Ok(context)
    }
    fn generate_builtin(
        &mut self,
        module: ModuleId,
        site: Location,
        grammar: MacroContext,
        input: &Input,
        builtin: Builtin,
    ) -> Result<builtin::Expansion, Error> {
        if builtin == Builtin::Cfg {
            return self.cfg_expansion(site.file, input, grammar, site.span);
        }
        let file = site.file;
        let original = site.span;
        let physical =
            if (builtin.needs_position() || builtin == Builtin::File) && input.tokens.is_empty() {
                Some(self.call_location(Location {
                    file,
                    span: original,
                })?)
            } else {
                None
            };
        let position = physical
            .filter(|_| builtin.needs_position())
            .map(|physical| {
                self.loader
                    .package
                    .unit(physical.file)
                    .source
                    .position(physical.span.start)
                    .map_err(|e| Error::at(physical.file, physical.span.start, e))
            })
            .transpose()?;
        let text = if builtin == Builtin::Stringify {
            Some(self.stringify(file, input)?)
        } else if input.tokens.is_empty() {
            match builtin {
                Builtin::File => physical.map(|physical| {
                    self.loader.package.file_paths[physical.file.0 as usize]
                        .to_string_lossy()
                        .into_owned()
                }),
                Builtin::ModulePath => Some(self.module_path(module)),
                _ => None,
            }
        } else {
            None
        };
        let package = &mut self.loader.package;
        let view = self
            .captures
            .view(|id| package.units.get(id.0 as usize).map(|u| u.source.text()));
        builtin::expand(
            builtin,
            package.unit(file).source.text(),
            input,
            &view,
            &builtin::Environment {
                config: self.config,
                grammar,
                position,
                text: text.as_deref(),
            },
        )
        .map_err(|e| {
            Error::at(
                file,
                if e.offset == 0 {
                    original.start
                } else {
                    e.offset
                },
                e.message,
            )
        })
    }
    fn install_builtin(
        &mut self,
        file: FileId,
        target: NodeId,
        original: Span,
        context: ExpansionId,
        mode: Transparency,
        generated: builtin::Expansion,
    ) -> Result<Span, Error> {
        let span = Span {
            start: 0,
            end: u32::try_from(generated.source.len())
                .map_err(|_| Error::at(file, original.start, "builtin output too large"))?,
        };
        let call_site = super::unit::contexts::callee(&self.loader.package.units, file, target)?;
        let syntax = self
            .loader
            .package
            .marks
            .apply(SyntaxContextId::ROOT, context, mode, call_site)
            .map_err(|message| Error::at(file, original.start, message))?;
        let origins = [Origin {
            generated: span,
            original: Location {
                file,
                span: original,
            },
            context: Some(syntax),
            introduction: context,
            joint: None,
        }];
        let generated = self.loader.package.units[file.0 as usize]
            .install_expansion(target, &generated.source, generated.syntax, &origins)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        Ok(generated)
    }
    fn invoke(
        &mut self,
        visit: Visit,
        file: FileId,
        capture: u32,
        grammar: MacroContext,
        definition: usize,
        parent: ExpansionId,
    ) -> Result<ExpansionId, Error> {
        let offset = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(visit.node)
            .span
            .start;
        self.require_invocable(definition, builtin::BuiltinKind::Bang, file, offset)?;
        let docs = matches!(
            self.definitions[definition].body,
            Body::Builtin(Builtin::Stringify)
        );
        let input = self.input(file, capture, docs)?;
        let context = ExpansionId(
            u32::try_from(self.invocations.len() + 1)
                .map_err(|_| Error::at(file, 0, "too many expansion contexts"))?,
        );
        if let Body::Builtin(builtin) = self.definitions[definition].body {
            if builtin == Builtin::CfgSelect {
                return self.invoke_choice(visit, grammar, definition, context, &input);
            }
            if builtin.source_only() {
                return self.invoke_panic(visit, grammar, definition, context, &input, builtin);
            }
            return self.invoke_builtin(visit, grammar, definition, context, &input, builtin);
        }
        let transcription::Transcribed {
            stream,
            rule,
            parser_calls,
        } = self.transcribe(file, visit.node, definition, context, &input)?;
        let package = &mut self.loader.package;
        let units = &package.units;
        let original = units[file.0 as usize].configured.ast.node(visit.node).span;
        let view = self
            .captures
            .view(|id| units.get(id.0 as usize).map(|u| u.source.text()));
        let inputs = WithEditions::new(&view, |token| {
            let parent = editions::parent(
                &stream.origins,
                Span {
                    start: token.start,
                    end: token.end,
                },
            )?;
            editions::at(parent, |id| units.get(id.0 as usize))
        });
        let ast =
            fragment::parse_expansion_with_inputs(&stream.source, &stream.tokens, grammar, &inputs)
                .map_err(|e| diagnostics::stream_error(&stream, file, original, e))?;
        let configured =
            crate_syntax::configure_with_inputs(&stream.source, ast, self.config, &inputs)
                .map_err(|e| diagnostics::stream_error(&stream, file, original, e))?;
        let generated = package.units[file.0 as usize]
            .install_expansion(visit.node, &stream.source, configured, &stream.origins)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        self.paths
            .invocation(context, parent, &self.definitions[definition])
            .map_err(|message| Error::at(file, original.start, message))?;
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file,
                node: visit.node,
            },
            original,
            definition,
            rule: Some(rule),
            generated,
            parser_calls,
            parent,
        });
        Ok(context)
    }
}
