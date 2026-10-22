//! Opaque trait results are existential outputs whose published clauses must hold.
use super::{Contract, Declarations};
use crate::support::hash::HashMap;
use crate::types::{
    Kind, TypeId,
    infer::{Context, Instantiation},
    parameters::{
        clauses::Clause,
        predicates::{Bound as PredicateBound, Predicate},
    },
    substitute::Bound,
};

/// Match complete signatures in a private graph. The result substitutions are
/// local to this conformance proof; they never equate distinct public identities.
pub(super) fn instantiate(
    facts: &mut Declarations,
    contract: &mut Contract,
    evidence: &mut crate::types::lifetime_proof::Obligations,
) -> Result<Vec<(crate::types::opaque::OpaqueId, TypeId)>, String> {
    if contract.outputs.is_empty() {
        return Ok(Vec::new());
    }
    let ids = contract
        .outputs
        .iter()
        .map(|(ty, _)| match facts.types.kind(*ty) {
            Some(Kind::Opaque(id, _)) => Ok(id),
            _ => Err("missing trait opaque result identity"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = Context::default();
    let mut substitution = Instantiation::opaque_results(&mut context, &facts.types, &ids)
        .map_err(|e| e.to_string())?;
    let expected = context
        .instantiate_with(&mut facts.types, contract.expected, &mut substitution)
        .map_err(|e| e.to_string())?;
    let actual = context
        .known(&facts.types, contract.actual)
        .map_err(|e| e.to_string())?;
    context
        .equate(&facts.types, actual, expected)
        .map_err(|e| e.to_string())?;
    evidence
        .equalities
        .extend(context.region_equalities.iter().copied());
    let mut equations = crate::types::substitute::RegionEquations::default();
    equations.extend(context.region_equalities.iter().copied())?;
    let mut closure = Bound::close_projection_regions(1, 0, HashMap::default())?;
    closure.exchange_projection_equations(&mut equations)?;
    let mut bindings = HashMap::default();
    // Closing each result can intern types. Stable rows require stable allocation
    // order here, before the final inspection writer sorts its records.
    let mut terms = substitution.opaque_terms().collect::<Vec<_>>();
    terms.sort_unstable_by_key(|&(id, _)| id.0);
    for (id, term) in terms {
        let ty = context
            .resolved(&mut facts.types, term)
            .map_err(|e| e.to_string())?;
        bindings.insert(id, closure.apply(&mut facts.types, ty)?);
    }
    let mut replacements = HashMap::<TypeId, TypeId>::default();
    for &(ty, _) in &contract.outputs {
        let Some(Kind::Opaque(id, _)) = facts.types.kind(ty) else {
            return Err("missing trait opaque output".into());
        };
        let result = *bindings.get(&id).ok_or("unconstrained trait output")?;
        let mut captures = crate::types::bound_uses::Uses::default();
        captures.collect(&facts.types, ty)?;
        let mut used = crate::types::bound_uses::Uses::default();
        used.collect(&facts.types, result)?;
        if !used.indices.is_subset(&captures.indices) {
            return Err("implementation result captures a lifetime excluded by the trait".into());
        }
        contract.obligations.push(Clause::Predicate(Predicate {
            ty: result,
            bound: PredicateBound::Sized,
        }));
        replacements.insert(ty, result);
    }
    let mut folder = Bound::defining_results(&facts.types, replacements)?;
    // The ordinary shape and lifetime conformance gates now see the chosen
    // output. Every declared result clause remains a separate obligation.
    contract.expected = folder.signature(&mut facts.types, contract.expected, 0)?;
    for (_, bounds) in &contract.outputs {
        for &clause in bounds {
            contract
                .obligations
                .push(folder.clause(&mut facts.types, clause)?);
        }
    }
    Ok(bindings.into_iter().collect())
}

/// Map the declared result promises with the same rigid method environment as
/// the signature. No hidden implementation type participates in this mapping.
pub(super) fn contracts(
    facts: &mut Declarations,
    method: super::DefId,
    folder: &mut Bound,
) -> Result<Vec<(TypeId, Vec<Clause>)>, &'static str> {
    let mut outputs = Vec::new();
    for id in facts.opaques.owners.get(&method).into_iter().flatten() {
        let declaration = facts
            .opaques
            .declarations
            .get(id)
            .ok_or("missing trait result declaration")?;
        let application = folder.apply(&mut facts.types, declaration.application)?;
        let bounds = declaration
            .bounds
            .iter()
            .map(|&clause| folder.clause(&mut facts.types, clause))
            .collect::<Result<Vec<_>, _>>()?;
        outputs.push((application, bounds));
    }
    Ok(outputs)
}
