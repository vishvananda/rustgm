//! Requested receivers retain the complete substituted source lifetime layout.
use super::{borrowed, validate_execution};
use crate::{
    sema::{
        closures::{Body, Execution},
        model::Declarations,
    },
    types::{closure::Adapter, instance::Instance, lifetime_proof::Obligations, outlives::Pair},
};

pub(crate) struct Prepared {
    pub canonical: Execution,
    pub execution: Execution,
    pub equalities: Vec<Pair>,
    pub proofs: Vec<Obligations>,
}
pub(super) fn prepare(
    facts: &mut Declarations,
    source: Body,
    instance: Instance,
    adapter: Option<Adapter>,
) -> Result<Prepared, &'static str> {
    let mut canonical = validate_execution(&facts.types, source)?;
    let generic = source
        .owner
        .is_some_and(|owner| crate::body::templates::owns(facts, owner));
    let mut proofs = Vec::new();
    let mut equalities = Vec::new();
    let execution = if generic && let Some(adapter) = adapter {
        let execution =
            crate::body::instantiate::substitute_closure_execution(facts, instance, canonical)?;
        let prepared = crate::body::instantiate::closure_self_type(facts, execution)?;
        canonical = prepared.execution;
        proofs.extend(prepared.proof);
        let prepared = borrowed::prepared(
            &mut facts.types,
            prepared.execution,
            source.universals,
            adapter,
            true,
        )?;
        equalities = prepared.equalities;
        let prepared =
            crate::body::instantiate::normalize_closure_execution(facts, prepared.execution)?;
        proofs.extend(prepared.proof);
        prepared.execution
    } else {
        let mut execution = if let Some(adapter) = adapter {
            let prepared = borrowed::execution(&mut facts.types, source, adapter)?;
            equalities = prepared.equalities;
            prepared.execution
        } else {
            canonical
        };
        if generic {
            let prepared = crate::body::instantiate::closure_execution(facts, instance, execution)?;
            execution = prepared.execution;
            proofs.extend(prepared.proof);
        }
        execution
    };
    Ok(Prepared {
        canonical,
        execution,
        equalities,
        proofs,
    })
}
