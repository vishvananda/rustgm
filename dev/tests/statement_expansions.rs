//! Raw statement expansion boundaries preserve forced and opaque expressions.
use rustgm::{
    expand::config::Config,
    resolve::{
        expansion,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa8/tests/spec/statement-forwarding";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

fn analyze(name: &str) -> Result<expansion::Report> {
    let source = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(PACKET)
            .join(format!("{name}.rs")),
    )?;
    Ok(expansion::analyze(
        InputFile {
            identity: "statement.rs".into(),
            path: "statement.rs".into(),
            source: SourceFile::new(source)?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}

#[test]
fn raw_nested_calls_keep_their_statement_expansion_boundary() -> Result {
    for name in [
        "recursive-cfg",
        "bracket-call",
        "local-items",
        "empty-output",
        "brace-block-tail",
        "final-value",
    ] {
        let report = analyze(name).map_err(|error| format!("{name}: {error}"))?;
        assert!(report.invocations.len() >= 2, "{name}");
    }
    Ok(())
}

#[test]
fn forced_and_opaque_expressions_keep_their_expression_grammar() -> Result {
    for name in [
        "initializer-rejected",
        "parenthesized-rejected",
        "ordinary-tail-rejected",
        "opaque-expression-rejected",
    ] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name}: statement output entered expression grammar"))?;
        assert!(
            error
                .to_string()
                .contains("unexpected token after macro expansion"),
            "{name}: {error}"
        );
    }
    Ok(())
}
