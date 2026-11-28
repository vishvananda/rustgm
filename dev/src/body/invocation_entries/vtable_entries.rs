//! Vtable conversion entries retain exact source Self before runtime erasure.
use super::{Entry, call_signature, select_execution};
use crate::{
    body::{
        ir::{Body, ValueId},
        regions::Contract,
    },
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind, RegionId, TypeId, substitute::Bound, unsizing},
};

#[derive(Clone, Copy)]
pub(super) struct Conversion {
    concrete: TypeId,
    signature: TypeId,
    region: RegionId,
    permissions: [bool; 3],
}

pub(super) fn select(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    site: ValueId,
    operand: ValueId,
    conversion: unsizing::Conversion<TypeId>,
    proofs: &mut crate::body::instantiate::ProofCursor,
) -> Result<Option<[Option<Entry>; 3]>, &'static str> {
    let (concrete, object, nominal_region) = match conversion {
        unsizing::Conversion::Object { concrete, target } => (concrete, target, None),
        unsizing::Conversion::Nominal { source, target } => {
            let object = if body.source.source().is_some() {
                let row = body.nominal_use(&facts.types, site)?;
                crate::sema::values::Reader::new(facts)
                    .nominal_object(row.source, row.from, row.to)?
            } else {
                crate::sema::values::Reader::new(facts).nominal_object_plan(source, target)?
            };
            let Some((concrete, object, region)) = object else {
                return Ok(None);
            };
            (concrete, object, Some(region))
        }
        _ => return Ok(None),
    };
    let (leaf, visits) = crate::types::closure::reference_leaf(&facts.types, concrete)?;
    facts.body_work.canonical_steps += visits;
    let Some(leaf) = leaf else {
        return Ok(None);
    };
    let Some(Kind::Closure {
        signature,
        environment,
        ..
    }) = facts.types.kind(leaf)
    else {
        return Ok(None);
    };
    if environment == TypeId::UNIT {
        return Ok(None);
    }
    let mut permissions = crate::sema::vtable_plans::captured_permissions(facts, object)?;
    if nominal_region.is_none()
        && let Some(operand_ty) = body.values.get(operand.0 as usize).map(|value| value.ty)
        && matches!(facts.types.kind(operand_ty), Some(Kind::Reference { .. }))
    {
        permissions[2] = false;
    }
    if !permissions.iter().any(|&required| required) {
        return Ok(None);
    }
    let Some(Kind::Function { bound_regions, .. }) = facts.types.kind(signature) else {
        return Err("captured object entry lacks its public signature");
    };
    let region = if let Some(region) = nominal_region {
        // The checked pointer field supplies its borrow bound. For raw storage
        // the object's own lifetime bounds the callable receiver.
        region
    } else {
        let ty = body
            .values
            .get(operand.0 as usize)
            .ok_or("captured conversion has an invalid input")?
            .ty;
        match facts.types.kind(ty) {
            Some(Kind::Reference { region, .. }) => region,
            Some(Kind::Pointer { .. }) => RegionId::STATIC,
            _ => return Err("captured object entry requires a checked reference conversion"),
        }
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
    ) -> Result<[Option<Entry>; 3], &'static str> {
        let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
            return Err("captured object entry lacks its opened signature");
        };
        let mut entries = [None; 3];
        let source = contract.source;
        for (index, kind) in [CallTrait::Fn, CallTrait::FnMut, CallTrait::FnOnce]
            .into_iter()
            .enumerate()
        {
            if !self.permissions[index] {
                continue;
            }
            let receiver = if kind == CallTrait::FnOnce {
                self.concrete
            } else {
                facts.types.intern(Kind::Reference {
                    referent: self.concrete,
                    mutable: kind == CallTrait::FnMut,
                    region: self.region,
                })?
            };
            let selected = call_signature(&mut facts.types, receiver, signature, result)?;
            entries[index] = Some(
                select_execution(facts, contract, source, kind, receiver, selected, proofs)?
                    .ok_or("captured object target lost its environment entry")?,
            );
        }
        Ok(entries)
    }
}
