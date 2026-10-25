//! Matching publishes one retained stream before parsing or installing it.
use super::{
    Body, Engine, Environment, Error, ExpansionId, FileId, Input, NodeId, Source, Sources, Stream,
    Transparency, WithEditions, editions, transcribe,
};

pub(super) struct Transcribed {
    pub stream: Stream,
    pub rule: usize,
    pub parser_calls: usize,
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn transcribe(
        &mut self,
        file: FileId,
        call: NodeId,
        definition: usize,
        context: ExpansionId,
        input: &Input,
    ) -> Result<Transcribed, Error> {
        let offset = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(call)
            .span
            .start;
        self.require_invocable(
            definition,
            crate::expand::builtin::BuiltinKind::Bang,
            file,
            offset,
        )?;
        let call_site =
            crate::resolve::unit::contexts::callee(&self.loader.package.units, file, call)?;
        let package = &mut self.loader.package;
        let units = &package.units;
        let original = units[file.0 as usize].configured.ast.node(call).span;
        let entry = &mut self.definitions[definition];
        let node = entry
            .node
            .ok_or_else(|| Error::at(file, original.start, "declarative macro has no source"))?;
        let (compiled, mode) = match &mut entry.body {
            Body::Declarative(compiled) => (compiled, Transparency::SemiOpaque),
            Body::Scoped { compiled, mode } => (compiled.as_mut(), *mode),
            Body::Builtin(_) => {
                return Err(Error::at(
                    file,
                    original.start,
                    "invalid declarative macro body",
                ));
            }
        };
        let definition_source = units[node.file.0 as usize].source.text();
        let source = units[file.0 as usize].source.text();
        let view = self
            .captures
            .view(|id| units.get(id.0 as usize).map(|u| u.source.text()));
        let inputs = WithEditions::new(&view, |token| {
            editions::token(file, token, |id| units.get(id.0 as usize))
        });
        let mut matched = compiled
            .invoke_with_inputs(
                definition_source,
                source,
                input,
                &mut package.names,
                &mut self.workspace,
                &inputs,
            )
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        let transcription = transcribe::transcribe(&compiled.definition, &matched, source, input)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        let environment = Environment {
            definition: Source {
                file: node.file,
                text: definition_source,
            },
            invocation: Source { file, text: source },
            context: entry.context,
            introduction: context,
            crate_id: 0,
        };
        let mut stream = Stream::transcribe_with(
            &transcription,
            &matched,
            input,
            &environment,
            |location, origin| {
                let raw =
                    crate::resolve::unit::contexts::at(units, location.file, location.span.start)
                        .map_err(|_| {
                        crate::syntax::token_tree::error(
                            location.span.start,
                            "invalid retained token context",
                        )
                    })?;
                if origin == transcribe::Origin::Invocation {
                    return Ok(Some(raw));
                }
                package
                    .marks
                    .apply(raw, context, mode, call_site)
                    .map(Some)
                    .map_err(|message| {
                        crate::syntax::token_tree::error(location.span.start, message)
                    })
            },
        )
        .map_err(|e| Error::at(file, original.start, e.message))?;
        self.captures
            .adopt(&mut stream, &mut matched, file, source, input)
            .map_err(|e| Error::at(file, original.start, e.message))?;
        Ok(Transcribed {
            stream,
            rule: matched.rule,
            parser_calls: matched.captures.parser_calls,
        })
    }
}
