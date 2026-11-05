//! One selected deferred body owns its still-required formation proof separately.
use super::{Bound, Cleanup, Context, Declarations, Inputs, Materialized, Premises, contracts};
use crate::{
    body::{
        coroutines,
        ir::{Body, BodyKey},
        regions::Contract,
    },
    types::{Kind, TypeId, lifetime_proof::Obligations},
};
use std::cell::Cell;

/// A transient selected application, never a constructor, original or runtime permit.
/// The generated poll transform consumes its one owned materialization.
pub struct FutureApplication {
    materialized: Materialized,
    formation: Option<Box<Obligations>>,
}
impl FutureApplication {
    #[must_use]
    pub const fn body(&self) -> &Body {
        &self.materialized.body
    }
    #[must_use]
    pub const fn contract(&self) -> &Contract {
        &self.materialized.contract
    }
    /// These are requirements in a separate proof unit, not callee assumptions.
    #[must_use]
    pub fn formation_requirements(&self) -> Option<&Obligations> {
        self.formation.as_deref()
    }
}

/// Select only the actual checked original for the complete genuine future application.
/// No arbitrary template, constructor instance or source marker supplies this stage.
/// # Errors
/// Rejects foreign/stale families, malformed captures, failed closed bounds and work limits.
pub(crate) fn future_application(
    facts: &mut Declarations,
    template: &crate::body::templates::Template,
    original: &Contract,
    future: TypeId,
) -> Result<FutureApplication, String> {
    let Some(Kind::Future {
        identity,
        arguments,
    }) = facts.types.kind(future)
    else {
        return Err("async application requires its complete genuine future type".into());
    };
    if coroutines::original_template(facts, template, original)?
        != facts.checked_future_definition(future)?
    {
        return Err("async application belongs to another actual original".into());
    }
    let application = facts
        .opaques
        .declarations
        .get(&identity)
        .ok_or("async application lost its actual opaque declaration")?
        .application;
    let Some(Kind::Opaque(actual, formal)) = facts.types.kind(application) else {
        return Err("async application lost its actual declaration capture frame".into());
    };
    if actual != identity {
        return Err("async application changed its declaration identity".into());
    }
    let scope = crate::body::copy_requirements::materialized::capture_async(
        facts,
        &template.body,
        original,
        future,
    )?;
    let mut inputs = captured_inputs(facts, formal, arguments, future)?;
    let (mut fold, mut contract) =
        contracts::specialize(facts, original, &mut inputs, Premises::Required)?;
    let mut formation_fold = Bound::parameters_captured_charged(
        &facts.types,
        &inputs.bindings,
        inputs.captures,
        &|work, bytes| facts.charge_caller_inputs(work, bytes),
    )?;
    let formation =
        facts.authored_future_requirements(future, &mut formation_fold, contract.universals)?;
    contract.key = BodyKey::AsyncApplication(future);
    let signature = contract
        .signature
        .ok_or("async application lost its actual internal Output signature")?;
    facts.charge_caller_inputs(
        template
            .body
            .values
            .len()
            .saturating_add(template.body.locals.len())
            .saturating_add(1),
        template
            .retained_bytes()
            .saturating_add(contract.retained_bytes()),
    )?;
    let mut cleanup = Cleanup::new(facts);
    let (body, destruction) = super::materialize(
        facts,
        template,
        super::anonymous::Selection::AsyncBody { future, signature },
        &mut fold,
        Some(&mut contract),
        None,
        Context {
            premises: Premises::Required,
            caller: None,
            cleanup: &mut cleanup,
        },
    )?;
    let copy_scope = scope
        .map(|scope| scope.finish(facts, &body, &mut contract))
        .transpose()?;
    let mut materialized = Materialized {
        body,
        destruction,
        contract,
        copy_scope,
    };
    materialized.resolve(facts)?;
    let formation = if formation.is_empty() {
        None
    } else {
        facts.charge_caller_inputs(
            1,
            size_of::<Obligations>().saturating_add(formation.allocation_bytes()),
        )?;
        Some(Box::new(formation))
    };
    Ok(FutureApplication {
        materialized,
        formation,
    })
}

fn captured_inputs(
    facts: &mut Declarations,
    formal: crate::types::GenericArgs,
    actual: crate::types::GenericArgs,
    future: TypeId,
) -> Result<Inputs, &'static str> {
    let work = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let base = facts.constant_work.template_bytes;
    let result =
        Bound::captured_inputs_charged(&facts.types, formal, actual, &|steps, allocation| {
            work.set(work.get().saturating_add(steps));
            bytes.set(bytes.get().saturating_add(allocation));
            if work.get() > 1_000_000 || base.saturating_add(bytes.get()) > 64 * 1024 * 1024 {
                return Err("async application capture mapping budget exhausted");
            }
            facts.charge_caller_inputs(steps, allocation)
        });
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.get());
    let result = result?;
    Ok(Inputs {
        caller: crate::sema::values::execution::CallerInstance::AsyncBody(future),
        instance: None,
        bindings: result.bindings,
        captures: result.regions,
        comparison: None,
        connections: None,
    })
}
