//! Dispatch representation is checked on each actual impl, independently of Unsize bounds.
use super::{Checker, Error, NodeKey};
use crate::{
    sema::{model::Declarations, nominals::Proof},
    support::{bindings::LangItem, constness::Constness},
    types::{
        Kind, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

impl Checker<'_> {
    pub(super) fn validate_source_dispatch_impl(
        &mut self,
        source: NodeKey,
        predicate: Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let result = validate(&mut self.facts, predicate, proof);
        result.map_err(|message| self.error(source, message))
    }
}

fn validate(
    facts: &mut Declarations,
    predicate: Predicate,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    let Some((id, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Ok(());
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("invalid DispatchFromDyn impl application")?;
    if crate::sema::compiler_items::unsizing_role(
        facts,
        crate::resolve::model::DefId(application.item.0),
    )? != Some(LangItem::DispatchFromDyn)
    {
        return Ok(());
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or("DispatchFromDyn impl lost its source catalog")?
        .header_role(LangItem::CoerceUnsized)?
        .ok_or("DispatchFromDyn impl requires actual CoerceUnsized")?;
    let mut arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing DispatchFromDyn impl arguments")?;
    let (Some(Argument::Type(target)), None) = (arguments.next(), arguments.next()) else {
        return Err("DispatchFromDyn impl requires exactly its actual target type");
    };
    match (facts.types.kind(predicate.ty), facts.types.kind(target)) {
        (
            Some(Kind::Reference {
                region: a,
                mutable: ma,
                ..
            }),
            Some(Kind::Reference {
                region: b,
                mutable: mb,
                ..
            }),
        ) if a == b && ma == mb => Ok(()),
        (Some(Kind::Pointer { mutable: a, .. }), Some(Kind::Pointer { mutable: b, .. }))
            if a == b =>
        {
            Ok(())
        }
        (Some(Kind::Pattern { constraint: a, .. }), Some(Kind::Pattern { constraint: b, .. }))
            if a == b =>
        {
            Ok(())
        }
        (Some(Kind::Adt(a, _)), Some(Kind::Adt(b, _))) if a == b => {
            nominal(facts, predicate.ty, target, application.item, proof)
        }
        _ => Err(
            "DispatchFromDyn impl must preserve its reference lifetime, pointer mutability or nominal owner",
        ),
    }
}

fn nominal(
    facts: &mut Declarations,
    source: TypeId,
    target: TypeId,
    role: crate::types::ItemId,
    proof: &mut Proof,
) -> Result<(), &'static str> {
    let Some(Kind::Adt(item, _)) = facts.types.kind(source) else {
        return Err("missing DispatchFromDyn nominal source");
    };
    let definition = facts
        .aggregates
        .get(item)
        .ok_or("missing DispatchFromDyn nominal declaration")?;
    if definition.kind != crate::types::aggregate::Kind::Struct
        || definition.repr.representation == crate::types::aggregate::Representation::C
        || definition.repr.packed.is_some()
    {
        return Err("DispatchFromDyn nominal impl requires an unpacked non-C struct");
    }
    facts.aggregates.instantiate(&mut facts.types, source)?;
    facts.aggregates.instantiate(&mut facts.types, target)?;
    let from = fields(facts, source)?;
    let to = fields(facts, target)?;
    let count = facts
        .types
        .list(from)
        .ok_or("missing DispatchFromDyn source fields")?
        .len();
    if facts
        .types
        .list(to)
        .is_none_or(|fields| fields.len() != count)
    {
        return Err("DispatchFromDyn nominal field coverage changed");
    }
    let mut changed = None;
    for index in 0..count {
        crate::sema::header_attributes::charge_source_contract_query(facts, 1)
            .map_err(|_| "DispatchFromDyn field work exhausted")?;
        let a = facts
            .types
            .list(from)
            .ok_or("missing DispatchFromDyn source fields")?[index];
        let b = facts
            .types
            .list(to)
            .ok_or("missing DispatchFromDyn target fields")?[index];
        if matches!(facts.types.kind(a), Some(Kind::Adt(item, _)) if facts.aggregates.get(item).is_some_and(|d| d.kind == crate::types::aggregate::Kind::PhantomData))
        {
            continue;
        }
        if a == b {
            unchanged(facts, a)?;
        } else if changed.replace((a, b)).is_some() {
            return Err(
                "DispatchFromDyn nominal impl has more than one changed non-PhantomData field",
            );
        }
    }
    let (a, b) = changed.ok_or("DispatchFromDyn nominal impl requires one changed field")?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(b)])?;
    let application = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: role,
            arguments,
        })?;
    if !proof.prove(
        facts,
        Predicate {
            ty: a,
            bound: Bound::Trait(application),
        },
    )? {
        return Err("DispatchFromDyn changed field does not prove its actual recursive relation");
    }
    Ok(())
}

fn fields(facts: &Declarations, ty: TypeId) -> Result<crate::types::ListId, &'static str> {
    facts
        .aggregates
        .raw_shape(&facts.types, ty)?
        .variant(0)
        .map(|variant| variant.fields)
        .ok_or("DispatchFromDyn struct has no field declaration")
}

fn unchanged(facts: &mut Declarations, ty: TypeId) -> Result<(), &'static str> {
    if facts.types.contains_parameters(ty) != Some(false) {
        return Err(
            "DispatchFromDyn unchanged field requires a concrete one-aligned zero-sized type",
        );
    }
    facts
        .aggregates
        .prepare_storage_fields(&mut facts.types, ty)?;
    facts
        .layouts
        .demand(&facts.types, &facts.aggregates, ty)
        .map_err(|error| error.message)?;
    let layout = facts
        .layouts
        .get(ty)
        .ok_or("DispatchFromDyn unchanged field layout is missing")?;
    if layout.size != 0 || layout.align != 1 {
        return Err("DispatchFromDyn unchanged field must be one-aligned and zero-sized");
    }
    Ok(())
}
