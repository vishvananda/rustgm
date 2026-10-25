//! Error-only source projection retains physical definition and invocation traces.
use super::{
    Definition, Engine, Error, ExpansionId, FileId, Invocation, Location, Package, Report, Sources,
    Span, Stream,
};
use crate::support::hash::HashSet;
use crate::{lex::token::LexError, resolve::package::ExpansionTrace};

impl<S: Sources> Engine<'_, S> {
    pub(super) fn diagnostic(&self, error: Error) -> Error {
        project(
            &self.loader.package,
            &self.definitions,
            &self.invocations,
            error,
        )
    }
}
impl Report {
    /// Project a phase error through retained source and expansion provenance.
    #[must_use]
    pub fn diagnostic(&self, error: Error) -> Error {
        project(&self.package, &self.definitions, &self.invocations, error)
    }
}
pub(super) fn frame(package: &Package, definition: &Definition, call: Location) -> ExpansionTrace {
    ExpansionTrace {
        macro_name: package.names.get(definition.name).to_owned(),
        call,
        definition: definition.node.map(|key| Location {
            file: key.file,
            span: package.unit(key.file).configured.ast.node(key.node).span,
        }),
    }
}
pub(super) fn stream_error(
    stream: &Stream,
    file: FileId,
    original: Span,
    error: LexError,
) -> Error {
    if let Some(origin) = stream.origin(error.offset) {
        let offset = origin.original.span.start
            + (error.offset - origin.generated.start)
                .min(origin.original.span.end - origin.original.span.start);
        Error::at(origin.original.file, offset, error.message)
    } else {
        Error::at(file, original.start, error.message)
    }
}

fn physical(
    package: &Package,
    invocations: &[Invocation],
    mut location: Location,
) -> Result<(Location, Option<ExpansionId>), &'static str> {
    let mut context = None;
    for _ in 0..4096 {
        // A new file can fail parsing before its unit is installed. Its error
        // offsets are already physical and have no generated provenance.
        if package.units.get(location.file.0 as usize).is_none() {
            return Ok((location, context));
        }
        let Some(origin) = package.origin(location.file, location.span.start) else {
            return Ok((location, context));
        };
        if context.is_none() {
            context = (origin.introduction.0 != 0).then_some(origin.introduction);
            // Includes preserve source hygiene, so their origins have no syntax
            // context. Error-only lookup recovers the separate invocation ancestry.
            if context.is_none() && package.unit(origin.original.file).include_context.is_some() {
                context = invocations
                    .iter()
                    .find(|call| {
                        call.node.file == location.file && call.generated == origin.generated
                    })
                    .map(|call| call.context);
            }
        }
        let extent = origin.original.span.end - origin.original.span.start;
        location = Location {
            file: origin.original.file,
            span: Span {
                start: origin.original.span.start
                    + (location.span.start - origin.generated.start).min(extent),
                end: origin.original.span.start
                    + (location.span.end - origin.generated.start).min(extent),
            },
        };
    }
    Err("diagnostic provenance exceeds 4096")
}
fn project_frame(
    package: &Package,
    invocations: &[Invocation],
    trace: &mut ExpansionTrace,
) -> Result<Option<ExpansionId>, &'static str> {
    let (call, context) = physical(package, invocations, trace.call)?;
    trace.call = call;
    if let Some(definition) = trace.definition {
        trace.definition = Some(physical(package, invocations, definition)?.0);
    }
    Ok(context)
}
fn project(
    package: &Package,
    definitions: &[Definition],
    invocations: &[Invocation],
    mut error: Error,
) -> Error {
    let Some(file) = error.file.source() else {
        return error;
    };
    let result = (|| {
        let (location, mut context) = physical(
            package,
            invocations,
            Location {
                file,
                span: error.span,
            },
        )?;
        error.file = crate::resolve::package::DiagnosticFile::Source(location.file);
        error.span = location.span;
        for trace in &mut error.expansions {
            let parent = project_frame(package, invocations, trace)?;
            if context.is_none() {
                context = parent;
            }
        }
        if context.is_none() && error.introduction.0 != 0 {
            context = Some(error.introduction);
        }
        let mut seen = HashSet::default();
        while let Some(current) = context.filter(|context| context.0 != 0) {
            if seen.len() >= 4096 || !seen.insert(current) {
                return Err("invalid diagnostic expansion ancestry");
            }
            let call = invocations
                .get(current.0 as usize - 1)
                .ok_or("invalid diagnostic expansion identity")?;
            let definition = definitions
                .get(call.definition)
                .ok_or("invalid diagnostic macro identity")?;
            let mut trace = frame(
                package,
                definition,
                Location {
                    file: call.node.file,
                    span: call.original,
                },
            );
            project_frame(package, invocations, &mut trace)?;
            error.expansions.push(trace);
            context = Some(call.parent);
        }
        Ok(())
    })();
    error.introduction = ExpansionId(0);
    if let Err(message) = result {
        error.message.push_str("; ");
        error.message.push_str(message);
    }
    error
}
