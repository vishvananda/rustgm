//! Only the actual sealed caller can supply a target's non-type conditions.
use super::*;
use crate::{
    body::ir::Owner,
    lower::instances::demand::{self, Caller, Specialization},
    types::parameters::{clauses::Clause, environment::Environment},
};
use std::sync::Arc;
const SOURCE: &str =
    "trait Gate{}impl Gate for u8{}fn id<T:Gate>(x:T)->T{x}fn caller()->u8{id(7u8)}";

fn definition(facts: &Declarations, templates: &Templates) -> TestResult<resolve::model::DefId> {
    templates
        .bodies
        .iter()
        .find_map(|template| match template.body.owner {
            Owner::Function { definition, .. } if facts.generics.has_parameters(definition) => {
                Some(definition)
            }
            _ => None,
        })
        .ok_or_else(|| "actual generic target".into())
}
fn install_conditions(mode: &str, fixture: &mut Fixture) -> TestResult<resolve::model::DefId> {
    let target = definition(&fixture.facts, &fixture.templates)?;
    let caller = fixture
        .program
        .bodies
        .iter()
        .find_map(|body| match body.owner {
            Owner::Function { definition, .. } => Some(definition),
            _ => None,
        })
        .ok_or("actual ordinary caller")?;
    let id = fixture
        .facts
        .types
        .intern_feature_charged("port_gate", &|_, _| Ok(()))?;
    let other = fixture
        .facts
        .types
        .intern_feature_charged("unrelated", &|_, _| Ok(()))?;
    fixture.facts.publish_feature_policy(&[], &|_, _| Ok(()))?;
    fixture
        .facts
        .generics
        .predicates
        .get_mut(&target)
        .ok_or("actual written target condition")?[0]
        .value = Clause::UnstableFeature(id);
    let environment = Arc::new(
        fixture
            .facts
            .generics
            .parameter_scope(&fixture.facts.types, target)?
            .with_feature_assumptions_charged(&fixture.facts.types, &[id], &|_, _| Ok(()))?,
    );
    fixture.templates.bodies.iter_mut().find(|template| matches!(template.body.owner, Owner::Function {definition,..} if definition==target))
        .ok_or("actual target template")?.environment = environment;
    if matches!(
        mode,
        "matching" | "unrelated" | "root" | "wrong-root" | "missing-index"
    ) {
        let id = if mode == "unrelated" { other } else { id };
        fixture.facts.generics.environments.insert(
            caller,
            Arc::new(
                Environment::new(&fixture.facts.types, &[])?.with_feature_assumptions_charged(
                    &fixture.facts.types,
                    &[id],
                    &|_, _| Ok(()),
                )?,
            ),
        );
    } else if mode == "uncompleted" {
        fixture.facts.generics.predicates.insert(caller, Vec::new());
    }
    Ok(target)
}
fn control(mode: &str) -> TestResult {
    let (report, mut fixture) = fixture_with_report(SOURCE)?;
    let target = install_conditions(mode, &mut fixture)?;
    let checked = originals(&mut fixture.facts, &fixture.templates, &fixture.inputs)?;
    let (mut builder, bodies) = super::super::Builder::new(fixture.program)?;
    let mut current = None;
    for body in bodies {
        let input = fixture
            .inputs
            .iter()
            .find(|input| input.key == body.key())
            .ok_or("actual caller contract")?;
        let index = builder.push(&mut fixture.facts, &checked, body, input, None)?;
        current = Some(index);
    }
    let current = current.ok_or("actual sealed caller index")?;
    let source = if mode == "wrong-root" {
        checked
            .get(crate::types::ItemId(target.0))
            .ok_or("target template")?
            .body
            .source
            .root_site()
    } else {
        builder.body(current)?.source.root_site()
    };
    let caller = match mode {
        "root" => Caller::ROOT,
        "missing-index" => Caller::body(current + 1)?,
        _ => Caller::body(current)?,
    };
    let instance = *fixture.instances.first().ok_or("actual target instance")?;
    let mut cleanup = body::instantiate::Cleanup::new(&fixture.facts);
    let result = demand::specialize(
        &report,
        &mut fixture.facts,
        &checked,
        &mut builder,
        &mut cleanup,
        Specialization {
            template: checked
                .get(crate::types::ItemId(target.0))
                .ok_or("actual target")?,
            input: checked
                .input(crate::types::ItemId(target.0))
                .ok_or("target input")?,
            instance,
            source,
            caller,
        },
    );
    if mode == "matching" {
        let index = result?;
        assert!(
            matches!(builder.body(index)?.owner, Owner::Function {definition,..} if definition==target)
        );
    } else {
        let error = result
            .err()
            .ok_or("foreign or missing caller accepted feature transfer")?
            .to_string();
        let expected = match mode {
            "wrong-root" => "root provenance",
            "missing-index" => "proved instance body",
            "uncompleted" => "completed declaration environment",
            _ => "caller evidence",
        };
        assert!(error.contains(expected), "{mode}: {error}");
    }
    Ok(())
}
#[test]
fn actual_ordinary_caller_feature_scope_reaches_materialization() -> TestResult {
    for mode in [
        "matching",
        "missing",
        "unrelated",
        "root",
        "uncompleted",
        "wrong-root",
        "missing-index",
    ] {
        control(mode)?;
    }
    Ok(())
}
#[test]
fn checked_template_environments_keep_owner_and_source_authority() -> TestResult {
    let mut fixture = fixture(
        "fn id<T:Copy>(x:T)->T{x}fn other<U:Copy>(x:U)->U{x}fn caller(){let _=id(1u8);let _=other(2u8);}",
    )?;
    let checked = originals(&mut fixture.facts, &fixture.templates, &fixture.inputs)?;
    let first = fixture
        .templates
        .bodies
        .first()
        .ok_or("first actual template")?;
    let other = fixture
        .templates
        .bodies
        .get(1)
        .ok_or("other actual template")?;
    assert!(Arc::ptr_eq(
        checked
            .environment_for_body(&first.body)?
            .ok_or("actual checked environment")?,
        &first.environment
    ));
    let mut wrong = first.body.clone();
    wrong.source = other.body.source;
    assert!(checked.environment_for_body(&wrong).is_err());
    wrong.source = first.body.source;
    wrong.owner = other.body.owner;
    assert!(checked.environment_for_body(&wrong).is_err());
    Ok(())
}
#[test]
fn compact_caller_keeps_the_existing_cold_queue_and_dense_layouts() -> TestResult {
    let (old, new, caller) = demand::caller_work_layouts();
    println!("caller queue old={old} new={new} caller={caller}");
    assert_eq!(old, 36);
    assert_eq!(new, 40);
    assert_eq!(new - old, 4);
    assert_eq!(caller, 4);
    assert_eq!(size_of::<crate::types::Store>(), 472);
    assert_eq!(size_of::<Environment>(), 64);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    let mut fixture = fixture(SOURCE)?;
    let key = fixture
        .program
        .bodies
        .first()
        .ok_or("actual body queue key")?
        .key();
    demand::caller_work_payments(&mut fixture.facts, key)?;
    Ok(())
}
