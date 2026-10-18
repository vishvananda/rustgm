//! Concrete projection results retain their selection debts in the instance contract.
mod assumptions;
mod debts;
use crate::{
    body::{
        ir::{Body, Rvalue},
        regions::{Contract, Kind},
    },
    sema::{
        model::Declarations,
        projections::{self, Concrete},
    },
    types::{
        TypeId,
        lifetime_proof::{Obligations, ScopeId},
    },
};

/// Nominal fields can contain projections absent from the nominal's own arguments.
pub(super) fn needs_field_normalization(
    facts: &Declarations,
    body: &Body,
) -> Result<bool, &'static str> {
    use crate::types::{Kind as Type, arguments::Argument};
    if !facts.aggregates.has_projected_templates() {
        return Ok(false);
    }
    let mut work = std::iter::once(body.result)
        .chain(body.locals.iter().map(|local| local.ty))
        .chain(body.values.iter().map(|value| value.ty))
        .chain(body.projections.iter().map(|projection| projection.ty))
        .collect::<Vec<_>>();
    let mut seen = std::collections::HashSet::new();
    let mut steps = 0_usize;
    while let Some(ty) = work.pop() {
        steps = steps.saturating_add(1);
        if steps.saturating_add(work.len()) > 1_000_000 {
            return Err("instance storage dependency work budget exceeded");
        }
        if !seen.insert(ty) {
            continue;
        }
        if let Some(projected) = facts
            .stored_fields
            .as_ref()
            .and_then(|fields| fields.contains_projection(ty))
        {
            if projected {
                return Ok(true);
            }
            continue;
        }
        match facts
            .types
            .kind(ty)
            .ok_or("invalid instance storage dependency")?
        {
            Type::Adt(_, arguments) => {
                if facts.aggregates.has_projected_fields(ty) {
                    return Ok(true);
                }
                for variant in facts.aggregates.raw_shape(&facts.types, ty)?.variants() {
                    work.extend_from_slice(
                        facts
                            .types
                            .list(variant.fields)
                            .ok_or("missing instance storage fields")?,
                    );
                }
                work.extend(
                    facts
                        .types
                        .arguments(arguments)
                        .ok_or("missing instance storage arguments")?
                        .filter_map(|argument| match argument {
                            Argument::Type(ty) => Some(ty),
                            _ => None,
                        }),
                );
            }
            Type::Tuple(list) => work.extend_from_slice(
                facts
                    .types
                    .list(list)
                    .ok_or("missing instance storage tuple")?,
            ),
            Type::Array { element, .. }
            | Type::ArrayTerm { element, .. }
            | Type::Slice(element) => work.push(element),
            _ => {}
        }
    }
    Ok(false)
}

pub(super) fn normalize(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    comparison: Option<(TypeId, TypeId)>,
) -> Result<(), &'static str> {
    let mut proof = Obligations::new(first_region(contract)?);
    let mut session = projections::concrete(&mut proof, ScopeId::ROOT)?;
    let mut storage = crate::sema::stored_fields::Preparation::default();
    if let Some((expected, actual)) = comparison
        && session.normalize(facts, expected)?.0 != session.normalize(facts, actual)?.0
    {
        return Err("instance signature differs from normalized template");
    }
    body.result = ty(facts, &mut session, &mut storage, body.result)?;
    for local in &mut body.locals {
        local.ty = ty(facts, &mut session, &mut storage, local.ty)?;
    }
    for projection in &mut body.projections {
        projection.ty = ty(facts, &mut session, &mut storage, projection.ty)?;
    }
    for value in &mut body.values {
        value.ty = ty(facts, &mut session, &mut storage, value.ty)?;
        if let Rvalue::Call { signature, .. } = &mut value.kind {
            *signature = ty(facts, &mut session, &mut storage, *signature)?;
        }
    }
    rewrite_contract(facts, contract, &mut session)?;
    assumptions::normalize(facts, contract, &mut proof)?;
    let next = proof.regions.end;
    if !proof.is_empty() {
        contract.proofs.push((body.source, proof));
    }
    debts::normalize(facts, contract, next)?;
    Ok(())
}

fn first_region(contract: &Contract) -> Result<u32, &'static str> {
    contract
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
                .map(|(_, proof)| proof.regions.end.saturating_sub(1)),
        )
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("instance projection region identity space exhausted")
}

fn ty(
    facts: &mut Declarations,
    session: &mut Concrete<'_>,
    storage: &mut crate::sema::stored_fields::Preparation,
    original: TypeId,
) -> Result<TypeId, &'static str> {
    let result = session.normalize(facts, original)?.0;
    if facts.types.contains_projections(result) != Some(false) {
        return Err("concrete instance retains an associated projection");
    }
    if facts.aggregates.has_projected_templates() {
        storage.prepare(facts, result, session)?;
    } else {
        facts.aggregates.prepare(&mut facts.types, result)?;
    }
    Ok(result)
}

fn rewrite_contract(
    facts: &mut Declarations,
    contract: &mut Contract,
    session: &mut Concrete<'_>,
) -> Result<(), &'static str> {
    if let Some(signature) = &mut contract.signature {
        // This binder is the body's existing universal scope, not a fresh
        // nested callable scope. Its parameter/result roots live at ROOT.
        let Some(crate::types::Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
        }) = facts.types.kind(*signature)
        else {
            return Err("instance contract has no function signature");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("invalid instance contract parameters")?
            .to_vec();
        let mut normalized = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            normalized.push(session.normalize(facts, parameter)?.0);
        }
        let result = session.normalize(facts, result)?.0;
        let parameters = facts.types.intern_list(&normalized)?;
        *signature = facts.types.intern(crate::types::Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
        })?;
    }
    for clause in &mut contract.type_assumptions {
        if let crate::types::parameters::clauses::Clause::Predicate(predicate) = clause {
            predicate.ty = session.normalize(facts, predicate.ty)?.0;
        }
    }
    for obligation in &mut contract.obligations {
        match &mut obligation.kind {
            Kind::Coercion {
                actual, expected, ..
            } => {
                *actual = session.normalize(facts, *actual)?.0;
                *expected = session.normalize(facts, *expected)?.0;
            }
            Kind::WellFormed(ty) | Kind::TypeOutlives { ty, .. } => {
                *ty = session.normalize(facts, *ty)?.0;
            }
            _ => {}
        }
    }
    Ok(())
}
