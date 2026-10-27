//! Captured cfg values retain literal roots and independently demanded feature uses.
use crate::{
    expand::config::{CfgGate, Config},
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{edition::Edition, source::SourceFile},
    syntax::{
        ast::Kind,
        expression,
        interpolation::{Nonterminal, ParsedSyntax, Splice},
        token_tree::Span,
    },
};
use std::{borrow::Cow, fmt::Write as _, path::Path};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool)] = &[
    ("literal", true),
    ("raw", true),
    ("escaped", true),
    ("expr-literal", true),
    ("expr-group", false),
    ("expression", false),
    ("bytes", false),
    ("integer", false),
    ("boolean", false),
    ("suffix", false),
    ("forwarded", true),
    ("metadata", true),
    ("short-circuit-invalid", false),
    ("disabled-inner-invalid", true),
    ("disabled-predicate-invalid", false),
    ("target-widths", true),
    ("target-bare", true),
    ("target-captured", true),
    ("missing-feature", false),
    ("false-missing-feature", false),
    ("short-circuit-feature", false),
    ("disabled-attr-feature", false),
    ("disabled-inner-feature", true),
    ("removed-first-feature", true),
    ("feature-first-removed", false),
];
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, edition: Edition) -> Result<Report> {
    let config = Config::in_edition(edition);
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "cfg.rs".into(),
            path: "cfg.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &config,
    )?)?)
}
#[test]
fn captured_strings_and_demanded_gates_follow_both_pinned_editions() -> Result {
    for &(name, expected) in CASES {
        let path = format!("pa8/tests/spec/captured-cfg/{name}.rs");
        let input = crate::test_source::fixture(&path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let actual = report(&input, edition);
            assert_eq!(
                actual.is_ok(),
                expected,
                "{path} {edition:?}: {:?}",
                actual.err()
            );
        }
    }
    for (feature, predicate, missing) in [
        ("cfg_target_has_atomic", "ub_checks", "cfg_ub_checks"),
        (
            "cfg_ub_checks",
            "target_has_atomic_load_store=\"128\"",
            "cfg_target_has_atomic",
        ),
    ] {
        let input =
            format!("#![feature({feature})]#[cfg({predicate})]fn omitted(){{}}fn main(){{}}");
        let error = report(&input, Edition::Rust2024)
            .err()
            .ok_or("wrong feature granted permission")?;
        assert!(error.to_string().contains(missing), "{error}");
    }
    Ok(())
}
#[test]
fn stored_and_deferred_strings_borrow_only_their_actual_literal_roots()
-> std::result::Result<(), crate::lex::token::LexError> {
    for (source, expected) in [
        ("\"linux\"", "linux"),
        ("r#\"linux\"#", "linux"),
        ("\"\\x6cinux\"", "linux"),
        ("\"\"", ""),
    ] {
        let parsed = ParsedSyntax::own(source, expression::parse(source)?)?;
        let span = parsed.ast.node(parsed.ast.root).span;
        let splice = Splice {
            fragment: Nonterminal::Syntax {
                kind: expression::fragment::FragmentKind::Literal,
                parsed: &parsed,
            },
            original_source: source,
            original: span,
            destination: Span {
                start: 48,
                end: 48 + span.end,
            },
        };
        let bytes = parsed.literals.decoded_bytes();
        for _ in 0..1000 {
            assert!(matches!(splice.string_literal()?, Cow::Borrowed(value) if value == expected));
        }
        assert_eq!(parsed.literals.decoded_bytes(), bytes);
        assert!(
            Splice {
                original: Span { start: 1, ..span },
                ..splice
            }
            .string_literal()
            .is_err()
        );
        let token = crate::lex::Lexer::new(source)?
            .next_token()?
            .ok_or_else(|| crate::syntax::token_tree::error(0, "literal token"))?;
        let tokens = [token];
        let deferred = expression::fragment::parse(
            source,
            &tokens,
            expression::fragment::FragmentKind::Literal,
        )?;
        let deferred = ParsedSyntax::own(source, deferred.ast)?;
        let splice = Splice {
            fragment: Nonterminal::Syntax {
                kind: expression::fragment::FragmentKind::Literal,
                parsed: &deferred,
            },
            ..splice
        };
        assert_eq!(splice.string_literal()?, expected);
    }
    for source in [
        "8",
        "b\"linux\"",
        "(\"linux\")",
        "{\"linux\"}",
        "concat!(\"lin\",\"ux\")",
    ] {
        let parsed = ParsedSyntax::own(source, expression::parse(source)?)?;
        let span = parsed.ast.node(parsed.ast.root).span;
        assert!(
            Splice {
                fragment: Nonterminal::Syntax {
                    kind: expression::fragment::FragmentKind::Expression,
                    parsed: &parsed
                },
                original_source: source,
                original: span,
                destination: span
            }
            .string_literal()
            .is_err()
        );
    }
    Ok(())
}
#[test]
fn generated_gate_counts_reuse_one_capture_and_keep_real_atomic_impls() -> Result {
    let config = Config::default();
    assert!(config.contains("target_has_atomic_load_store", None));
    for width in ["8", "16", "32", "64", "ptr"] {
        assert!(config.contains("target_has_atomic_load_store", Some(width)));
    }
    assert!(!config.contains("target_has_atomic_load_store", Some("128")));
    for width in [1_usize, 8, 32, 128] {
        let mut input =
            String::from("#![feature(cfg_target_has_atomic)]macro_rules! many{($s:literal)=>{");
        for index in 0..width {
            write!(
                input,
                "#[cfg(target_has_atomic_load_store=$s)]fn f{index}(){{}}"
            )?;
        }
        input.push_str("}}many!(\"8\");fn main(){}");
        let actual = report(&input, Edition::Rust2024)?;
        assert_eq!(actual.invocations[0].parser_calls, 1);
        assert_eq!(actual.captures.len(), 1);
        let unit = &actual.package.units[0];
        let uses = unit.configured.cfg_uses.as_ref().ok_or("gate uses")?;
        assert_eq!(uses.len(), width);
        assert!(
            uses.iter()
                .all(|used| used.predicate.gate == CfgGate::TargetHasAtomic)
        );
        assert!(unit.configured.ast.nodes.len() <= 128 + 32 * width);
        eprintln!(
            "cfg occurrences={width} capture-parses=1 gate-uses={} nodes={} use-bytes={}",
            uses.len(),
            unit.configured.ast.nodes.len(),
            size_of::<crate::expand::configured::ConfiguredUse>() * uses.len()
        );
    }
    let input = crate::test_source::fixture("pa31/tests/spec/captured-cfg/atomic-primitive.rs")?;
    let actual = report(&input, Edition::Rust2024)?;
    let unit = &actual.package.units[0];
    let implementations = unit.configured.ast.nodes.iter().enumerate().filter(|(index,node)| {
        unit.configured.active[*index] && matches!(node.kind, Kind::Item(item) if matches!(unit.configured.ast.items[item as usize].kind, crate::syntax::item::ItemKind::Impl { .. }))
    }).count();
    assert_eq!(implementations, 9);
    assert_eq!(
        unit.configured
            .cfg_uses
            .as_ref()
            .ok_or("actual core gate uses")?
            .len(),
        11
    );
    assert_eq!(actual.invocations.len(), 11);
    Ok(())
}
