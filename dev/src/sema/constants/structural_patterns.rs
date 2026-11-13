//! Source patterns classify real traits modulo regions, without publishing trait evidence.
use super::{Declarations, Error, NodeKey, Report};
use crate::{
    sema::{compiler_items, dispatch::Dispatcher},
    support::{bindings::LangItem, comparisons::Method},
    types::{
        GenericArgs, ItemId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};

pub(super) fn equality(
    report: &Report,
    facts: &mut Declarations,
    dispatch: &mut Dispatcher,
    source: NodeKey,
    ty: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, source, message);
    let comparison = compiler_items::source_comparison(report, facts, source, Method::Eq)?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(ty)])
        .map_err(error)?;
    prove(
        facts,
        dispatch,
        ty,
        ItemId(comparison.definition.0),
        arguments,
    )
    .map_err(|message| {
        Declarations::error(
            report,
            source,
            format!("source constant pattern requires PartialEq: {message}"),
        )
    })
}

pub(super) fn nominal(
    report: &Report,
    facts: &mut Declarations,
    dispatch: &mut Dispatcher,
    source: NodeKey,
    ty: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, source, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 8)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source pattern lacks its actual trait catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    let marker = catalog
        .validated_language(LangItem::StructuralPartialEq)
        .map_err(error)?
        .ok_or_else(|| error("source constant pattern requires actual StructuralPartialEq"))?;
    prove(
        facts,
        dispatch,
        ty,
        ItemId(marker.definition.0),
        GenericArgs::EMPTY,
    )
    .map_err(|message| {
        Declarations::error(
            report,
            source,
            format!("source constant pattern requires StructuralPartialEq: {message}"),
        )
    })
}

fn prove(
    facts: &mut Declarations,
    dispatch: &mut Dispatcher,
    ty: TypeId,
    item: ItemId,
    arguments: GenericArgs,
) -> Result<(), String> {
    let application = facts
        .types
        .intern_trait_application(Application { item, arguments })?;
    dispatch.pattern_implements_modulo_regions(
        facts,
        Predicate {
            ty,
            bound: Bound::Trait(application),
        },
    )
}
