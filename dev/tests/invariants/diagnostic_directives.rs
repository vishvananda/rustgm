//! Independent capped outcomes preserve actual semantic checks and cold storage.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{diagnostics::LintCap, edition::Edition, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, config: &Config) -> Result<Report> {
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "directives.rs".into(),
            path: "directives.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
#[test]
fn capped_directives_match_independent_edition_outcomes() -> Result {
    let cases = crate::test_source::fixture("pa40/tests/spec/diagnostic-directives/cases-v3.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status] = columns.as_slice() else {
            return Err("malformed diagnostic directive case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            config.set_lint_cap(LintCap::Allow);
            let actual = (|| -> Result {
                let report = report(&source, &config)?;
                let facts = crate::sema::declarations::analyze(&report)?;
                crate::pipeline::analyze(&report, facts)?;
                Ok(())
            })();
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "{name} {edition:?}: {:?}",
                actual.err()
            );
        }
        count += 1;
    }
    assert_eq!(count, 59);
    Ok(())
}
#[test]
fn uncapped_rendering_remains_explicit_and_capped_tables_stay_cold() -> Result {
    for name in ["const-valid", "macro-valid"] {
        let source = crate::test_source::fixture(&format!(
            "pa40/tests/spec/diagnostic-directives/{name}.rs"
        ))?;
        let strict = report(&source, &Config::default())?;
        let Err(error) = crate::sema::declarations::analyze(&strict) else {
            return Err("uncapped directive rendering unexpectedly accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("uncapped const/macro diagnostic")
        );
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        let capped = report(&source, &config)?;
        let facts = crate::sema::declarations::analyze(&capped)?;
        assert!(
            facts
                .must_use_contracts
                .as_deref()
                .and_then(|table| table.diagnostics.as_deref())
                .is_none()
        );
        assert!(facts.require_lint_context(&strict).is_err());
        crate::pipeline::analyze(&capped, facts)?;
    }
    Ok(())
}
#[test]
fn attribute_lexical_validity_is_separate_from_suffix_policy() -> Result {
    use crate::{lex::Lexer, literal};
    let token = |source| -> Result<crate::lex::token::Token> {
        Ok(Lexer::new(source)
            .map_err(|error| error.message)?
            .next_token()
            .map_err(|error| error.message)?
            .ok_or("text token")?)
    };
    for source in [r#""plain"u8"#, r##"r#"raw"#suffix"##, r#""\u{41}"u8"#] {
        let token = token(source)?;
        literal::validate_attribute_text(source, token).map_err(|error| error.message)?;
        assert!(literal::interpret(source, token).is_err());
    }
    for source in [r#""\q"suffix"#, r#""\u{d800}"u8"#, r#"b"\u{41}""#] {
        let token = token(source)?;
        assert!(literal::validate_attribute_text(source, token).is_err());
    }
    Ok(())
}
