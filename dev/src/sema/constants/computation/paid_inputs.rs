//! Only raw debts solved under the actual caller's inputs seal declared premises.
use super::{Declarations, Instance, catalog};
use crate::sema::{
    callee_premises::Inputs,
    dispatch::{ConstantDecision, ConstantProof},
    traits::{implementations::Availability, obligations::Obligations},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Paid {
    owner: catalog::CallerOwner,
    availability: Availability,
    selected: Instance,
}
impl Paid {
    pub(super) fn selects(self, selected: Instance) -> bool {
        self.selected == selected
    }
    pub(super) fn validate(
        self,
        facts: &Declarations,
        caller: catalog::CallerId,
        key: catalog::CallerKey,
        selected: Instance,
    ) -> Result<(), &'static str> {
        if !self.selects(selected) || self.availability != Availability::Validated {
            return Err("paid source premise lost its selected instance or availability");
        }
        facts
            .source_computations
            .as_deref()
            .ok_or("missing paid source catalog")?
            .0
            .validate_caller_owner(self.owner, caller, key)
    }
}

pub(super) fn select(
    facts: &mut Declarations,
    dispatcher: &mut crate::sema::dispatch::Dispatcher,
    key: catalog::CallerKey,
    publication: catalog::Publication,
    inputs: Inputs<'_>,
    input: Instance,
) -> Result<(Instance, Option<Paid>), String> {
    let opening = dispatcher.source_opening(key.universals);
    facts.body_work.canonical_steps += key.universals as usize;
    let opening = opening?;
    let mut paid = None;
    let selected = dispatcher.source_constant(facts, input, &mut |facts, proof| {
        let root = proof.input == input;
        let (decision, receipt) = observe(facts, key, publication, inputs, &opening, proof)?;
        if root && receipt.is_some() {
            paid = receipt;
        }
        Ok(decision)
    })?;
    Ok((selected, paid))
}

pub(super) fn observe(
    facts: &mut Declarations,
    key: catalog::CallerKey,
    publication: catalog::Publication,
    inputs: Inputs<'_>,
    opening: &Obligations,
    selected: ConstantProof<'_>,
) -> Result<(ConstantDecision, Option<Paid>), String> {
    let Some(proof) = selected.proof else {
        // A cached Instance alone carries no evidence. This route deliberately
        // reselects until a full-context paid reuse policy is qualified.
        return Ok((ConstantDecision::Fresh, None));
    };
    if inputs.source != key.source
        || inputs.universals != key.universals
        || selected.availability != Availability::Validated
    {
        return Err("source premise inputs have another caller or availability".into());
    }
    let mut fold = selected.selected.substitution(facts)?;
    let requirements = crate::sema::callee_premises::requirements(
        facts,
        selected.selected.definition,
        &mut fold,
        0,
        selected.first,
    );
    facts.body_work.canonical_steps += fold.steps;
    let requirements = requirements?;
    let Some(requirements) = requirements else {
        return Ok((ConstantDecision::Checked(None), None));
    };
    for (_, ty) in requirements.type_roots() {
        Inputs::prepare_type(facts, ty)?;
    }
    let mut work = 0;
    let checked = inputs.selection(facts, opening, proof, Some(&requirements), &mut work);
    facts.body_work.canonical_steps += work;
    checked?;
    if facts.source_computations.is_none() {
        facts.source_computations = Some(Box::new(super::Computations(catalog::Catalog::new()?)));
    }
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source caller catalog")?
        .0;
    let caller = catalog.caller(key, publication)?;
    let owner = catalog.caller_owner(caller)?;
    Ok((
        ConstantDecision::Checked(Some(requirements)),
        Some(Paid {
            owner,
            availability: selected.availability,
            selected: selected.selected,
        }),
    ))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/paid_callers.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/invariants/paid_work.rs"]
mod work_tests;
