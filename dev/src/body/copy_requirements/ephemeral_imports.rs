//! One immutable owner keeps raw substitution inputs separate from local proof inputs.
use super::{Body, Contract, Declarations};
use crate::{
    body::transported_originals::ImportedOriginals,
    types::{Store, arena::Identity},
};
use std::rc::Rc;

pub(crate) struct ImportedInputs {
    originals: Rc<ImportedOriginals>,
    contracts: Vec<Contract>,
    arena: Identity,
    bytes: usize,
    base: usize,
}
impl ImportedInputs {
    pub(crate) fn build(
        facts: &mut Declarations,
        originals: Rc<ImportedOriginals>,
        mut produce: impl FnMut(&mut Declarations, &Body, &Contract) -> Result<Contract, String>,
    ) -> Result<Self, String> {
        let base = facts.constant_work.template_bytes;
        let mut reserved = size_of::<Self>().saturating_add(Identity::allocation_bytes());
        for row in originals.iter() {
            facts.body_work.canonical_steps = facts
                .body_work
                .canonical_steps
                .saturating_add(1 + row.contract().proofs.len());
            reserved = reserved.saturating_add(row.contract().retained_bytes());
        }
        if originals.len() > crate::support::crates::MAX_COMPONENTS
            || base.saturating_add(reserved) > 64 * 1024 * 1024
        {
            return Err("prepared imported contracts exceed aggregate storage budget".into());
        }
        // Reserve every destination Contract slot before any cloned vector can grow.
        let mut contracts = Vec::with_capacity(originals.len());
        let mut bytes = size_of::<Self>()
            .saturating_add(Identity::allocation_bytes())
            .saturating_add(crate::support::retained::vector(&contracts));
        for original in originals.iter() {
            let template = original.template();
            let contract = original.contract();
            let prepared = produce(facts, &template.body, contract)?;
            if template.body.instance.is_some()
                || prepared.key != contract.key
                || prepared.source != contract.source
                || prepared.signature != contract.signature
            {
                return Err("prepared imported input changed its immutable original owner".into());
            }
            bytes = bytes.saturating_add(
                prepared
                    .retained_bytes()
                    .saturating_sub(size_of::<Contract>()),
            );
            if base.saturating_add(bytes) > 64 * 1024 * 1024 {
                return Err("prepared imported contracts exceed aggregate storage budget".into());
            }
            contracts.push(prepared);
        }
        let arena = facts.types.arena_identity();
        Ok(Self {
            originals,
            contracts,
            arena,
            bytes,
            base,
        })
    }

    pub(crate) fn inputs<'a>(
        &'a self,
        types: &Store,
        originals: Option<&Rc<ImportedOriginals>>,
    ) -> Result<&'a [Contract], &'static str> {
        if !types.has_identity(&self.arena)
            || originals.is_none_or(|originals| !Rc::ptr_eq(originals, &self.originals))
        {
            return Err("prepared imported inputs belong to another catalog or type arena");
        }
        Ok(&self.contracts)
    }

    pub(crate) const fn retained_bytes(&self) -> usize {
        self.bytes
    }
    pub(crate) const fn combined_bytes(&self) -> usize {
        self.base.saturating_add(self.retained_bytes())
    }
}
