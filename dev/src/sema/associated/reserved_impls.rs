//! Reservation polarity is bound only from the authenticated declaration annotation.
use super::{ImplPolarity, OwnerKind};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Definition},
        package::Error,
        unit::NodeKey,
    },
    sema::{header_attributes::reserve_source_contracts, model::Declarations},
    syntax::{ast::Kind, attribute::Attribute, item::ItemKind},
};
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    key: NodeKey,
    attribute: Attribute,
    effect: usize,
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    reserve_source_contracts(facts, 1, 0)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let unit = report.package.unit(key.file);
    let actual = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("reservation marker lost its actual effect"))?;
    if owner.file != key.file
        || actual.0 != attribute
        || actual.0.name(actual.1, unit.source.text()) != Some("rustc_reservation_impl")
    {
        return Err(error("reservation marker changed its source attribute"));
    }
    let name = actual
        .1
        .tree(attribute.path.start)
        .ok_or_else(|| error("reservation marker lost its actual name"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !report.macro_allowances().allows_at(
            key.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error(
            "rustc_reservation_impl requires its rustc_attrs request",
        ));
    }
    let annotation = report
        .lint_producer()
        .ok_or_else(|| error("reservation marker requires its declaration producer"))?
        .declaration_annotation(&report.package, key)
        .map_err(Error::from)?
        .ok_or_else(|| error("reservation marker lacks its canonical annotation"))?;
    if annotation.owner != owner
        || annotation
            .reservation
            .is_none_or(|(actual, _)| actual != attribute)
    {
        return Err(error(
            "reservation marker changed its canonical declaration",
        ));
    }
    if !matches!(unit.configured.ast.node(key.node).kind, Kind::Item(index)
        if matches!(unit.configured.ast.items[index as usize].kind,
            ItemKind::Impl { trait_path: Some(_), negative: false, .. }))
    {
        return Err(error(
            "reservation impl requires an actual positive trait impl declaration",
        ));
    }
    Ok(())
}
pub(in crate::sema) fn bind(
    report: &Report,
    facts: &mut Declarations,
    index: usize,
    definition: &Definition,
) -> Result<(), Error> {
    if definition.kind != DefKind::SelfType {
        return Ok(());
    }
    let Some(key) = definition.node else {
        return Ok(());
    };
    let Some(producer) = report.lint_producer() else {
        return Ok(());
    };
    if !producer.has_impl_annotations() {
        return Ok(());
    }
    let Some(annotation) = producer
        .declaration_annotation(&report.package, key)
        .map_err(Error::from)?
    else {
        return Ok(());
    };
    let Some((attribute, _message)) = annotation.reservation else {
        return Ok(());
    };
    reserve_source_contracts(facts, 1, 0)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let id = DefId(u32::try_from(index).map_err(|_| error("too many reservation owners"))?);
    let slot = facts
        .associated
        .owner(id)
        .ok_or_else(|| error("reservation lost its actual impl owner"))?;
    let owner = facts
        .associated
        .owners
        .get_mut(slot.0 as usize)
        .ok_or_else(|| error("reservation impl owner is outside its inventory"))?;
    if owner.source != Some(key) || owner.self_definition != id {
        return Err(error("reservation changed its actual source owner"));
    }
    let OwnerKind::Impl {
        trait_type: Some(_),
        polarity,
        ..
    } = &mut owner.kind
    else {
        return Err(error(
            "reservation marker requires an actual trait impl owner",
        ));
    };
    if *polarity != ImplPolarity::Positive {
        return Err(error("reservation impl cannot be negative or bound twice"));
    }
    *polarity = ImplPolarity::Reservation;
    Ok(())
}
