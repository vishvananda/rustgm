//! The one imported owner serves data queries; body readiness remains distinct.
use super::{Header, Headers, mapped_source};
use crate::{
    resolve::{
        body_origin::{BodyOrigin, Catalog},
        model::DefId,
    },
    sema::{
        crates::ImportedOwners, declaration_inputs::Input, generics::Generics, model::Declarations,
    },
};
impl ImportedOwners {
    pub(super) fn function(
        &self,
        catalog: &Catalog,
        definition: DefId,
        body: Option<BodyOrigin>,
        universals: u32,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.function(catalog, definition, body, universals),
            Self::Provided(owner) => owner.function(catalog, definition, body, universals),
        }
    }
    pub(super) fn declaration_inputs<'a>(
        &'a self,
        catalog: &Catalog,
        generics: &Generics,
        definition: DefId,
        declared: bool,
    ) -> Result<Option<&'a [Input]>, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.declaration_inputs(catalog, definition, declared),
            Self::Provided(owner) => {
                owner.declaration_inputs(catalog, generics, definition, declared)
            }
        }
    }
    pub(super) fn owns_original(
        &self,
        catalog: &Catalog,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.owns_original(catalog, definition, steps),
            Self::Provided(owner) => owner.owns_original(catalog, definition, steps),
        }
    }
    pub(super) fn source_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &Declarations,
        headers: &'a Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Option<mapped_source::SourceOriginal<'a>>, &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.source_original(catalog, facts, headers, definition, steps)
            }
            Self::Provided(owner) => {
                owner.require_store(&facts.types)?;
                owner.owns_original(catalog, definition, steps)?;
                Err("selected original requires its actual pending body/domain proof port")
            }
        }
    }
    pub(super) fn require_method_regions(
        &self,
        catalog: &Catalog,
        definition: DefId,
        scheme: &crate::sema::lifetimes::FunctionRegions,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.require_method_regions(catalog, definition, scheme, steps)
            }
            Self::Provided(_) => {
                Err("selected method requires its actual pending callable-scheme consumer")
            }
        }
    }
    pub(super) fn require_method_environment(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        header: &Header,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.require_method_environment(catalog, facts, definition, header, steps)
            }
            Self::Provided(_) => {
                Err("selected method requires its actual pending declaration-scope consumer")
            }
        }
    }
    pub(super) fn trait_header<'a>(
        &self,
        catalog: &Catalog,
        facts: &'a Declarations,
        role: crate::sema::compiler_items::Role,
        steps: &mut usize,
    ) -> Result<crate::sema::compiler_items::headers::Trait<'a>, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.trait_header(catalog, facts, role, steps),
            Self::Provided(_) => {
                Err("selected trait requires its actual pending canonical role-header consumer")
            }
        }
    }
    pub(super) fn readiness(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        headers: &Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<mapped_source::Readiness, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.readiness(catalog, facts, headers, definition, steps),
            Self::Provided(_) => {
                Err("selected declaration requires its actual pending readiness consumer")
            }
        }
    }
}
