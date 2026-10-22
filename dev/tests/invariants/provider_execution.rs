//! Demanded annotation readiness retains the existing compact worklist storage.
use super::*;

#[test]
fn preparation_phase_retains_boolean_storage() {
    assert_eq!(size_of::<Phase>(), size_of::<bool>());
    println!(
        "declaration Work={} Checker={} Pending={} Phase={}",
        size_of::<Work>(),
        size_of::<Checker<'_>>(),
        size_of::<Pending>(),
        size_of::<Phase>()
    );
}

#[test]
fn repeated_demand_cannot_replay_prepared_function_annotations() -> Result<(), String> {
    let text = "const fn wrapper<T>()->usize{4}type A=[u8;wrapper::<u32>()];";
    let report = crate::test_source::resolve(text, true)?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run().map_err(|e| e.to_string())?;
    let function = *checker.facts.functions.first().ok_or("function")?;
    assert!(
        checker
            .facts
            .typed_function_bodies
            .contains(&function.definition)
    );
    let root = function.body.ok_or("body")?;
    let context = checker.local_context(function.node, super::super::regions::Context::ALIAS);
    let visited = checker.constant_dependencies.len();
    let pending = checker.work.len();
    assert!(visited > 0);
    for _ in 0..128 {
        checker
            .constant_dependencies_in(root, context)
            .map_err(|e| e.to_string())?;
        assert_eq!(checker.constant_dependencies.len(), visited);
        assert_eq!(checker.work.len(), pending);
    }
    Ok(())
}

#[test]
fn prepared_and_resume_states_require_their_exact_pending_owner() -> Result<(), String> {
    let report = crate::test_source::resolve("const fn value()->usize{4}", true)?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker.run().map_err(|e| e.to_string())?;
    let function = checker.facts.functions.first().ok_or("function")?;
    let ty = checker.facts.definitions[function.definition.0 as usize].ok_or("type")?;
    let instance = crate::types::instance::Instance::new(&checker.facts.types, ty)?;
    assert!(
        checker
            .execution_body(Requirement::FunctionBody(instance), Phase::Prepared)
            .is_err()
    );
    let error = checker
        .execution_body(Requirement::FunctionBody(instance), Phase::Resume)
        .err()
        .ok_or("missing continuation accepted")?;
    assert!(
        error
            .to_string()
            .contains("missing suspended const function body")
    );
    Ok(())
}

#[test]
fn closure_input_completion_filters_exact_owners_and_preserves_memoized_inputs()
-> Result<(), String> {
    let report = crate::test_source::resolve(
        "fn f(){let _=|x:&u8|{let _=x;};let _=|y:&u8|{let _=y;};}",
        true,
    )?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker.run().map_err(|e| e.to_string())?;
    let mut closures = checker
        .facts
        .closures
        .inputs
        .keys()
        .copied()
        .collect::<Vec<_>>();
    closures.sort_by_key(|key| key.node.0);
    assert_eq!(closures.len(), 2);
    let key = closures[0];
    let original = checker.facts.closures.inputs[&key].clone();
    assert_eq!(original.len(), 1);
    let mut steps = 0;
    let mut scan = Vec::new();
    let first = checker.annotation_inputs.begin(key)?;
    for region in &original {
        checker.annotation_inputs.record(key, *region);
    }
    checker
        .closure_annotation(
            key,
            super::super::regions::Context::ALIAS,
            Some(first),
            &mut steps,
            1000,
            &mut scan,
        )
        .map_err(|e| e.to_string())?;
    assert_eq!(checker.facts.closures.inputs[&key], original);
    let first = checker.annotation_inputs.begin(key)?;
    // Unrelated production cannot enter this closure's pending input scope.
    checker.annotation_inputs.record(closures[1], original[0]);
    checker
        .closure_annotation(
            key,
            super::super::regions::Context::ALIAS,
            Some(first),
            &mut steps,
            1000,
            &mut scan,
        )
        .map_err(|e| e.to_string())?;
    assert_eq!(checker.facts.closures.inputs[&key], original);
    checker.annotation_inputs.begin(key)?;
    assert!(
        checker
            .closure_annotation(
                key,
                super::super::regions::Context::ALIAS,
                Some(usize::MAX),
                &mut steps,
                1000,
                &mut scan
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn input_completion_rejects_forged_regions_before_changing_completed_inputs() -> Result<(), String>
{
    let report = crate::test_source::resolve(
        "fn f(){let _=|x:&u8|{let _=x;};let _=|y:&u8|{let _=y;};}",
        true,
    )?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker.run().map_err(|e| e.to_string())?;
    let mut keys = checker
        .facts
        .closures
        .inputs
        .keys()
        .copied()
        .collect::<Vec<_>>();
    keys.sort_by_key(|key| key.node.0);
    let own = keys[0];
    let previous = checker.facts.closures.inputs[&own].clone();
    for region in [
        crate::types::RegionId(0),
        checker.facts.closures.inputs[&keys[1]][0],
    ] {
        let first = checker.annotation_inputs.begin(own)?;
        checker.annotation_inputs.record(own, region);
        assert!(
            checker
                .closure_annotation(
                    own,
                    super::super::regions::Context::ALIAS,
                    Some(first),
                    &mut 0,
                    1000,
                    &mut Vec::new(),
                )
                .is_err()
        );
        assert_eq!(checker.facts.closures.inputs[&own], previous);
    }
    let first = checker.annotation_inputs.begin(own)?;
    checker.annotation_inputs.record(own, previous[0]);
    assert!(
        checker
            .closure_annotation(
                own,
                super::super::regions::Context::ALIAS,
                Some(first),
                &mut 0,
                0,
                &mut Vec::new(),
            )
            .is_err()
    );
    assert_eq!(checker.facts.closures.inputs[&own], previous);
    Ok(())
}
