//! Vtable conversion entries retain exact source Self before runtime erasure.
use super::{Entry, call_signature, select_execution};
use crate::{
    body::{ir::Body, regions::Contract},
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind, RegionId, TypeId, substitute::Bound},
};

#[derive(Clone, Copy)]
pub(super) struct Conversion {
    concrete: TypeId,
    signature: TypeId,
    region: RegionId,
    permissions: [bool; 2],
}

pub(super) fn select(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    concrete: TypeId,
    object: TypeId,
    operand: TypeId,
    proofs: &mut crate::body::instantiate::ProofCursor,
) -> Result<Option<[Option<Entry>; 2]>, &'static str> {
    let Some(Kind::Closure {
        signature,
        environment,
        ..
    }) = facts.types.kind(concrete)
    else {
        return Ok(None);
    };
    if environment == TypeId::UNIT {
        return Ok(None);
    }
    let permissions = crate::sema::vtable_plans::captured_permissions(facts, object)?;
    if !permissions.iter().any(|&required| required) {
        return Ok(None);
    }
    let Some(Kind::Function { bound_regions, .. }) = facts.types.kind(signature) else {
        return Err("captured object entry lacks its public signature");
    };
    let Some(Kind::Reference { region, .. }) = facts.types.kind(operand) else {
        return Err("captured object entry requires a checked reference conversion");
    };
    if body.source != contract.source {
        return Err("captured conversion differs from its lifetime owner");
    }
    let conversion = Conversion {
        concrete,
        signature,
        region,
        permissions,
    };
    if bound_regions != 0 {
        return super::vtable_scopes::select(facts, contract, conversion, bound_regions, proofs)
            .map(Some);
    }
    let signature = conversion.open(facts, Vec::new())?;
    conversion
        .entries(facts, contract, signature, proofs)
        .map(Some)
}

impl Conversion {
    /// Remove only the public call binder. Nested callable binders and outer
    /// capture lifetimes retain their own identities.
    pub(super) fn open(
        self,
        facts: &mut Declarations,
        arguments: Vec<RegionId>,
    ) -> Result<TypeId, &'static str> {
        let mut open = Bound::new(arguments);
        let signature = open.signature(&mut facts.types, self.signature, 0)?;
        facts.body_work.canonical_steps += open.steps;
        Ok(signature)
    }

    pub(super) fn entries(
        self,
        facts: &mut Declarations,
        contract: &mut Contract,
        signature: TypeId,
        proofs: &mut crate::body::instantiate::ProofCursor,
    ) -> Result<[Option<Entry>; 2], &'static str> {
        let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
            return Err("captured object entry lacks its opened signature");
        };
        let mut entries = [None; 2];
        let source = contract.source;
        for (index, kind) in [CallTrait::Fn, CallTrait::FnMut].into_iter().enumerate() {
            if !self.permissions[index] {
                continue;
            }
            let receiver = facts.types.intern(Kind::Reference {
                referent: self.concrete,
                mutable: kind == CallTrait::FnMut,
                region: self.region,
            })?;
            let selected = call_signature(&mut facts.types, receiver, signature, result)?;
            entries[index] = Some(
                select_execution(facts, contract, source, kind, receiver, selected, proofs)?
                    .ok_or("captured object target lost its environment entry")?,
            );
        }
        Ok(entries)
    }
}
