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
    if original.key != template.body.key()
        || original.source != template.body.source
        || original.output.is_some()
    {
        return Err("selected initializer output interface is not implemented");
    }
    let mut inputs = anonymous::initializer_inputs(facts, instance)?;
    let (substitution, contract) = contracts::initializer(facts, original, &mut inputs)?;
    materialize(
        facts,
        template,
        anonymous::Selection::Initializer(instance),
        substitution,
        contract,
        Premises::Required,
    )
}
fn materialize(
    facts: &mut Declarations,
    template: &Template,
    selection: anonymous::Selection,
    mut substitution: crate::types::substitute::Bound,
    mut contract: Contract,
    premises: Premises,
) -> Result<Materialized, &'static str> {
    let (crate::body::ir::Owner::InitializerInstance { instance, .. }, _, _) =
        selection.owner(facts, &template.body)?
    else {
        return Err("initializer materialization selects another body kind");
    };
    contract.key = BodyKey::InitializerInstance(instance);
    let mut cleanup = Cleanup::new(facts);
    let (body, destruction) = super::materialize(
        facts,
        template,
        selection,
        &mut substitution,
        Some(&mut contract),
        None,
        Context {
            premises,
            cleanup: &mut cleanup,
        },
    )?;
    if let anonymous::Selection::SourceInitializer(witness) = selection
        && !witness.rigid_parameters(facts)?.is_empty()
    {
        super::rigid_values::check(facts, &body)?;
    }
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
    let selected = witness.resolve(facts, result)?;
    if original.key != BodyKey::Declaration(selected.selected.definition)
        || original.source != selected.original
        || template.body.source != selected.original
        || original.key != template.body.key()
        || original.output.is_some()
    {
        return Err("source computation selects another original initializer contract");
    }
    let premises = if witness.paid_premises(facts)? {
        Premises::Declared
    } else {
        Premises::Required
    };
    if template.body.result == result && facts.types.needs_region_context(result) == Some(false) {
        let mut inputs = anonymous::initializer_inputs(facts, selected.selected)?;
        let parameters = witness.rigid_parameters(facts)?;
        let (substitution, contract) = if parameters.is_empty() {
            contracts::initializer(facts, original, &mut inputs)?
        } else {
            contracts::rigid_initializer(facts, original, &mut inputs, parameters)?
        };
        return Ok(SourceMaterialized {
            materialized: materialize(
                facts,
                template,
                anonymous::Selection::SourceInitializer(witness),
                substitution,
                contract,
                premises,
            )?,
            interface: None,
        });
    }
    let mut inputs = anonymous::initializer_inputs(facts, selected.selected)?;
    let (mut substitution, mut contract, imported) =
        contracts::source_result(facts, original, &mut inputs, result)?;
    let actual = super::caller_transport::result_root(
        facts,
        template.body.source,
        crate::sema::values::execution::CallerInstance::Initializer(selected.selected),
        template.body.result,
    )?;
    let before = substitution.steps;
    let actual = substitution.apply(&mut facts.types, actual)?;
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
        materialized: materialize(
            facts,
            template,
            anonymous::Selection::SourceInitializer(witness),
            substitution,
            contract,
            premises,
        )?,
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
    pub(super) fn plain(materialized: Materialized) -> Self {
        Self {
            materialized,
            interface: None,
        }
    }
    pub(super) fn interface(
        materialized: Materialized,
        requested: crate::types::TypeId,
        imported: crate::types::TypeId,
        obligation: usize,
    ) -> Self {
        Self {
            materialized,
            interface: Some(ResultInterface {
                requested,
                imported,
                obligation,
            }),
        }
    }
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
