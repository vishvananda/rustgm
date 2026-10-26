//! Error-only source projection retains physical definition and invocation traces.
use super::{
    Definition, Engine, Error, ExpansionId, FileId, Invocation, Location, Package, Report, Sources,
    Span, Stream,
};
use crate::support::hash::HashSet;
use crate::{lex::token::LexError, resolve::package::ExpansionTrace};

impl<S: Sources> Engine<'_, S> {
    pub(super) fn failed_invocation(
        &mut self,
        index: usize,
        definition: usize,
        mut error: Error,
    ) -> Error {
        self.calls[index].state = super::State::Failed;
        let call = &self.calls[index];
        error.expansions.push(frame(
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
    pub(in crate::resolve) fn diagnostic_charged(
        &self,
        error: Error,
        charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Error, Error> {
        checked_projection(
            &self.loader.package,
            &self.definitions,
            &self.invocations,
            error,
            charge,
        )
    }
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
    pub(in crate::resolve) fn diagnostic_charged(
        &self,
        error: Error,
        charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Error, Error> {
        checked_projection(
            &self.package,
            &self.definitions,
            &self.invocations,
            error,
            charge,
        )
    }
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
    charge: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
) -> Result<(Location, Option<ExpansionId>), &'static str> {
    let mut context = None;
    for _ in 0..4096 {
        charge(1, 0)?;
        // A new file can fail parsing before its unit is installed. Its error
        // offsets are already physical and have no generated provenance.
        if package.units.get(location.file.0 as usize).is_none() {
            return Ok((location, context));
        }
        charge(
            1 + package
                .unit(location.file)
                .origins
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize,
            0,
        )?;
        let Some(origin) = package.origin(location.file, location.span.start) else {
            return Ok((location, context));
        };
        let original_unit = package
            .units
            .get(origin.original.file.0 as usize)
            .ok_or("diagnostic origin source is missing")?;
        let extent = origin
            .original
            .span
            .end
            .checked_sub(origin.original.span.start)
            .filter(|_| origin.original.span.end as usize <= original_unit.source.text().len())
            .ok_or("diagnostic origin span changed")?;
        if context.is_none() {
            context = (origin.introduction.0 != 0).then_some(origin.introduction);
            // Includes preserve source hygiene, so their origins have no syntax
            // context. Error-only lookup recovers the separate invocation ancestry.
            if context.is_none() && original_unit.include_context.is_some() {
                for call in invocations {
                    charge(1, 0)?;
                    if call.node.file == location.file && call.generated == origin.generated {
                        context = Some(call.context);
                        break;
                    }
                }
            }
        }
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
    charge: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
) -> Result<Option<ExpansionId>, &'static str> {
    let (call, context) = physical(package, invocations, trace.call, charge)?;
    trace.call = call;
    if let Some(definition) = trace.definition {
        trace.definition = Some(physical(package, invocations, definition, charge)?.0);
    }
    Ok(context)
}
fn project(
    package: &Package,
    definitions: &[Definition],
    invocations: &[Invocation],
    error: Error,
) -> Error {
    let (mut error, result) = projected(package, definitions, invocations, error, |_, _| Ok(()));
    if let Err(message) = result {
        error.message.push_str("; ");
        error.message.push_str(message);
    }
    error
}
fn projected(
    package: &Package,
    definitions: &[Definition],
    invocations: &[Invocation],
    mut error: Error,
    mut charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
) -> (Error, Result<(), &'static str>) {
    let Some(file) = error.file.source() else {
        return (error, Ok(()));
    };
    let result = (|| {
        let (location, mut context) = physical(
            package,
            invocations,
            Location {
                file,
                span: error.span,
            },
            &mut charge,
        )?;
        error.file = crate::resolve::package::DiagnosticFile::Source(location.file);
        error.span = location.span;
        for trace in &mut error.expansions {
            let parent = project_frame(package, invocations, trace, &mut charge)?;
            if context.is_none() {
                context = parent;
            }
        }
        if context.is_none() && error.introduction.0 != 0 {
            context = Some(error.introduction);
        }
        let mut seen = HashSet::default();
        while let Some(current) = context.filter(|context| context.0 != 0) {
            charge(1, crate::support::retained::new_map::<ExpansionId, ()>(1))?;
            if seen.len() >= 4096 || !seen.insert(current) {
                return Err("invalid diagnostic expansion ancestry");
            }
            let call = invocations
                .get(current.0 as usize - 1)
                .ok_or("invalid diagnostic expansion identity")?;
            let definition = definitions
                .get(call.definition)
                .ok_or("invalid diagnostic macro identity")?;
            charge(
                1,
                4 * size_of::<ExpansionTrace>() + package.names.get(definition.name).len(),
            )?;
            let mut trace = frame(
                package,
                definition,
                Location {
                    file: call.node.file,
                    span: call.original,
                },
            );
            project_frame(package, invocations, &mut trace, &mut charge)?;
            error.expansions.push(trace);
            context = Some(call.parent);
        }
        Ok(())
    })();
    error.introduction = ExpansionId(0);
    (error, result)
}

fn checked_projection(
    package: &Package,
    definitions: &[Definition],
    invocations: &[Invocation],
    error: Error,
    charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
) -> Result<Error, Error> {
    // This checked path owns a fresh source diagnostic, never a nested carrier.
    // A failed callback returns its original identity without partial traces.
    if !error.expansions.is_empty() || !error.warnings().is_empty() {
        return Err(error);
    }
    let Some(file) = error.file.source() else {
        return Err(error);
    };
    let original = (file, error.span, error.introduction);
    let (mut error, result) = projected(package, definitions, invocations, error, charge);
    if result.is_err() {
        error.file = crate::resolve::package::DiagnosticFile::Source(original.0);
        error.span = original.1;
        error.introduction = original.2;
        error.expansions.clear();
        return Err(error);
    }
    Ok(error)
}
