//! The source box write keeps the owning box and its actual uninitialized content.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::bindings::{LangItem, SourceIntrinsic},
    types::{Kind, TypeId, arguments::Argument, callable::Convention},
};

pub(crate) const NAME: &str = "write_box_via_move";

pub(crate) struct Selection {
    pub boxed: TypeId,
    pub value: TypeId,
    pub storage: TypeId,
}

pub(crate) fn checked_type(
    facts: &Declarations,
    target: TypeId,
    opened: TypeId,
) -> Result<Selection, &'static str> {
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = types.kind(target)
    else {
        return Err("source box write requires its complete function item");
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source box write requires its source catalog")?;
    if !matches!(
        catalog.intrinsic_kind(DefId(item.0))?,
        Some(SourceIntrinsic::Declaration(identity)) if identity.name() == NAME
    ) {
        return Err("source box write lost its attributed declaration");
    }
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source box write lost its type argument")?;
    let (Some(Argument::Type(value)), None) = (arguments.next(), arguments.next()) else {
        return Err("source box write requires exactly one type argument");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source box write requires its safe Rust signature");
    };
    let Some(&[public_box, public_value]) = types.list(parameters) else {
        return Err("source box write requires two public inputs");
    };
    let roots =
        super::intrinsic_contracts::item_roots(types, [public_box, public_value], |work| {
            crate::sema::header_attributes::charge_source_contract_query(facts, work)
                .map_err(|_| "source box write instance query work exhausted")
        })?;
    let Some([boxed, applied]) = roots else {
        return Err("source box write public inputs lost their caller types");
    };
    if public_box != result
        || applied != value
        || !super::intrinsic_contracts::opened_signature_matches(
            types,
            opened,
            &[boxed, value],
            boxed,
            false,
            Convention::RUST,
        )
    {
        return Err("source box write must return its moved box and take the selected value");
    }
    let storage = super::boxes::content(types, &facts.aggregates, boxed)?
        .ok_or("source box write requires the completed owning box")?;
    let uninit = catalog
        .validated_language(LangItem::MaybeUninit)?
        .ok_or("source box write requires the completed MaybeUninit role")?;
    let Some(Kind::Adt(owner, args)) = types.kind(storage) else {
        return Err("source box write requires its actual MaybeUninit content");
    };
    let mut args = types
        .arguments(args)
        .ok_or("source box write lost the MaybeUninit substitution")?;
    if owner.0 != uninit.definition.0
        || args.next() != Some(Argument::Type(value))
        || args.next().is_some()
    {
        return Err("source box write requires MaybeUninit of the value being moved");
    }
    Ok(Selection {
        boxed,
        value,
        storage,
    })
}
