//! Actual cold body targets preserve ordinary contexts, source bodies and lint policy.
use rustgm::{pipeline, resolve::expansion::Report};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/source-cold";
fn input(name: &str) -> std::io::Result<String> {
    crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))
}
fn full(report: &Report) -> std::result::Result<(), String> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
#[test]
fn ordinary_body_capture_and_const_function_controls_lower_completely() -> Result {
    for name in [
        "closure",
        "statement-no-feature",
        "tail-no-feature",
        "macro-allowed",
        "many-no-feature",
        "capture",
        "unsafe-context",
        "function",
        "duplicate-allow",
        "wrong-target-allow",
    ] {
        let report = crate::resolve::resolve(&input(name)?, true)?;
        full(&report).map_err(|e| format!("{name}: {e}"))?;
    }
    let report = crate::resolve::source_provider(&input("source-native")?, "source_cold")?;
    full(&report)?;
    Ok(())
}
#[test]
fn cold_never_pays_feature_safety_move_lifetime_const_or_attribute_obligations() -> Result {
    let mut failures = Vec::new();
    for (name, expected) in [
        ("missing-expression-feature", "stmt_expr_attributes"),
        ("macro-forwarded", "stmt_expr_attributes"),
        ("many-with-illegal", "stmt_expr_attributes"),
        ("wrong-target-deny", "cold attribute has no effect"),
        ("duplicate-deny", "duplicate cold attribute"),
        ("unsafe-body", "unsafe"),
        ("move-borrowed", "move"),
        ("lifetime", "lifetime"),
        ("nonconst", "const"),
        ("malformed", "cold requires a plain word"),
        ("unsafe-wrapper", "cold requires a plain word"),
    ] {
        let input = input(name)?;
        let actual = crate::resolve::resolve(&input, true).and_then(|r| full(&r));
        match actual {
            Ok(()) => failures.push(format!("accepted {name}")),
            Err(error) if !error.contains(expected) || error.contains("not implemented") => {
                failures.push(format!("{name}: {error}"));
            }
            Err(error) => eprintln!("{name}: {error}"),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn const_target_has_a_checked_body_and_async_headers_keep_their_execution_requirements() -> Result {
    let report = crate::resolve::resolve(&input("const-closure")?, true)?;
    full(&report)?;
    for (name, required) in [
        (
            "async-block",
            "body construct requires later semantic implementation",
        ),
        ("async-closure", "async closure bodies are not implemented"),
    ] {
        let report = crate::resolve::resolve(&input(name)?, true)?;
        rustgm::sema::declarations::analyze(&report).map_err(|e| format!("{name}: {e}"))?;
        if let Err(error) = full(&report) {
            assert!(
                error.contains(required),
                "unrelated body failure {name}: {error}"
            );
            eprintln!("required body profile {name}: {error}");
        }
    }
    for name in ["prototype-required", "foreign-required"] {
        let report = crate::resolve::resolve(&input(name)?, true)?;
        full(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn actual_source_cold_closure_keeps_runtime_captures_and_control_flow() -> Result {
    let report = crate::resolve::source_provider(&input("source-native")?, "source_cold")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/source-transport/source-cold/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "source-cold",
        &program,
        &crate::resolve::fixture(&format!("{ROOT}/observer.c"))?,
        "cold 23 71\n",
    )?;
    Ok(())
}
