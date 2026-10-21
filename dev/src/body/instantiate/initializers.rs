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
    let (mut substitution, mut contract) = contracts::specialize(facts, original, &mut inputs)?;
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
