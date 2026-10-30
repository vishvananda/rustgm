//! The source lint marker never forbids const calls or grants mutation permission.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "advisory_replays.rs"]
mod replays;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-const-item-calls/{name}.rs"
    ))?)
}
fn checked(name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(&input(name)?, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    Ok((report, analysis))
}
fn lower(
    (report, analysis): (Report, pipeline::Analysis),
) -> TestResult<crate::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
fn function<'a>(report: &'a Report, name: &str) -> TestResult<(usize, &'a Definition)> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == name
        })
        .ok_or_else(|| "actual source method".into())
}
#[test]
fn genuine_impl_targets_keep_ordinary_const_mutation_and_unsafe_semantics() -> TestResult {
    for name in [
        "inherent",
        "associated-static",
        "trait-implementation",
        "const-allowed",
        "const-item-runtime",
        "const-item-ufcs",
        "mutation",
        "unsafe-allowed",
        "deny-unused",
        "macro-allowance",
        "macro-input-missing-gate",
        "cfg-removed",
        "independent-markers",
    ] {
        let result = checked(name)?;
        if name == "const-allowed" {
            let (report, analysis) = &result;
            let initializer = analysis
                .facts
                .initializers
                .iter()
                .find(|row| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[row.definition.0 as usize].name)
                        == "V"
                })
                .ok_or("actual V initializer")?;
            let reader = crate::sema::values::Reader::new(&analysis.facts);
            assert!(matches!(
                reader.view(
                    reader
                        .definition(initializer.definition)
                        .ok_or("actual completed V")?
                )?,
                crate::sema::values::View::Scalar(crate::sema::model::Scalar::Integer(7))
            ));
        }
        lower(result).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn real_form_gate_and_method_owner_boundaries_are_hard_errors() -> TestResult {
    for name in [
        "free",
        "trait-required",
        "trait-default",
        "foreign",
        "constant",
        "structure",
        "impl-target",
        "nested-free",
        "closure",
        "list",
        "value",
        "unsafe-wrapper",
        "duplicate",
        "missing-gate",
        "macro-without-allowance",
        "macro-item-input-missing-gate",
    ] {
        let error = checked(name)
            .err()
            .ok_or_else(|| format!("{name}: invalid source marker accepted"))?;
        let text = error.to_string();
        assert!(
            text.contains("const-item")
                || text.contains("rustc_should_not_be_called_on_const_items")
                || text.contains("rustc_attrs"),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn marker_grants_no_const_unsafe_or_shared_reference_mutation_permission() -> TestResult {
    for name in [
        "const-denied",
        "unsafe-denied",
        "mutation-denied",
        "const-body-denied",
    ] {
        let error = checked(name)
            .and_then(lower)
            .err()
            .ok_or_else(|| format!("{name}: permission was invented"))?;
        let text = error.to_string();
        let expected = if name == "unsafe-denied" {
            &["unsafe"][..]
        } else if name == "mutation-denied" {
            &["mutable", "assign", "mutation"][..]
        } else {
            &["const"][..]
        };
        assert!(
            expected.iter().any(|word| text.contains(word)),
            "{name}: {text}"
        );
    }
    Ok(())
}

#[test]
fn actual_advisory_bodies_emit_local_native_roots_without_metadata_publication() -> TestResult {
    for (name, cluster, fixture, expression) in [
        (
            "advisory-mutation",
            "source-const-item-calls",
            "mutation",
            "observe()",
        ),
        (
            "advisory-const",
            "source-const-item-calls",
            "const-allowed",
            "V",
        ),
        (
            "advisory-unsafe",
            "source-const-item-calls",
            "unsafe-allowed",
            "observe()",
        ),
        (
            "confusable-mutation",
            "source-confusable-methods",
            "mutation",
            "observe()",
        ),
        (
            "confusable-const",
            "source-confusable-methods",
            "const-allowed",
            "V",
        ),
        (
            "suspension-mutation",
            "source-suspend-advisory",
            "mutation",
            "observe()",
        ),
        (
            "suspension-drop",
            "source-suspend-advisory",
            "drop-body",
            "observe()",
        ),
    ] {
        let source = format!(
            "{}\n#[unsafe(no_mangle)]pub extern \"C\" fn native_observed()->u64{{{expression} as u64}}\n",
            test_source::fixture(&format!("pa31/tests/spec/{cluster}/{fixture}.rs"))?
        );
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let program = lower((report, analysis))?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/legacy-advisory-join-v1/candidate-native")
            .join(name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("source.rs"), source)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
