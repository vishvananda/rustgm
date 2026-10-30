//! Source LLVM Writable suppression does not confer language permissions.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "writable_payments.rs"]
mod payments;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-no-writable/{name}.rs"
    ))?)
}
fn checked(name: &str) -> TestResult<(Report, pipeline::Analysis)> {
    let report = test_source::resolve(&input(name)?, true)?;
    let analysis = pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
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
#[test]
fn real_body_and_closure_targets_preserve_hints_mutation_and_const_execution() -> TestResult {
    for name in [
        "word",
        "with-inline",
        "with-always",
        "with-never",
        "const",
        "unsafe",
        "inherent",
        "default",
        "implementation",
        "extern-body",
        "deny-unused",
        "closure",
        "closure-call",
        "mutation",
    ] {
        let (report, analysis) = checked(name)?;
        if name.starts_with("with-") {
            let (index, _) = function(&report, "f")?;
            let hint = analysis
                .facts
                .inline_hint_for_definition(&report, DefId(u32::try_from(index)?))?
                .ok_or("actual independent inline hint")?;
            let expected = match name {
                "with-always" => crate::sema::inlining::Hint::Always,
                "with-never" => crate::sema::inlining::Hint::Never,
                _ => crate::sema::inlining::Hint::Hint,
            };
            assert_eq!(hint.hint, expected);
        }
        if name == "const" {
            let initial = analysis
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
                        .definition(initial.definition)
                        .ok_or("actual completed V")?
                )?,
                crate::sema::values::View::Scalar(crate::sema::model::Scalar::Integer(7))
            ));
        }
        lower((report, analysis)).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn forms_owner_gates_and_const_unsafe_borrow_boundaries_are_independent() -> TestResult {
    for name in [
        "required",
        "foreign",
        "struct",
        "constant",
        "missing-gate",
        "duplicate",
        "duplicate-allowed",
        "empty-list",
        "list",
        "value",
        "unsafe-wrapper",
        "unsafe-denied",
        "nonconst-denied",
        "borrow-denied",
        "closure-duplicate",
        "closure-missing-expression-gate",
        "closure-unsafe-denied",
    ] {
        let outcome = checked(name).and_then(lower);
        let error = outcome
            .err()
            .ok_or_else(|| format!("{name}: invalid contract accepted"))?;
        let text = error.to_string();
        let expected: &[&str] = match name {
            "unsafe-denied" | "closure-unsafe-denied" => &["unsafe"],
            "nonconst-denied" => &["const"],
            "borrow-denied" => &["borrow", "mutable", "immutable", "shared"],
            "closure-missing-expression-gate" => &["stmt_expr_attributes"],
            _ => &["rustc_no_writable"],
        };
        assert!(
            expected.iter().any(|part| text.contains(part)),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn configured_removal_macro_allowance_and_distinct_optimization_axes_stay_real() -> TestResult {
    for name in [
        "cfg-removed-duplicate",
        "cfg-removed-bad-target",
        "macro-allowance",
        "same-owner-distinct",
        "with-mir",
    ] {
        lower(checked(name)?)?;
    }
    for (name, word) in [
        ("mixed-duplicate-mir", "rustc_no_mir_inline"),
        ("mixed-duplicate-writable", "rustc_no_writable"),
    ] {
        let error = checked(name)
            .err()
            .ok_or("duplicate independent marker accepted")?;
        assert!(error.to_string().contains(word), "{error}");
    }
    Ok(())
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
        .ok_or_else(|| "actual source function".into())
}
#[test]
fn current_module_imports_keep_real_hints_without_inventing_an_optimizer_permission() -> TestResult
{
    let artifact = crate::module_packets::exported_profile(
        &input("module-provider")?,
        crate::lower::instances::SourcePublication::ModuleCallables,
        "writable_boundary",
    )?;
    let bytes = crate::support::crates::encode(&artifact)?;
    let decoded = crate::support::crates::decode(&bytes)?;
    assert_eq!(crate::support::crates::encode(&decoded)?, bytes);
    let (report, analysis, _) = crate::source_packets::client_analysis(
        decoded,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observed()->u32{base::marked_hint(7)}",
    )?;
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported catalog")?;
    let id = imported
        .definitions()
        .find_map(|(external, id)| {
            external
                .path()
                .last()
                .is_some_and(|name| name == "marked_hint")
                .then_some(id)
        })
        .ok_or("actual imported marked function")?;
    assert_eq!(
        analysis
            .facts
            .inline_hint_for_definition(&report, id)?
            .ok_or("independent hint")?
            .hint,
        crate::sema::inlining::Hint::Always
    );
    Ok(())
}
#[test]
fn actual_mutating_source_body_produces_native_qualification_input() -> TestResult {
    let source = input("native-body")?;
    let program = lower(checked("native-body")?)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-no-writable-v1/candidate");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
