//! Vtable conversion entries retain exact source Self before runtime erasure.
use super::{Entry, call_signature, select_execution};
use crate::{
    body::{ir::Body, regions::Contract},
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind, TypeId, substitute::Bound},
};

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
    let Some(Kind::Function {
        bound_regions: 0, ..
    }) = facts.types.kind(signature)
    else {
        return Err("captured object entry requires quantified call proof transport");
    };
    let Some(Kind::Reference { region, .. }) = facts.types.kind(operand) else {
        return Err("captured object entry requires a checked reference conversion");
    };
    // Open the public function's empty binder, preserving every outer capture
    // lifetime and any callable binders nested inside its argument/result types.
    let mut open = Bound::new(Vec::new());
    let signature = open.signature(&mut facts.types, signature, 0)?;
    facts.body_work.canonical_steps += open.steps;
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("captured object entry lacks its opened signature");
    };
    let mut entries = [None; 2];
    for (index, kind) in [CallTrait::Fn, CallTrait::FnMut].into_iter().enumerate() {
        if !permissions[index] {
            continue;
        }
        let receiver = facts.types.intern(Kind::Reference {
            referent: concrete,
            mutable: kind == CallTrait::FnMut,
            region,
        })?;
        let selected = call_signature(&mut facts.types, receiver, signature, result)?;
        entries[index] = Some(
            select_execution(
                facts,
                contract,
                body.source,
                kind,
                receiver,
                selected,
                proofs,
            )?
            .ok_or("captured object target lost its environment entry")?,
        );
    }
    Ok(Some(entries))
}
