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
    Ok(resolve::expansion::analyze(
        InputFile {
            identity: "declaration.rs".into(),
            path: "declaration.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}
fn checked(source: &str) -> TestResult<Report> {
    Ok(resolve::analyze_expanded(expanded(source)?)?)
}
fn lower(source: &str) -> TestResult<rustgm::lowir::program::Program> {
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
        let modes = report
            .definitions
            .iter()
            .filter_map(|definition| {
                if let Body::ScopedDeclaration(mode) = definition.body {
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
        ("private-import", "private macro name"),
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
    let report = expanded(&source("original-assert-matches")?)?;
    let definition = report
        .definitions
        .iter()
        .find(|definition| report.package.names.get(definition.name) == "assert_matches")
        .ok_or("original attributed declaration was not published")?;
    assert!(definition.node.is_some());
    assert!(matches!(
        definition.body,
        Body::ScopedDeclaration(Transparency::SemiOpaque)
    ));
    assert!(report.invocations.is_empty());
    let error = resolve::analyze_expanded(report)
        .err()
        .ok_or("scoped stability contract silently accepted")?;
    assert!(
        error
            .message
            .contains("scoped macro library stability declarations are not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
fn scoped_source_invocations_and_builtin_claims_keep_their_exact_unsupported_boundary() -> TestResult
{
    for name in [
        "direct",
        "qualified",
        "alias",
        "forward",
        "shadow-builtin",
        "nested-legacy",
        "eager",
    ] {
        let error = checked(&source(&format!("invoke-{name}"))?)
            .err()
            .ok_or("scoped invocation accepted")?;
        assert!(
            error
                .to_string()
                .contains("scoped macro invocation is not implemented"),
            "{name}: {error}"
        );
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
    for name in POSITIVE {
        let program = lower(&source(name)?)?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer, "7\n")?;
    }
    Ok(())
}
