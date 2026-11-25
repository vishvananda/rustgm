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
    pub(super) fn owns_closure_original(
        &self,
        catalog: &Catalog,
        identity: crate::types::closure::ClosureId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.owns_closure_original(catalog, identity, steps),
            Self::Provided(_) => Ok(false),
        }
    }
    pub(super) fn owns_anonymous_original(
        &self,
        catalog: &Catalog,
        body: crate::resolve::body_origin::ImportedBodyId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.owns_anonymous_original(catalog, body, steps),
            Self::Provided(_) => Ok(false),
        }
    }
    pub(super) fn closure_identity_for_body(
        &self,
        catalog: &Catalog,
        body: crate::support::crates::owner_catalog::BodyRef,
        steps: &mut usize,
    ) -> Result<Option<crate::types::closure::ClosureId>, &'static str> {
        match self {
            Self::Ordinary(graph) => graph.closure_identity_for_body(catalog, body, steps),
            Self::Provided(_) => Ok(None),
        }
    }
    pub(super) fn source_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
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
    pub(super) fn source_closure_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a Headers,
        identity: crate::types::closure::ClosureId,
        steps: &mut usize,
    ) -> Result<Option<mapped_source::SourceOriginal<'a>>, &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.source_closure_original(catalog, facts, headers, identity, steps)
            }
            Self::Provided(_) => {
                Err("selected original does not own ordinary source closure bodies")
            }
        }
    }
    pub(super) fn source_anonymous_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a Headers,
        body: crate::resolve::body_origin::ImportedBodyId,
        steps: &mut usize,
    ) -> Result<Option<mapped_source::SourceOriginal<'a>>, &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.source_anonymous_original(catalog, facts, headers, body, steps)
            }
            Self::Provided(_) => {
                Err("selected original does not own ordinary source anonymous bodies")
            }
        }
    }
    pub(super) fn require_method_regions(
        &self,
        catalog: &Catalog,
        store: &crate::types::Store,
        definition: DefId,
        scheme: &crate::sema::lifetimes::FunctionRegions,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        match self {
            Self::Ordinary(graph) => {
                graph.require_method_regions(catalog, definition, scheme, steps)
            }
            Self::Provided(owner) => {
                owner.require_method_regions(catalog, store, definition, scheme, steps)
            }
        }
    }
    pub(super) fn require_method_declaration(
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
            Self::Provided(owner) => {
                owner.require_method_declaration(catalog, facts, definition, header, steps)
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
            Self::Provided(owner) => owner.trait_header(catalog, facts, role, steps),
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

impl super::super::Linkage {
    pub(crate) fn constant_initializer(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<Option<crate::resolve::body_origin::BodyOrigin>, &'static str> {
        let Some(headers) = self.originals.as_deref() else {
            return Ok(None);
        };
        let Some(ImportedOwners::Ordinary(graph)) = headers.source.as_ref() else {
            return Ok(None);
        };
        let catalog = self
            .origin_catalog()
            .ok_or("imported initializer lost its declaration catalog")?;
        headers
            .charge(6)
            .map_err(|_| "source constant lookup exceeds header work budget")?;
        let Some(raw) = graph.constant_original(catalog, facts, definition)? else {
            return Ok(None);
        };
        let (owner, _, _) = catalog.declaration_coordinate(catalog.declaration(definition)?)?;
        catalog.origin(catalog.body_at(owner, raw)?).map(Some)
    }

    pub(in crate::sema) fn source_associated_constant(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<
        Option<(
            DefId,
            crate::support::crates::generics::source::ConstantBody,
        )>,
        &'static str,
    > {
        let Some(headers) = self.originals.as_deref() else {
            return Ok(None);
        };
        let Some(ImportedOwners::Ordinary(graph)) = headers.source.as_ref() else {
            return Ok(None);
        };
        let catalog = self
            .origin_catalog()
            .ok_or("source constant lookup lost its current catalog")?;
        headers
            .charge(6)
            .map_err(|_| "source constant lookup exceeds header work budget")?;
        graph.associated_constant(catalog, facts, definition)
    }

    pub(crate) fn selected_caller_tracking_absent(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        let Some(headers) = self.originals.as_deref() else {
            return Ok(false);
        };
        let Some(ImportedOwners::Provided(owner)) = headers.source.as_ref() else {
            return Ok(false);
        };
        let mut steps = 3_usize;
        let result = (|| {
            let actual = self
                .checked_original_header(definition)?
                .ok_or("selected caller policy lacks its installed canonical header")?;
            let semantic = facts
                .declaration_header(definition)?
                .ok_or("selected caller policy lacks its actual semantic function")?;
            if actual.declaration != semantic.declaration {
                return Err("selected caller policy changed its canonical declaration");
            }
            let catalog = self
                .origin_catalog()
                .ok_or("selected caller policy lost its catalog")?;
            owner.caller_tracking_absent(catalog, facts, definition, &mut steps)?;
            Ok(true)
        })();
        // Immutable policy queries pay through the same installed imported
        // header owner, including attempted authentication on rejection.
        headers
            .charge(steps)
            .map_err(|_| "selected caller policy work budget exceeded")?;
        result
    }
}
