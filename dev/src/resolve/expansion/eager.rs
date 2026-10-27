//! Eager builtin arguments join the existing worklist and retain one parsed arena.
#[cfg(test)]
#[path = "../../../tests/invariants/eager_sources.rs"]
mod tests;
use super::{
    Body, Builtin, Engine, Error, ExpansionId, FileId, Input, Invocation, Location, NodeId,
    NodeKey, Sources, State, Visit, Work, builtin, diagnostics,
};
use crate::{
    resolve::bindings::panic_contracts,
    syntax::{interpolation::Inputs, token_tree::error},
};

pub(super) struct Arguments {
    root: NodeId,
    context: ExpansionId,
    pub(super) pending: usize,
}

impl<S: Sources> Engine<'_, S> {
    pub(super) fn source_eager_definition(&self, index: usize) -> Result<Builtin, Error> {
        let definition = self
            .definitions
            .get(index)
            .ok_or_else(|| Error::unlocated("missing actual source eager builtin definition"))?;
        let Body::Builtin(role) = definition.body else {
            return Err(Error::unlocated(
                "source eager builtin changed its actual role",
            ));
        };
        if !role.source_eager() {
            return Err(Error::unlocated(
                "source eager builtin changed its actual role",
            ));
        }
        let key = self
            .paths
            .checked_source_macro(index, definition)
            .map_err(Error::unlocated)?;
        let header =
            panic_contracts::published(&self.loader.package, key, definition.header.as_ref(), 1)?;
        if header.builtin != Some(role) || header.edition != definition.edition {
            return Err(Error::unlocated(
                "source eager builtin differs from its defining header",
            ));
        }
        Ok(role)
    }
    pub(super) fn prepare_source_eager_input(
        &self,
        file: FileId,
        capture: u32,
        definition: usize,
    ) -> Result<(), Error> {
        if !matches!(self.definitions[definition].body,Body::Builtin(role)if role.source_eager()) {
            return Ok(());
        }
        self.source_eager_definition(definition)?;
        let forest = &self.loader.package.unit(file).configured.ast.captures[capture as usize];
        let source = self.loader.package.source_core().ok_or_else(|| {
            Error::at(
                file,
                0,
                "source eager builtin requires its local source provider",
            )
        })?;
        // Preflight before flattening/parsing the real argument forest. Actual
        // opaque payload copies are separately paid through their registry view.
        if matches!(
            self.definitions[definition].body,
            Body::Builtin(Builtin::Asm)
        ) {
            source
                .macros
                .charge(
                    forest.nodes().len(),
                    forest.nodes().len().saturating_mul(512),
                )
                .map_err(|message| Error::at(file, 0, message))?;
        }
        source
            .macros
            .charge(
                forest.nodes().len(),
                forest.payload_bytes().saturating_mul(20),
            )
            .map_err(|message| Error::at(file, 0, message))
    }
    pub(super) fn charge_source_eager_opaque(
        &self,
        file: FileId,
        definition: usize,
        input: &Input,
    ) -> Result<(), Error> {
        if !matches!(self.definitions[definition].body,Body::Builtin(role)if role.source_eager()) {
            return Ok(());
        }
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| Error::at(file, 0, "missing source eager builtin provider"))?;
        let view = self.captures.view(|id| {
            self.loader
                .package
                .units
                .get(id.0 as usize)
                .map(|unit| unit.source.text())
        });
        let charge = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| error(0, message))
        };
        let counted = builtin::choice::Counted {
            inputs: &view,
            charge: &charge,
        };
        for &token in &input.tokens {
            counted
                .get(token)
                .map_err(|error| Error::at(file, error.offset, error.message))?;
        }
        Ok(())
    }
    pub(super) fn completed(&mut self, index: usize) -> Result<(), Error> {
        self.calls[index].state = State::Complete;
        if self.calls[index].notified {
            return Ok(());
        }
        self.calls[index].notified = true;
        if let Some(parent) = self.calls[index].visit.eager {
            if self.derive_outer(parent) {
                return self.derive_completed(parent);
            }
            let call = &mut self.calls[parent];
            let file = call.file;
            let offset = call.offset;
            let arguments = call
                .eager
                .as_mut()
                .ok_or_else(|| Error::at(file, offset, "missing eager parent"))?;
            arguments.pending = arguments
                .pending
                .checked_sub(1)
                .ok_or_else(|| Error::at(file, offset, "eager dependency counter underflow"))?;
            if arguments.pending == 0 {
                self.wake(parent);
            }
        }
        Ok(())
    }

    pub(super) fn begin_eager(&mut self, index: usize, definition: usize) -> Result<(), Error> {
        let call = &self.calls[index];
        let (file, capture, visit, scope) = (call.file, call.capture, call.visit, call.scope);
        if matches!(
            self.definitions[definition].body,
            Body::Builtin(Builtin::Asm)
        ) && !matches!(
            call.grammar.bang().map_err(|e| call.diagnostic(e))?,
            crate::syntax::ast::MacroContext::Expression
                | crate::syntax::ast::MacroContext::Statements
        ) {
            return Err(call.diagnostic("inline assembly requires an expression context"));
        }
        self.prepare_source_eager_input(file, capture, definition)?;
        let input = self
            .input(file, capture, false)
            .map_err(|e| self.eager_error(index, definition, e))?;
        self.charge_source_eager_opaque(file, definition, &input)?;
        let (before, tail) = self.loader.package.units.split_at_mut(file.0 as usize);
        let (unit, after) = tail
            .split_first_mut()
            .ok_or_else(|| Error::at(file, 0, "missing eager source"))?;
        let original = unit.configured.ast.node(visit.node).span;
        let root = unit
            .retain_arguments(
                file,
                &input.tokens,
                &self.captures,
                |id| {
                    if id.0 < file.0 {
                        before.get(id.0 as usize)
                    } else {
                        after.get((id.0 - file.0 - 1) as usize)
                    }
                },
                self.config,
                matches!(
                    self.definitions[definition].body,
                    Body::Builtin(Builtin::Asm)
                ),
            )
            .map_err(|e| Error::at(file, e.offset, e.message));
        let root = root.map_err(|e| self.eager_error(index, definition, e))?;
        let context = ExpansionId(
            u32::try_from(self.invocations.len() + 1)
                .map_err(|_| Error::at(file, original.start, "too many expansion contexts"))?,
        );
        self.paths
            .invocation(context, visit.introduction, &self.definitions[definition])
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
            generated: original,
            parser_calls: 1,
            parent: visit.introduction,
        });
        self.calls[index].selected = Some(definition);
        self.calls[index].eager = Some(Arguments {
            root,
            context,
            pending: 0,
        });
        self.work.push(Work::Restore(self.head));
        self.head = scope;
        let tail = self.lint_call_tail(index)?;
        self.push_visit_with_tail(
            Visit {
                node: root,
                depth: visit.depth + 1,
                introduction: context,
                eager: Some(index),
                ..visit
            },
            tail,
        )?;
        self.scan()?;
        if self.calls[index]
            .eager
            .as_ref()
            .is_some_and(|arguments| arguments.pending == 0)
        {
            self.wake(index);
        }
        Ok(())
    }

    fn concat_bytes_value(
        &self,
        index: usize,
        definition: usize,
        root: NodeId,
    ) -> Result<builtin::Expansion, Error> {
        if self.source_eager_definition(definition)? != builtin::Builtin::ConcatBytes {
            return Err(Error::unlocated(
                "byte concatenation changed its actual source role",
            ));
        }
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let source = self.loader.package.source_core().ok_or_else(|| {
            Error::at(
                call.file,
                call.offset,
                "byte concatenation requires its actual local provider",
            )
        })?;
        builtin::bytes_concat::expand(
            &unit.configured.ast,
            root,
            |literal| unit.literal(literal),
            |steps, bytes| {
                source
                    .macros
                    .charge(steps, bytes)
                    .map_err(|message| error(call.offset, message))
            },
            call.grammar
                .bang()
                .map_err(|message| call.diagnostic(message))?,
            self.config,
        )
        .map_err(|failure| Error::at(call.file, failure.offset, failure.message))
    }
    fn eager_error(&self, index: usize, definition: usize, mut error: Error) -> Error {
        let call = &self.calls[index];
        error.expansions.push(diagnostics::frame(
            &self.loader.package,
            &self.definitions[definition],
            Location {
                file: call.file,
                span: self
                    .loader
                    .package
                    .unit(call.file)
                    .configured
                    .ast
                    .node(call.visit.node)
                    .span,
            },
        ));
        error
    }

    pub(super) fn finish_eager(&mut self, index: usize, definition: usize) -> Result<(), Error> {
        let call = &self.calls[index];
        let (file, visit, grammar, scope, output) = (
            call.file,
            call.visit,
            call.grammar
                .bang()
                .map_err(|message| call.diagnostic(message))?,
            call.scope,
            call.output,
        );
        let arguments = call
            .eager
            .as_ref()
            .ok_or_else(|| call.diagnostic("missing eager arguments"))?;
        let (root, context) = (arguments.root, arguments.context);
        let unit = self.loader.package.unit(file);
        let original = unit.configured.ast.node(visit.node).span;
        let assembly = matches!(
            self.definitions[definition].body,
            Body::Builtin(Builtin::Asm)
        );
        let (generated, include) = if assembly {
            if self.source_eager_definition(definition)? != Builtin::Asm {
                return Err(call.diagnostic("assembly builtin changed its actual source role"));
            }
            let unit = &self.loader.package.units[file.0 as usize];
            let crate::syntax::ast::Kind::InlineAssembly(args) =
                unit.configured.ast.node(root).kind
            else {
                return Err(Error::at(
                    file,
                    original.start,
                    "missing actual assembly arguments",
                ));
            };
            let mut bytes = 0usize;
            for &arg in unit.configured.ast.list(args) {
                if let crate::syntax::ast::Kind::AssemblyTemplate { value, .. } =
                    unit.configured.ast.node(arg).kind
                {
                    if let crate::syntax::ast::Kind::Literal(index) =
                        unit.configured.ast.node(value).kind
                    {
                        if let crate::literal::Value::String(text) = unit
                            .literal(index)
                            .map_err(|e| Error::at(file, e.offset, e.message))?
                        {
                            bytes = bytes.saturating_add(text.len());
                        }
                    }
                }
            }
            self.loader
                .package
                .source_core()
                .ok_or_else(|| {
                    Error::at(
                        file,
                        original.start,
                        "missing actual source assembly provider",
                    )
                })?
                .macros
                .charge(
                    bytes.saturating_add(args.end as usize - args.start as usize),
                    bytes.saturating_mul(96).saturating_add(
                        (args.end as usize - args.start as usize).saturating_mul(512),
                    ),
                )
                .map_err(|message| Error::at(file, original.start, message))?;
            super::assembly_values::complete(&mut self.loader.package.units[file.0 as usize], root)
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            let generated = self.loader.package.units[file.0 as usize]
                .install_assembly(visit.node, root, context.0)
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            (generated, visit.include)
        } else if matches!(
            self.definitions[definition].body,
            Body::Builtin(builtin::Builtin::Include)
        ) {
            let (span, included, parsed) = self
                .include_builtin(file, visit.node, root, grammar)
                .map_err(|e| self.eager_error(index, definition, e))?;
            self.invocations[context.0 as usize - 1].parser_calls += usize::from(parsed);
            (span, Some(included))
        } else {
            let generated = self
                .eager_value(index, definition, root)
                .map_err(|e| self.eager_error(index, definition, e))?;
            // Formatting's constructor path is generated at the actual builtin
            // definition site. An enclosing wrapper cannot lend it privacy.
            let mode = if matches!(self.definitions[definition].body,
                Body::Builtin(role) if role.formatting())
            {
                super::Transparency::Opaque
            } else {
                super::Transparency::SemiOpaque
            };
            (
                self.install_builtin(file, visit.node, original, context, mode, generated)?,
                visit.include,
            )
        };
        self.invocations[context.0 as usize - 1].generated = generated;
        if !assembly {
            self.loader.package.units[file.0 as usize].retire_arguments(root);
        }
        self.calls[index].eager = None;
        self.work.push(Work::Finish {
            output,
            restore: self.head,
        });
        self.head = scope;
        let tail = self.lint_call_tail(index)?;
        self.push_visit_with_tail(
            Visit {
                depth: visit.depth + 1,
                introduction: context,
                include,
                ..visit
            },
            tail,
        )?;
        self.scan()?;
        self.completed(index)
    }
    fn eager_value(
        &mut self,
        index: usize,
        definition: usize,
        root: NodeId,
    ) -> Result<builtin::Expansion, Error> {
        let call = &self.calls[index];
        let (file, visit, grammar) = (
            call.file,
            call.visit,
            call.grammar
                .bang()
                .map_err(|message| call.diagnostic(message))?,
        );
        let unit = self.loader.package.unit(file);
        if matches!(self.definitions[definition].body,Body::Builtin(role) if role.formatting()) {
            return self.format_literal_value(index, definition, root);
        }
        match self.definitions[definition].body {
            Body::Builtin(builtin::Builtin::ConcatBytes) => {
                self.concat_bytes_value(index, definition, root)
            }
            Body::Builtin(builtin::Builtin::Concat) => builtin::concat(
                &unit.configured.ast,
                root,
                |index| unit.literal(index),
                grammar,
                self.config,
            )
            .map_err(|e| Error::at(file, e.offset, e.message)),
            Body::Builtin(builtin::Builtin::CompileError) => Err(
                match builtin::string_argument(&unit.configured.ast, root, |index| {
                    unit.literal(index)
                }) {
                    Ok((span, message)) => Error::at(file, span.start, message),
                    Err(error) => Error::at(file, error.offset, error.message),
                },
            ),
            Body::Builtin(
                builtin @ (builtin::Builtin::OptionEnv
                | builtin::Builtin::Env
                | builtin::Builtin::IncludeStr
                | builtin::Builtin::IncludeBytes),
            ) => self.resource_builtin(file, visit.node, root, grammar, builtin),
            _ => Err(call.diagnostic("invalid eager builtin definition")),
        }
    }
}
