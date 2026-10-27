//! Opaque const arguments retain their typed grammar and ordinary proof path.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{edition::Edition, hash::HashSet, source::SourceFile},
    syntax::ast::Kind,
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa18/tests/spec/forwarded-consts/{name}.rs"
    ))?)
}
fn report(source: &str, edition: Edition) -> Result<Report> {
    let mut config = Config::default();
    config.set_edition(edition);
    Ok(resolve::expansion::analyze(
        InputFile {
            identity: "forwarded.rs".into(),
            path: "forwarded.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &config,
    )?)
}
fn compile(source: &str, edition: Edition) -> Result {
    let mut report = resolve::analyze_expanded(report(source, edition)?)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    let program = crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    Ok(())
}
#[test]
fn forwarded_scalars_and_blocks_follow_both_editions_const_grammar() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "integer-repetition",
            "bool-literals",
            "char-literal",
            "signed-literal",
            "forwarded-twice",
            "braced-arithmetic",
            "opaque-block-expression",
            "opaque-expression-literal",
            "opaque-expression-signed",
            "opaque-block-fragment",
            "opaque-unsafe-block",
        ] {
            compile(&source(name)?, edition)
                .map_err(|error| format!("{edition:?}/{name}: {error}"))?;
        }
    }
    Ok(())
}
#[test]
fn wrong_const_values_and_type_positions_remain_checked() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "wrong-const-type",
            "negative-usize",
            "missing-separator",
            "unbraced-arithmetic",
            "literal-in-type-context",
            "opaque-expression",
            "opaque-runtime-expression",
            "opaque-expression-type-context",
        ] {
            let error = compile(&source(name)?, edition)
                .err()
                .ok_or("invalid const argument accepted")?;
            let text = error.to_string();
            if matches!(
                name,
                "wrong-const-type" | "negative-usize" | "opaque-expression-type-context"
            ) {
                assert!(!text.contains("expected path segment"), "{name}: {text}");
            }
        }
    }
    Ok(())
}
#[test]
fn opaque_argument_imports_reuse_capture_and_grow_with_occurrences() -> Result {
    use std::fmt::Write as _;
    for width in [1_usize, 8, 32, 128] {
        let mut input = String::from(
            "fn chunk<const N:usize>()->usize{N}macro_rules! many{($n:literal)=>{fn main(){",
        );
        for _ in 0..width {
            write!(input, "let _=chunk::<$n>();")?;
        }
        input.push_str("}}}many!(7);");
        let actual = report(&input, Edition::Rust2024)?;
        assert_eq!(actual.invocations.len(), 1);
        assert_eq!(actual.invocations[0].parser_calls, 1);
        assert_eq!(actual.captures.len(), 1);
        let ast = &actual.package.units[0].configured.ast;
        let mut arguments = HashSet::default();
        for node in &ast.nodes {
            if let Kind::Applied {
                arguments: range, ..
            } = node.kind
            {
                for &argument in ast.list(range) {
                    if matches!(
                        ast.node(argument).kind,
                        Kind::Literal(_) | Kind::DeferredLiteral(_)
                    ) {
                        assert!(arguments.insert(argument));
                    }
                }
            }
        }
        assert_eq!(arguments.len(), width);
        assert!(ast.nodes.len() <= 128 + 32 * width);
        eprintln!(
            "const fragment occurrences={width} capture-parses=1 distinct-arguments={} syntax-nodes={}",
            arguments.len(),
            ast.nodes.len()
        );
    }
    Ok(())
}
#[test]
fn retained_core_swap_prefix_uses_the_actual_expansion_parser() -> Result {
    let input =
        crate::test_source::fixture("pa31/tests/spec/forwarded-literal-consts/swap-prefix.rs")?;
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let actual = report(&input, edition)?;
        assert_eq!(actual.invocations.len(), 1);
        assert_eq!(actual.invocations[0].parser_calls, 3);
        assert_eq!(actual.captures.len(), 3);
        let ast = &actual.package.units[0].configured.ast;
        let applications = ast
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, Kind::Applied { .. }))
            .count();
        assert_eq!(applications, 3);
    }
    Ok(())
}
