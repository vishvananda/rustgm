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

/// Borrow both representations without copying source predicates or losing sites.
#[derive(Clone, Copy)]
pub(crate) struct Inputs<'a> {
    source: &'a [Predicate],
    imported: &'a [Input],
}
impl<'a> Inputs<'a> {
    pub(crate) fn new(source: &'a [Predicate], imported: &'a [Input]) -> Self {
        Self { source, imported }
    }
    pub(crate) fn iter(self) -> impl Iterator<Item = Input> + 'a {
        self.source
            .iter()
            .map(Input::from)
            .chain(self.imported.iter().copied())
    }
    pub(crate) fn len(self) -> usize {
        self.source.len().saturating_add(self.imported.len())
    }
    pub(crate) fn get(self, index: usize) -> Option<Input> {
        if index < self.source.len() {
            self.source.get(index).map(Input::from)
        } else {
            self.imported.get(index - self.source.len()).copied()
        }
    }
}

pub(crate) fn own_view<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<Inputs<'a>, &'static str> {
    Ok(Inputs::new(
        generics.predicates.get(&owner).map_or(&[], Vec::as_slice),
        imported(generics, linkage, owner, false)?,
    ))
}

fn imported<'a>(
    generics: &Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
    declared: bool,
) -> Result<&'a [Input], &'static str> {
    linkage.map_or(Ok(&[]), |linkage| {
        linkage.original_inputs(generics, owner, declared)
    })
}
pub(crate) fn all<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(generics
        .predicates_in_scope(owner)
        .map(Input::from)
        .chain(imported(generics, linkage, owner, false)?.iter().copied()))
}

/// A call needs each actual parent scope's own clauses. Imported function
/// headers deliberately retain only their own input/prefix, just as source
/// declarations do; that header slice cannot stand in for the whole chain.
pub(crate) fn collect_inherited(
    generics: &Generics,
    linkage: Option<&super::crates::Linkage>,
    owner: DefId,
    mut pay: impl FnMut(usize, usize) -> Result<(), String>,
) -> Result<Vec<Input>, String> {
    let mut result = Vec::new();
    let limit = generics.scope_limit();
    for (visited, scope) in generics.scopes(owner).enumerate() {
        pay(16, 0)?;
        if visited == limit {
            return Err("declaration input scopes contain a parent cycle".into());
        }
        let inputs = own_view(generics, linkage, scope).map_err(str::to_owned)?;
        let count = inputs.len();
        if count == 0 {
            continue;
        }
        let growth = crate::support::retained::Growth::vector::<Input>(
            result.len(),
            result.capacity(),
            count,
        );
        pay(growth.work.saturating_add(count), growth.bytes)?;
        result
            .try_reserve(count)
            .map_err(|_| "declaration input scratch allocation failed".to_owned())?;
        result.extend(inputs.iter());
    }
    Ok(result)
}
pub(crate) fn declared<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(generics
        .declared_predicates_in_scope(owner)
        .map(Input::from)
        .chain(imported(generics, linkage, owner, true)?.iter().copied()))
}
pub(crate) fn own<'a>(
    generics: &'a Generics,
    linkage: Option<&'a super::crates::Linkage>,
    owner: DefId,
) -> Result<impl Iterator<Item = Input> + 'a, &'static str> {
    Ok(own_view(generics, linkage, owner)?.iter())
}

#[cfg(test)]
#[path = "../../tests/invariants/declaration_inputs.rs"]
mod tests;
