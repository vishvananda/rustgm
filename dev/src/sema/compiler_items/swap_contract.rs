//! Native swap uses the exact instantiated signature of the attributed intrinsic.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::bindings::SourceIntrinsic,
    types::{Kind, TypeId, arguments::Argument, callable::Convention},
};

pub(crate) const NAME: &str = "typed_swap_nonoverlapping";

pub(crate) fn checked_type(
    facts: &Declarations,
    callee: TypeId,
    opened: TypeId,
) -> Result<(TypeId, TypeId), &'static str> {
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = types.kind(callee)
    else {
        return Err("source typed swap requires its complete function item");
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source typed swap requires its source catalog")?;
    let definition = DefId(item.0);
    if !matches!(
        catalog.intrinsic_kind(definition)?,
        Some(SourceIntrinsic::Declaration(identity)) if identity.name() == NAME
    ) {
        return Err("source typed swap lost its attributed declaration");
    }
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source typed swap lost its type argument")?;
    let (Some(Argument::Type(element)), None) = (arguments.next(), arguments.next()) else {
        return Err("source typed swap requires exactly one type argument");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: true,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source typed swap requires its unsafe Rust signature");
    };
    let Some(&[left, right]) = types.list(parameters) else {
        return Err("source typed swap requires two public pointers");
    };
    let Some([pointer]) = super::intrinsic_contracts::item_roots(types, [left], |work| {
        crate::sema::header_attributes::charge_source_contract_query(facts, work)
            .map_err(|_| "source typed swap instance query work exhausted")
    })?
    else {
        return Err("source typed swap public pointer lost its caller type");
    };
    if left != right
        || types.kind(pointer)
            != Some(Kind::Pointer {
                mutable: true,
                pointee: element,
            })
        || !super::intrinsic_contracts::opened_signature_matches(
            types,
            opened,
            &[pointer; 2],
            TypeId::UNIT,
            true,
            Convention::RUST,
        )
    {
        return Err("source typed swap requires two mutable pointers to its exact type argument");
    }
    Ok((element, pointer))
}
