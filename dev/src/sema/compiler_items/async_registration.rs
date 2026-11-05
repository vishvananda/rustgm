//! Async cleanup absence requires the exhaustive actual implementation pass.
use super::Catalog;
use crate::{resolve::body_origin::DeclarationOrigin, support::bindings::LangItem};

impl Catalog {
    /// Source registration completes separately from the ordinary trait header.
    pub(in crate::sema) fn complete_async_destructor_registration(
        &mut self,
    ) -> Result<(), &'static str> {
        let Some(declaration) = self.validated_role(LangItem::AsyncDrop)? else {
            return Ok(());
        };
        if matches!(declaration.origin, DeclarationOrigin::Imported(_)) {
            return Err("imported async destruction requires its complete source profile");
        }
        if self.async_destruction_registration.is_some() {
            return Err("source async destructor registration was completed twice");
        }
        self.async_destruction_registration = Some(declaration.definition);
        Ok(())
    }

    /// A reserved trait never authorizes an absent asynchronous destructor.
    pub(crate) fn require_async_destructor_registration(&self) -> Result<(), &'static str> {
        let Some(declaration) = self.validated_role(LangItem::AsyncDrop)? else {
            return if self.async_destruction_registration.is_none() {
                Ok(())
            } else {
                Err("async destructor registration has no actual source declaration")
            };
        };
        if matches!(declaration.origin, DeclarationOrigin::Imported(_)) {
            return Err("imported async destruction requires its complete source profile");
        }
        if self.async_destruction_registration != Some(declaration.definition) {
            return Err("source async destructor registration requires exhaustive preparation");
        }
        Ok(())
    }
}
