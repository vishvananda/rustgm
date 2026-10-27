//! Declaration-only source programs retain real items without execution permission.
use rustgm::{
    expand::{config::Config, marks::Transparency},
    resolve::{
        self,
        expansion::{Body, Report},
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const POSITIVE: &[&str] = &[
    "unused-concise",
    "unused-braced",
    "forward-alias",
    "restricted-scope",
    "module-identities",
    "shadow-builtin-unused",
    "cfg-disabled-no-gate",
    "unused-transparent",
    "unused-semiopaque",
    "unused-opaque",
    "captured-generated-names",
];
const INVOKED: &[&str] = &[
    "invoke-direct",
    "invoke-qualified",
    "invoke-alias",
    "invoke-forward",
    "invoke-shadow-builtin",
    "invoke-nested-legacy",
    "invoke-eager",
];
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn source(name: &str) -> TestResult<String> {
    Ok(super::fixture(&format!("declaration-{name}"))?)
}
fn expanded(source: &str) -> TestResult<Report> {
    expanded_with_context(source, &resolve::bindings::Context::default())
}
fn expanded_with_context(source: &str, context: &resolve::bindings::Context) -> TestResult<Report> {
    Ok(resolve::expansion::analyze_with_context(
        InputFile {
            identity: "declaration.rs".into(),
            path: "declaration.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        context,
    )?)
}
fn checked(source: &str) -> TestResult<Report> {
    Ok(resolve::analyze_expanded(expanded(source)?)?)
}
pub(super) fn lower(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = checked(source)?;
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
fn unused_scoped_source_items_keep_context_visibility_and_validate_without_executing() -> TestResult
{
    for name in POSITIVE {
        let source = source(name)?;
        let report = checked(&source).map_err(|error| format!("{name}: {error}"))?;
        let mut rendered = Vec::new();
        resolve::render::expanded(&report, &mut rendered)?;
        let rendered = String::from_utf8(rendered)?;
        assert_eq!(
            rendered.contains(" scoped="),
            *name != "cfg-disabled-no-gate",
            "{name}"
        );
        assert!(
            rendered
                .lines()
                .filter(|line| line.contains(" scoped="))
                .all(|line| line.contains(" rules=") && !line.contains(" execution=unsupported"))
        );
        let modes = report
            .definitions
            .iter()
            .filter_map(|definition| {
                if let Body::Scoped { mode, .. } = definition.body {
                    Some(mode)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if *name == "cfg-disabled-no-gate" {
            assert!(modes.is_empty());
        } else {
            assert!(!modes.is_empty(), "{name}");
            let expected = match *name {
                "unused-transparent" => Transparency::Transparent,
                "unused-semiopaque" => Transparency::SemiOpaque,
                _ => Transparency::Opaque,
            };
            assert!(modes.iter().all(|mode| *mode == expected), "{name}");
        }
        lower(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
fn unused_unknown_template_substitution_is_retained_as_upstream_declaration_data() -> TestResult {
    // The original negative hypothesis was falsified by both pinned compile modes.
    // This does not promise that invoking an unbound substitution is supported.
    let source = source("unbound-variable")?;
    checked(&source)?;
    lower(&source)?;
    Ok(())
}

#[test]
fn malformed_scoped_declarations_reject_at_their_actual_language_boundary() -> TestResult {
    for (name, diagnostic) in [
        ("missing-gate", "scoped macros require decl_macro"),
        (
            "wrong-separator",
            "scoped macro rules require comma separation",
        ),
        ("duplicate", "duplicate macro name"),
        ("private-import", "private import or widened visibility"),
        (
            "nonancestor-restriction",
            "visibility must name an ancestor module",
        ),
        ("unknown-transparency", "unknown macro transparency mode"),
        (
            "duplicate-transparency",
            "multiple macro transparency attributes",
        ),
        (
            "transparency-missing-gate",
            "macro transparency requires rustc_attrs",
        ),
        (
            "transparency-wrong-target",
            "macro transparency requires a macro declaration",
        ),
    ] {
        let error = checked(&source(name)?)
            .err()
            .ok_or("invalid declaration accepted")?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn original_assert_matches_is_a_real_declaration_with_an_explicit_stability_frontier() -> TestResult
{
    let original = source("original-assert-matches")?;
    let ordinary = expanded(&original)?;
    assert!(!ordinary.package.uses_source_bindings());
    assert!(ordinary.package.source_core().is_none());
    assert!(ordinary.package.source_client().is_none());
    let identity = rustgm::support::crates::CrateIdentity::new("minicore", "scoped_header")?;
    let context = resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: rustgm::support::bindings::Mode::Source { provider: identity },
        ..Default::default()
    };
    let provider = expanded_with_context(&source("source-assert-matches")?, &context)?;
    assert!(provider.package.source_core().is_some());
    for report in [ordinary, provider] {
        let definition = report
            .definitions
            .iter()
            .find(|definition| report.package.names.get(definition.name) == "assert_matches")
            .ok_or("original attributed declaration was not published")?;
        assert!(definition.node.is_some());
        assert!(matches!(
            definition.body,
            Body::Scoped {
                mode: Transparency::SemiOpaque,
                ..
            }
        ));
        assert!(report.invocations.is_empty());
        let report = resolve::analyze_expanded(report)?;
        crate::provider_traits::require_macro_policies(&report)?;
        rustgm::pipeline::analyze_source(&report)?;
    }
    Ok(())
}

#[test]
fn scoped_source_invocations_execute_and_builtin_claims_keep_their_boundary() -> TestResult {
    for name in INVOKED {
        lower(&source(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in ["claimed-cfg-select", "unknown-builtin-claim"] {
        let error = expanded(&source(name)?)
            .err()
            .ok_or("scoped builtin stub accepted")?;
        assert!(
            error
                .to_string()
                .contains("scoped builtin macro declarations are not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn unused_scoped_source_items_preserve_their_independently_declared_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/scoped-declarations-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer =
        std::fs::read_to_string(root.join("pa31/tests/spec/source-panic-macros/consumer.c"))?;
    for name in POSITIVE
        .iter()
        .copied()
        .chain(["unbound-variable"])
        .chain(INVOKED.iter().copied())
    {
        let program = lower(&source(name)?)?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer, "7\n")?;
    }
    Ok(())
}
