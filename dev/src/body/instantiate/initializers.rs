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
    let (substitution, contract) = contracts::initializer(facts, original, &mut inputs)?;
    materialize(facts, template, instance, substitution, contract)
}
fn materialize(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    mut substitution: crate::types::substitute::Bound,
    mut contract: Contract,
) -> Result<Materialized, &'static str> {
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
) -> Result<SourceMaterialized, &'static str> {
    witness.validate(facts, result)?;
    if original.key != BodyKey::Declaration(witness.selected().definition)
        || original.source != witness.original()
        || template.body.source != witness.original()
        || original.key != template.body.key()
        || original.output.is_some()
    {
        return Err("source computation selects another original initializer contract");
    }
    if template.body.result == result && facts.types.needs_region_context(result) == Some(false) {
        return Ok(SourceMaterialized {
            materialized: with_contract(facts, template, witness.selected(), original)?,
            interface: None,
        });
    }
    let mut inputs = anonymous::initializer_inputs(facts, witness.selected())?;
    let (mut substitution, mut contract, imported) =
        contracts::source_result(facts, original, &mut inputs, result)?;
    let before = substitution.steps;
    let actual = substitution.apply(&mut facts.types, template.body.result)?;
    facts.body_work.canonical_steps += substitution.steps - before;
    let obligation = contract.obligations.len();
    contract.obligations.push(crate::body::regions::Obligation {
        source: original.source,
        kind: crate::body::regions::Kind::Coercion {
            actual,
            expected: imported,
            unsize: false,
            reference: None,
        },
    });
    Ok(SourceMaterialized {
        materialized: materialize(facts, template, witness.selected(), substitution, contract)?,
        interface: Some(ResultInterface {
            requested: result,
            imported,
            obligation,
        }),
    })
}
struct ResultInterface {
    requested: crate::types::TypeId,
    imported: crate::types::TypeId,
    obligation: usize,
}
pub(crate) struct SourceMaterialized {
    materialized: Materialized,
    interface: Option<ResultInterface>,
}
impl SourceMaterialized {
    pub(crate) fn into_requested(
        self,
        requested: crate::types::TypeId,
    ) -> Result<Materialized, &'static str> {
        let Some(interface) = self.interface else {
            return if self.materialized.body.result == requested {
                Ok(self.materialized)
            } else {
                Err("source initializer materialized result changed")
            };
        };
        let obligation = self
            .materialized
            .contract
            .obligations
            .get(interface.obligation);
        if requested != interface.requested
            || self.materialized.contract.key != self.materialized.body.key()
            || self.materialized.contract.source != self.materialized.body.source
            || !obligation.is_some_and(|obligation| {
                matches!(obligation.kind, crate::body::regions::Kind::Coercion {
                    actual, expected, unsize: false, reference: None,
                } if actual == self.materialized.body.result && expected == interface.imported)
            })
        {
            return Err("source initializer result interface changed");
        }
        Ok(self.materialized)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/returned_interfaces.rs"]
mod tests;
