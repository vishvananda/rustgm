//! Source atomic loads and stores consume their checked call and exact ordering argument.
use crate::{
    resolve::model::DefId,
    sema::{model::Declarations, values::Reader},
    support::atomic_ordering::Ordering,
    types::{
        GenericArgs, Kind, Store, TypeId,
        arguments::{Argument, ConstId, ConstKind, trees::Aggregate},
        callable::Convention,
        parameters::ParameterKind,
    },
};

#[derive(Clone, Copy)]
pub(crate) struct Selection {
    pub pointer: TypeId,
    pub stored: Option<TypeId>,
    pub result: TypeId,
    pub ordering: Ordering,
}

pub(crate) fn select(
    facts: &Declarations,
    name: &str,
    callee: TypeId,
    signature: TypeId,
) -> Result<Option<Selection>, &'static str> {
    let storing = match name {
        "atomic_load" => false,
        "atomic_store" => true,
        _ => return Ok(None),
    };
    let types = &facts.types;
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = types.kind(callee)
    else {
        return Err("source atomic access requires its function item");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("missing source atomic arguments")?;
    let (Some(Argument::Type(word)), Some(Argument::Const(order)), None) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err("source atomic access requires its type and ordering argument");
    };
    let Some(ordering) = ordering(facts, DefId(item.0), 1, order)? else {
        return Ok(None);
    };
    let valid_order = if storing {
        ordering.store_valid()
    } else {
        ordering.load_valid()
    };
    if !valid_order {
        return Err("invalid source atomic access ordering");
    }
    word_kind(types, word)?;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: true,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source atomic access requires its opened unsafe signature");
    };
    let (pointer, stored) = match (storing, types.list(parameters)) {
        (false, Some(&[pointer])) if result == word => (pointer, None),
        (true, Some(&[pointer, value])) if value == word && result == TypeId::UNIT => {
            (pointer, Some(value))
        }
        _ => return Err("source atomic access has different inputs or result"),
    };
    if !matches!(types.kind(pointer), Some(Kind::Pointer { mutable, pointee })
        if mutable == storing && pointee == word)
    {
        return Err("source atomic access has a different pointer type");
    }
    Ok(Some(Selection {
        pointer,
        stored,
        result,
        ordering,
    }))
}

pub(super) enum Word {
    Integer { signed: bool },
    Pointer,
}

pub(super) fn word_kind(types: &Store, word: TypeId) -> Result<Word, &'static str> {
    match types.kind(word) {
        Some(Kind::Pointer { .. }) => Ok(Word::Pointer),
        Some(Kind::Primitive(primitive)) => primitive
            .integer()
            .filter(|(_, bits)| (8..=64).contains(bits))
            .map(|(signed, _)| Word::Integer { signed })
            .ok_or("source atomic access requires a supported integer word"),
        _ => Err("source atomic access requires an integer or pointer word"),
    }
}

pub(super) fn ordering(
    facts: &Declarations,
    definition: DefId,
    index: usize,
    order: ConstId,
) -> Result<Option<Ordering>, &'static str> {
    let types = &facts.types;
    let Some(formal) = facts
        .generics
        .parameters
        .get(&definition)
        .and_then(|parameters| parameters.get(index))
    else {
        return Err("source atomic access lacks its checked ordering parameter");
    };
    let Argument::Const(formal) = formal.argument else {
        return Err("source atomic ordering formal is not a constant");
    };
    let Some(ConstKind::Parameter(formal)) = types.const_kind(formal) else {
        return Err("source atomic ordering lacks its declared formal");
    };
    let Some(ParameterKind::Const { ty: order_type }) = types.parameter(formal).map(|row| row.kind)
    else {
        return Err("source atomic ordering has a different formal kind");
    };
    let Some(Kind::Adt(owner, GenericArgs::EMPTY)) = types.kind(order_type) else {
        return Ok(None);
    };
    let variants = facts.members.variants(owner);
    if variants.len() != Ordering::ALL.len() {
        return Ok(None);
    }
    let Some(ConstKind::Aggregate(order)) = types.const_kind(order) else {
        return Err("source atomic ordering has no evaluated enum value");
    };
    let Some(Aggregate::Fields {
        ty,
        variant: Some(tag),
        values: [],
    }) = types.aggregate_constant(order)
    else {
        return Err("source atomic ordering is not a fieldless enum value");
    };
    let variant = variants
        .get(tag as usize)
        .copied()
        .ok_or("unknown source atomic ordering")?;
    if ty != order_type
        || !facts.members.fields(variant).is_some_and(<[_]>::is_empty)
        || Reader::new(facts)
            .discriminant(variant)?
            .map(|(_, bits)| bits)
            != Some(u128::from(tag))
    {
        return Err("source atomic ordering differs from its checked enum tag");
    }
    Ordering::from_tag(tag)
        .map(Some)
        .ok_or("unknown source atomic ordering")
}
