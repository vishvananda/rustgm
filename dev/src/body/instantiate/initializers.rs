//! Selected initializer bodies retain full argument identity without callable permission.
use super::{Cleanup, Context, Declarations, Materialized, Premises, anonymous, contracts};
use crate::{
    body::{ir::BodyKey, regions::Contract, templates::Template},
    sema::values::Instance,
};
pub(crate) fn with_contract(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    original: &Contract,
) -> Result<Materialized, &'static str> {
    if original.key != template.body.key() || original.output.is_some() {
        return Err("selected initializer output interface is not implemented");
    }
    let mut inputs = anonymous::initializer_inputs(facts, instance)?;
    let (mut substitution, mut contract) =
        contracts::specialize(facts, original, &mut inputs, super::Premises::Required)?;
    contract.key = BodyKey::InitializerInstance(instance);
    let mut cleanup = Cleanup::new(facts);
    let (body, destruction) = super::materialize(
        facts,
        template,
        anonymous::Selection::Initializer(instance),
        &mut substitution,
        Some(&mut contract),
        None,
        Context {
            premises: Premises::Required,
            cleanup: &mut cleanup,
        },
    )?;
    Ok(Materialized {
        body,
        destruction,
        contract,
    })
}

/// Source computation retains the full instance but publishes no materialized value.
pub(crate) fn source(
    facts: &mut Declarations,
    template: &Template,
    witness: crate::sema::values::execution::SourceComputation,
    result: crate::types::TypeId,
    original: &Contract,
) -> Result<Materialized, &'static str> {
    witness.validate(facts, result)?;
    if original.key != BodyKey::Declaration(witness.selected().definition)
        || original.source != witness.original()
        || template.body.source != witness.original()
    {
        return Err("source computation selects another original initializer contract");
    }
    with_contract(facts, template, witness.selected(), original)
}
