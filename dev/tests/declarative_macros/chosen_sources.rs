//! The frozen upstream corpus supplies actual source-role and fragment contracts.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn root() -> Result<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "missing root".into())
}
const POSITIVE: &[&str] = &[
    "first-match",
    "fallback",
    "unbraced",
    "braced-commas",
    "nested-predicates",
    "wildcard-first",
    "repeated-wildcard",
    "item-position",
    "empty-items",
    "type-position",
    "impl-position",
    "trait-position",
    "trait-impl-position",
    "foreign-position",
    "import-alias",
    "qualified-path",
    "configured-on",
    "configured-off",
    "authored-same-spelling",
    "legacy-builtin-shape",
    "width-8",
    "width-32",
    "width-128",
    "depth-8",
    "depth-16",
    "depth-32",
    "statement-complete-provider",
    "pattern-complete-provider",
    "captured-complete-provider",
    "empty-unit-expression",
    "opaque-selected",
    "opaque-unselected",
    "empty-statement-tail",
    "empty-statement-semicolon",
    "empty-unselected-statement-tail",
    "item-statement-tail",
    "selected-statement-tail-value",
    "tokens-empty-statement-semicolon",
    "tokens-empty-brace-statement-tail",
];
fn source(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        root()?
            .join("pa31/tests/spec/cfg-select")
            .join(format!("{name}.rs")),
    )?)
}
fn context() -> Result<resolve::bindings::Context> {
    let identity = rustgm::support::crates::CrateIdentity::new("minicore", "cfg_selected")?;
    Ok(resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: rustgm::support::bindings::Mode::Source { provider: identity },
        ..Default::default()
    })
}
fn expanded(name: &str, text: &str) -> Result<Report> {
    let mut config = Config::default();
    if name == "configured-on" {
        config
            .add("rustgm_selected")
            .map_err(|error| error.message)?;
    }
    Ok(resolve::expansion::analyze_with_context(
        InputFile {
            identity: "chosen.rs".into(),
            path: "chosen.rs".into(),
            source: SourceFile::new(text.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &config,
        &context()?,
    )?)
}
fn lower(name: &str, text: &str) -> Result<rustgm::lowir::program::Program> {
    let report = resolve::analyze_expanded(expanded(name, text)?)?;
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
fn genuine_selected_source_contexts_lower_with_the_independent_contracts() -> Result {
    let mut count = 0;
    for &name in POSITIVE {
        let text = source(name)?;
        lower(name, &text).map_err(|error| format!("{name}: {error}"))?;
        count += 1;
        eprintln!("cfg_select lower accepted {name}");
    }
    assert_eq!(count, 39);
    Ok(())
}
#[test]
fn malformed_unchosen_syntax_and_expression_contexts_reject_at_expansion() -> Result {
    for name in [
        "unselected-expression-syntax",
        "unselected-item-syntax",
        "unselected-type-syntax",
        "unreachable-syntax",
        "empty-let-expression",
        "empty-parenthesized-expression",
        "empty-unselected-expression",
        "opaque-empty-expression-tail",
        "opaque-empty-expression-semicolon",
        "tokens-empty-statement-tail",
    ] {
        let error = expanded(name, &source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: malformed fragment accepted"))?;
        let diagnostic = error.to_string();
        assert!(
            diagnostic.contains("expected") || diagnostic.contains("expression"),
            "{name}: {diagnostic}"
        );
        eprintln!("cfg_select syntax rejected {name}: {diagnostic}");
    }
    let error = lower("empty-expression", &source("empty-expression")?)
        .err()
        .ok_or("unit silently became u32")?;
    assert!(error.to_string().contains("type"), "{error}");
    Ok(())
}
#[test]
fn predicate_and_branch_framing_reject_at_their_actual_boundary() -> Result {
    for (name, message) in [
        ("no-match", "no matching branch"),
        ("empty-input", "no matching branch"),
        ("invalid-predicate-after-match", "not"),
        ("missing-arrow", "requires =>"),
        ("missing-comma", "comma"),
        ("branch-attribute", "attributes"),
        ("branch-doc-comment", "documentation"),
    ] {
        let error = expanded(name, &source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: invalid cfg_select accepted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn original_staged_header_is_a_real_builtin_with_its_separate_stability_frontier() -> Result {
    let report = expanded("original-provider", &source("original-provider")?)?;
    assert!(report.definitions.iter().any(|entry| matches!(
        entry.body,
        resolve::expansion::Body::Builtin(rustgm::expand::builtin::Builtin::CfgSelect)
    )));
    assert_eq!(report.invocations.len(), 1);
    let error = resolve::analyze_expanded(report)
        .err()
        .ok_or("staged macro contract silently admitted")?;
    assert!(error.message.contains("stability"), "{error}");
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM backend and host C linker"]
fn genuine_chosen_source_cases_preserve_independently_observed_native_answers() -> Result {
    let folder = root()?
        .join("obj/parallel/pa31/cfg-select-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer =
        std::fs::read_to_string(root()?.join("pa31/tests/spec/source-panic-macros/consumer.c"))?;
    let mut count = 0;
    for &name in POSITIVE {
        let program = lower(name, &source(name)?)?;
        let expected = match name {
            "configured-on" | "authored-same-spelling" => "9\n",
            "selected-statement-tail-value" => "17\n",
            _ => "7\n",
        };
        crate::ffi_program::check(root()?, &folder, name, &program, &consumer, expected)?;
        count += 1;
    }
    assert_eq!(count, 39);
    Ok(())
}
