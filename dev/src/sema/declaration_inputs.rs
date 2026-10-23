//! Borrow canonical declaration inputs without storing syntax in imported contracts.
use super::generics::{Generics, Predicate};
use crate::{
    resolve::{body_origin::ProofSite, model::DefId},
    types::parameters::clauses::Clause,
};

/// A view, not a new dense source predicate table or evidence of satisfaction.
#[derive(Clone, Copy)]
pub(crate) struct Input {
    pub source: ProofSite,
    pub value: Clause,
}
impl From<&Predicate> for Input {
    fn from(predicate: &Predicate) -> Self {
        Self {
            source: predicate.source.into(),
            value: predicate.value,
        }
    }
}

fn imported(
    linkage: Option<&super::crates::Linkage>,
    owner: DefId,
    declared: bool,
) -> Result<&[Input], &'static str> {
    linkage.map_or(Ok(&[]), |linkage| linkage.original_inputs(owner, declared))
}
pub(crate) fn all<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(generics
        .predicates_in_scope(owner)
        .map(Input::from)
        .chain(imported(linkage, owner, false)?.iter().copied()))
}
pub(crate) fn declared<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(generics
        .declared_predicates_in_scope(owner)
        .map(Input::from)
        .chain(imported(linkage, owner, true)?.iter().copied()))
}
pub(crate) fn own<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(generics
        .predicates
        .get(&owner)
        .into_iter()
        .flatten()
        .map(Input::from)
        .chain(imported(linkage, owner, false)?.iter().copied()))
}

#[cfg(test)]
#[path = "../../tests/invariants/declaration_inputs.rs"]
mod tests;
