//! Closed consumers discharge retained evidence in an empty lifetime environment.
use super::model::Declarations;
use crate::types::{lifetime_proof::Obligations, outlives::Queries, relations::Constraints};

pub(crate) fn closed(
    facts: &Declarations,
    proof: &Obligations,
    failure: &'static str,
) -> Result<(), String> {
    if proof.is_empty() {
        return Ok(());
    }
    let mut constraints = Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 1_000_000)
        .map_err(|error| error.to_string())?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    constraints
        .solve()
        .map(|_| ())
        .map_err(|error| format!("{failure}: {error}"))
}
