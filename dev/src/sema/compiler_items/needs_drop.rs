//! A completed actual registry supplies data, never caller execution authority.
use super::Catalog;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::bindings::{LangItem, SourceIntrinsic},
    types::{TypeId, instance::Instance},
};

#[derive(Clone, Copy)]
pub(crate) struct NeedsDropSelection {
    pub definition: DefId,
    pub element: TypeId,
    pub fallback: bool,
}
impl Catalog {
    /// Absence of a role is different from its incomplete implementation inventory.
    pub(crate) fn require_source_destructors_prepared(&self) -> Result<(), &'static str> {
        let Some(declaration) = self.validated_language(LangItem::Drop)? else {
            return if self.destruction_registration.is_none() {
                Ok(())
            } else {
                Err("source destructor registration has no actual Drop declaration")
            };
        };
        if self.destruction_registration != Some(declaration.definition) {
            return Err("source destructor registration requires exhaustive preparation");
        }
        Ok(())
    }
    /// Only the exhaustive declaration pass publishes completion, including empty.
    pub(in crate::sema) fn complete_destructor_registration(&mut self) -> Result<(), &'static str> {
        let Some(declaration) = self.validated_language(LangItem::Drop)? else {
            return Ok(());
        };
        if self.destruction_registration.is_some() {
            return Err("source destructor registration was completed twice");
        }
        self.destruction_registration = Some(declaration.definition);
        Ok(())
    }
    /// A selection remains data. Every executing caller must supply its own witness.
    pub(crate) fn needs_drop_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<NeedsDropSelection>, &'static str> {
        let Some(selected) = self.selected_intrinsic(facts, instance)? else {
            return Ok(None);
        };
        if selected.kind != SourceIntrinsic::NeedsDrop {
            return Ok(None);
        }
        self.require_needs_drop_roles()?;
        Ok(Some(NeedsDropSelection {
            definition: selected.definition,
            element: selected.element,
            fallback: selected.fallback,
        }))
    }
    pub(crate) fn require_needs_drop_roles(&self) -> Result<(), &'static str> {
        for role in [LangItem::Copy, LangItem::Drop] {
            if self.validated_language(role)?.is_none() {
                return Err("source needs_drop requires completed source Drop and Copy roles");
            }
        }
        self.require_source_destructors_prepared()
    }
}
