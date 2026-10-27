//! Local source panic roles delegate to a real handler with unchanged arguments.
use super::{
    Body, Builtin, Definition, Engine, Error, ExpansionId, FileId, Input, Invocation, Location,
    MacroContext, NodeKey, Origin, Sources, Span, SyntaxContextId, Transparency, Visit, editions,
};
use crate::{
    expand::{
        builtin::{
            assertion,
            delegation::{self, Handler, Site},
        },
        stream::Source,
    },
    resolve::{
        bindings::panic_contracts::{self, Frame},
        model::DefKind,
    },
    support::edition::Edition,
    syntax::ast::Kind,
};

struct Call {
    original: Location,
    target: super::NodeId,
    context: ExpansionId,
    handler: Handler,
    definition_context: SyntaxContextId,
    grammar: MacroContext,
    edition: Edition,
}

impl<S: Sources> Engine<'_, S> {
    pub(super) fn definition_contract(
        &self,
        visit: Visit,
        file: FileId,
    ) -> Result<Option<std::rc::Rc<panic_contracts::HeaderData>>, Error> {
        let contract = panic_contracts::checked(
            &self.loader.package,
            NodeKey {
                file,
                node: visit.owner.unwrap_or(visit.node),
            },
        )?;
        if contract.as_ref().is_some_and(|contract| {
            contract.summary().node
                != NodeKey {
                    file,
                    node: visit.node,
                }
        }) {
            return Err(Error::at(
                file,
                0,
                "source macro contract names another declaration",
            ));
        }
        Ok(contract.and_then(panic_contracts::CheckedHeader::into_data))
    }

    fn frame_view(&self) -> super::permission_frames::FrameView<'_> {
        super::permission_frames::FrameView {
            package: &self.loader.package,
            definitions: &self.definitions,
            invocations: &self.invocations,
            facts: &self.paths.state.facts,
        }
    }
    fn panic_definition(&self, index: usize) -> Result<&Definition, Error> {
        self.frame_view().definition(index)
    }
    fn panic_frame(&self, context: ExpansionId) -> Result<Frame, Error> {
        self.frame_view().frame(context)
    }

    fn panic_edition(&self, call: NodeKey, original: Span) -> Result<Edition, Error> {
        let package = &self.loader.package;
        let error = |message| Error::at(call.file, original.start, message);
        let unit = package
            .units
            .get(call.file.0 as usize)
            .ok_or_else(|| error("unknown panic call file"))?;
        let node = unit
            .configured
            .ast
            .nodes
            .get(call.node.get() as usize)
            .ok_or_else(|| error("unknown panic call node"))?;
        if node.span != original
            || !matches!(node.kind, Kind::Macro { .. })
            || !unit.configured.active[call.node.get() as usize]
        {
            return Err(error("panic edition query changed its actual call site"));
        }
        let source = package
            .source_core()
            .ok_or_else(|| error("panic delegation requires its actual source provider"))?;
        let mut location = Location {
            file: call.file,
            span: original,
        };
        let mut pending = Vec::new();
        let edition = loop {
            source.macros.charge(1, 0).map_err(error)?;
            let raw = package.syntax_context(location.file, location.span.start)?;
            let Some(context) = package.marks.outer(raw).map_err(error)? else {
                break editions::introducer(location.file, location.span, |id| {
                    package.units.get(id.0 as usize)
                })
                .map_err(|failure| Error::at(location.file, failure.offset, failure.message))?;
            };
            let frame = self.panic_frame(context)?;
            if let Some(edition) = source.macros.edition(frame).map_err(error)? {
                break edition;
            }
            source
                .macros
                .charge(1, 4 * size_of::<Frame>())
                .map_err(error)?;
            pending.push(frame);
            if !frame.skip {
                break frame.edition;
            }
            location = Location {
                file: frame.call.file,
                span: frame.original,
            };
            let next = package.syntax_context(location.file, location.span.start)?;
            if package
                .marks
                .outer(next)
                .map_err(error)?
                .is_some_and(|parent| parent.0 >= context.0)
            {
                return Err(error(
                    "panic edition call site does not precede its expansion",
                ));
            }
        };
        source
            .macros
            .retain_editions(&pending, edition)
            .map_err(error)?;
        Ok(edition)
    }

    fn prepare_panic(
        &self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        builtin: Builtin,
    ) -> Result<Call, Error> {
        let entry = self.panic_definition(definition)?;
        let node = entry
            .node
            .ok_or_else(|| Error::at(FileId(0), 0, "source panic declaration is missing"))?;
        let module = self
            .loader
            .package
            .modules
            .get(visit.module.0 as usize)
            .ok_or_else(|| Error::at(node.file, 0, "unknown panic caller module"))?;
        let file = module.file;
        let original = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(visit.node)
            .span;
        let error = |message| Error::at(file, original.start, message);
        if !matches!(entry.body, Body::Builtin(actual) if actual == builtin && actual.source_only())
        {
            return Err(error("panic delegation changed its selected source role"));
        }
        let root = self
            .paths
            .state
            .facts
            .crate_root
            .and_then(|root| self.paths.state.facts.definitions.get(root.0 as usize));
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| error("missing source panic provider"))?;
        if source.current != source.provider
            || source.current.name() != self.loader.package.crate_name
            || root.is_none_or(|root| root.kind != DefKind::Module || root.node.is_some())
        {
            return Err(error("panic defining crate identity changed"));
        }
        let edition = self.panic_edition(
            NodeKey {
                file,
                node: visit.node,
            },
            original,
        )?;
        let handler = match (builtin, edition) {
            (
                Builtin::Assert | Builtin::CorePanic | Builtin::StdPanic,
                Edition::Rust2021 | Edition::Rust2024,
            ) => Handler::Panic2021,
            (Builtin::Unreachable, Edition::Rust2021 | Edition::Rust2024) => {
                Handler::Unreachable2021
            }
            _ => return Err(error("invalid source panic delegation role")),
        };
        Ok(Call {
            original: Location {
                file,
                span: original,
            },
            target: visit.node,
            context,
            handler,
            definition_context: entry.context,
            edition: entry.edition,
            grammar,
        })
    }

    fn panic_storage(&self, call: &Call, input: &Input) -> Result<usize, Error> {
        let file = call.original.file;
        let original = call.original.span;
        let error = |message| Error::at(file, original.start, message);
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| error("missing source panic provider"))?;
        source.macros.charge(input.tokens.len(), 0).map_err(error)?;
        let bytes = input.tokens.iter().try_fold(0_usize, |total, token| {
            total
                .checked_add(
                    token
                        .end
                        .checked_sub(token.start)
                        .ok_or_else(|| error("invalid panic argument span"))?
                        as usize,
                )
                .ok_or_else(|| error("panic argument size overflow"))
        })?;
        let storage = bytes.saturating_add(input.tokens.len().saturating_add(16).saturating_mul(
            size_of::<Origin>()
                + size_of::<crate::lex::token::Token>()
                + size_of::<crate::syntax::token_tree::Tree>()
                + 64,
        ));
        let reserved = storage.saturating_mul(4);
        source.macros.charge(0, reserved).map_err(error)?;
        Ok(reserved)
    }
    fn panic_output(
        &mut self,
        call: &Call,
        input: &Input,
        builtin: Builtin,
    ) -> Result<(delegation::Delegated, usize), Error> {
        let Location {
            file,
            span: original,
        } = call.original;
        let error = |message| Error::at(file, original.start, message);
        let reserved = self.panic_storage(call, input)?;
        let call_site = self.loader.package.syntax_context(file, original.start)?;
        let marks = &mut self.loader.package.marks;
        let call_context = marks
            .apply(
                SyntaxContextId::ROOT,
                call.context,
                Transparency::SemiOpaque,
                call_site,
            )
            .map_err(error)?;
        let path_context = marks
            .apply(
                call.definition_context,
                call.context,
                Transparency::Opaque,
                call_site,
            )
            .map_err(error)?;
        let site = Site {
            source: Source {
                file,
                text: self.loader.package.unit(file).source.text(),
            },
            original: Location {
                file,
                span: original,
            },
            introduction: call.context,
            call_context,
            path_context,
            grammar: call.grammar,
        };
        let mut provenance = |token: crate::lex::token::Token| {
            let raw = self
                .loader
                .package
                .syntax_context(file, token.start)
                .map_err(|_| {
                    crate::syntax::token_tree::error(token.start, "invalid forwarded panic context")
                })?;
            Ok((raw, self.token_joint(file, token)?))
        };
        let (generated, parser_calls) = if builtin == Builtin::Assert {
            self.assert_output(call, input, site, &mut provenance)?
        } else {
            (
                delegation::build(
                    &site,
                    call.handler,
                    input,
                    &self.captures,
                    self.config,
                    &mut provenance,
                )
                .map_err(|failure| Error::at(file, failure.offset, failure.message))?,
                0,
            )
        };
        let actual = generated.payload_bytes();
        if actual > reserved {
            self.loader
                .package
                .source_core()
                .ok_or_else(|| error("missing source panic provider"))?
                .macros
                .charge(0, actual - reserved)
                .map_err(error)?;
        }
        Ok((generated, parser_calls))
    }

    fn assert_output(
        &self,
        call: &Call,
        input: &Input,
        site: Site<'_>,
        provenance: impl FnMut(
            crate::lex::token::Token,
        ) -> Result<(SyntaxContextId, bool), crate::lex::token::LexError>,
    ) -> Result<(delegation::Delegated, usize), Error> {
        let file = call.original.file;
        let original = call.original.span;
        let error = |message| Error::at(file, original.start, message);
        let units = &self.loader.package.units;
        let view = self
            .captures
            .view(|id| units.get(id.0 as usize).map(|unit| unit.source.text()));
        let inputs = crate::syntax::edition_context::WithEditions::new(&view, |token| {
            editions::token(file, token, |id| units.get(id.0 as usize))
        });
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| error("missing actual Assert source provider"))?;
        let charge = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| crate::syntax::token_tree::error(original.start, message))
        };
        assertion::build(
            &assertion::Request {
                site,
                edition: call.edition,
                input,
                registry: &self.captures,
                inputs: &inputs,
                config: self.config,
                charge: &charge,
            },
            provenance,
        )
        .map_err(|failure| Error::at(file, failure.offset, failure.message))
    }

    pub(super) fn invoke_panic(
        &mut self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        input: &Input,
        builtin: Builtin,
    ) -> Result<ExpansionId, Error> {
        let call = self.prepare_panic(visit, grammar, definition, context, builtin)?;
        let (generated, parser_calls) = self.panic_output(&call, input, builtin)?;
        let Location {
            file,
            span: original,
        } = call.original;
        let error = |message| Error::at(file, original.start, message);
        let span = self.loader.package.units[file.0 as usize]
            .install_expansion(
                call.target,
                &generated.stream.source,
                generated.syntax,
                &generated.stream.origins,
            )
            .map_err(|failure| Error::at(file, original.start, failure.message))?;
        self.paths
            .invocation(context, visit.introduction, &self.definitions[definition])
            .map_err(error)?;
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file,
                node: visit.node,
            },
            original,
            definition,
            rule: None,
            generated: span,
            parser_calls,
            parent: visit.introduction,
        });
        Ok(context)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/assert_dispatch.rs"]
mod assert_tests;
