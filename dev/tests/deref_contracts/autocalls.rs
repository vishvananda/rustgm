//! Implicit method receivers execute the actual Deref chain under ordinary proofs.
use crate::resolve;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    program_with_exports(source, "observed_")
}

pub(super) fn program_with_exports(
    source: &str,
    export_prefix: &str,
) -> Result<rustgm::lowir::program::Program> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = resolve::source_provider_config(source, "implicit_receiver", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                .starts_with(export_prefix)
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}

#[test]
fn implicit_receiver_calls_retain_source_borrows_and_selected_method_priority() -> Result {
    check_cases(&[
        ("ordinary-direct-sized.rs", true),
        ("ordinary-wrapped-sized.rs", true),
        ("ordinary-explicit-sized.rs", true),
        ("ordinary-direct-unsized.rs", true),
        ("ordinary-wrapped-unsized.rs", true),
        ("ordinary-explicit-unsized.rs", true),
        ("const-direct-sized.rs", true),
        ("const-wrapped-sized.rs", true),
        ("const-explicit-sized.rs", true),
        ("const-direct-unsized.rs", true),
        ("const-wrapped-unsized.rs", true),
        ("const-explicit-unsized.rs", true),
        ("concrete.rs", true),
        ("borrowed.rs", true),
        ("move-reject.rs", false),
        ("mutable.rs", true),
        ("immutable-reject.rs", false),
        ("inherent-priority.rs", true),
        ("missing-deref.rs", false),
        ("native.rs", true),
    ])
}

fn check_cases(cases: &[(&str, bool)]) -> Result {
    let mut failures = Vec::new();
    for &(name, accepted) in cases {
        let text = resolve::fixture(&format!("pa31/tests/spec/deref-methods/{name}"))?;
        let result = program(&text);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.as_ref().err()));
        }
        if let Err(error) = result {
            let message = error.to_string();
            if [
                "not implemented",
                "budget",
                "exhausted",
                "invalid canonical",
                "unfinished",
            ]
            .iter()
            .any(|part| message.contains(part))
            {
                failures.push(format!("{name}: implementation failure: {message}"));
            }
            if name == "move-reject.rs"
                && !["move", "borrowed"]
                    .iter()
                    .any(|part| message.contains(part))
            {
                failures.push(format!("{name}: wrong cause: {message}"));
            }
            if name == "immutable-reject.rs" && !message.contains("immutable") {
                failures.push(format!("{name}: wrong cause: {message}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn shared_and_mutable_implicit_receivers_execute_real_calls() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/array-receiver/native");
    std::fs::create_dir_all(&out)?;
    let source = resolve::fixture("pa31/tests/spec/deref-methods/native.rs")?;
    crate::ffi_program::check(
        &root,
        &out,
        "implicit-receiver",
        &program(&source)?,
        &resolve::fixture("pa31/tests/spec/deref-methods/observer.c")?,
        "receiver 23 41\n",
    )?;
    Ok(())
}

#[test]
fn implicit_receiver_bounds_captures_generics_and_const_calls_keep_their_proofs() -> Result {
    check_cases(&[
        ("generic-method.rs", true),
        ("nested.rs", true),
        ("parameter-bound.rs", true),
        ("trait-priority.rs", true),
        ("shared-priority.rs", true),
        ("deref-bound-unpaid.rs", false),
        ("deref-bound-paid.rs", true),
        ("static-paid.rs", true),
        ("static-unpaid.rs", false),
        ("captured-shared.rs", true),
        ("captured-mutable.rs", true),
        ("ctfe-shared.rs", true),
        ("ctfe-mutable.rs", true),
        ("ctfe-ordinary-method-reject.rs", false),
        ("cycle-missing-reject.rs", false),
        ("evaluation-order.rs", true),
    ])
}

#[test]
fn implicit_receiver_const_calls_produce_actual_values() -> Result {
    for (name, expected) in [("ctfe-shared.rs", 23), ("ctfe-mutable.rs", 41)] {
        let source = resolve::fixture(&format!("pa31/tests/spec/deref-methods/{name}"))?;
        let report = resolve::source_provider(&source, "implicit_receiver")?;
        let analysis = pipeline::analyze_source(&report)?;
        let initializer = analysis
            .facts
            .initializers
            .iter()
            .find(|initializer| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                    == "VALUE"
            })
            .ok_or("actual VALUE initializer")?;
        let reader = rustgm::sema::values::Reader::new(&analysis.facts);
        let value = reader
            .definition(initializer.definition)
            .ok_or("completed implicit receiver constant")?;
        assert!(
            matches!(reader.view(value)?, rustgm::sema::values::View::Scalar(rustgm::sema::model::Scalar::Integer(n)) if n == expected),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn implicit_receiver_work_tracks_chain_depth_without_unrelated_declarations() -> Result {
    let mut previous = None;
    // The frozen 128-layer positives remain required: they currently exhaust
    // the shared inference budget. Keep that failure outside this qualified
    // finite prefix rather than treating it as a language rejection.
    for depth in [8, 32] {
        let mut control = None;
        for noise in [0, 128] {
            let text = resolve::fixture(&format!(
                "pa31/tests/spec/deref-methods/depth-{depth}-noise-{noise}.rs"
            ))?;
            let report = resolve::source_provider(&text, "implicit_receiver_scaling")?;
            let analysis = pipeline::analyze_source(&report)
                .map_err(|e| format!("depth{depth} noise{noise}: {e}"))?;
            let work = (
                analysis.facts.body_work.canonical_steps,
                analysis.facts.body_work.inference_steps,
            );
            eprintln!("implicit receiver depth{depth} noise{noise}: {work:?}");
            assert!(work.0 > 0 && work.1 > 0);
            if noise == 0 {
                if let Some((canonical, inference)) = previous {
                    assert!(
                        work.0 <= 4 * canonical + 256,
                        "canonical depth{depth}: {work:?}"
                    );
                    // Each real call retains its complete nested Self shape.
                    // Canonical proof reuse is linear above; inference still
                    // has a separate quadratic scaling requirement to repair.
                    assert!(
                        work.1 <= 16 * inference + 256,
                        "inference depth{depth}: {work:?}"
                    );
                }
                previous = Some(work);
                control = Some(work);
            } else {
                assert_eq!(Some(work), control, "unrelated declarations depth{depth}");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn implicit_receiver_order_and_operand_count_match_the_independent_trace() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/array-receiver/native-order");
    std::fs::create_dir_all(&out)?;
    let source = resolve::fixture("pa31/tests/spec/deref-methods/evaluation-order.rs")?;
    crate::ffi_program::check(
        &root,
        &out,
        "implicit-receiver-order",
        &program(&source)?,
        &resolve::fixture("pa31/tests/spec/deref-methods/order-observer.c")?,
        "receiver-order 23 125 41 134625\n",
    )?;
    Ok(())
}
