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
    pub(super) fn formatting_requires_inputs(
        &self,
        index: usize,
        root: NodeId,
    ) -> Result<bool, Error> {
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let Kind::Expansion {
            children,
            tail: None,
        } = unit.configured.ast.node(root).kind
        else {
            return Err(call.diagnostic("missing actual formatting input list"));
        };
        let inputs = unit.configured.ast.list(children);
        let [first] = inputs else {
            return Ok(true);
        };
        let Kind::Literal(literal) = unit.configured.ast.node(*first).kind else {
            return Ok(true);
        };
        let Value::String(text) = unit
            .literal(literal)
            .map_err(|e| Error::at(call.file, e.offset, e.message))?
        else {
            return Ok(true);
        };
        self.loader
            .package
            .source_core()
            .ok_or_else(|| call.diagnostic("missing formatting source provider"))?
            .macros
            .charge(text.len().saturating_add(1), 0)
            .map_err(|message| call.diagnostic(message))?;
        // This route discriminator does not parse arguments or resolve names. The
        // selected producer validates the complete literal exactly as before.
        let text = std::str::from_utf8(&text)
            .map_err(|_| call.diagnostic("formatting template is not UTF-8"))?;
        let mut chars = text.chars();
        while let Some(character) = chars.next() {
            if character == '{' && chars.next() != Some('{') {
                return Ok(true);
            }
        }
        Ok(false)
    }
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
        let (draft, first, values) = self.prepare_format_inputs(index, definition, arguments)?;
        let adopted = self.prepare_format_adoption(file, &values, &draft, original)?;
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
        let captured = self.format_capture_origins(file, first, &draft, context, origins[0])?;
        let origins = captured.as_deref().unwrap_or(&origins);
        let unit = &mut self.loader.package.units[file.0 as usize];
        let generated = unit
            .install_expansion(
                target,
                &draft.expansion.source,
                draft.expansion.syntax,
                origins,
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
    ) -> Result<(builtin::format_arguments::Draft, NodeId, Vec<NodeId>), Error> {
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
        let actual =
            super::argument_bindings::collect(unit, file, arguments, call.capture, &payer)?;
        let input_names: Vec<_> = actual.names.iter().map(|name| name.as_deref()).collect();
        let allow_capture = super::argument_bindings::direct_literal(
            unit,
            call.capture,
            &self.captures,
            &payer,
            |id| package.units.get(id.0 as usize),
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        let mut names = [crate::support::names::NameId(0); FormatConstructor::ALL.len()];
        for (slot, constructor) in FormatConstructor::ALL.into_iter().enumerate() {
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
            &input_names,
            allow_capture,
            builtin::format_arguments::Environment {
                symbols: &mut package.names,
                grammar,
                config: self.config,
            },
            role == builtin::Builtin::FormatArgsNl,
            names,
            &payer,
        )
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        Ok((draft, first, actual.values))
    }
    fn prepare_format_adoption(
        &self,
        file: super::FileId,
        values: &[NodeId],
        draft: &builtin::format_arguments::Draft,
        original: Span,
    ) -> Result<Vec<(NodeId, NodeId)>, Error> {
        let unit = self.loader.package.unit(file);
        let count = values.len();
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| Error::at(file, original.start, "missing formatting provider"))?;
        source
            .macros
            .charge(
                count.saturating_add(12),
                count.saturating_mul(size_of::<(NodeId, NodeId)>()),
            )
            .map_err(|message| Error::at(file, original.start, message))?;
        let (input_base, sites) =
            draft_input_sites(draft, count.saturating_add(draft.captured.len()))
                .map_err(|message| Error::at(file, original.start, message))?;
        if draft.inputs.len() != count {
            return Err(Error::at(
                file,
                original.start,
                "formatting input cardinality changed",
            ));
        }
        for (ordinal, &captured) in draft.captured.iter().enumerate() {
            source
                .macros
                .charge(3, 0)
                .map_err(|message| Error::at(file, original.start, message))?;
            let site = sites.get(count + ordinal).copied().ok_or_else(|| {
                Error::at(file, original.start, "formatting capture site changed")
            })?;
            if !matches!(draft.expansion.syntax.ast.node(site).kind, Kind::Group(name) if name == captured)
                || !matches!(
                    draft.expansion.syntax.ast.node(captured).kind,
                    Kind::Name(_, Some(_))
                )
            {
                return Err(Error::at(
                    file,
                    original.start,
                    "formatting capture occurrence changed",
                ));
            }
        }
        let mut adopted = Vec::with_capacity(count);
        let base = u32::try_from(unit.configured.ast.nodes.len())
            .map_err(|_| Error::at(file, original.start, "formatting node overflow"))?;
        for (expected, &(node, ordinal)) in draft.inputs.iter().enumerate() {
            let input = *values.get(ordinal).ok_or_else(|| {
                Error::at(file, original.start, "formatting input ordinal changed")
            })?;
            if ordinal != expected
                || sites.get(ordinal) != Some(&node)
                || unit.configured.active.get(input.get() as usize) != Some(&true)
                || !matches!(draft.expansion.syntax.ast.node(node).kind,
                    Kind::LibraryValue {kind:LibraryValue::FormatInput {slot},value:None}
                    if slot as usize == input_base+ordinal)
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

// Borrow only the actual draft inventory; no second metadata owner or cache.
fn draft_input_sites(
    draft: &builtin::format_arguments::Draft,
    count: usize,
) -> Result<(usize, &[NodeId]), &'static str> {
    let ast = &draft.expansion.syntax.ast;
    let kind = |node: NodeId| {
        ast.nodes
            .get(node.get() as usize)
            .map(|node| node.kind)
            .ok_or("formatting draft node changed")
    };
    let root = match kind(ast.root)? {
        Kind::Expansion {
            children,
            tail: Some(root),
        } if children.start == children.end => root,
        _ => ast.root,
    };
    let Kind::LibraryValue {
        kind: LibraryValue::FormatOutput { sites },
        value: Some(_),
    } = kind(root)?
    else {
        return Err("formatting draft root changed");
    };
    let sites = ast
        .edges
        .get(sites.start as usize..sites.end as usize)
        .ok_or("formatting draft inventory changed")?;
    let base = sites
        .len()
        .checked_sub(count)
        .filter(|base| {
            *base >= 3 + count && *base <= 3 + count.saturating_mul(FormatConstructor::PAIR_COUNT)
        })
        .ok_or("formatting draft pair cardinality changed")?;
    Ok((base, &sites[base..]))
}
