//! Crate output declarations consume an actual root and explicit caller policy.
use super::{
    package::Error,
    unit::{FileId, Unit},
};
use crate::{
    lex::token::Kind,
    literal::{self, Value},
    support::crates::ArtifactContext,
    syntax::{
        attribute::{Attribute, Input, Style},
        token_tree::Forest,
    },
};
const ORDINARY: &str = "source crate_type artifact selection requires source compiler mode";

#[derive(PartialEq, Eq)]
enum Request {
    Binary,
    Library,
    RustLibrary,
    DynamicRustLibrary,
    DynamicSystemLibrary,
    StaticSystemLibrary,
    ProceduralMacro,
}

pub(super) fn host(unit: &Unit, context: &super::bindings::Context) -> Result<(), Error> {
    if matches!(context.artifacts, ArtifactContext::Source { .. })
        && matches!(&context.mode, crate::support::bindings::Mode::Bootstrap)
    {
        return Err(Error::at(
            FileId(0),
            0,
            "source artifact policy requires source compiler bindings",
        ));
    }
    root(unit, FileId(0), context.artifacts)
}

pub(super) fn root(unit: &Unit, file: FileId, context: ArtifactContext) -> Result<(), Error> {
    let node = unit.configured.ast.root;
    let Some(owner) = unit.owner_indices[node.get() as usize] else {
        return Ok(());
    };
    let owner = &unit.configured.owners[owner.get() as usize - 1];
    if !owner.enabled || owner.node != node {
        return Ok(());
    }
    let attributes = owner.attributes.clone();
    for (attribute, forest) in unit.retained_attributes(attributes) {
        if attribute.name(forest, unit.source.text()) != Some("crate_type") {
            continue;
        }
        let requested = request(unit, file, attribute, forest)?;
        let error = |message| Error::at(file, attribute.span.start, message);
        if attribute.style != Style::Inner {
            return Err(error(
                "crate_type selection requires an inner crate attribute",
            ));
        }
        match context {
            ArtifactContext::Inspection | ArtifactContext::Source { selected: Some(_) } => {}
            ArtifactContext::Ordinary | ArtifactContext::BootstrapLibrary => {
                return Err(error(ORDINARY));
            }
            ArtifactContext::Source { selected: None } if requested == Request::Binary => {}
            ArtifactContext::Source { selected: None } => {
                return Err(error(
                    "source-selected library artifacts require an explicit --crate-type=bin or --crate-type=lib override",
                ));
            }
        }
    }
    Ok(())
}

/// Non-root occurrences are recognized metadata, with no artifact authority.
/// Their forms still receive the builtin's validation before macro lookup.
pub(super) fn form(
    unit: &Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
    context: ArtifactContext,
) -> Result<(), Error> {
    request(unit, file, attribute, forest)?;
    if matches!(
        context,
        ArtifactContext::Ordinary | ArtifactContext::BootstrapLibrary
    ) {
        return Err(Error::at(file, attribute.span.start, ORDINARY));
    }
    Ok(())
}

fn request(
    unit: &Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Request, Error> {
    let error = || {
        Error::at(
            file,
            attribute.span.start,
            "crate_type requires a string literal",
        )
    };
    if attribute.unsafe_wrapper {
        return Err(error());
    }
    let Input::Equals(range) = attribute.input else {
        return Err(error());
    };
    if range.end != range.start + 1 {
        return Err(error());
    }
    let token = forest.token(range.start).ok_or_else(error)?;
    if !matches!(token.kind, Kind::String | Kind::RawString) {
        return Err(error());
    }
    let Some(Value::String(bytes)) = literal::interpret(unit.source.text(), token)
        .map_err(|failure| Error::at(file, failure.offset, failure.message))?
    else {
        return Err(error());
    };
    match bytes.as_ref() {
        b"bin" => Ok(Request::Binary),
        b"lib" => Ok(Request::Library),
        b"rlib" => Ok(Request::RustLibrary),
        b"dylib" => Ok(Request::DynamicRustLibrary),
        b"cdylib" => Ok(Request::DynamicSystemLibrary),
        b"staticlib" => Ok(Request::StaticSystemLibrary),
        b"proc-macro" => Ok(Request::ProceduralMacro),
        _ => Err(Error::at(
            file,
            attribute.span.start,
            "crate_type value is outside the supported artifact profile",
        )),
    }
}
