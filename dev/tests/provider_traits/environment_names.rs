//! Source macro identities do not request a synthetic Option library by spelling.
use super::{TestResult, report};
use rustgm::resolve::model::DefKind;
use std::path::Path;

fn source(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-macro-paths")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn actual_forward_and_renamed_macros_do_not_create_bootstrap_declarations() -> TestResult {
    for name in [
        "forward-import",
        "forward-alias-call",
        "forward-qualified-call",
        "renamed-user-macro",
    ] {
        let report = report(&source(name)?).map_err(|error| format!("{name}: {error}"))?;
        assert!(report.resolution.builtin_traits.library.is_none(), "{name}");
        assert!(report.resolution.drop_trait.is_none(), "{name}");
        assert!(report.resolution.macro_prelude.is_empty(), "{name}");
        assert!(
            report
                .resolution
                .definitions
                .iter()
                .all(|definition| definition.kind != DefKind::BuiltinMacro),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn absent_source_macros_keep_actual_import_and_expansion_failures() -> TestResult {
    for (name, diagnostic) in [
        ("missing-import", "unresolved import"),
        (
            "missing-qualified-call",
            "unresolved macro is not in textual scope or path scope",
        ),
    ] {
        let error = report(&source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: missing macro was accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}
