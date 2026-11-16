//! Explicit call-trait methods retain their validated tuple ABI and current effect.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/explicit-call-members";
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    packet_report(PACKET, name)
}
fn packet_report(packet: &str, name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let source = crate::resolve::fixture(&format!("{packet}/{name}.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(crate::resolve::source_provider_config(
        &source,
        "explicit_calls",
        &config,
    )?)
}
fn program(report: &rustgm::resolve::expansion::Report) -> Result<rustgm::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            matches!(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name),
                "entry" | "probe"
            )
        })
        .map(|function| function.definition)
        .collect::<Vec<_>>();
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}
#[test]
fn explicit_const_members_require_their_actual_role_tuple_and_receiver() -> Result {
    let mut failures = Vec::new();
    for name in [
        "direct-mut",
        "direct-shared",
        "direct-once",
        "relay",
        "original-drain",
        "native-runtime",
    ] {
        if let Err(error) = report(name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "ordinary-impl",
        "immutable",
        "wrong-tuple",
        "unsafe-impl",
    ] {
        let error = report(name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        if [
            "not implemented",
            "unsupported",
            "exhausted",
            "budget",
            "current conditional trait premise",
            "normalized safe source signature",
        ]
        .iter()
        .any(|message| error.contains(message))
        {
            failures.push(format!("unqualified {name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn explicit_member_work_tracks_actual_typed_calls() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = report(&format!("scaling-{count}"))?;
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
        println!("explicit members={count} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= 4 * old + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn explicit_const_delegation_executes_the_actual_callable_and_mutable_receiver() -> Result {
    native(PACKET, "obj/parallel/explicit-call-members/native")
}
fn native(packet: &str, output: &str) -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    crate::observed_program::execute(
        &root.join(output),
        &program(&packet_report(packet, "native-runtime")?)?,
        &root.join(packet).join("observe.c"),
        b"41 59\n",
    )
}

const BORROWED: &str = "pa31/tests/spec/borrowed-callables";
const CAPTURED: &str = "pa31/tests/spec/captured-const-calls";
#[test]
fn captured_const_calls_keep_the_actual_closure_and_enclosing_formals() -> Result {
    for name in [
        "borrowed-mut",
        "borrowed-shared",
        "opaque-adapter",
        "inherited-adapter",
        "returned-shared",
        "returned-two-args",
        "native-runtime",
    ] {
        packet_report(CAPTURED, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("plain-bound", "known function type"),
        ("immutable-adapter", "mutable"),
        ("wrong-arity", "argument count"),
        ("wrong-argument", "incompatible types"),
    ] {
        let error = packet_report(CAPTURED, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn captured_const_call_work_tracks_actual_closures() -> Result {
    callable_scaling(CAPTURED, "adapter-scaling")
}
fn callable_scaling(packet: &str, prefix: &str) -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = packet_report(packet, &format!("{prefix}-{count}"))?;
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
        println!("captured callable {count}: {work:?}");
        if let Some(before) = previous {
            for (actual, old) in work.into_iter().zip(before) {
                assert!(actual <= old.saturating_mul(5));
            }
        }
        previous = Some(work);
    }
    Ok(())
}
#[test]
fn captured_local_cleanup_and_ctfe_remain_accepting_requirements() -> Result {
    // Independent Rust accepts every positive below. Keep the concrete unfinished
    // boundary explicit rather than recording those programs as expected rejects.
    for name in [
        "owned-mut",
        "owned-shared",
        "owned-once",
        "two-args",
        "ordinary-control",
        "scaling-8",
        "scaling-32",
        "scaling-128",
    ] {
        let error = packet_report(CAPTURED, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("{name}: update completed accepting requirement"))?;
        assert!(
            error
                .to_string()
                .contains("active cleanup requires const Destruct"),
            "{name}: {error}"
        );
    }
    let error = packet_report(CAPTURED, "native-ctfe")
        .and_then(|report| program(&report))
        .err()
        .ok_or("update completed CTFE accepting requirement")?;
    assert!(error.to_string().contains("known function type"), "{error}");
    let error = packet_report(CAPTURED, "returned-once")
        .and_then(|report| program(&report))
        .err()
        .ok_or("update completed consuming closure cleanup requirement")?;
    assert!(
        error
            .to_string()
            .contains("const cleanup selection requires its actual source function"),
        "{error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn captured_const_adapter_executes_its_actual_function_and_receiver() -> Result {
    native(CAPTURED, "obj/parallel/captured-const-calls/native")
}
#[test]
fn borrowed_const_callables_keep_source_adjustments_and_permissions() -> Result {
    let mut failures = Vec::new();
    for name in [
        "direct-mut",
        "field-mut",
        "direct-shared",
        "field-shared",
        "nested-mut",
        "nested-shared",
        "shared-through-mut",
        "native-runtime",
        "borrowed-relay",
    ] {
        if let Err(error) = packet_report(BORROWED, name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "shared-cannot-mutate",
        "field-cannot-mutate",
        "once-cannot-move",
        "wrong-argument",
    ] {
        let error = packet_report(BORROWED, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        if [
            "not implemented",
            "unsupported",
            "exhausted",
            "budget",
            "differs from its defining",
            "requires its source syntax",
            "changed its source",
            "changed its exact source",
        ]
        .iter()
        .any(|message| error.contains(message))
        {
            failures.push(format!("unqualified {name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn borrowed_callable_work_tracks_actual_reference_paths() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = packet_report(BORROWED, &format!("scaling-{count}"))?;
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
        println!("borrowed callable {count}: {work:?}");
        if let Some(before) = previous {
            for (actual, old) in work.into_iter().zip(before) {
                assert!(actual <= old.saturating_mul(5));
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn borrowed_callable_fields_execute_the_actual_mutable_reborrow() -> Result {
    native(BORROWED, "obj/parallel/borrowed-callables/native")
}

const MEMBERS: &str = "pa31/tests/spec/closure-members";
#[test]
fn const_closure_members_keep_their_actual_source_owner_and_effect() -> Result {
    for name in [
        "generic-ufcs",
        "generic-method",
        "concrete",
        "native-runtime",
    ] {
        packet_report(MEMBERS, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("plain-bound", "trait obligation requires"),
        ("ordinary-closure", "quantified body requirement needs"),
        ("wrong-argument", "incompatible types"),
    ] {
        let error = packet_report(MEMBERS, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn const_closure_member_work_tracks_actual_source_calls() -> Result {
    callable_scaling(MEMBERS, "scaling")
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn const_closure_member_executes_its_selected_implementation() -> Result {
    native(MEMBERS, "obj/parallel/closure-members/native")
}

const OPERATORS: &str = "pa31/tests/spec/source-call-members";
const OPERATOR_CASES: [&str; 5] = ["mutable", "once", "nested", "const-generator", "owned"];
/// Call syntax on a generic callable whose actual type is a nominal receiver,
/// as core's `try_from_fn_erased` calls `&mut Drain`, runs the selected impl.
#[test]
fn call_operators_on_nominal_receivers_select_their_source_members() -> Result {
    for name in OPERATOR_CASES {
        packet_report(OPERATORS, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    let error = packet_report(OPERATORS, "shared")
        .and_then(|report| program(&report))
        .err()
        .ok_or("accepted shared")?;
    assert!(
        error
            .to_string()
            .contains("associated type requires an unambiguous validated implementation"),
        "shared: {error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn call_operators_on_nominal_receivers_execute_their_source_members() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in OPERATOR_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/source-call-members/native/{name}")),
            &program(&packet_report(OPERATORS, name)?)?,
            &root.join(OPERATORS).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}

const RUNTIME_CONST: &str = "pa31/tests/spec/runtime-const-closures";
const RUNTIME_CONST_CASES: [&str; 4] = ["ordinary", "pair", "capturing", "direct"];
/// A const closure executed natively, as core's `wrap_mut_1(f)` is from
/// `Iterator::fold`, may call an ordinary closure; const evaluation may not.
#[test]
fn native_const_closures_call_ordinary_closures_that_const_evaluation_rejects() -> Result {
    for name in RUNTIME_CONST_CASES {
        packet_report(RUNTIME_CONST, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        (
            "ctfe-callee",
            "trait obligation requires an unambiguous matching assumption or implementation",
        ),
        (
            "plain-callee",
            "non-const calls are not allowed in constant initializers",
        ),
    ] {
        let error = packet_report(RUNTIME_CONST, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn native_const_closures_execute_their_ordinary_callees() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in RUNTIME_CONST_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/runtime-const-closures/native/{name}")),
            &program(&packet_report(RUNTIME_CONST, name)?)?,
            &root.join(RUNTIME_CONST).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}

const PROJECTED: &str = "pa31/tests/spec/projected-captures";
const PROJECTED_CASES: [&str; 4] = ["front", "written", "flatten", "plain"];
/// A returned closure capturing `&mut Option<T::IntoIter>`, as core's
/// `flatten` does, is called with its environment normalized.
#[test]
fn returned_closures_with_projected_captures_match_their_checked_execution() -> Result {
    for name in PROJECTED_CASES {
        packet_report(PROJECTED, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    let error = packet_report(PROJECTED, "borrowed")
        .and_then(|report| program(&report))
        .err()
        .ok_or("accepted borrowed")?;
    assert!(
        error
            .to_string()
            .contains("access conflicts with a live reference loan"),
        "borrowed: {error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn returned_closures_with_projected_captures_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in PROJECTED_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/projected-captures/native/{name}")),
            &program(&packet_report(PROJECTED, name)?)?,
            &root.join(PROJECTED).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}

#[test]
fn const_closure_ctfe_and_default_premises_remain_accepting_requirements() -> Result {
    for (name, cause) in [
        ("native-ctfe", "known function type"),
        (
            "default-owner",
            "const call operators require their actual source function owner",
        ),
    ] {
        let error = packet_report(MEMBERS, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("{name}: update completed accepting requirement"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

const COPIES: &str = "pa31/tests/spec/captured-copies";
const COPY_CASES: [&str; 6] = [
    "captured-read",
    "captured-reference",
    "captured-callee",
    "once",
    "copy-fold",
    "take-copied",
];
/// A capturing closure keeps its creator's `T: Copy`, as core's `copy_fold`
/// needs for `move |acc, &elt| f(acc, elt)`.
#[test]
fn capturing_closures_copy_values_their_creator_bounds_copy() -> Result {
    for name in COPY_CASES {
        packet_report(COPIES, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    let error = packet_report(COPIES, "not-copy")
        .and_then(|report| program(&report))
        .err()
        .ok_or("accepted not-copy")?;
    assert!(
        error
            .to_string()
            .contains("cannot move a non-Copy value out of a reference"),
        "not-copy: {error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn capturing_closures_copy_values_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in COPY_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/captured-copies/native/{name}")),
            &program(&packet_report(COPIES, name)?)?,
            &root.join(COPIES).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}

const FORWARDED: &str = "pa31/tests/spec/forwarded-calls";
const FORWARDED_CASES: [&str; 5] = ["mutable", "once", "nested", "shared", "filter"];
/// A reference forwards its call to a nominal referent's impl, as core's
/// `Filter::next` calls `&mut IsNotEmpty` for `split_whitespace`.
#[test]
fn references_forward_calls_to_their_nominal_referents() -> Result {
    for name in FORWARDED_CASES {
        packet_report(FORWARDED, name)
            .and_then(|report| program(&report))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    let error = packet_report(FORWARDED, "mutable-shared")
        .and_then(|report| program(&report))
        .err()
        .ok_or("accepted mutable-shared")?;
    assert!(
        error.to_string().contains(
            "trait obligation requires an unambiguous matching assumption or implementation"
        ),
        "mutable-shared: {error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn references_forward_calls_to_their_nominal_referents_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in FORWARDED_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/forwarded-calls/native/{name}")),
            &program(&packet_report(FORWARDED, name)?)?,
            &root.join(FORWARDED).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}
