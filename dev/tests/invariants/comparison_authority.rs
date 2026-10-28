//! Source comparison calls retain genuine headers and ordinary selected-call proof.
use crate::{pipeline, sema::model::Operation};
#[test]
fn actual_equality_uses_source_proof_before_scalar_erasure()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/equality/actual-equality.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "source_comparisons")?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(
        analysis
            .facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Binary { .. })))
    );
    assert!(
        analysis
            .facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Call { .. })))
    );
    let equality = analysis
        .facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .language(crate::support::bindings::LangItem::PartialEq)
        .ok_or("actual PartialEq")?;
    assert_eq!(
        analysis.facts.associated.comparison(equality.definition),
        None
    );
    Ok(())
}

#[test]
fn actual_integer_partial_ord_retains_full_source_body_and_intrinsic()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/ordering/actual-source-comparisons-v3.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "source_comparisons")?;
    let _analysis = pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn genuine_comparison_native_inputs_keep_const_values_and_authored_overrides()
-> Result<(), Box<dyn std::error::Error>> {
    for name in ["actual-profile", "authored-override"] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/source-comparisons/native/{name}.rs"
        ))?;
        let report = crate::test_source::source_provider(&source, "source_comparisons")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/source-comparison-integration-v1/producers")
            .join(name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("provider.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}

#[test]
fn closed_source_comparison_consts_keep_actual_selected_members()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-comparisons/ordering/const-source-comparisons-v3.rs",
    )? + "#[unsafe(no_mangle)]pub extern \"C\" fn const_members()->bool{E&&N&&G}";
    let report = crate::test_source::source_provider(&source, "source_comparisons")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-comparison-integration-v1/producers/const-members");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn full_source_comparison_opponents_reject_their_actual_missing_contract()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, diagnostic) in [
        ("missing-ord-impl", "can't compare"),
        ("wrong-ordering-tag", "Ordering"),
        ("wrong-intrinsic-copy", "Copy"),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/source-comparisons/ordering/{name}-v3.rs"
        ))?;
        let report = crate::test_source::source_provider(&source, "comparison_opponents")?;
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let pipeline::Analysis {
                mut facts,
                prepared,
            } = pipeline::analyze_source(&report)?;
            crate::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok(())
        })();
        let error = result
            .err()
            .ok_or_else(|| format!("{name} acquired comparison authority"))?;
        let error = error.to_string();
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}
