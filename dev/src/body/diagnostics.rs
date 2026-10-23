//! Body-owned locations resolve through exact metadata, never fabricated syntax.
use super::ir::ProofSite;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
};
impl Report {
    #[must_use]
    pub fn body_optional_error(
        &self,
        site: Option<ProofSite>,
        message: impl Into<String>,
    ) -> Error {
        match site {
            Some(site) => self.body_error(site, message),
            None => Error::unlocated(message),
        }
    }
    #[must_use]
    pub fn body_error(&self, site: impl Into<ProofSite>, message: impl Into<String>) -> Error {
        let site = site.into();
        if let Some(source) = site.source() {
            return Declarations::error(self, source, message);
        }
        let location = self
            .resolution
            .crates
            .as_deref()
            .ok_or("imported diagnostic has no current ownership catalog")
            .and_then(crate::resolve::crates::Imported::origin_catalog)
            .and_then(|catalog| {
                catalog.ok_or("imported diagnostic has no current ownership catalog")
            })
            .and_then(|catalog| catalog.diagnostic(site));
        match location {
            Ok(location) => Error::imported(location, message),
            Err(error) => Error::unlocated(format!("{}; {error}", message.into())),
        }
    }
}

/// Borrowed rendering metadata; no body, original or proof permission is retained.
#[derive(Clone, Copy)]
pub struct BodyOrigins<'a> {
    catalog: Option<&'a crate::resolve::body_origin::Catalog>,
}
impl Declarations {
    #[must_use]
    pub fn body_origins(&self) -> BodyOrigins<'_> {
        self.crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::origin_catalog)
            .map_or_else(BodyOrigins::source_only, BodyOrigins::from_catalog)
    }
}
impl<'a> BodyOrigins<'a> {
    pub(crate) const fn source_only() -> Self {
        Self { catalog: None }
    }
    pub(crate) const fn from_catalog(catalog: &'a crate::resolve::body_origin::Catalog) -> Self {
        Self {
            catalog: Some(catalog),
        }
    }
    fn catalog(self) -> std::io::Result<&'a crate::resolve::body_origin::Catalog> {
        self.catalog
            .ok_or_else(|| std::io::Error::other("imported view has no ownership catalog"))
    }
    pub(crate) fn body(
        self,
        origin: super::ir::BodyOrigin,
        out: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        if let Some(source) = origin.source() {
            return write!(out, "{}:{}", source.file.0, source.node.get());
        }
        let super::ir::BodyOrigin::Imported(imported) = origin else {
            return Err(std::io::Error::other("missing imported origin"));
        };
        let catalog = self.catalog()?;
        if catalog
            .origin(imported.id())
            .map_err(std::io::Error::other)?
            != origin
        {
            return Err(std::io::Error::other(
                "imported view changed body site bounds",
            ));
        }
        self.proof(origin.root_site(), out)
    }
    pub(crate) fn proof(
        self,
        site: ProofSite,
        out: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        if let Some(source) = site.source() {
            return write!(out, "{}:{}", source.file.0, source.node.get());
        }
        let (owner, identity, ordinal) = self
            .catalog()?
            .proof_coordinate(site)
            .map_err(std::io::Error::other)?;
        write!(out, "crate:{}[{}]:", owner.name(), owner.disambiguator())?;
        match identity {
            crate::resolve::body_origin::LocationOwner::Body(body) => {
                write!(out, "body:{}", body.0)?;
            }
            crate::resolve::body_origin::LocationOwner::Declaration(declaration) => {
                write!(out, "declaration:{}", declaration.0)?;
            }
        }
        write!(out, ":site:{ordinal}")
    }
}
