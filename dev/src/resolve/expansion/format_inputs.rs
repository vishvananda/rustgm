//! Adopt eager input nodes once, preserving their original typed spans and contexts.
use super::{Engine, Error, ExpansionId, NodeId, Sources, Transparency, builtin};
use crate::{
    expand::{
        marks::SyntaxContextId,
        stream::{Location, Origin, OriginKind},
    },
    literal::Value,
    support::library_types::{FormatConstructor, LibraryValue},
    syntax::{
        ast::Kind,
        token_tree::{Span, error},
    },
};
impl<S: Sources> Engine<'_, S> {
    pub(super) fn finish_format_inputs(
        &mut self,
        index: usize,
        definition: usize,
        arguments: NodeId,
        context: ExpansionId,
    ) -> Result<Span, Error> {
        let call = &self.calls[index];
        let (file, target) = (call.file, call.visit.node);
        let original = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(target)
            .span;
        let (draft, first) = self.prepare_format_inputs(index, definition, arguments)?;
        let adopted = self.prepare_format_adoption(file, arguments, &draft, original)?;
        let call_site =
            crate::resolve::unit::contexts::callee(&self.loader.package.units, file, target)?;
        let syntax = self
            .loader
            .package
            .marks
            .apply(
                SyntaxContextId::ROOT,
                context,
                Transparency::Opaque,
                call_site,
            )
            .map_err(|message| Error::at(file, original.start, message))?;
        let generated = Span {
            start: 0,
            end: u32::try_from(draft.expansion.source.len())
                .map_err(|_| Error::at(file, original.start, "formatting image overflow"))?,
        };
        let origins = [Origin {
            generated,
            original: Location {
                file,
                span: original,
            },
            context: Some(syntax),
            introduction: context,
            joint: None,
            kind: OriginKind::Synthetic(self.definitions[definition].edition),
        }];
        let unit = &mut self.loader.package.units[file.0 as usize];
        let generated = unit
            .install_expansion(
                target,
                &draft.expansion.source,
                draft.expansion.syntax,
                &origins,
            )
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        // Preflight above authenticates every destination against this append. No
        // argument is copied, reparsed, or retired as part of the detached list.
        for (destination, input) in adopted {
            unit.configured.ast.nodes[destination.get() as usize].kind = Kind::Group(input);
        }
        unit.configured.active[arguments.get() as usize] = false;
        unit.configured.active[first.get() as usize] = false;
        Ok(generated)
    }
    fn prepare_format_inputs(
        &mut self,
        index: usize,
        definition: usize,
        arguments: NodeId,
    ) -> Result<(builtin::format_arguments::Draft, NodeId), Error> {
        let role = self.source_eager_definition(definition)?;
        if !role.formatting() {
            return Err(Error::unlocated(
                "formatting changed its actual source role",
            ));
        }
        let call = &self.calls[index];
        let (file, target, grammar) = (
            call.file,
            call.visit.node,
            call.grammar.bang().map_err(|m| call.diagnostic(m))?,
        );
        let package = &mut self.loader.package;
        let unit = &package.units[file.0 as usize];
        let original = unit.configured.ast.node(target).span;
        let Kind::Expansion {
            children,
            tail: None,
        } = unit.configured.ast.node(arguments).kind
        else {
            return Err(call.diagnostic("missing actual formatting input list"));
        };
        let inputs = unit.configured.ast.list(children);
        let count = inputs.len().saturating_sub(1);
        let Some(first) = inputs.first().copied() else {
            return Err(call.diagnostic("missing formatting template"));
        };
        let Kind::Literal(literal) = unit.configured.ast.node(first).kind else {
            return Err(call.diagnostic("formatting requires an actual string literal"));
        };
        let Value::String(text) = unit
            .literal(literal)
            .map_err(|e| Error::at(file, e.offset, e.message))?
        else {
            return Err(call.diagnostic("formatting requires an actual string literal"));
        };
        let Some(crate::resolve::bindings::Source::Core(source)) = package.source.as_deref() else {
            return Err(call.diagnostic("missing formatting source provider"));
        };
        let payer = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| error(original.start, message))
        };
        let mut names = [crate::support::names::NameId(0); 2];
        for (slot, constructor) in [FormatConstructor::Arguments, FormatConstructor::Debug]
            .into_iter()
            .enumerate()
        {
            names[slot] = package
                .names
                .intern_exact_paid(constructor.name(), &mut |growth| {
                    source.macros.charge(growth.work, growth.bytes)
                })
                .map_err(|message| Error::at(file, original.start, message))?;
        }
        let draft = builtin::format_arguments::build(
            std::str::from_utf8(&text)
                .map_err(|_| Error::at(file, original.start, "formatting template is not UTF-8"))?,
            count,
            role == builtin::Builtin::FormatArgsNl,
            names,
            grammar,
            self.config,
            &payer,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        Ok((draft, first))
    }
    fn prepare_format_adoption(
        &self,
        file: super::FileId,
        arguments: NodeId,
        draft: &builtin::format_arguments::Draft,
        original: Span,
    ) -> Result<Vec<(NodeId, NodeId)>, Error> {
        let unit = self.loader.package.unit(file);
        let Kind::Expansion {
            children,
            tail: None,
        } = unit.configured.ast.node(arguments).kind
        else {
            return Err(Error::at(
                file,
                original.start,
                "missing actual formatting input list",
            ));
        };
        let count = unit.configured.ast.list(children).len().saturating_sub(1);
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| Error::at(file, original.start, "missing formatting provider"))?;
        source
            .macros
            .charge(
                count.saturating_add(1),
                count.saturating_mul(size_of::<(NodeId, NodeId)>()),
            )
            .map_err(|message| Error::at(file, original.start, message))?;
        let mut adopted = Vec::with_capacity(count);
        let base = u32::try_from(unit.configured.ast.nodes.len())
            .map_err(|_| Error::at(file, original.start, "formatting node overflow"))?;
        for &(node, ordinal) in &draft.inputs {
            let input = *unit
                .configured
                .ast
                .list(children)
                .get(ordinal + 1)
                .ok_or_else(|| {
                    Error::at(file, original.start, "formatting input ordinal changed")
                })?;
            if unit.configured.active.get(input.get() as usize) != Some(&true)
                || !matches!(draft.expansion.syntax.ast.node(node).kind,
                    Kind::LibraryValue {kind:LibraryValue::FormatInput {slot},value:None}
                    if slot as usize == 3+count+ordinal)
            {
                return Err(Error::at(
                    file,
                    original.start,
                    "formatting input occurrence changed",
                ));
            }
            let destination = node
                .get()
                .checked_add(base)
                .and_then(NodeId::checked)
                .ok_or_else(|| Error::at(file, original.start, "formatting node overflow"))?;
            adopted.push((destination, input));
        }
        Ok(adopted)
    }
}
