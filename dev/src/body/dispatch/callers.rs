//! Recover copied source operands and prepare their exact receiving caller debts.
use super::{Body, Contract, Declarations, Kind, Rvalue};
use crate::sema::{dispatch::Dispatcher, values::execution::BodyCaller};

pub(super) fn prepare(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &Contract,
) -> Result<(Option<BodyCaller>, Dispatcher), String> {
    let has_source = body
        .values
        .iter()
        .any(|value| matches!(value.kind, Rvalue::SourceAssociated(_)));
    if has_source && (contract.key != body.key() || contract.source != body.source) {
        return Err("source dispatch contract belongs to another body".into());
    }
    for value in &mut body.values {
        if let Rvalue::SourceAssociated(source) = value.kind {
            source.validate_use(
                facts,
                body.source,
                body.owner
                    .source_caller(facts, body.source, body.instance)?,
                value.ty,
            )?;
            // A copied/rebuilt contract must mint its own normalization debts.
            // The old handle supplies identity only, never authority to skip them.
            value.kind = Rvalue::AssociatedConstant(source.input(facts)?);
        }
    }
    let source_path = body
        .values
        .iter()
        .map(|value| match value.kind {
            Rvalue::AssociatedConstant(instance) => {
                crate::sema::values::execution::BodyCaller::needed(facts, instance, value.ty)
            }
            _ => Ok(false),
        })
        .try_fold(false, |needed, next| next.map(|next| needed || next))?;
    if source_path && (contract.key != body.key() || contract.source != body.source) {
        return Err("source dispatch contract belongs to another body".into());
    }
    let caller = if source_path {
        let enclosing = body
            .owner
            .source_caller(facts, body.source, body.instance)?;
        Some(crate::sema::values::execution::BodyCaller::new(
            facts,
            body.source,
            enclosing,
            contract.universals,
            contract
                .obligations
                .iter()
                .filter_map(|obligation| {
                    if let Kind::Equality(pair) = obligation.kind {
                        Some(pair)
                    } else {
                        None
                    }
                })
                .chain(
                    contract
                        .proofs
                        .iter()
                        .flat_map(|proof| proof.proof.root_equalities()),
                ),
        )?)
    } else {
        None
    };
    let dispatcher = Dispatcher::new(
        contract.universals,
        first_region(facts, contract, source_path)?,
    )?;
    Ok((caller, dispatcher))
}

fn first_region(
    facts: &Declarations,
    contract: &Contract,
    source_path: bool,
) -> Result<u32, &'static str> {
    let first = contract
        .obligations
        .iter()
        .filter_map(|obligation| {
            if let Kind::Variable { id, .. } = obligation.kind {
                Some(id.0)
            } else {
                None
            }
        })
        .chain(
            contract
                .proofs
                .iter()
                .map(|evidence| evidence.proof.regions.end.saturating_sub(1)),
        )
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("dispatch region identity space exhausted")?;
    let first = if source_path {
        // Source callers reserve these identities in the body's unit. Its
        // registered namespace already numbered them past every actual.
        facts
            .body_regions
            .frontier(contract.source)
            .map_err(|_| "source caller region identity space exhausted")?
            .max(first)
    } else {
        first
    };
    Ok(first)
}
