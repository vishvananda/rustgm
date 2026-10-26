//! A source abort declaration is a safe non-const claim, never call permission.
use crate::{
    sema::model::{Declarations, Function},
    types::{Kind, callable::Convention},
};

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source abort requires a safe Rust signature without late regions");
    };
    if facts.types.list(parameters).is_none_or(|p| !p.is_empty())
        || facts.types.kind(result) != Some(Kind::Never)
        || facts
            .generics
            .parameters
            .get(&function.definition)
            .is_some_and(|p| !p.is_empty())
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
        || facts
            .generics
            .declared_predicates_in_scope(function.definition)
            .next()
            .is_some()
    {
        return Err("source abort requires no parameters or predicates and never result");
    }
    if facts.is_const_function(function.definition) || function.body.is_some() {
        return Err("source abort requires a non-const bodyless declaration");
    }
    Ok(())
}
