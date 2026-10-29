//! Real source macros retain their typed field and original pending range debt.
use super::*;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/derive-type-macros/{name}.rs"
    ))?)
}
#[test]
fn real_nested_type_macros_preserve_source_clone_proofs_and_original_range_frontier() -> TestResult
{
    let source = input("source-runtime")?;
    let report = test_source::source_provider(&source, "nested_type_macros")?;
    let payer = &report.package.source_core().ok_or("actual payer")?.macros;
    let before = payer.usage();
    let derives = report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("actual derive")?;
    for entry in &derives.entries {
        report.expanded_derive(entry.owner, entry.effect)?;
    }
    let after = payer.usage();
    assert!(after.0 > before.0);
    assert_eq!(after.1, before.1);
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
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
        .join("../obj/parallel/derive-type-macros-v1/candidate");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    for name in ["source-wrong-type", "source-missing-feature"] {
        let error = test_source::source_provider(&input(name)?, "type_macro_opponent")
            .and_then(|r| {
                crate::pipeline::analyze_source(&r)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("this parsed field type"), "{name}: {error}");
        if name == "source-missing-feature" {
            assert!(
                error.contains("pattern types require pattern_types"),
                "{error}"
            );
        }
        eprintln!("type macro opponent {name}: {error}");
    }
    match expanded_provider(&input("original-source-frontier")?, "original_niche") {
        Ok(report) => {
            let entries = &report
                .attribute_effects
                .as_deref()
                .and_then(|effects| effects.derives.as_deref())
                .ok_or("actual original derive")?
                .entries;
            assert!(!entries.is_empty());
            for entry in entries {
                report.expanded_derive(entry.owner, entry.effect)?;
            }
            assert!(
                !report.names_resolved,
                "expansion alone grants no semantic readiness"
            );
        }
        Err(error) => {
            let error = error.to_string();
            assert!(
                error.contains("expected punctuation") && error.contains("in macro pattern_type"),
                "{error}"
            );
            eprintln!("original niche Required range parser frontier: {error}");
        }
    }
    Ok(())
}
