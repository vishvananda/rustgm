//! Anonymous bodies retain enclosing declaration bounds without acquiring a callable ABI.
use super::{Contract, Declarations};
/// Anonymous templates retain their enclosing parameter scope, without a callable ABI.
pub(super) fn header(facts: &Declarations, contract: &mut Contract) -> Result<(), &'static str> {
    let source = contract
        .source
        .source()
        .ok_or("source anonymous header requires a real source owner")?;
    if let Some(inline) = facts
        .inline_constants
        .as_deref()
        .and_then(|r| r.get(source))
    {
        contract.universals = contract.universals.max(inline.universals);
        if let Some(owner) = inline.declaration {
            let header = super::parameter_contract(facts, owner, source, None)?;
            contract.parameters = header.parameters;
            contract.type_assumptions = header.type_assumptions;
            if let Some(scheme) = facts.lifetimes.functions.get(&owner) {
                contract.assumptions.extend_from_slice(&scheme.bounds);
            }
        }
    }
    Ok(())
}
