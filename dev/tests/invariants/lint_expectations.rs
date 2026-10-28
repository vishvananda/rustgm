//! Genuine declaration/use events, effective identity and completed demand scopes.
use crate::{pipeline, sema, support::diagnostics::WarningCode};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn analyze(source: &str) -> TestResult<Vec<WarningCode>> {
    let report = crate::test_source::resolve(source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok(analysis.facts.warnings().map(|row| row.code).collect())
}
#[test]
fn real_deprecated_function_protocol() -> TestResult {
    for (source, expected) in [
        ("fulfilled.rs", vec![]),
        (
            "unfulfilled.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("ordinary-warning.rs", vec![WarningCode::Deprecated]),
        ("deny-overridden.rs", vec![]),
        (
            "nested-expect.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "overridden-allow.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "self-expect.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("self-expect-allowed.rs", vec![]),
        ("allow-unfulfilled.rs", vec![]),
        ("reason.rs", vec![WarningCode::UnfulfilledLintExpectations]),
        ("cfg-disabled.rs", vec![]),
        ("cfg-attr-disabled.rs", vec![]),
        ("two-events-one-expect.rs", vec![]),
        (
            "two-entries-one-event.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "duplicate-entries.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("must-use-event.rs", vec![]),
        ("unsafe-operation-event.rs", vec![]),
        ("static-reference-event.rs", vec![]),
        ("unreachable-event.rs", vec![]),
        (
            "unreachable-warning.rs",
            vec![WarningCode::UnreachablePatterns],
        ),
        ("same-package-import.rs", vec![WarningCode::Deprecated]),
        ("same-origin-module.rs", vec![WarningCode::Deprecated]),
        ("different-origin.rs", vec![WarningCode::Deprecated]),
        ("group-event.rs", vec![]),
        (
            "group-unfulfilled.rs",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        ("atomic-order-event.rs", vec![]),
        (
            "atomic-order-warning.rs",
            vec![WarningCode::InvalidAtomicOrdering],
        ),
    ] {
        assert_eq!(analyze(&fixture(source)?)?, expected, "{source}");
    }
    for (source, message) in [
        ("ordinary-deny.rs", "use of deprecated declaration"),
        (
            "deny-unfulfilled.rs",
            "this lint expectation is unfulfilled",
        ),
        (
            "forbid-not-overridden.rs",
            "forbidden deprecated lint cannot be weakened",
        ),
        (
            "unsafe-permission.rs",
            "raw pointer dereference requires an unsafe context",
        ),
    ] {
        let Err(error) = analyze(&fixture(source)?) else {
            return Err(format!("accepted {source}").into());
        };
        assert!(error.to_string().contains(message), "{source}: {error}");
    }
    Ok(())
}
#[test]
fn real_deprecated_macro_invocation_and_reexport() -> TestResult {
    for source in ["macro-invocation.rs", "macro-reexport.rs"] {
        assert!(analyze(&fixture(source)?)?.is_empty(), "{source}");
    }
    assert_eq!(
        analyze(&fixture("nondeprecated-reexport.rs")?)?,
        vec![WarningCode::UnfulfilledLintExpectations]
    );
    Ok(())
}
#[test]
fn completed_scope_and_repeated_report_sessions_are_independent() -> TestResult {
    let source = "#[deprecated]fn old(){} #[expect(deprecated)]fn f(){old()}";
    let report = crate::test_source::resolve(source, true)?;
    for _ in 0..3 {
        let facts = sema::declarations::analyze(&report)?;
        assert!(facts.warnings().next().is_none());
        let analysis = pipeline::analyze(&report, facts)?;
        assert!(analysis.facts.warnings().next().is_none());
    }
    Ok(())
}

fn fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa40/tests/spec/lint-expectations")
            .join(name),
    )
}

#[test]
fn primary_failure_preserves_actual_prior_warnings_without_false_finalization() -> TestResult {
    for (name, expected) in [
        ("warning-before-error.rs", vec![WarningCode::Deprecated]),
        ("primary-error-no-completion.rs", vec![]),
        ("import-expect-before-error.rs", vec![]),
    ] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let Err(error) = pipeline::analyze_source(&report) else {
            return Err(format!("accepted {name}").into());
        };
        assert_eq!(
            error
                .warnings()
                .iter()
                .map(|row| row.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn selected_runtime_closes_only_completed_function_scope() -> TestResult {
    let source = fixture("selected-scopes.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let root = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| report.package.names.get(definition.name) == "checked")
        .map(|(index, _)| u32::try_from(index))
        .transpose()?
        .ok_or("checked function identity")?;
    for _ in 0..2 {
        let facts = sema::declarations::analyze(&report)?;
        assert!(facts.warnings().next().is_none());
        let (facts, _) =
            pipeline::prepare_local_runtime(&report, &[crate::resolve::model::DefId(root)])?;
        assert_eq!(
            facts.warnings().map(|row| row.code).collect::<Vec<_>>(),
            vec![WarningCode::UnfulfilledLintExpectations]
        );
        assert_eq!(
            facts
                .warnings()
                .next()
                .ok_or("missing checked owner warning")?
                .diagnostic
                .span
                .start,
            0
        );
    }
    Ok(())
}
#[test]
fn genuine_source_grouped_try_reexport_has_opposing_identity_controls() -> TestResult {
    for (name, expected) in [
        ("grouped-try", vec![]),
        (
            "ordinary-try",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
        (
            "omitted-try",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa31/tests/spec/source-deprecation/{name}.rs"));
        let report =
            crate::test_source::source_provider(&std::fs::read_to_string(path)?, "lint_source")?;
        let analysis = pipeline::analyze_source(&report)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .map(|row| row.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn expectation_storage_scales_in_cold_rows_with_four_byte_dense_nodes() -> TestResult {
    use std::fmt::Write as _;
    assert_eq!(size_of::<crate::resolve::lint_policy::Level>(), 1);
    for width in [1, 8, 32, 128] {
        let mut source = String::from("#[deprecated]fn old(){}");
        for index in 0..width {
            write!(
                source,
                "#[expect(deprecated, reason=\"intentional\")]fn f{index}(){{old();}}"
            )?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        assert!(analysis.facts.warnings().next().is_none());
        let (nodes, capacity, outer) = analysis.facts.retained_policy_storage();
        let (rows, row_capacity, row_size) = analysis.facts.retained_policy_rows();
        assert_eq!(rows, width + 2);
        assert!(row_capacity >= rows && capacity >= nodes);
        let producer = report.lint_producer().ok_or("producer")?;
        let expectation_capacity: usize = producer
            .owners
            .iter()
            .map(|owner| owner.parsed.expectations.capacity())
            .sum();
        let expectation_bytes = expectation_capacity * size_of::<super::Expectation>();
        let work = producer.work();
        assert_eq!(work.expectations, width);
        assert!(work.steps < 20_000 * (width + 1));
        assert!(work.bytes < 20_000 * (width + 1));
        println!(
            "lint storage width={width} dense_nodes={nodes} dense_bytes={} cold_rows={rows} cold_capacity={row_capacity} row_bytes={row_size} expectation_capacity={expectation_capacity} expectation_bytes={expectation_bytes} outer={outer} work={} paid_bytes={}",
            capacity * size_of::<u32>(),
            work.steps,
            work.bytes
        );
    }
    Ok(())
}
#[test]
fn warm_expectation_queries_reject_changed_headers_and_keep_failed_budget_work() -> TestResult {
    use crate::resolve::lint_policy::Lint;
    let mut report =
        crate::test_source::resolve("#[expect(deprecated,reason=\"kept\")]fn f(){}", true)?;
    let mut producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("producer")?;
    let key = producer.owners[0].key;
    let tail = producer.index[&key];
    let policy = producer
        .policy(&report.package, Some(tail), key)
        .map_err(crate::resolve::package::Error::from)?;
    let id = policy
        .expectation(Lint::Deprecated)
        .ok_or("expectation identity")?;
    assert!(
        producer
            .fulfill_early(&report.package, id, key, Lint::MustUse)
            .is_err()
    );
    assert!(!producer.owners[0].parsed.expectations[0].early.get());
    let cold = producer.work();
    producer
        .policy(&report.package, Some(tail), key)
        .map_err(crate::resolve::package::Error::from)?;
    let warm = producer.work();
    assert_eq!(cold.compositions, warm.compositions);
    assert!(warm.steps > cold.steps);
    let node = &mut report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[key.node.get() as usize];
    let original = node.kind;
    if let crate::syntax::ast::Kind::Attributed { attributes, .. } = node.kind {
        node.kind = crate::syntax::ast::Kind::Attributed {
            attributes,
            value: crate::syntax::ast::NodeId::new(0),
        };
    }
    assert!(producer.policy(&report.package, Some(tail), key).is_err());
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[key.node.get() as usize]
        .kind = original;
    producer
        .policy(&report.package, Some(tail), key)
        .map_err(crate::resolve::package::Error::from)?;
    let before = producer.work();
    assert!(producer.diagnostic_read(key, usize::MAX).is_err());
    assert!(
        producer
            .fulfill_early(&report.package, id, key, Lint::Deprecated)
            .is_err()
    );
    assert!(!producer.owners[0].parsed.expectations[0].early.get());
    assert!(producer.work().steps > before.steps);
    println!(
        "lint warm cold_steps={} warm_steps={} invalidated_steps={} failed_steps={} owner={} expectation={} policy={}",
        cold.steps,
        warm.steps,
        before.steps,
        producer.work().steps,
        size_of_val(&producer.owners[0]),
        size_of_val(&producer.owners[0].parsed.expectations[0]),
        size_of::<crate::resolve::lint_policy::Policy>()
    );
    Ok(())
}

#[test]
fn interleaved_requests_retain_actual_import_fulfillment_without_new_body_demand() -> TestResult {
    let source = fixture("interleaved-import.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut first = sema::declarations::analyze(&report)?;
    assert!(first.warnings().next().is_none());
    assert!(first.typed_function_bodies.is_empty());
    let second = pipeline::analyze_source(&report)?;
    assert!(second.facts.warnings().next().is_none());
    assert!(!second.facts.typed_function_bodies.is_empty());
    first.finalize_lints(&report, false)?;
    assert!(first.warnings().next().is_none());
    assert!(first.typed_function_bodies.is_empty());
    Ok(())
}
#[test]
fn sparse_pending_queries_do_not_scan_unrelated_attribute_owners() -> TestResult {
    use std::fmt::Write as _;
    let mut expected = None;
    for width in [0, 128, 512] {
        let mut source = String::from("#[expect(deprecated)]fn wanted(){}");
        for index in 0..width {
            write!(source, "#[doc=\"unrelated\"]fn unrelated{index}(){{}}")?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let producer = report.lint_producer().ok_or("producer")?;
        let key = producer.owners[0].key;
        let progress = producer
            .new_progress(key)
            .map_err(crate::resolve::package::Error::from)?
            .ok_or("request progress")?;
        assert_eq!(producer.expectation_owners.len(), 1);
        assert_eq!(producer.owners.len(), width + 1);
        let before = producer.work();
        let pending = producer
            .pending(&report.package, &progress)
            .map_err(crate::resolve::package::Error::from)?;
        assert_eq!(pending.len(), 1);
        let steps = producer.work().steps - before.steps;
        assert_eq!(steps, *expected.get_or_insert(steps));
        let foreign = crate::test_source::resolve(&source, true)?;
        assert!(
            foreign
                .lint_producer()
                .ok_or("foreign producer")?
                .pending(&foreign.package, &progress)
                .is_err()
        );
        println!(
            "lint pending unrelated_owners={width} expectation_owners=1 steps={steps} progress_bytes={}",
            size_of_val(progress.as_ref())
        );
    }
    Ok(())
}
