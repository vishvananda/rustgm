//! Public call quantifiers own conversion debts without granting caller permissions.
#[cfg(test)]
#[path = "../../../tests/invariants/vtable_quantifiers.rs"]
mod tests;
use super::{Entry, vtable_entries::Conversion};
use crate::{
    body::{
        instantiate::ProofCursor,
        regions::{Contract, Kind, Proof},
    },
    sema::model::Declarations,
    types::{
        RegionId,
        lifetime_proof::{Obligations, ScopeId},
        substitute::Bound,
    },
};

pub(super) fn select(
    facts: &mut Declarations,
    caller: &mut Contract,
    conversion: Conversion,
    count: u32,
    cursor: &mut ProofCursor,
) -> Result<[Option<Entry>; 2], &'static str> {
    let first = cursor.first(facts, caller)?;
    let end = first
        .checked_add(count)
        .filter(|&end| end < 0x8000_0000)
        .ok_or("captured call quantifier identity space exhausted")?;
    let mut opening = Obligations::new(first);
    let scope = opening.scope(ScopeId::ROOT, count)?;
    opening.append_universals(first..end, scope)?;
    let mut selected = selection_contract(caller, opening);
    let signature = conversion.open(facts, (first..end).map(RegionId).collect())?;
    let entries =
        conversion.entries(facts, &mut selected, signature, &mut ProofCursor::default())?;
    let proof = close(facts, selected, scope)?;
    cursor.append(facts, caller, proof)?;
    Ok(entries)
}

/// Selection only emits debts. Retain the caller identity but copy no assumptions,
/// conditional output conclusions, body vectors or enclosing signature.
fn selection_contract(caller: &Contract, opening: Obligations) -> Contract {
    Contract {
        key: caller.key,
        source: caller.source,
        signature: None,
        universals: 0,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: vec![Proof::new(caller.source, opening)],
        builtins: None,
        output: None,
        require_result_formation: false,
    }
}

fn close(
    facts: &mut Declarations,
    selected: Contract,
    scope: ScopeId,
) -> Result<Obligations, &'static str> {
    let mut proofs = selected.proofs.into_iter();
    let mut opening = proofs.next().ok_or("missing captured call opening")?.proof;
    let mut shift = Bound::shift(1);
    for mut evidence in proofs {
        if evidence.source != selected.source || !evidence.sources.is_empty() {
            return Err("captured call proof changed its diagnostic owner");
        }
        // Actual numeric aliases keep their scope. Outer body-bound lifetimes
        // cross the new public binder; proof-local binders remain untouched.
        evidence.proof.substitute(&mut facts.types, &mut shift)?;
        opening.graft(evidence.proof, scope)?;
    }
    let mut debts = Obligations::new(opening.regions.end);
    for obligation in selected.obligations {
        if obligation.source != selected.source {
            return Err("captured call debt changed its diagnostic owner");
        }
        match obligation.kind {
            Kind::Equality(pair) => debts.equalities.push(pair),
            Kind::Outlives(pair) => debts.outlives.push(pair),
            _ => return Err("captured call selection emitted a nonrelational debt"),
        }
    }
    debts.substitute(&mut facts.types, &mut shift)?;
    opening.append_at(debts, scope)?;
    facts.body_work.canonical_steps += shift.steps;
    Ok(opening)
}
