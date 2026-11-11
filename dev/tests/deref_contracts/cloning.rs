//! A bound-selected target method and its implicit Deref retain distinct debts.
use super::{TestResult, program, test_source};

#[test]
fn concrete_explicit_and_priority_controls_keep_their_actual_receivers() -> TestResult {
    for name in [
        "owned-known",
        "explicit-generic",
        "factory-qualified",
        "full-clone-owned-known",
        "full-clone-factory-qualified",
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/deref-clone/{name}.rs"))?;
        program(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, cause) in [
        ("reference-priority-reject", "incompatible types"),
        ("wrapper-priority-reject", "incompatible types"),
        ("unpaid-reject", "no applicable method for this receiver"),
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/deref-clone/{name}.rs"))?;
        let error = program(&source).err().ok_or("accepted invalid clone")?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    let source = test_source::fixture("pa31/tests/spec/deref-clone/reference-return.rs")?;
    let report = test_source::source_provider(&source, "reference_clone")?;
    rustgm::pipeline::analyze_source(&report)?;
    // The complete RootBuilder artifact remains a required positive in
    // pending.json. Inference completion does not pay aggregate preparation.
    Ok(())
}

#[test]
fn generic_target_clone_keeps_deref_implementation_and_caller_evidence_separate() -> TestResult {
    let mut failures = Vec::new();
    for name in [
        "owned-generic",
        "factory-generic",
        "associated-name",
        "full-clone-owned-generic",
        "full-clone-factory-generic",
        "native-order",
        "same-member-implicit",
        "same-member-explicit",
        "same-member-native",
        "result-context-trait",
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/deref-clone/{name}.rs"))?;
        if let Err(error) = super::autocalls::program_with_exports(&source, "code") {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn result_context_and_protocol_work_scales_without_unrelated_types() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = test_source::fixture(&format!(
                "pa31/tests/spec/deref-clone/width-{width}-noise-{noise}.rs"
            ))?;
            let report = test_source::source_provider(&text, "clone_scaling")?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.trait_selections,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            println!("clone width={width} noise={noise} work={work:?}");
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

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn generic_target_clone_executes_the_actual_receiver_chain_once() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = root.join("obj/parallel/refcell-clone/native");
    std::fs::create_dir_all(&out)?;
    for (name, observer, expected) in [
        ("native-order", "observer.c", "deref-clone 23 123\n"),
        (
            "same-member-native",
            "member-observer.c",
            "deref-member 23 12\n",
        ),
    ] {
        let source = test_source::fixture(&format!("pa31/tests/spec/deref-clone/{name}.rs"))?;
        crate::ffi_program::check(
            &root,
            &out,
            name,
            &super::autocalls::program_with_exports(&source, "code")?,
            &test_source::fixture(&format!("pa30/tests/spec/deref-clone/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
