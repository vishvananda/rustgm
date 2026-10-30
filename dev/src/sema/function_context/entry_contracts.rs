//! A default's defining context is not permission to execute it as a root.
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    types::{Kind, instance::Instance},
};

impl Declarations {
    /// Candidate data only; the reached call and fresh implementation proof remain.
    pub(crate) fn const_default_member(
        &self,
        instance: Instance,
    ) -> Result<Option<DefId>, &'static str> {
        let Some(Kind::FunctionItem { item, .. }) = self.types.kind(instance.ty()) else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some(owner) = self.associated.trait_owner(definition) else {
            return Ok(None);
        };
        if self.associated.constant_trait(owner) != Some(true) {
            return Ok(None);
        }
        if self.nonconst_trait_member(definition)? {
            return Err("non-const trait methods cannot be called in const contexts");
        }
        let header = self
            .function_header(definition)?
            .ok_or("const default requires its completed actual function header")?;
        if header.body.is_none() {
            return Err("source abstract const-trait member execution is not implemented");
        }
        if header.declaration.source().is_none() {
            return Err("const default requires its actual source body");
        }
        Ok(Some(definition))
    }

    /// No proof is cached as call permission. Ordinary body/Code preparation follows.
    pub(crate) fn check_const_default(
        &mut self,
        report: &Report,
        instance: Instance,
    ) -> Result<(), &'static str> {
        self.charge_caller_inputs(16, 0)?;
        let definition = self
            .const_default_member(instance)?
            .ok_or("const default changed its actual trait member")?;
        if !matches!(
            self.source_non_const_trait_method_contract(report, definition)
                .map_err(|_| "const default changed its current source member header")?,
            ContractKnowledge::KnownAbsent
        ) {
            return Err("const default lost its completed current member contract");
        }
        super::super::dispatch::check_const_default(self, instance)
    }
}
