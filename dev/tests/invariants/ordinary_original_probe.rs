//! Borrow an actual ordinary proof before its paired input is released.
use super::*;
use crate::{borrow::incremental::Proved, sema::model::Function};
use std::cell::Cell;

thread_local! {
    static ACTIVE: Cell<Option<(Function, usize)>> = const { Cell::new(None) };
}

pub(super) struct Probe;
impl Probe {
    pub(super) fn new(function: Function) -> Self {
        assert!(ACTIVE.replace(Some((function, 0))).is_none());
        Self
    }
    pub(super) fn finish(self) {
        assert_eq!(ACTIVE.get().map(|(_, visits)| visits), Some(1));
        drop(self);
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}

pub(in crate::lower::instances) fn observe(
    facts: &Declarations,
    proved: &Proved,
    input: &body::regions::Contract,
) {
    let Some((function, visits)) = ACTIVE.get() else {
        return;
    };
    let body = proved.body();
    let body::ir::Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    else {
        return;
    };
    if definition != function.definition {
        return;
    }
    assert_eq!(
        visits, 0,
        "the same ordinary original must be observed once"
    );
    ACTIVE.set(Some((function, visits + 1)));
    assert!(body.instance.is_none());
    assert_eq!(body.source.source(), function.body);
    assert_eq!(body.key(), input.key);
    assert_eq!(body.source, input.source);
    assert_eq!(signature, function.signature);
    assert_eq!(input.signature, Some(execution.unwrap_or(signature)));
    println!(
        "ordinary clone public={signature:?} {:?} owned={execution:?} {:?} const-context={} contract-universals={}",
        facts.types.kind(signature),
        execution.and_then(|ty| facts.types.kind(ty)),
        body.const_context,
        input.universals
    );
    trace_shape(facts, "clone", body, input);
    trace_source_obligations("clone", input);
    println!(
        "ordinary clone actual cleanup-plan={:?} separate-source-environment={}",
        proved
            .plan()
            .map(|plan| (plan.cleanups.len(), plan.events.len())),
        facts.generics.body_environment(definition).is_some()
    );
    assert!(body.projections.iter().any(|projection| matches!(
        projection.kind,
        body::ir::ProjectionKind::Dereference { .. }
    )));
}
