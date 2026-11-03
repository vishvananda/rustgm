//! Real wrapper identities affect ordinary source typing and actual cleanup.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-wrappers/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, crate::pipeline::Analysis)> {
    let capture = crate::sema::copy_trace::Capture::new()?;
    let report = crate::test_source::source_provider(source, "source_wrappers")?;
    let analysis = crate::pipeline::analyze_source(&report).inspect_err(|_| {
        if let Ok(rows) = capture.rows() {
            for row in rows {
                eprintln!("wrapper proof: {row}");
            }
        }
    })?;
    Ok((report, analysis))
}
#[test]
fn source_wrapper_targets_and_generic_arity_match_the_actual_declarations() -> TestResult {
    for (name, accepted) in [
        ("manual-header", true),
        ("maybe-header", true),
        ("manual-no-repr", true),
        ("manual-lifetime", true),
        ("manual-const", true),
        ("manual-two-fields", true),
        ("maybe-two-fields", true),
        ("manual-zero", false),
        ("manual-two-generics", false),
        ("maybe-zero", false),
        ("maybe-two-generics", false),
        ("manual-wrong-target", false),
        ("maybe-wrong-target", false),
        ("manual-duplicate", false),
        ("maybe-duplicate", false),
        ("combined-roles", false),
    ] {
        let source = fixture(name)?;
        let result = analyze(&source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|e| e.to_string())
        );
    }
    Ok(())
}
#[test]
fn source_wrappers_preserve_actual_constructors_drop_policy_and_ordinary_lookalikes() -> TestResult
{
    for name in [
        "drop-pair",
        "manual-own-destructor",
        "const-layout",
        "manual-move-out",
    ] {
        let (
            report,
            crate::pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = analyze(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
        assert!(
            crate::support::library_types::LibraryType::ALL
                .into_iter()
                .all(|kind| report.resolution.library_types.get(kind).is_none())
        );
        let manual = facts
            .source_bindings
            .as_deref()
            .ok_or("source")?
            .validated_language(LangItem::ManuallyDrop)?
            .ok_or("actual manual role")?;
        assert_eq!(
            facts.aggregates.destruction(ItemId(manual.definition.0)),
            Some(FieldDestruction::Never)
        );
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/source-wrappers-v1/candidate-v1");
        std::fs::create_dir_all(&out)?;
        let mut bytes = Vec::new();
        crate::lowir::writer::write(&program, &mut bytes)?;
        std::fs::write(out.join(format!("{name}.lowir")), bytes)?;
    }
    Ok(())
}

#[test]
fn source_dangling_permission_is_typed_scoped_and_never_scalar_or_borrow_authority() -> TestResult {
    for (name, accepted) in [
        ("maybe-valid-dangling", true),
        ("manual-nested-valid-dangling", true),
        ("maybe-null", false),
        ("maybe-unaligned", false),
        ("ordinary-dangling", false),
        ("manual-alone-dangling", false),
        ("lookalike-dangling", false),
        ("maybe-invalid-bool", false),
        ("manual-invalid-bool", false),
        ("maybe-extracted-dangling", false),
        ("maybe-sibling-memo", false),
        ("maybe-deref", false),
        ("maybe-borrow-escape", false),
    ] {
        let source = fixture(name)?;
        let result = analyze(&source).and_then(|(report, analysis)| {
            let crate::pipeline::Analysis {
                mut facts,
                prepared,
            } = analysis;
            crate::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok((report, facts))
        });
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.as_ref().err().map(ToString::to_string)
        );
        if let Ok((report, facts)) = result {
            let index = report
                .resolution
                .definitions
                .iter()
                .position(|d| report.package.names.get(d.name) == "VALUE")
                .ok_or("actual constant")?;
            let reader = crate::sema::values::Reader::new(&facts);
            let mut value = reader
                .definition(crate::resolve::model::DefId(u32::try_from(index)?))
                .ok_or("computed constant")?;
            for _ in 0..if name == "manual-nested-valid-dangling" {
                2
            } else {
                1
            } {
                value = reader.field(value, 0)?;
            }
            assert!(matches!(
                reader.view(value)?,
                crate::sema::values::View::Address { bits: 1 }
            ));
            assert!(value.location().is_none());
            assert!(value.write_permission().is_none());
        }
    }
    Ok(())
}

#[test]
fn source_maybe_uninit_invalid_values_fail_for_their_actual_const_rules() -> TestResult {
    for (name, reason) in [
        ("invalid-bool", "invalid bool"),
        ("null-reference", "null"),
        (
            "wrong-size-transmute",
            "constant execution reached source panic_display",
        ),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-maybe-uninit/{name}.rs"))?;
        let report = crate::test_source::source_provider(&source, "source_wrappers")?;
        let error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid value accepted")?
            .to_string();
        assert!(error.contains(reason), "{name}: {error}");
    }
    Ok(())
}
