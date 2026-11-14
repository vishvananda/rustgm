//! Real source dereference dispatch preserves effects, places and ownership.
#[path = "deref_contracts/autocalls.rs"]
mod autocalls;
#[path = "deref_contracts/cloning.rs"]
mod cloning;
#[path = "deref_contracts/coercions.rs"]
mod coercions;
use crate::resolve as test_source;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "deref_source")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_source_deref_and_mutable_places_keep_real_bodies_and_proofs() -> TestResult {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in test_source::fixture("pa31/tests/spec/dereference-source/cases.tsv")?.lines() {
        let (path, status) = line.split_once('\t').ok_or("case columns")?;
        let result = program(&test_source::fixture(path)?);
        let failure = match (status, result) {
            ("0", Err(e)) => Some(format!("{path}: {e}")),
            ("1", Ok(_)) => Some(format!("accepted {path}")),
            ("1", Err(e))
                if e.to_string()
                    .contains("semantic contract is not implemented") =>
            {
                Some(format!("{path}: {e}"))
            }
            ("0", Ok(_)) | ("1", Err(_)) => None,
            _ => return Err("invalid prediction".into()),
        };
        if let Some(failure) = failure {
            failures.push(failure);
        }
        count += 1;
    }
    assert_eq!(count, 39);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn ordinary_deref_method_effects_and_one_time_place_evaluation_reach_native() -> TestResult {
    let source = test_source::fixture("pa31/tests/spec/dereference-source/native.rs")?;
    let program = program(&source)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/deref-source-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn nested_pin_borrow_mutates_the_actual_operand_before_directed_conversion() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/nested-native");
    std::fs::create_dir_all(&folder)?;
    let text = test_source::fixture("pa31/tests/spec/dereference-source/nested-borrow.rs")?;
    crate::ffi_program::check(
        &root,
        &folder,
        "nested-borrow",
        &program(&text)?,
        &test_source::fixture("pa31/tests/spec/dereference-source/nested-observer.c")?,
        "23\n",
    )?;
    Ok(())
}

#[test]
fn generic_receiver_requirements_share_one_current_caller_proof_graph() -> TestResult {
    let source =
        test_source::fixture("pa31/tests/spec/dereference-source/receiver-alias-depth.rs")?;
    let mut steps = Vec::new();
    for depth in [8, 16, 32, 64, 128] {
        let text = source.replace("self:Layer128", &format!("self:Layer{depth}"));
        let report = test_source::source_provider(&text, "receiver_scaling")?;
        let analysis = pipeline::analyze_source(&report)
            .map_err(|e| format!("depth {depth}; preceding work {steps:?}: {e}"))?;
        steps.push(analysis.facts.body_work.canonical_steps);
    }
    eprintln!("receiver canonical work: {steps:?}");
    assert!(steps.iter().all(|&work| work > 0));
    assert!(steps.windows(2).all(|pair| pair[1] <= pair[0] * 2));
    Ok(())
}

const CONDITIONAL_MEMBERS: &str = "pa31/tests/spec/conditional-member-calls";

#[test]
fn generic_const_member_calls_keep_checked_source_applications() -> TestResult {
    for name in [
        "shared-field",
        "mutable-field",
        "direct-method",
        "abstract-premise",
        "ordinary-control",
        "original-fields",
        "native",
    ] {
        let source = test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/{name}.rs"))?;
        program(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in ["abstract-unpaid", "nonconst-impl"] {
        let source = test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/{name}.rs"))?;
        let error = program(&source)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(
            error.to_string().contains("trait obligation requires"),
            "{name}: {error}"
        );
    }
    for (name, cause) in [
        ("lifetime-escape", "lifetime"),
        ("shared-mutation", "borrow"),
    ] {
        let source = test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/{name}.rs"))?;
        let error = program(&source)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn ordinary_const_member_minima_keep_their_defining_effect() -> TestResult {
    for (name, accepted) in [
        ("generic", true),
        ("abstract-premise", true),
        ("ordinary-control", true),
        ("abstract-unpaid", false),
        ("nonconst-impl", false),
    ] {
        let text = test_source::fixture(&format!(
            "pa25/tests/spec/conditional-member-calls/{name}.rs"
        ))?;
        let result = (|| -> TestResult {
            let report = test_source::resolve(&text, true)?;
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report)?;
            lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok(())
        })();
        if accepted {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result.err().ok_or_else(|| format!("accepted {name}"))?;
            assert!(
                error.to_string().contains("trait obligation requires"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}

#[test]
fn generic_const_member_work_scales_with_source_calls() -> TestResult {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let text = test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/scaling-{count}.rs"))?;
        let report = test_source::source_provider(&text, "conditional_members_scaling")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ];
        println!("conditional member calls={count} work={work:?}");
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
fn generic_const_member_instances_execute_selected_shared_and_unique_calls() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/conditional-members/native");
    std::fs::create_dir_all(&out)?;
    let text = test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/native.rs"))?;
    crate::ffi_program::check(
        &root,
        &out,
        "members",
        &program(&text)?,
        &test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/observe.c"))?,
        &test_source::fixture(&format!("{CONDITIONAL_MEMBERS}/expected.stdout"))?,
    )?;
    Ok(())
}

const EXPLICIT_CONST: &str = "pa31/tests/spec/explicit-const-dereference";

#[test]
fn explicit_const_dereference_preserves_the_selected_reference_and_place() -> TestResult {
    for name in [
        "shared",
        "unique",
        "impl-shared",
        "impl-unique",
        "ordinary",
        "native",
        "constant",
    ] {
        let source = test_source::fixture(&format!("{EXPLICIT_CONST}/{name}.rs"))?;
        program(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("nonconst", "trait obligation requires"),
        ("shared-mutation", "borrow"),
        ("lifetime-escape", "lifetime"),
    ] {
        let source = test_source::fixture(&format!("{EXPLICIT_CONST}/{name}.rs"))?;
        let error = program(&source)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn explicit_const_place_work_includes_prepared_validation() -> TestResult {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let source = test_source::fixture(&format!("{EXPLICIT_CONST}/scaling-{count}.rs"))?;
        let report = test_source::source_provider(&source, "explicit_const_places")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ];
        println!("explicit const places={count} work={work:?}");
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
fn explicit_const_dereference_executes_mutable_and_constant_instances() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/explicit-members/native");
    std::fs::create_dir_all(&out)?;
    for name in ["native", "constant"] {
        let text = test_source::fixture(&format!("{EXPLICIT_CONST}/{name}.rs"))?;
        crate::ffi_program::check(
            &root,
            &out,
            name,
            &program(&text)?,
            &test_source::fixture(&format!("{EXPLICIT_CONST}/observe.c"))?,
            &test_source::fixture(&format!("{EXPLICIT_CONST}/expected.stdout"))?,
        )?;
    }
    Ok(())
}
