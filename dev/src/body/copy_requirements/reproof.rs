//! Instance assumptions are rebuilt from the actual transported Contract.
use super::{Arc, Contract, Declarations, Environment};

pub(super) fn fresh(
    facts: &mut Declarations,
    contract: &Contract,
) -> Result<Arc<Environment>, &'static str> {
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(contract.parameters.len())
        .saturating_add(contract.type_assumptions.len());
    let parameters = contract
        .parameters
        .iter()
        .filter(|&&id| {
            matches!(
                facts.types.parameter(id).map(|parameter| parameter.kind),
                Some(crate::types::parameters::ParameterKind::Type { .. })
            )
        })
        .map(|&id| {
            (
                id,
                crate::types::parameters::environment::Abilities::default(),
            )
        })
        .collect::<Vec<_>>();
    Ok(Arc::new(
        Environment::new(&facts.types, &parameters)?
            .with_clauses(&facts.types, &contract.type_assumptions)?,
    ))
}
