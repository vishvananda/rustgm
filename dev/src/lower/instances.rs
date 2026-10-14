//! Demand concrete CFGs from checked templates before ordinary executable lowering.
mod demand;
mod dependencies;
mod proof;
use crate::{
    body::{
        self,
        ir::{NodeKey, Selection},
        regions::Contracts,
    },
    borrow,
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    types::{Kind, TypeId},
};

/// Compile selected instances using retained syntax/type facts and checked templates.
/// Each demanded body has its own substituted contract; source keys are diagnostic
/// identities, never keys that combine different instances' lifetime contexts.
/// # Errors
/// Rejects invalid unused templates, unsupported demanded bodies, proof failures
/// and bounded instance growth.
pub fn build(
    report: &Report,
    facts: &mut Declarations,
    entry: DefId,
) -> Result<crate::lowir::program::Program, Error> {
    let mut templates = body::templates::build(report, facts)?;
    let mut inputs = body::regions::collect(facts)
        .map_err(|e| diagnostic(report, None, e.into()))?
        .bodies;
    body::templates::constructors(report, facts, &mut templates, &mut inputs)?;
    body::promote::analyze_templates(report, facts, &mut templates)?;
    let contracts = Contracts {
        bodies: inputs,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    let mut copies = crate::types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &custom,
        |ty, field| position(facts, ty, field),
    )
    .map_err(|e| diagnostic(report, e.node, e.message))?;
    let inputs = contracts.bodies;
    let program = demand::prepare(report, facts, entry, &checked, inputs)?;
    program
        .with_checked(|checked| super::build(facts, checked, entry))
        .map_err(|e| diagnostic(report, e.node, e.message))?
        .map_err(|e| diagnostic(report, Some(e.node), e.message.into()))
}

fn diagnostic(report: &Report, node: Option<NodeKey>, message: String) -> Error {
    if let Some(key) = node {
        report.diagnostic(Declarations::error(report, key, message))
    } else {
        Error::at(crate::resolve::unit::FileId(0), 0, message)
    }
}
fn position(
    facts: &Declarations,
    ty: TypeId,
    field: Selection,
) -> Result<(u32, u32), &'static str> {
    let (Some(Kind::Adt(item, _)), Selection::Nominal(field)) = (facts.types.kind(ty), field)
    else {
        return Err("invalid instance cleanup field selection");
    };
    let variant = facts
        .members
        .variant(field)
        .and_then(|id| facts.members.variants.get(id.0 as usize))
        .ok_or("missing instance cleanup variant")?;
    if variant.owner != item {
        return Err("instance cleanup field belongs to another type");
    }
    Ok((
        variant.ordinal,
        facts
            .members
            .ordinal(field)
            .ok_or("missing instance cleanup field")?,
    ))
}
