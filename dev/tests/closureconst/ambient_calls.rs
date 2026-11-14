//! Returned const closures keep the selected owner's defining requirements.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn analyze(input: &str) -> Result<rustgm::sema::model::Declarations> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(input, "const_adapter", &config)?;
    Ok(pipeline::analyze_source(&report)?.facts)
}

#[test]
fn conditional_closure_results_pay_their_instantiated_defining_contracts() -> Result {
    let mut failures = Vec::new();
    for (name, accepted) in [
        ("self", true),
        ("binary", true),
        ("forward", true),
        ("direct", true),
        ("ordinary", true),
        ("const-factory", true),
        ("ordinary-owner-paid", true),
        ("ordinary-owner-unpaid", false),
        ("once-via-mut", true),
        ("lifetime-via-mut", true),
        ("local-unpaid-child-reject", false),
        ("ordinary-bound-reject", false),
        ("ordinary-closure-reject", false),
        ("wrong-output-reject", false),
        ("runtime-creation-reject", false),
    ] {
        let input = crate::resolve::fixture(&format!("pa31/tests/spec/const-adapters/{name}.rs"))?;
        let result = analyze(&input);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.as_ref().err()));
        }
        if let Err(error) = result {
            let message = error.to_string();
            if [
                "not implemented",
                "scoped continuation",
                "budget",
                "invalid canonical",
            ]
            .iter()
            .any(|part| message.contains(part))
            {
                failures.push(format!("{name}: implementation failure: {message}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn defining_clause_work_tracks_functions_without_unrelated_headers() -> Result {
    scaling_work("width")
}

#[test]
fn local_const_completion_retries_one_frontier_after_all_children() -> Result {
    scaling_work("local-width")
}

#[test]
fn completed_local_const_children_still_check_direct_call_legality() -> Result {
    let input =
        crate::resolve::fixture("pa31/tests/spec/const-adapters/local-ordinary-child-reject.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(&input, "local_const_child", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let error = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .err()
    .ok_or("const child accepted a non-const direct call")?;
    assert!(error.to_string().contains("non-const"), "{error}");
    Ok(())
}

fn scaling_work(prefix: &str) -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let output = root.join("obj/parallel/const-adapter/scaling");
    std::fs::create_dir_all(&output)?;
    let mut observations = Vec::new();
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let source = crate::resolve::fixture(&format!(
                "pa31/tests/spec/const-adapters/{prefix}-{width}-noise-{noise}.rs"
            ))?;
            let facts = analyze(&source)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
                facts.body_work.projection_steps,
                facts.body_work.projection_selections,
                facts.body_work.unsizing_inputs,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            observations.push(format!("width={width} noise={noise} work={work:?}"));
            std::fs::write(
                output.join(format!("{prefix}-observations.txt")),
                observations.join("\n"),
            )?;
            if noise == 0 {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 1024, "{before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}

const GENERIC_CALLS: &str = "pa31/tests/spec/generic-const-calls";
fn generic_program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let text = crate::resolve::fixture(&format!("{GENERIC_CALLS}/{name}.rs"))?;
    let report = crate::resolve::source_provider(&text, "generic_calls")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            let name = report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name);
            matches!(name, "entry" | "probe")
        })
        .map(|function| function.definition)
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
fn unused_generic_const_calls_keep_actual_formals_borrows_and_move_checks() -> Result {
    for name in [
        "once-unused",
        "mut-unused",
        "shared-unused",
        "mut-reference",
        "mut-twice",
        "native-mut",
        "native-shared",
        "native-mut-runtime",
        "original-min-by-key",
    ] {
        generic_program(name).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("plain-bound", "known function type"),
        ("once-twice", "moved"),
        ("immutable-mut", "mutable"),
    ] {
        let error = generic_program(name)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn generic_const_receiver_preparation_and_calls_scale_with_functions() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let text = crate::resolve::fixture(&format!("{GENERIC_CALLS}/scaling-{count}.rs"))?;
        let report = crate::resolve::source_provider(&text, "generic_calls")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ];
        println!("generic const calls={count} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= old * 4 + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn generic_const_mutable_function_item_reaches_native() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/generic-const-calls/native");
    std::fs::create_dir_all(&out)?;
    for (name, observer, expected) in [
        ("native-mut", "observe.c", "41\n"),
        ("native-shared", "observe.c", "41\n"),
        ("native-mut-runtime", "runtime-observe.c", "41 59\n"),
    ] {
        crate::ffi_program::check(
            &root,
            &out,
            name,
            &generic_program(name)?,
            &crate::resolve::fixture(&format!("{GENERIC_CALLS}/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
