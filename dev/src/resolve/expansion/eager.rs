//! Eager builtin arguments join the existing worklist and retain one parsed arena.
use super::{
    Body, Engine, Error, ExpansionId, Invocation, Location, NodeId, NodeKey, Sources, State, Visit,
    Work, builtin, diagnostics,
};

pub(super) struct Arguments {
    root: NodeId,
    context: ExpansionId,
    pub(super) pending: usize,
}

impl<S: Sources> Engine<'_, S> {
    pub(super) fn completed(&mut self, index: usize) -> Result<(), Error> {
        self.calls[index].state = State::Complete;
        if self.calls[index].notified {
            return Ok(());
        }
        self.calls[index].notified = true;
        if let Some(parent) = self.calls[index].visit.eager {
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
        self.prepare_format_input(file, capture, definition)?;
        let input = self
            .input(file, capture, false)
            .map_err(|e| self.eager_error(index, definition, e))?;
        self.charge_format_opaque(file, definition, &input)?;
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
        self.push_visit(Visit {
            node: root,
            depth: visit.depth + 1,
            introduction: context,
            eager: Some(index),
            ..visit
        })?;
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
        let (file, visit, grammar, scope, output) =
            (call.file, call.visit, call.grammar, call.scope, call.output);
        let arguments = call
            .eager
            .as_ref()
            .ok_or_else(|| call.diagnostic("missing eager arguments"))?;
        let (root, context) = (arguments.root, arguments.context);
        let unit = self.loader.package.unit(file);
        let original = unit.configured.ast.node(visit.node).span;
        let (generated, include) = if matches!(
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
        self.loader.package.units[file.0 as usize].retire_arguments(root);
        self.calls[index].eager = None;
        self.work.push(Work::Finish {
            output,
            restore: self.head,
        });
        self.head = scope;
        self.push_visit(Visit {
            depth: visit.depth + 1,
            introduction: context,
            include,
            ..visit
        })?;
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
        let (file, visit, grammar) = (call.file, call.visit, call.grammar);
        let unit = self.loader.package.unit(file);
        if matches!(self.definitions[definition].body,Body::Builtin(role) if role.formatting()) {
            return self.format_literal_value(index, definition, root);
        }
        match self.definitions[definition].body {
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
