//! Independent source cases preserve actual marker identity, child proofs and ownership.
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[test]
fn source_no_drop_matches_actual_union_rules_without_copy_or_borrow_grants() -> TestResult {
    for (name, accepted) in [
        ("header", true),
        ("header-unsafe", true),
        ("header-constant", true),
        ("header-members", true),
        ("header-auto", true),
        ("fallback-long-tuple", false),
        ("fallback-copy-tuple", true),
        ("marker-long-tuple", true),
        ("fallback-scalar-array", true),
        ("marker-empty-array", false),
        ("wrong-target", false),
        ("generic-type", false),
        ("generic-lifetime", false),
        ("generic-const", false),
        ("duplicate-role", false),
        ("manual-union", true),
        ("mutable-reference-union", true),
        ("nested-components", true),
        ("zero-array-noncopy", false),
        ("ordinary-nondrop", false),
        ("copy-premise", true),
        ("manual-premise", true),
        ("reference-premise", true),
        ("nominal-copy", true),
        ("manual-lookalike", false),
        ("role-lookalike", false),
        ("explicit-impl", false),
        ("fallback-manual", false),
        ("fallback-mutable-reference", false),
        ("fallback-shared-reference", true),
        ("fallback-scalar-without-copy", true),
        ("bound-scalar-without-copy", false),
        ("borrow-escape", false),
        ("no-copy-grant", false),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-no-drop/{name}.rs"))?;
        let result = prove_source(&source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|e| e.to_string())
        );
    }
    Ok(())
}
fn prove_source(source: &str) -> TestResult<crate::lowir::program::Program> {
    let capture = crate::sema::copy_trace::Capture::new()?;
    let report = crate::test_source::source_provider(source, "source_no_drop")?;
    let analysis = crate::pipeline::analyze_source(&report).inspect_err(|_| trace(&capture))?;
    let crate::pipeline::Analysis {
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
fn actual_no_drop_relay_lowers_real_mutation_and_owned_source_fields() -> TestResult {
    let source = crate::test_source::fixture("pa31/tests/spec/source-no-drop/native-relay.rs")?;
    let program = prove_source(&source)?;
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-no-drop-v1/candidate-v1");
    std::fs::create_dir_all(&out)?;
    let mut bytes = Vec::new();
    crate::lowir::writer::write(&program, &mut bytes)?;
    std::fs::write(out.join("native-relay.lowir"), bytes)?;
    Ok(())
}
#[test]
fn source_union_admission_shares_repeated_type_proofs_and_trivial_copy_dependencies() -> TestResult
{
    for (fixture, generic, field, unique) in [
        ("manual-union", "<T>", "Hold<T>", 1),
        ("fallback-scalar-array", "", "[u16;16]", 1),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-no-drop/{fixture}.rs"))?;
        let prefix = source
            .split_once("union U")
            .ok_or("source union declaration")?
            .0;
        for width in [8, 32, 128] {
            let mut fields = String::new();
            for index in 0..width {
                write!(&mut fields, "f{index}:{field},")?;
            }
            let source = format!("{prefix}union U{generic}{{{fields}}}");
            let report = crate::test_source::source_provider(&source, "source_no_drop")?;
            let capture = crate::sema::copy_trace::Capture::new()?;
            let analysis =
                crate::pipeline::analyze_source(&report).inspect_err(|_| trace(&capture))?;
            let work = analysis.facts.union_work;
            assert_eq!(work.fields, width);
            assert_eq!(work.unique_types, unique);
            assert_eq!(work.steps, width, "width={width}");
            eprintln!(
                "source union {fixture}: width={width}, steps={}, unique={}",
                work.steps, work.unique_types
            );
        }
    }
    Ok(())
}

fn trace(capture: &crate::sema::copy_trace::Capture) {
    if let Ok(rows) = capture.rows() {
        for row in rows {
            eprintln!("source NoDrop proof: {row}");
        }
    }
}

#[test]
fn source_union_shape_facts_are_shared_across_declaration_count_and_depth() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-no-drop/fallback-scalar-array.rs")?;
    let prefix = source.split_once("union U").ok_or("source prefix")?.0;
    for depth in [8, 32, 64] {
        for count in [8, 32, 64] {
            let mut source = format!("{prefix}type A0=u16;");
            for index in 1..=depth {
                write!(&mut source, "type A{index}=[A{};1];", index - 1)?;
            }
            for index in 0..count {
                write!(&mut source, "union U{index}{{field:A{depth}}}")?;
            }
            let report = crate::test_source::source_provider(&source, "source_no_drop")?;
            let analysis = crate::pipeline::analyze_source(&report)?;
            assert_eq!(analysis.facts.union_work.fields, count);
            assert_eq!(analysis.facts.union_work.steps, count);
            assert_eq!(analysis.facts.union_work.unique_types, count);
            eprintln!(
                "shared union shape depth={depth}, declarations={count}, steps={}",
                analysis.facts.union_work.steps
            );
        }
    }
    Ok(())
}
