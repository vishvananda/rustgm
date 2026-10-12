//! Select the executable root from resolved module bindings, including imports.
use super::model::Declarations;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace},
        package::Error,
        unit::FileId,
    },
    types::{Kind, TypeId},
};

/// Select a safe, nongeneric, parameterless unit/never Rust main.
///
/// # Errors
/// Diagnoses missing/ambiguous entry bindings and incompatible entry signatures.
pub fn select(report: &mut Report, facts: &Declarations) -> Result<DefId, Error> {
    let error = |message| Error::at(FileId(0), 0, message);
    let name = report.package.names.intern("main").map_err(error)?;
    let root = *report
        .resolution
        .module_scopes
        .first()
        .ok_or_else(|| error("missing root scope"))?;
    let binding = report.resolution.scopes[root.0 as usize]
        .bindings
        .get(&(name, Namespace::Value))
        .ok_or_else(|| error("an executable requires a root main function"))?;
    if binding.ambiguous {
        return Err(error("ambiguous main function"));
    }
    let definition = &report.resolution.definitions[binding.target.0 as usize];
    if definition.kind != DefKind::Function {
        return Err(error("main must name a function"));
    }
    let ty =
        facts.definitions[binding.target.0 as usize].ok_or_else(|| error("main has no type"))?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err(error("main has no function signature"));
    };
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
    }) = facts.types.kind(signature)
    else {
        return Err(error("invalid main signature"));
    };
    if bound_regions != 0
        || !facts
            .types
            .list(parameters)
            .ok_or_else(|| error("missing main parameter list"))?
            .is_empty()
        || unsafe_
        || (result != TypeId::UNIT && result != TypeId::NEVER)
    {
        return Err(error("main must be safe, parameterless and return () or !"));
    }
    Ok(binding.target)
}
