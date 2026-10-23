//! Version-one application tables follow headers and precede body records.
use super::{Input, Writer, crates, g, id, range};
pub(super) fn encode(out: &mut Writer, calls: &g::Calls) -> Result<(), &'static str> {
    id(out, calls.header_types)?;
    out.u32(calls.applications.len())?;
    for application in &calls.applications {
        id(out, application.target.0)?;
        range(out, application.arguments)?;
    }
    out.u32(calls.arguments.len())?;
    for argument in &calls.arguments {
        id(out, argument.0)?;
    }
    Ok(())
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<g::Calls, &'static str> {
    read.reserve(size_of::<g::Calls>())?;
    let header_types = read.id()?;
    let count = read.count::<g::Application>(crates::MAX_COMPONENTS, 12)?;
    let mut applications = Vec::with_capacity(count);
    for _ in 0..count {
        applications.push(g::Application {
            target: super::DeclarationRef(read.id()?),
            arguments: read.range()?,
        });
    }
    let count = read.count::<g::TypeRef>(crates::MAX_COMPONENTS, 4)?;
    let mut arguments = Vec::with_capacity(count);
    for _ in 0..count {
        arguments.push(g::TypeRef(read.id()?));
    }
    Ok(g::Calls {
        header_types,
        applications,
        arguments,
    })
}
