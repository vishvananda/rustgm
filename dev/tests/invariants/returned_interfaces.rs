//! A result record is identity transport; the ordinary proof remains mandatory.
use super::*;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::values::execution::{Executor, Progress, Request, SourceComputation, Target},
    types::{TypeId, outlives::Queries},
};
use std::{cell::Cell, rc::Rc};
struct Observe {
    inner: Box<dyn Executor>,
    seen: Rc<Cell<Option<(SourceComputation, TypeId)>>>,
}
impl Executor for Observe {
    fn check(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error> {
        if let Target::SourceAssociated { source, result } = target {
            self.seen.set(Some((source, result)));
        }
        self.inner.check(report, facts, target)
    }
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> Result<Progress, Error> {
        self.inner.advance(report, facts, request)
    }
}
struct Fixture {
    report: Report,
    facts: Declarations,
    template: Template,
    contract: Contract,
    witness: SourceComputation,
    requested: TypeId,
}
fn prepare() -> Result<Fixture, Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/lifetime-initializer-open/reference.rs");
    let text = std::fs::read_to_string(path)?;
    let report = crate::test_source::resolve(&text, true)?;
    let context = crate::pipeline::execution::Context::default();
    let seen = Rc::new(Cell::new(None));
    let facts = crate::sema::declarations::analyze_with_executor(
        &report,
        Box::new(Observe {
            inner: context.executor(),
            seen: Rc::clone(&seen),
        }),
    )?;
    let mut facts = crate::pipeline::analyze(&report, facts)?.facts;
    let (witness, result) = seen.get().ok_or("missing actual source request")?;
    let initializer = *facts
        .initializers
        .iter()
        .find(|initializer| initializer.definition == witness.selected().definition)
        .ok_or("missing initializer")?;
    let mut template =
        crate::body::templates::Prepared::default().initializer(&report, &facts, &initializer)?;
    crate::body::promote::instance_reusing(
        &report,
        &mut facts,
        &mut template.body,
        &template.destruction,
        &mut crate::body::promote::Scratch::default(),
    )?;
    let contract = crate::body::regions::Collector::default().initializer(&facts, &initializer)?;
    Ok(Fixture {
        report,
        facts,
        template,
        contract,
        witness,
        requested: result,
    })
}
#[test]
fn result_transport_is_not_permission_without_the_capture_identity_proof()
-> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "source-only layout: result interface={} wrapper={} materialized={}",
        size_of::<ResultInterface>(),
        size_of::<SourceMaterialized>(),
        size_of::<Materialized>(),
    );
    let Fixture {
        report,
        mut facts,
        template,
        contract,
        witness,
        requested,
    } = prepare()?;
    for tamper in 0..4 {
        let mut built = source(&mut facts, &template, witness, requested, &contract)?;
        crate::body::promote::instance_reusing(
            &report,
            &mut facts,
            &mut built.materialized.body,
            &built.materialized.destruction,
            &mut crate::body::promote::Scratch::default(),
        )?;
        let interface = built.interface.as_ref().ok_or("missing result interface")?;
        assert_ne!(built.materialized.body.result, requested);
        assert_ne!(built.materialized.body.result, interface.imported);
        assert_eq!(built.materialized.body.result, template.body.result);
        match tamper {
            0 => {
                built.materialized.contract.assumptions.clear();
            }
            1 => {
                built
                    .materialized
                    .contract
                    .obligations
                    .remove(interface.obligation);
            }
            2 => {
                assert!(
                    built
                        .into_requested(TypeId::primitive(crate::types::Primitive::U16))
                        .is_err()
                );
                continue;
            }
            _ => {}
        }
        let materialized = built.into_requested(requested);
        if tamper == 1 {
            assert!(materialized.is_err());
            continue;
        }
        let materialized = materialized?;
        let mut query = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let proof = crate::borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut query,
            &materialized.contract,
            &materialized.body,
        );
        if tamper == 0 {
            assert!(proof.is_err(), "unrelated body slots became equivalent");
        } else {
            proof?;
        }
    }
    Ok(())
}
