//! Concrete projection results retain their selection debts in the instance contract.
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
use std::collections::HashMap;

pub(super) fn normalize(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    comparison: Option<(TypeId, TypeId)>,
) -> Result<(), &'static str> {
    let mut proof = Obligations::new(first_region(contract)?);
    let mut session = projections::concrete(&mut proof, ScopeId::ROOT)?;
    if let Some((expected, actual)) = comparison
        && session.normalize(facts, expected)?.0 != session.normalize(facts, actual)?.0
    {
        return Err("instance signature differs from normalized template");
    }
    body.result = ty(facts, &mut session, body.result)?;
    for local in &mut body.locals {
        local.ty = ty(facts, &mut session, local.ty)?;
    }
    for projection in &mut body.projections {
        projection.ty = ty(facts, &mut session, projection.ty)?;
    }
    for value in &mut body.values {
        value.ty = ty(facts, &mut session, value.ty)?;
        if let Rvalue::Call { signature, .. } = &mut value.kind {
            *signature = ty(facts, &mut session, *signature)?;
        }
    }
    rewrite_contract(facts, contract, &mut session)?;
    let mut next = proof.regions.end;
    if !proof.is_empty() {
        contract.proofs.push((body.source, proof));
    }
    let mut additions = Vec::new();
    for (source, original) in &mut contract.proofs {
        let mut roots = Vec::new();
        for (scope, ty) in original.type_roots() {
            if facts.types.contains_projections(ty) == Some(true) {
                // A new sibling forall cannot own the existing proof's private
                // variables. Keep this gate until extensions can share that
                // actual proof scope and its discontiguous allocation intervals.
                if scope != ScopeId::ROOT {
                    return Err("quantified instance projection debt requires its original scope");
                }
                roots.push(ty);
            }
        }
        if roots.is_empty() {
            continue;
        }
        let mut mapped = HashMap::new();
        let mut proof = Obligations::new(next);
        let mut session = projections::concrete(&mut proof, ScopeId::ROOT)?;
        for root in roots {
            mapped.insert(root, session.normalize(facts, root)?.0);
        }
        next = proof.regions.end;
        if !proof.is_empty() {
            additions.push((*source, proof));
        }
        original.map_type_roots(|_, root| mapped.get(&root).copied().unwrap_or(root));
    }
    contract.proofs.extend(additions);
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
    original: TypeId,
) -> Result<TypeId, &'static str> {
    let result = session.normalize(facts, original)?.0;
    if facts.types.contains_projections(result) != Some(false) {
        return Err("concrete instance retains an associated projection");
    }
    facts.aggregates.prepare(&mut facts.types, result)?;
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
    for (ty, _) in &mut contract.type_assumptions {
        *ty = session.normalize(facts, *ty)?.0;
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
