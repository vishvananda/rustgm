//! A source copying view exists only for the exact completed body-region proof.
use crate::{
    body::ir::Body,
    types::{
        ItemId, Store, TypeId,
        aggregate::Definitions,
        copy::{Copies, Declaration},
    },
};
use std::hash::BuildHasher;

pub(crate) struct CopyView<'a> {
    ordinary: &'a mut Copies,
    source: Option<(&'a crate::body::regions::BuiltinRequirements, &'a Body)>,
}
impl<'a> CopyView<'a> {
    pub(crate) fn ordinary(copies: &'a mut Copies) -> Self {
        Self {
            ordinary: copies,
            source: None,
        }
    }
    pub(crate) fn checked(
        copies: &'a mut Copies,
        body: &'a Body,
        proof: &'a super::regions::Body,
        source_copy: bool,
    ) -> Result<Self, &'static str> {
        if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance
        {
            return Err("source Copy view requires this body's completed region proof");
        }
        if source_copy
            && proof
                .builtins
                .as_ref()
                .is_none_or(|r| r.source_copy.is_none())
        {
            return Err("source Copy view requires an explicit body-local source policy");
        }
        Ok(Self {
            ordinary: copies,
            source: proof
                .builtins
                .as_deref()
                .map(|requirements| (requirements, body)),
        })
    }
    pub(crate) fn demand<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        derives: &std::collections::HashMap<ItemId, Declaration, S>,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        if let Some((requirements, body)) = self.source
            && let Some(copy) = requirements.copy_decision(body, ty)?
        {
            return Ok(copy);
        }
        self.ordinary
            .demand(types, definitions, derives, ty)
            .map(|properties| properties.copy)
    }
}
