//! Builtin coherence shape choices supply no operation lifetime permission.
use super::declaration;
use crate::{
    sema::{model::Declarations, nominals::Proof},
    support::bindings::LangItem,
    types::{
        Kind, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

pub(super) fn reborrow(
    facts: &mut Declarations,
    field: &super::structure::Field,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    if matches!(
        facts.types.kind(field.declared),
        Some(Kind::Reference { mutable: true, .. })
    ) {
        return Ok(());
    }
    let ty = proof.normalize(facts, field.ty)?;
    if prove(facts, ty, None, LangItem::Reborrow, proof)?
        || prove(facts, ty, None, LangItem::Copy, proof)?
    {
        return Ok(());
    }
    Err("Reborrow requires every data field to implement actual Reborrow or Copy")
}

pub(super) fn shared(
    facts: &mut Declarations,
    from: &super::structure::Field,
    to: &super::structure::Field,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    if matches!(
        (
            facts.types.kind(from.declared),
            facts.types.kind(to.declared)
        ),
        (
            Some(Kind::Reference { mutable: true, .. }),
            Some(Kind::Reference { mutable: false, .. })
        )
    ) {
        let a = peel_references(facts, from.declared)?;
        let b = peel_references(facts, to.declared)?;
        if corresponding(facts, a, b)? {
            return Ok(());
        }
    }
    let from = proof.normalize(facts, from.ty)?;
    let to = proof.normalize(facts, to.ty)?;
    if corresponding(facts, from, to)? {
        // The pinned builtin does not resolve temporary equality regions in
        // either early branch. Unused differing-field-lifetime impls are legal;
        // the actual operation must independently prove its directed field loans.
        if prove(facts, from, None, LangItem::Copy, proof)? {
            return Ok(());
        }
        return Err("CoerceShared equal data fields must implement actual Copy");
    }
    if prove(facts, from, Some(to), LangItem::CoerceShared, proof)? {
        return Ok(());
    }
    Err("CoerceShared data fields do not prove their actual relation")
}

fn corresponding(facts: &Declarations, a: TypeId, b: TypeId) -> Result<bool, &'static str> {
    if a == b {
        return Ok(true);
    }
    let pay = crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "reborrowing field comparison lacks its source payer")?;
    let charge = |w, b| pay.charge(w, b);
    crate::types::structural::Normalized::<false>::new_charged(
        &facts.types,
        &charge,
        crate::types::structural::Memo::default(),
    )
    .matches_charged(a, b)
}

fn prove(
    facts: &mut Declarations,
    ty: TypeId,
    target: Option<TypeId>,
    role: LangItem,
    proof: &mut Proof,
) -> Result<bool, &'static str> {
    let item = declaration(facts, role)?;
    let arguments = if let Some(target) = target {
        facts.types.intern_arguments(&[Argument::Type(target)])?
    } else {
        facts.types.intern_arguments(&[])?
    };
    let application = facts
        .types
        .intern_trait_application(crate::types::traits::Application { item, arguments })?;
    proof.prove(
        facts,
        Predicate {
            ty,
            bound: Bound::Trait(application),
        },
    )
}

fn peel_references(facts: &Declarations, mut ty: TypeId) -> Result<TypeId, &'static str> {
    let pay = crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "reborrowing reference traversal lacks its source payer")?;
    for _ in 0..1_000_000 {
        pay.charge(1, 0)?;
        match facts
            .types
            .kind(ty)
            .ok_or("reborrowing reference traversal has an invalid type")?
        {
            Kind::Reference { referent, .. } => ty = referent,
            _ => return Ok(ty),
        }
    }
    Err("reborrowing reference traversal work budget exceeded")
}
