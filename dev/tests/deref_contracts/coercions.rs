//! Direct and explicit places anchor the separately frozen implicit conversions.
use super::{TestResult, program, test_source};

#[test]
fn direct_and_explicit_reference_controls_retain_checked_body_contracts() -> TestResult {
    for name in ["direct-known", "explicit-place", "explicit-temporary"] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        program(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        (
            "implicit-mutability-reject",
            "cannot coerce a shared reference to a mutable reference",
        ),
        ("implicit-wrong-target-reject", "incompatible types"),
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        let error = program(&source)
            .err()
            .ok_or("accepted invalid conversion")?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    // Seven implicit positives, the actual escape rejection, and native order
    // remain required in both manifests. These controls do not qualify them.
    Ok(())
}

#[test]
fn implicit_reference_conversions_keep_actual_calls_and_their_loans() -> TestResult {
    let mut failures = Vec::new();
    for name in [
        "implicit-known",
        "implicit-inferred",
        "implicit-temporary-known",
        "implicit-temporary-inferred",
        "implicit-borrowed",
        "implicit-shared",
        "implicit-annotated",
        "native-order",
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        if let Err(error) = super::autocalls::program(&source) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let source =
        test_source::fixture("pa31/tests/spec/borrow-coercions/implicit-escape-reject.rs")?;
    let error = program(&source)
        .err()
        .ok_or("accepted escaping wrapper borrow")?;
    let message = error.to_string();
    assert!(
        message.contains("borrow") || message.contains("lifetime"),
        "{message}"
    );
    assert!(!message.contains("incompatible types"), "{message}");
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn mutable_and_shared_conversions_execute_once_in_argument_order() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/refcell-coercion/native");
    std::fs::create_dir_all(&out)?;
    for (name, observer, expected) in [
        ("native-order", "observer.c", "borrow-coercion 23 41 1342\n"),
        (
            "native-guard",
            "guard-observer.c",
            "borrow-guard 23 41 1345\n",
        ),
        ("const-paid", "constant-observer.c", "borrow-const 41\n"),
        (
            "const-unforced-wrapper",
            "constant-observer.c",
            "borrow-const 41\n",
        ),
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/borrow-coercions/{name}.rs"))?;
        crate::ffi_program::check(
            &root,
            &out,
            &format!("borrow-{name}"),
            &super::autocalls::program(&source)?,
            &test_source::fixture(&format!("pa30/tests/spec/borrow-coercions/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn source_reference_conversions_keep_const_effects_contexts_and_captures() -> TestResult {
    let mut failures = Vec::new();
    for line in test_source::fixture("pa31/tests/spec/borrow-coercions/expanded-cases.tsv")?.lines()
    {
        let (path, status) = line.split_once('\t').ok_or("conversion case columns")?;
        let result = super::autocalls::program(&test_source::fixture(path)?);
        match (status, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error)) => {
                let message = error.to_string();
                if [
                    "budget",
                    "not implemented",
                    "lost its",
                    "changed its",
                    "missing",
                    "invalid",
                    "panicked",
                ]
                .iter()
                .any(|part| message.contains(part))
                {
                    failures.push(format!("{path}: implementation failure: {message}"));
                }
                if path.ends_with("borrowed-lifetime-reject.rs")
                    && !["borrow", "lifetime"]
                        .iter()
                        .any(|part| message.contains(part))
                {
                    failures.push(format!("{path}: wrong loan rejection: {message}"));
                }
            }
            (_, result) => failures.push(format!(
                "{path}: required {status}, actual {:?}",
                result.err()
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn reference_conversion_work_scales_without_unrelated_types() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = test_source::fixture(&format!(
                "pa31/tests/spec/borrow-coercions/paid-width-{width}-noise-{noise}.rs"
            ))?;
            let report = test_source::source_provider(&text, "coercion_scaling")?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.trait_selections,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("coercion width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(old) = previous {
                    for (before, actual) in std::iter::zip(old, work) {
                        assert!(actual <= 4 * before + 2048, "{old:?}->{work:?}");
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
