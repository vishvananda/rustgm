//! Materialized scopes authenticate full instances and reselect concrete reads.
use super::*;
use crate::body::{instantiate::Materialized, ir::Owner};

fn applications(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
) -> Result<[Instance; 2]> {
    let mut selected = Vec::new();
    for node in facts.nodes.iter().flatten() {
        let Some(ty) = node.ty else {
            continue;
        };
        if matches!(facts.types.kind(ty), Some(Kind::FunctionItem { item, arguments, .. })
            if item.0 == definition.0 && arguments != GenericArgs::EMPTY)
            && facts.types.contains_parameters(ty) == Some(false)
            && !selected.contains(&ty)
        {
            selected.push(ty);
        }
    }
    assert_eq!(selected.len(), 2, "two actual call-site full instances");
    let public = |ty| match facts.types.kind(ty) {
        Some(Kind::FunctionItem { signature, .. }) => Some(signature),
        _ => None,
    };
    assert_eq!(
        public(selected[0]),
        public(selected[1]),
        "only full arguments differ"
    );
    let full = |ty| match facts.types.kind(ty) {
        Some(Kind::FunctionItem { arguments, .. }) => facts
            .types
            .arguments(arguments)
            .map(Iterator::collect::<Vec<_>>),
        _ => None,
    };
    let first = full(selected[0]).ok_or("first actual full application")?;
    let second = full(selected[1]).ok_or("second actual full application")?;
    assert_eq!((first.len(), second.len()), (2, 2));
    assert_eq!(
        first[0],
        crate::types::arguments::Argument::Lifetime(crate::types::RegionId::STATIC)
    );
    assert_eq!(first[0], second[0]);
    assert_ne!(first[1], second[1]);
    Ok([
        Instance::new(&facts.types, selected[0])?,
        Instance::new(&facts.types, selected[1])?,
    ])
}
fn rejected(
    facts: &mut Declarations,
    scope: &materialized::InstanceScope,
    body: &Body,
    contract: &Contract,
    environment: &Arc<Environment>,
) -> Result {
    let before = facts.body_work.canonical_steps;
    let error = scope
        .check(facts, body, contract, Some(environment))
        .err()
        .ok_or("foreign scope accepted")?;
    assert_eq!(
        error,
        "instance Copy scope belongs to another materialization or environment"
    );
    assert!(facts.body_work.canonical_steps > before);
    Ok(())
}
fn opponents(
    facts: &mut Declarations,
    concrete: &Materialized,
    other: Instance,
    original: &Arc<Environment>,
    foreign_source: crate::body::ir::BodyOrigin,
) -> Result {
    let scope = concrete
        .instance_copy_scope_for_test()
        .ok_or("instance scope")?;
    let environment = concrete
        .contract
        .builtins
        .as_deref()
        .and_then(|input| input.environment.as_ref())
        .ok_or("fresh environment")?;
    for change in 0..4 {
        let mut wrong = concrete.body.clone();
        let Owner::Function {
            signature,
            execution,
            ..
        } = &mut wrong.owner
        else {
            return Err("actual function".into());
        };
        match change {
            0 => wrong.instance = Some(other),
            1 => *signature = TypeId(u32::MAX),
            2 => *execution = Some(TypeId(u32::MAX)),
            _ => *execution = None,
        }
        rejected(facts, scope, &wrong, &concrete.contract, environment)?;
    }
    rejected(
        facts,
        scope,
        &concrete.body,
        &concrete.contract,
        &Arc::new(environment.as_ref().clone()),
    )?;
    rejected(facts, scope, &concrete.body, &concrete.contract, original)?;
    let mut wrong = concrete.contract.clone();
    wrong.source = foreign_source;
    rejected(facts, scope, &concrete.body, &wrong, environment)?;
    let mut wrong = concrete.contract.clone();
    wrong.universals = wrong
        .universals
        .checked_add(1)
        .ok_or("fixture scope overflow")?;
    rejected(facts, scope, &concrete.body, &wrong, environment)?;
    let source = concrete
        .body
        .source
        .source()
        .ok_or("actual instance source")?;
    let retained = facts
        .generics
        .builtin_requirements
        .get_mut(&source)
        .ok_or("actual original row")?
        .environment
        .replace(Arc::new(original.as_ref().clone()));
    rejected(
        facts,
        scope,
        &concrete.body,
        &concrete.contract,
        environment,
    )?;
    facts
        .generics
        .builtin_requirements
        .get_mut(&source)
        .ok_or("actual original row")?
        .environment = retained;
    let catalog = facts.source_bindings.take();
    rejected(
        facts,
        scope,
        &concrete.body,
        &concrete.contract,
        environment,
    )?;
    facts.source_bindings = catalog;
    Ok(())
}
#[test]
fn alias_materialization_reselects_in_a_fresh_exact_instance_environment() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/projections/body-opening/instance-arguments-scoped.rs",
    )?;
    let (mut facts, templates, contracts) = fixture_source(&source)?;
    let template = templates
        .bodies
        .iter()
        .find(|template| {
            template.environment.body_aliases().is_some()
                && contracts
                    .iter()
                    .any(|input| input.key == template.body.key() && input.parameters.len() == 1)
        })
        .ok_or("actual generic alias-bearing original")?;
    let original = contracts
        .iter()
        .find(|input| input.key == template.body.key())
        .ok_or("original")?;
    let Owner::Function { definition, .. } = template.body.owner else {
        return Err("actual function".into());
    };
    assert_eq!(
        template
            .environment
            .body_aliases()
            .map(|aliases| aliases.len()),
        Some(1)
    );
    let [instance, other] = applications(&facts, definition)?;
    let mut concrete = body::instantiate::with_contract(&mut facts, template, instance, original)?;
    let scope = concrete
        .instance_copy_scope_for_test()
        .ok_or("authenticated instance scope")?;
    let environment = concrete
        .contract
        .builtins
        .as_deref()
        .and_then(|input| input.environment.as_ref())
        .ok_or("fresh environment")?;
    assert!(!Arc::ptr_eq(environment, &template.environment));
    assert!(environment.body_aliases().is_none());
    scope.check(
        &mut facts,
        &concrete.body,
        &concrete.contract,
        Some(environment),
    )?;
    println!(
        "instance_scope_peak_bytes={}",
        scope.retained_bytes_for_test()
    );
    let mut reproved = concrete.contract.clone();
    concrete.prepare_source_copy(&mut facts, &concrete.body, &mut reproved)?;
    let mut raw = concrete.contract.clone();
    let error = prepare(&mut facts, &concrete.body, &mut raw)
        .err()
        .ok_or("raw preparation manufactured a scope")?;
    assert_eq!(
        error,
        "body alias opening belongs to another original or environment"
    );
    let foreign = contracts
        .iter()
        .find(|input| input.source != original.source)
        .ok_or("different actual source")?;
    opponents(
        &mut facts,
        &concrete,
        other,
        &template.environment,
        foreign.source,
    )?;
    concrete.resolve(&mut facts)?;
    let builtins = concrete
        .contract
        .builtins
        .as_ref()
        .ok_or("prepared instance")?;
    let prepared = builtins.prepared_for_test()?;
    let requirements = builtins
        .source_copy
        .as_ref()
        .ok_or("actual instance read requirements")?;
    assert!(!prepared.decisions.is_empty());
    for (&ty, &copy) in &prepared.decisions {
        assert!(copy);
        assert!(prepared.decision(
            &concrete.body,
            requirements,
            builtins.environment.as_ref(),
            ty
        )?);
    }
    println!(
        "instance alias carrier bytes={} body={} contract={}",
        size_of::<materialized::InstanceScope>(),
        size_of::<Body>(),
        size_of::<Contract>()
    );
    Ok(())
}
#[test]
fn actual_const_engine_reproves_the_materialized_alias_instance() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/projections/body-opening/instance-engine-scoped.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "alias_instance_engine")?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "inspect")
        .ok_or("actual selected helper")?;
    let definition = crate::resolve::model::DefId(u32::try_from(definition)?);
    let (analysis, bodies) =
        crate::pipeline::execution::materialized_bodies::during_scoped(&[definition], || {
            crate::pipeline::analyze_source(&report)
        })?;
    assert_eq!(bodies.len(), 1);
    let body = bodies.first().ok_or("actual proved materialized helper")?;
    assert!(body.instance.is_some());
    let source = body.source.source().ok_or("actual proved source owner")?;
    let environment = analysis
        .facts
        .generics
        .builtin_requirements
        .get(&source)
        .and_then(|input| input.environment.as_ref())
        .ok_or("actual source alias environment")?;
    assert_eq!(
        environment.body_aliases().map(|aliases| aliases.len()),
        Some(1)
    );
    assert!(matches!(
        body.owner,
        Owner::Function {
            execution: Some(_),
            ..
        }
    ));
    assert!(analysis.facts.constant_work.frames_started > 0);
    Ok(())
}
