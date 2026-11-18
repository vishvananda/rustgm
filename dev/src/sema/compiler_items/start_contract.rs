//! Complete the actual authored startup header without selecting an executable.
use super::{Catalog, headers::functions};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention,
        parameters::ParameterKind,
    },
};

pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<bool, Error> {
    let Some(declaration) = catalog.language(LangItem::Start) else {
        return Ok(false);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    if functions::complete(report, facts, catalog, LangItem::Start)?.is_none() {
        return Err(error("source start lost its actual completed function"));
    }
    let header = functions::current(facts, declaration).map_err(error)?;
    let parameters = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .map_or(&[][..], Vec::as_slice);
    let [parameter] = parameters else {
        return Err(error(
            "source start requires exactly one owned type parameter",
        ));
    };
    let Argument::Type(formal) = parameter.argument else {
        return Err(error(
            "source start requires exactly one owned type parameter",
        ));
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(formal) else {
        return Err(error(
            "source start requires its actual canonical type formal",
        ));
    };
    if facts.types.parameter(id).is_none_or(|parameter| {
        parameter.owner.0 != declaration.definition.0
            || parameter.index != 0
            || !matches!(parameter.kind, ParameterKind::Type { .. })
    }) {
        return Err(error("source start changed its actual type formal owner"));
    }
    signature(facts, header.signature, formal).map_err(error)?;
    Ok(true)
}

fn signature(facts: &Declarations, signature: TypeId, formal: TypeId) -> Result<(), &'static str> {
    const SHAPE: &str = "source start requires fn(fn() -> T, isize, *const *const u8, u8) -> isize";
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err(SHAPE);
    };
    let Some(&[callback, count, argv, sigpipe]) = facts.types.list(parameters) else {
        return Err(SHAPE);
    };
    let isize_ty = TypeId::primitive(Primitive::Isize);
    let byte_ty = TypeId::primitive(Primitive::U8);
    if result != isize_ty || count != isize_ty || sigpipe != byte_ty {
        return Err(SHAPE);
    }
    let Some(Kind::Pointer {
        mutable: false,
        pointee,
    }) = facts.types.kind(argv)
    else {
        return Err(SHAPE);
    };
    if !matches!(facts.types.kind(pointee), Some(Kind::Pointer { mutable: false, pointee }) if pointee == byte_ty)
        || !matches!(facts.types.kind(callback),
            Some(Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: false,
                convention: Convention::RUST,
            }) if result == formal && facts.types.list(parameters).is_some_and(<[_]>::is_empty))
    {
        return Err(SHAPE);
    }
    Ok(())
}

/// Until executable role consumers exist, refuse only actual selected contracts.
pub(in crate::sema) fn require_transport(facts: &Declarations) -> Result<(), &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    for role in [LangItem::Start, LangItem::Termination] {
        if catalog.role(role)?.is_some() {
            return Err("selected Source startup roles require their checked artifact transport");
        }
    }
    Ok(())
}
