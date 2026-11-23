//! Object representation equality opens the two independently bound shared inputs.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::bindings::SourceIntrinsic,
    types::{Kind, Primitive, RegionId, Store, TypeId, arguments::Argument, callable::Convention},
};

pub(crate) const NAME: &str = "raw_eq";

pub(crate) fn checked_type(
    facts: &Declarations,
    callee: TypeId,
    opened: TypeId,
) -> Result<(TypeId, [TypeId; 2]), &'static str> {
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = types.kind(callee)
    else {
        return Err("source raw equality requires its complete function item");
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source raw equality requires its source catalog")?;
    if !matches!(
        catalog.intrinsic_kind(DefId(item.0))?,
        Some(SourceIntrinsic::Declaration(identity)) if identity.name() == NAME
    ) {
        return Err("source raw equality lost its attributed declaration");
    }
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source raw equality lost its type argument")?;
    let (Some(Argument::Type(element)), None) = (arguments.next(), arguments.next()) else {
        return Err("source raw equality requires exactly one type argument");
    };
    let [left, right] = inputs(types, signature, 2)?;
    let (left_region, public_left) = shared_reference(types, left)?;
    let (right_region, public_right) = shared_reference(types, right)?;
    if left_region == right_region
        || ![left_region, right_region]
            .into_iter()
            .all(|region| matches!(region.bound_parts(), Some((0, 0 | 1))))
    {
        return Err("source raw equality requires two independently bound shared inputs");
    }
    let roots =
        super::intrinsic_contracts::item_roots(types, [public_left, public_right], |work| {
            crate::sema::header_attributes::charge_source_contract_query(facts, work)
                .map_err(|_| "source raw equality instance query work exhausted")
        })?;
    let actual @ [left, right] = inputs(types, opened, 0)?;
    if roots != Some([element; 2])
        || shared_reference(types, left)?.1 != element
        || shared_reference(types, right)?.1 != element
    {
        return Err("source raw equality inputs differ from its exact referent type");
    }
    Ok((element, actual))
}

fn inputs(types: &Store, signature: TypeId, regions: u32) -> Result<[TypeId; 2], &'static str> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: true,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source raw equality requires an unsafe Rust signature");
    };
    if bound_regions != regions || result != TypeId::primitive(Primitive::Bool) {
        return Err("source raw equality has a different binder or result");
    }
    let Some(&[left, right]) = types.list(parameters) else {
        return Err("source raw equality requires two inputs");
    };
    Ok([left, right])
}

fn shared_reference(types: &Store, ty: TypeId) -> Result<(RegionId, TypeId), &'static str> {
    match types.kind(ty) {
        Some(Kind::Reference {
            mutable: false,
            region,
            referent,
        }) => Ok((region, referent)),
        _ => Err("source raw equality requires shared references"),
    }
}
