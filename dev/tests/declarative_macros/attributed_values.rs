//! Frozen independent source attributes exercise real declarations and explicit boundaries.
use crate::provider_traits::TestResult;
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, edition::Edition, source::SourceFile},
};
use std::path::Path;
const ACCEPT: &[&str] = &[
    "forward-declaration",
    "reverse-declaration",
    "distinct-attribute-aliases",
    "holder-import",
    "forward-import",
    "disabled-unknown-attribute",
    "compiler-doc-lint",
    "unclaimed-same-spelling",
    "declarations-8",
    "imports-8",
    "declarations-32",
    "imports-32",
    "declarations-128",
    "imports-128",
    "predefined-diagnostic-tool",
    "predefined-rustfmt-tool",
    "predefined-clippy-tool",
];
fn root() -> TestResult<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn expanded(name: &str, edition: Edition) -> TestResult<Report> {
    let path = root()?
        .join("pa31/tests/spec/source-attribute-builtins")
        .join(format!("{name}.rs"));
    let identity = CrateIdentity::new("minicore", "actual_attribute_declarations")?;
    let context = resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    Ok(resolve::expansion::analyze_with_context(
        InputFile {
            identity: path.clone(),
            source: SourceFile::new(std::fs::read(&path)?)?,
            path,
        },
        &mut NoFiles,
        &Config::in_edition(edition),
        &context,
    )?)
}
fn checked(name: &str, edition: Edition) -> TestResult<Report> {
    Ok(resolve::analyze_expanded(expanded(name, edition)?)?)
}
fn lower(name: &str, edition: Edition) -> TestResult<rustgm::lowir::program::Program> {
    let report = checked(name, edition)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn authenticated_unused_attribute_declarations_and_predefined_tools_lower_in_both_editions()
-> TestResult {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for &name in ACCEPT {
            lower(name, edition).map_err(|error| format!("{name}/{edition:?}: {error}"))?;
        }
    }
    Ok(())
}
#[test]
fn opposing_block_macro_kinds_select_the_actual_eligible_outer_binding() -> TestResult {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for (name, invocations) in [
            ("block-outer-bang-control", 1),
            ("block-inner-attribute-outer-bang", 1),
            ("block-outer-attribute-control", 0),
            ("block-inner-bang-outer-attribute", 0),
        ] {
            let report = checked(name, edition)?;
            assert_eq!(report.invocations.len(), invocations, "{name}/{edition:?}");
            // A genuine Test removes the function with missing body names;
            // the opposite kind never supplies a substitute invocation frame.
            rustgm::pipeline::analyze_source(&report)
                .map_err(|error| format!("{name}/{edition:?}: {error}"))?;
        }
    }
    Ok(())
}
#[test]
fn actual_attribute_lookup_kind_owner_and_unavailable_execution_remain_distinct() -> TestResult {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for (name, message) in [
            ("wrong-owner", "builtin macro binding requires"),
            ("unknown-role", "builtin"),
            ("duplicate-claim", "builtin"),
            ("unsafe-claim", "unsafe wrapper"),
            ("unrequested-builtin", "rustc_attrs"),
            ("unknown-ordinary-attribute", "unresolved attribute macro"),
            ("unknown-qualified-attribute", "unresolved attribute macro"),
            ("bang-as-attribute", "unresolved attribute macro"),
            ("scoped-bang-as-attribute", "unresolved attribute macro"),
            ("attribute-as-bang", "unresolved macro"),
            ("alias-attribute-as-bang", "unresolved macro"),
            (
                "direct-attribute-empty",
                "builtin attribute execution is not implemented",
            ),
            (
                "qualified-attribute-empty",
                "builtin attribute execution is not implemented",
            ),
            (
                "alias-attribute-empty",
                "builtin attribute execution is not implemented",
            ),
            (
                "generated-captured-path",
                "builtin attribute execution is not implemented",
            ),
            (
                "generated-captured-metadata",
                "builtin attribute execution is not implemented",
            ),
            (
                "generated-literal-attribute",
                "builtin attribute execution is not implemented",
            ),
            (
                "configured-active-attribute",
                "builtin attribute execution is not implemented",
            ),
            ("private-attribute", "private"),
            ("missing-scoped-gate", "decl_macro"),
            ("unknown-tool-namespace", "unresolved attribute macro"),
            ("deceptive-clippy-prefix", "unresolved attribute macro"),
            ("deceptive-diagnostic-prefix", "unresolved attribute macro"),
        ] {
            let error = checked(name, edition)
                .err()
                .ok_or_else(|| format!("{name}: unsupported input accepted"))?;
            assert!(
                error.to_string().contains(message),
                "{name}/{edition:?}: {error}"
            );
        }
    }
    Ok(())
}
#[test]
fn independent_positive_frontiers_keep_stability_legacy_kind_and_tool_registration_explicit()
-> TestResult {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in ["original-autodiff_forward", "original-autodiff_reverse"] {
            let report = checked(name, edition)?;
            crate::provider_traits::require_macro_policies(&report)?;
            rustgm::pipeline::analyze_source(&report)?;
        }
        for (name, message) in [
            ("legacy-attribute-claim", "supported declaration kind"),
            (
                "registered-user-tool",
                "source tool registration is not implemented",
            ),
            (
                "registered-predefined-tool",
                "source tool registration is not implemented",
            ),
        ] {
            let error = checked(name, edition)
                .err()
                .ok_or_else(|| format!("{name}: unsupported frontier ignored"))?;
            assert!(
                error.to_string().contains(message),
                "{name}/{edition:?}: {error}"
            );
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM backend and host C linker"]
fn unused_attribute_declarations_preserve_independent_native_answers_in_both_editions() -> TestResult
{
    let folder = root()?
        .join("obj/parallel/pa31/attribute-builtins-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer = "#include <stdio.h>\nextern unsigned answer(void);int main(void){if(answer()!=7)return 1;puts(\"7\");return 0;}\n";
    for (label, edition) in [("2021", Edition::Rust2021), ("2024", Edition::Rust2024)] {
        for &name in ACCEPT {
            crate::ffi_program::check(
                root()?,
                &folder,
                &format!("{name}-{label}"),
                &lower(name, edition)?,
                consumer,
                "7\n",
            )?;
        }
    }
    Ok(())
}
