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
fn completed_source_callers_retain_identity_without_retaining_plain_literals() -> Result<(), String>
{
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )
    .map_err(|e| e.to_string())?;
    source.push_str("const fn plain_bound()->usize{3}type Called=[u8;plain_bound()];");
    let literal_start = u32::try_from(source.len()).map_err(|e| e.to_string())?;
    source.push_str("type Literal=[u8;3];");
    let report = crate::test_source::source_provider(&source, "plain_annotation")?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run().map_err(|e| e.to_string())?;
    let ast = &report.package.unit(FileId(0)).configured.ast;
    let mut found = 0;
    for (index, node) in ast.nodes.iter().enumerate() {
        let key = crate::resolve::unit::NodeKey {
            file: FileId(0),
            node: crate::syntax::ast::NodeId::new(u32::try_from(index).map_err(|e| e.to_string())?),
        };
        if matches!(node.kind, crate::syntax::ast::Kind::Call { .. }) {
            found += 1;
            assert_eq!(
                checker.facts.constant_body_type(key),
                Some(crate::types::TypeId::primitive(
                    crate::types::Primitive::Usize
                )),
            );
        } else if matches!(node.kind, crate::syntax::ast::Kind::Literal(_))
            && node.span.start >= literal_start
        {
            found += 1;
            assert_eq!(checker.facts.constant_body_type(key), None);
        }
    }
    assert_eq!(found, 2);
    Ok(())
}

#[test]
fn source_binary_annotation_prepares_its_actual_default_before_operator_contract()
-> Result<(), String> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-shift-or/const-followup/primitive-const-impl.rs",
    )
    .map_err(|e| e.to_string())?;
    source.push_str("type Arithmetic=[u8;3usize<<1|1];");
    let report = crate::test_source::source_provider(&source, "default_annotation")?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run().map_err(|e| e.to_string())?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "Arithmetic")
        .ok_or("actual alias")?;
    let ty = checker.facts.definitions[index].ok_or("actual array")?;
    assert!(matches!(
        checker.facts.types.kind(ty),
        Some(Kind::Array { length: 7, .. })
    ));
    Ok(())
}

#[test]
fn erased_scalar_anonymous_calls_keep_the_sparse_body_fast_path() -> Result<(), String> {
    let mut source =
        crate::test_source::fixture("pa31/tests/spec/source-scalar-patterns/source-runtime.rs")
            .map_err(|e| e.to_string())?;
    source.truncate(source.find("pub mod niche").ok_or("niche module")?);
    let start = u32::try_from(source.len()).map_err(|e| e.to_string())?;
    source.push_str("type Arithmetic=[u8;(-(-3i8)) as usize];");
    let report = crate::test_source::source_provider(&source, "erased_annotation")?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run().map_err(|e| e.to_string())?;
    let mut found = 0;
    for (index, node) in report
        .package
        .unit(FileId(0))
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        if node.span.start >= start
            && matches!(
                node.kind,
                crate::syntax::ast::Kind::Unary { .. } | crate::syntax::ast::Kind::Cast { .. }
            )
        {
            found += 1;
            let key = crate::resolve::unit::NodeKey {
                file: FileId(0),
                node: crate::syntax::ast::NodeId::new(
                    u32::try_from(index).map_err(|e| e.to_string())?,
                ),
            };
            assert_eq!(checker.facts.constant_body_type(key), None);
        }
    }
    assert_eq!(found, 3);
    Ok(())
}

#[test]
fn tracked_annotation_calls_prepare_only_actual_location_data_and_resume() -> Result<(), String> {
    let mut source =
        crate::test_source::fixture("pa31/tests/spec/source-location/caller-location.rs")
            .map_err(|e| e.to_string())?;
    source.push_str(
        "#[track_caller]const fn demanded_bound()->usize{3}\n\
         fn dormant_body()->u8{false}\n\
         type FirstBound=[u8;demanded_bound()];\n\
         type SecondBound=[u8;demanded_bound()];",
    );
    let report = crate::test_source::source_provider(&source, "tracked_annotation")?;
    let mut checker = Checker::new(&report).map_err(|e| e.to_string())?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run().map_err(|e| e.to_string())?;
    let mut found = 0;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let name = report.package.names.get(definition.name);
        if matches!(name, "FirstBound" | "SecondBound") {
            found += 1;
            let ty = checker.facts.definitions[index].ok_or("array declaration")?;
            assert!(matches!(
                checker.facts.types.kind(ty),
                Some(Kind::Array { length: 3, .. })
            ));
        } else if matches!(name, "demanded_bound" | "dormant_body") {
            found += 1;
            let id = DefId(u32::try_from(index).map_err(|e| e.to_string())?);
            assert_eq!(
                checker.facts.typed_function_bodies.contains(&id),
                name == "demanded_bound",
            );
        }
    }
    assert_eq!(found, 4);
    assert!(checker.facts.body_work.suspensions > 0);
    assert_eq!(
        crate::sema::compiler_items::site_records::requested_location(&report, &checker.facts)
            .map_err(|e| e.to_string())?,
        None,
    );
    assert!(
        checker
            .facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(crate::support::bindings::LangItem::PanicInfo))
            .is_none()
    );
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
            .execution_body(
                Request::Constant(Requirement::FunctionBody(instance)),
                Phase::Prepared
            )
            .is_err()
    );
    let error = checker
        .execution_body(
            Request::Constant(Requirement::FunctionBody(instance)),
            Phase::Resume,
        )
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
    closures.sort_by_key(|key| key.node.get());
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
    keys.sort_by_key(|key| key.node.get());
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
