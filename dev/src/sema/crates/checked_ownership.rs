//! Ordinary graphs and selected private mappings retain their actual owners.
use super::typed_headers::{mapped_source::Graph, unpaid_provided::ProvidedOwners};
use crate::{
    sema::{
        associated::{Owner, Schema},
        compiler_items::Role,
    },
    support::crates::generics::source as s,
    types::{Store, parameters::predicates::Predicate},
};
use std::rc::Rc;

pub(in crate::sema) enum ImportedOwners {
    Ordinary(Rc<Graph>),
    Provided(Rc<ProvidedOwners>),
}
impl ImportedOwners {
    pub(in crate::sema) const fn selected(&self) -> bool {
        matches!(self, Self::Provided(_))
    }
    pub(in crate::sema) fn same_owner(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ordinary(a), Self::Ordinary(b)) => Rc::ptr_eq(a, b),
            (Self::Provided(a), Self::Provided(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
    pub(in crate::sema) fn require_store(&self, store: &Store) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => graph.require_store(store),
            Self::Provided(owner) => owner.require_store(store),
        }
    }
    pub(in crate::sema) fn owner(&self, owner: Owner) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => graph.owner(owner),
            Self::Provided(mapping) => mapping.owner(owner),
        }
    }
    pub(in crate::sema) fn associated_count(&self) -> Result<usize, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.associated_count(),
            Self::Provided(_) => {
                Err("selected imported owners require their explicit selected append")
            }
        }
    }
    pub(in crate::sema) fn associated_row(
        &self,
        store: &Store,
        owner: Owner,
        schema: &Schema,
        members: &[crate::resolve::model::DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => graph.associated_row(store, owner, schema, members, steps),
            Self::Provided(mapping) => mapping.associated_row(store, owner, schema, members, steps),
        }
    }
    pub(in crate::sema) fn role_origin(&self, role: Role) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => graph.role_origin(role),
            Self::Provided(mapping) => mapping.role_origin(role),
        }
    }
    pub(in crate::sema) fn empty_module_trait(&self, owner: Owner) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.empty_module_trait(owner),
            Self::Provided(mapping) => {
                mapping.owner(owner)?;
                Ok(false)
            }
        }
    }
    pub(in crate::sema) fn denies_implementation(
        &self,
        owner: Owner,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.denies_implementation(owner),
            Self::Provided(mapping) => {
                Ok(mapping.trait_flags(owner)? & s::TraitFlags::DENY_IMPL != 0)
            }
        }
    }
    pub(in crate::sema) fn implementation_domain(
        &self,
        owner: Owner,
    ) -> Result<s::ImplementationDomain, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.implementation_domain(owner),
            Self::Provided(mapping) => {
                if mapping.trait_flags(owner)? & s::TraitFlags::RESTRICTED_IMPL != 0 {
                    return Err(
                        "selected restricted implementation domain requires its actual policy producer",
                    );
                }
                Ok(s::ImplementationDomain::Unrestricted)
            }
        }
    }
    pub(in crate::sema) fn implementation_request(
        &self,
        store: &Store,
        owner: Owner,
    ) -> Result<Predicate, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.implementation_request(store, owner),
            Self::Provided(mapping) => mapping.implementation_request(store, owner),
        }
    }
    pub(in crate::sema) fn implementation(
        &self,
        store: &Store,
        owner: Owner,
        predicate: Predicate,
    ) -> Result<(bool, bool), &'static str> {
        match self {
            Self::Ordinary(graph) => graph.implementation(store, owner, predicate),
            Self::Provided(mapping) => mapping.implementation(store, owner, predicate),
        }
    }
}
