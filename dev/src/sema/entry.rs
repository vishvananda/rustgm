//! Select the executable root from resolved module bindings, including imports.
mod startup;
use super::model::Declarations;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace},
        package::Error,
        unit::FileId,
    },
    support::bindings::LangItem,
    types::{Kind, TypeId},
};
pub(crate) use startup::{Startup, prepare as prepare_startup};

/// Select a safe, nongeneric, parameterless Rust main.
///
/// # Errors
/// Diagnoses missing/ambiguous entry bindings and incompatible entry signatures.
pub fn select(report: &mut Report, facts: &Declarations) -> Result<DefId, Error> {
    let error = |message| Error::at(FileId(0), 0, message);
    let name = report.package.names.intern("main").map_err(error)?;
    select_named(report, facts, name)
}

/// The private continuation reserves the actual name before borrowing its Report.
/// This shares every binding and signature check with ordinary entry selection.
pub(in crate::sema) fn select_named(
    report: &Report,
    facts: &Declarations,
    name: crate::support::names::NameId,
) -> Result<DefId, Error> {
    let error = |message| Error::at(FileId(0), 0, message);
    // A bring-up guest session's program is its last guest crate.
    let program = report
        .package
        .program_crate()
        .map_or(0, |program| program.root.0 as usize);
    let root = *report
        .resolution
        .module_scopes
        .get(program)
        .ok_or_else(|| error("missing root scope"))?;
    let binding = report
        .resolution
        .bindings
        .get(root, name, Namespace::Value)
        .ok_or_else(|| error("an executable requires a root main function"))?;
    if binding.ambiguous {
        return Err(error("ambiguous main function"));
    }
    let definition = &report.resolution.definitions[binding.target.0 as usize];
    if definition.kind != DefKind::Function {
        return Err(error("main must name a function"));
    }
    if report
        .resolution
        .foreign_owners
        .contains_key(&binding.target)
    {
        return Err(error("main cannot be declared in a foreign block"));
    }
    main_callable(facts, binding.target).map_err(error)?;
    Ok(binding.target)
}

fn main_callable(facts: &Declarations, entry: DefId) -> Result<(TypeId, TypeId), &'static str> {
    if facts.generics.parameters.contains_key(&entry) {
        return Err("main cannot have type or const parameters");
    }
    let ty = facts
        .definitions
        .get(entry.0 as usize)
        .copied()
        .flatten()
        .ok_or("main has no type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err("main has no function signature");
    };
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention,
    }) = facts.types.kind(signature)
    else {
        return Err("invalid main signature");
    };
    if bound_regions != 0
        || !facts
            .types
            .list(parameters)
            .ok_or("missing main parameter list")?
            .is_empty()
        || unsafe_
        || convention != crate::types::callable::Convention::RUST
    {
        return Err("main must be safe, parameterless and return () or !");
    }
    if source_start(facts)?.is_none() && result != TypeId::UNIT && result != TypeId::NEVER {
        return Err("main must be safe, parameterless and return () or !");
    }
    Ok((ty, signature))
}

fn source_role(facts: &Declarations, role: LangItem) -> Result<Option<DefId>, &'static str> {
    facts
        .source_bindings
        .as_deref()
        .map_or(Ok(None), |catalog| {
            Ok(catalog.validated_role(role)?.map(|role| role.definition))
        })
}

pub(crate) fn source_start(facts: &Declarations) -> Result<Option<DefId>, &'static str> {
    source_role(facts, LangItem::Start)
}
