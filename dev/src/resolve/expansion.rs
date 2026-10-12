//! Crate expansion visits new typed syntax and retains module-loading and capture state.
use super::{
    index::Context,
    load::{InputFile, Loader, Sources},
    model::{Resolution, ScopeId},
    package::{Error, ModuleId, Package},
    unit::{FileId, NodeKey},
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
        registry::Registry,
        stream::{ContextId, Environment, Location, Origin, Source, Stream},
        textual::{Lookup, Observer, Scope, Scopes},
    },
    support::names::NameId,
    syntax::{
        ast::{Kind, MacroContext, NodeId},
        expression::fragment,
        item::{ItemKind, Visibility},
        token_tree::Span,
    },
};
use std::collections::VecDeque;
mod attributes;
mod bindings;
mod location;
mod paths;

pub enum Body {
    Declarative(Compiled),
    Builtin(Builtin),
}
#[derive(Clone, Copy)]
enum MacroTarget {
    Declarative(usize),
    Builtin(Builtin),
}
pub struct Definition {
    pub node: Option<NodeKey>,
    pub module: ModuleId,
    pub name: NameId,
    pub body: Body,
    pub context: ContextId,
    pub introduction: ContextId,
    pub preceding_scope: Scope,
    pub local_inner_macros: bool,
}
pub struct Invocation {
    pub context: ContextId,
    pub node: NodeKey,
    pub original: Span,
    pub definition: usize,
    pub rule: Option<usize>,
    pub generated: Span,
    pub parser_calls: usize,
    pub parent: ContextId,
}
pub struct Report {
    pub package: Package,
    pub definitions: Vec<Definition>,
    pub invocations: Vec<Invocation>,
    pub captures: Registry,
    pub textual_scopes: Scopes,
    pub resolution: Resolution,
    pub(crate) names_resolved: bool,
}
#[derive(Clone, Copy)]
struct Visit {
    module: ModuleId,
    node: NodeId,
    owner: Option<NodeId>,
    in_block: bool,
    depth: usize,
    introduction: ContextId,
    path_scope: ScopeId,
}
enum Work {
    Visit(Visit),
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
    Failed,
}
struct Call {
    visit: Visit,
    file: FileId,
    capture: u32,
    grammar: MacroContext,
    absolute: bool,
    path: Vec<NameId>,
    scope: Scope,
    output: Option<Scope>,
    state: State,
    queued: bool,
    selected: Option<usize>,
    error: Option<Error>,
    offset: u32,
}
impl Call {
    fn diagnostic(&self, message: &str) -> Error {
        Error::at(self.file, self.offset, message)
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
    children: Vec<NodeId>,
    recursion_limit: usize,
    builtin_definitions: Vec<Option<usize>>,
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
    let mut loader = Loader::new(root, sources, config)?;
    loader.isolate_expansions();
    let root = loader.package.modules[0].root;
    let recursion_limit = attributes::recursion_limit(&loader.package)?;
    let paths = paths::Paths::new(&mut loader.package)?;
    let mut engine = Engine {
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
        work: vec![Work::Visit(Visit {
            module: ModuleId(0),
            node: root,
            owner: None,
            in_block: false,
            depth: 0,
            introduction: ContextId(0),
            path_scope: ScopeId(0),
        })],
        children: Vec::new(),
        recursion_limit,
        builtin_definitions: vec![None; Builtin::ALL.len()],
    };
    engine.scan()?;
    while let Some(index) = engine.ready.pop_front() {
        engine.calls[index].queued = false;
        engine.attempt(index)?;
    }
    if let Some(index) = engine
        .calls
        .iter()
        .position(|call| call.state != State::Complete)
    {
        if let Some(error) = engine.calls[index].error.take() {
            return Err(error);
        }
        return Err(engine.calls[index]
            .diagnostic("unresolved macro is not in textual scope or path scope"));
    }
    if let Some(error) = engine.paths.take_error() {
        return Err(error);
    }
    Ok(Report {
        package: engine.loader.package,
        definitions: engine.definitions,
        invocations: engine.invocations,
        captures: engine.captures,
        textual_scopes: engine.scopes,
        resolution: engine.paths.state.facts,
        names_resolved: false,
    })
}
impl<S: Sources> Engine<'_, S> {
    fn wake(&mut self, index: usize) {
        if !self.calls[index].queued {
            self.calls[index].queued = true;
            self.ready.push_back(index);
        }
    }
    fn scan(&mut self) -> Result<(), Error> {
        while let Some(work) = self.work.pop() {
            match work {
                Work::Visit(visit) => self.visit(visit)?,
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
                invocations: &self.invocations,
            },
        )? {
            self.wake(call);
        }
        Ok(())
    }
    fn macro_path(&mut self, file: FileId, path: NodeId) -> Result<(bool, Vec<NameId>), Error> {
        let (absolute, mut names) = self.loader.package.path_names(file, path)?;
        if !absolute && names.len() == 1 {
            let offset =
                super::hygiene::name_offset(&self.loader.package, NodeKey { file, node: path });
            let context = self.context(file, offset)?;
            let local_inner = context
                .0
                .checked_sub(1)
                .and_then(|index| self.invocations.get(index as usize))
                .is_some_and(|call| self.definitions[call.definition].local_inner_macros);
            if local_inner {
                let root = self
                    .loader
                    .package
                    .names
                    .intern("crate")
                    .map_err(|e| Error::at(file, offset, e))?;
                names.insert(0, root);
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
        self.ready.push_back(index);
        self.calls.push(Call {
            visit,
            file,
            capture,
            grammar,
            absolute,
            path: path_names,
            scope,
            output,
            state: State::Unresolved,
            queued: true,
            selected: None,
            error: None,
            offset,
        });
        Ok(())
    }
    fn attempt(&mut self, index: usize) -> Result<(), Error> {
        let found = match self.lookup_call(index) {
            Ok(found) => {
                self.calls[index].error = None;
                found
            }
            Err(error) => {
                self.calls[index].error = Some(error);
                None
            }
        };
        if let Some(previous) = self.calls[index].selected {
            if found == Some(previous) {
                self.calls[index].state = State::Complete;
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
        self.calls[index].state = State::InProgress;
        let call = &self.calls[index];
        let (visit, file, capture, grammar, scope, output, parent) = (
            call.visit,
            call.file,
            call.capture,
            call.grammar,
            call.scope,
            call.output,
            call.visit.introduction,
        );
        let introduction = match self.invoke(visit, file, capture, grammar, definition, parent) {
            Ok(context) => context,
            Err(error) => {
                self.calls[index].state = State::Failed;
                return Err(error);
            }
        };
        self.calls[index].selected = Some(definition);
        self.work.push(Work::Finish {
            output,
            restore: self.head,
        });
        self.head = scope;
        self.work.push(Work::Visit(Visit {
            depth: visit.depth + 1,
            introduction,
            ..visit
        }));
        self.scan()?;
        self.calls[index].state = State::Complete;
        Ok(())
    }

    fn visit(&mut self, mut visit: Visit) -> Result<(), Error> {
        let module = &self.loader.package.modules[visit.module.0 as usize];
        let file = module.file;
        let unit = self.loader.package.unit(file);
        if !unit.configured.active[visit.node.0 as usize] {
            return Ok(());
        }
        let kind = unit.configured.ast.node(visit.node).kind;
        let mut scope = matches!(kind, Kind::Block { .. });
        if let Kind::Item(index) = kind {
            let item = unit.configured.ast.items[index as usize];
            if let ItemKind::MacroRules { capture } = item.kind {
                return self.define(visit, file, index, capture);
            }
            if let ItemKind::Use(tree) = item.kind {
                return self.paths.import(
                    &mut self.loader.package,
                    Context {
                        module: visit.module,
                        file,
                        scope: visit.path_scope,
                    },
                    tree,
                    item.visibility,
                    (self.head, visit.introduction),
                );
            }
            if matches!(item.kind, ItemKind::Module { .. }) && visit.node != module.root {
                return self.module(visit, file, item.visibility);
            }
            scope |= matches!(item.kind, ItemKind::Trait { .. } | ItemKind::Impl { .. });
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
                    unit.configured.ast.node(visit.node).span.start,
                    format!("macro expansion recursion exceeds {}", self.recursion_limit),
                ));
            }
            if self.invocations.len() >= 1_000_000 {
                return Err(Error::at(file, 0, "crate expansion work budget exceeded"));
            }
            return self.register(visit, file, path, capture, context);
        }
        if scope {
            self.work.push(Work::Restore(self.head));
            visit.path_scope = self.paths.block(
                &mut self.loader.package,
                visit.path_scope,
                visit.module,
                visit.node,
            )?;
        }
        self.children.clear();
        self.loader
            .package
            .unit(file)
            .configured
            .ast
            .push_children(visit.node, &mut self.children);
        let owner = if matches!(kind, Kind::Attributed { .. }) {
            Some(visit.node)
        } else {
            None
        };
        self.work.extend(self.children.iter().rev().map(|node| {
            Work::Visit(Visit {
                node: *node,
                owner,
                in_block: visit.in_block || matches!(kind, Kind::Block { .. }),
                ..visit
            })
        }));
        Ok(())
    }
    fn module(&mut self, visit: Visit, file: FileId, visibility: Visibility) -> Result<(), Error> {
        let child =
            self.loader
                .load_module(visit.module, visit.node, visit.owner, visit.in_block)?;
        let path_scope = self.paths.module(
            &mut self.loader.package,
            Context {
                module: visit.module,
                file,
                scope: visit.path_scope,
            },
            child,
            visit.node,
            visibility,
            visit.introduction,
        )?;
        if !self.flags(file, visit.owner)?.macro_use {
            self.work.push(Work::Restore(self.head));
        }
        self.work.push(Work::Visit(Visit {
            module: child,
            node: self.loader.package.modules[child.0 as usize].root,
            owner: None,
            in_block: false,
            path_scope,
            ..visit
        }));
        Ok(())
    }
    fn define(
        &mut self,
        visit: Visit,
        file: FileId,
        index: u32,
        capture: u32,
    ) -> Result<(), Error> {
        let flags = self.flags(file, visit.owner)?;
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
        let definition = compile::compile(
            unit.source.text(),
            &unit.configured.ast.captures[capture as usize],
            &mut package.names,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        let index = self.definitions.len();
        self.definitions.push(Definition {
            node: Some(NodeKey {
                file,
                node: visit.node,
            }),
            module: visit.module,
            name,
            body: Body::Declarative(Compiled::new(definition)),
            context,
            introduction: visit.introduction,
            preceding_scope: self.head,
            local_inner_macros: flags.local_inner_macros,
        });
        self.head = self
            .scopes
            .define(self.head, name, index)
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        self.paths.definition(
            &mut self.loader.package,
            Context {
                module: visit.module,
                file,
                scope: visit.path_scope,
            },
            visit.node,
            name,
            flags.exported,
            (index, visit.introduction),
        )?;
        Ok(())
    }
    fn lookup_call(&mut self, index: usize) -> Result<Option<usize>, Error> {
        let call = &self.calls[index];
        if call.absolute {
            return Err(call.diagnostic("external macro paths require external metadata"));
        }
        if call.path.len() == 1 {
            let name = call.path[0];
            match (bindings::Bindings {
                definitions: &self.definitions,
                invocations: &self.invocations,
            })
            .textual(&mut self.scopes, call.scope, name, call.visit.introduction)
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
            Some(MacroTarget::Declarative(index)) => Ok(Some(index)),
            Some(MacroTarget::Builtin(builtin)) => self.builtin_definition(builtin).map(Some),
            None => Ok(None),
        }
    }
    fn builtin_definition(&mut self, builtin: Builtin) -> Result<usize, Error> {
        let slot = builtin as usize;
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
            context: ContextId(0),
            introduction: ContextId(0),
            preceding_scope: Scope::ROOT,
            local_inner_macros: false,
        });
        self.builtin_definitions[slot] = Some(index);
        Ok(index)
    }
    fn context(&self, file: FileId, offset: u32) -> Result<ContextId, Error> {
        self.loader.package.expansion_context(file, offset)
    }
    fn input(&mut self, file: FileId, capture: u32) -> Result<Input, Error> {
        let unit = &mut self.loader.package.units[file.0 as usize];
        let forest = &unit.configured.ast.captures[capture as usize];
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
        context: ContextId,
        input: &Input,
        builtin: Builtin,
    ) -> Result<ContextId, Error> {
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
        let position = if builtin.needs_position() && input.tokens.is_empty() {
            let physical = self.call_location(Location {
                file,
                span: original,
            })?;
            Some(
                self.loader
                    .package
                    .unit(physical.file)
                    .source
                    .position(physical.span.start)
                    .map_err(|e| Error::at(physical.file, physical.span.start, e))?,
            )
        } else {
            None
        };
        let package = &mut self.loader.package;
        let view = self
            .captures
            .view(|id| package.units.get(id.0 as usize).map(|u| u.source.text()));
        let generated = builtin::expand(
            builtin,
            package.unit(file).source.text(),
            input,
            &view,
            &builtin::Environment {
                config: self.config,
                grammar,
                position,
            },
        )
        .map_err(|e| Error::at(file, original.start, e.message))?;
        let span = Span {
            start: 0,
            end: u32::try_from(generated.source.len())
                .map_err(|_| Error::at(file, original.start, "builtin output too large"))?,
        };
        let origins = [Origin {
            generated: span,
            original: Location {
                file,
                span: original,
            },
            context: Some(context),
        }];
        let generated = package.units[file.0 as usize]
            .install_expansion(visit.node, &generated.source, generated.syntax, &origins)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        self.paths.state.facts.introductions.parents.push(parent);
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
    fn invoke(
        &mut self,
        visit: Visit,
        file: FileId,
        capture: u32,
        grammar: MacroContext,
        definition: usize,
        parent: ContextId,
    ) -> Result<ContextId, Error> {
        let input = self.input(file, capture)?;
        let context = ContextId(
            u32::try_from(self.invocations.len() + 1)
                .map_err(|_| Error::at(file, 0, "too many expansion contexts"))?,
        );
        if let Body::Builtin(builtin) = self.definitions[definition].body {
            return self.invoke_builtin(visit, grammar, definition, context, &input, builtin);
        }
        let package = &mut self.loader.package;
        let units = &package.units;
        let original = units[file.0 as usize].configured.ast.node(visit.node).span;
        let entry = &mut self.definitions[definition];
        let node = entry
            .node
            .ok_or_else(|| Error::at(file, original.start, "declarative macro has no source"))?;
        let Body::Declarative(compiled) = &mut entry.body else {
            return Err(Error::at(
                file,
                original.start,
                "invalid declarative macro body",
            ));
        };
        let definition_source = units[node.file.0 as usize].source.text();
        let source = units[file.0 as usize].source.text();
        let view = self
            .captures
            .view(|id| units.get(id.0 as usize).map(|u| u.source.text()));
        let mut matched = compiled
            .invoke_with_inputs(
                definition_source,
                source,
                &input,
                &mut package.names,
                &mut self.workspace,
                &view,
            )
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let transcription = transcribe::transcribe(&compiled.definition, &matched, source, &input)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        let environment = Environment {
            definition: Source {
                file: node.file,
                text: definition_source,
            },
            invocation: Source { file, text: source },
            context,
            crate_id: 0,
        };
        let mut stream = Stream::transcribe(&transcription, &matched, &input, &environment)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        self.captures
            .adopt(&mut stream, &mut matched, file)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        let view = self
            .captures
            .view(|id| units.get(id.0 as usize).map(|u| u.source.text()));
        let ast =
            fragment::parse_expansion_with_inputs(&stream.source, &stream.tokens, grammar, &view)
                .map_err(|e| Error::at(file, original.start, e.message))?;
        let configured = crate_syntax::configure(&stream.source, ast, self.config)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        let generated = package.units[file.0 as usize]
            .install_expansion(visit.node, &stream.source, configured, &stream.origins)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        self.paths.state.facts.introductions.parents.push(parent);
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file,
                node: visit.node,
            },
            original,
            definition,
            rule: Some(matched.rule),
            generated,
            parser_calls: matched.captures.parser_calls,
            parent,
        });
        Ok(context)
    }
}
