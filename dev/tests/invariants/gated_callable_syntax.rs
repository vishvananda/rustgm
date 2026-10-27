//! Const syntax preserves ordinary closure identity and its own feature rule.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        package::Error,
    },
    support::{edition::Edition, source::SourceFile},
    syntax::{
        ast::{Kind, Syntax},
        expression, gate_sites,
        interpolation::EMPTY_INPUTS,
        transfer,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, edition: Edition) -> std::result::Result<Report, Error> {
    resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "const-closure.rs".into(),
            path: "const-closure.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(Error::unlocated)?,
        },
        &mut NoFiles,
        &Config::in_edition(edition),
    )?)
}
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-const-closures/{name}.rs"
    ))?)
}
#[test]
fn grammar_keeps_the_ordinary_closure_and_const_blocks_distinct() -> Result {
    for source in [
        "const || 7",
        "const move |value: u8| value",
        "const |value| -> u8 {value}",
    ] {
        let ast = expression::parse(source).map_err(|error| error.message)?;
        let Kind::ConstClosure { closure, gate } = ast.node(ast.root).kind else {
            return Err("missing const wrapper".into());
        };
        assert!(matches!(
            ast.node(closure).kind,
            Kind::Closure {
                asynchronous: false,
                ..
            }
        ));
        let entry = ast.expression_gate(gate).ok_or("actual const gate")?;
        assert_eq!(
            entry.validate(&ast, gate).map_err(|error| error.message)?,
            closure
        );
        assert_eq!(
            &source[entry.keyword.start as usize..entry.keyword.end as usize],
            "const"
        );
        assert!(entry.context_key().is_none());
        let pretty =
            expression::pretty::expression(source, &ast, ast.root, &EMPTY_INPUTS, &|_, _| Ok(()))
                .map_err(|error| error.message)?;
        let reparsed = expression::parse(&pretty).map_err(|error| error.message)?;
        assert!(matches!(
            reparsed.node(reparsed.root).kind,
            Kind::ConstClosure { .. }
        ));
    }
    for source in ["const {7}", "|| 7", "move |value| value"] {
        let ast = expression::parse(source).map_err(|error| error.message)?;
        assert!(ast.gate_sites.is_none(), "{source}");
    }
    expression::parse("{const || 7;}").map_err(|error| error.message)?;
    for source in ["const 7", "const move {7}", "const |value| -> u8 value"] {
        assert!(expression::parse(source).is_err(), "{source}");
    }
    Ok(())
}
#[test]
fn real_keyword_authority_and_cfg_removed_hard_gate_match_pinned_rust() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in ["enabled", "literal-allowance", "forwarded-body"] {
            report(&fixture(name)?, edition).map_err(|error| format!("{name}: {error}"))?;
        }
        for name in [
            "missing-feature",
            "forwarded-keyword",
            "forwarded-expression",
            "inactive",
        ] {
            let error = report(&fixture(name)?, edition)
                .err()
                .ok_or_else(|| format!("{name} gained feature authority"))?;
            assert!(error.message.contains("const_closures"), "{name}: {error}");
        }
        assert!(report(&fixture("inactive-malformed")?, edition).is_err());
        let enabled = report(&fixture("enabled")?, edition)?;
        let error = crate::pipeline::analyze_source(&enabled)
            .err()
            .ok_or("syntax granted execution")?;
        assert!(
            error.message.contains("const closure callable semantics"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn transfer_failure_and_replay_keep_current_gate_edges_and_source() -> Result {
    let ast = expression::parse("const || const || 7").map_err(|error| error.message)?;
    let rejected = ast.expression_gate(1).ok_or("outer gate")?.keyword;
    let mut destination = Syntax::default();
    let mut paid = 0;
    for _ in 0..2 {
        assert!(
            transfer::append(
                &ast,
                &mut destination,
                |span| {
                    if span == rejected {
                        Err(crate::syntax::token_tree::error(
                            span.start,
                            "declared transfer rejection",
                        ))
                    } else {
                        Ok(span)
                    }
                },
                |value| Ok(value.clone())
            )
            .is_err()
        );
        assert!(destination.nodes.is_empty());
        let (rows, _, steps, _) = destination
            .gate_sites
            .as_ref()
            .ok_or("failed attempt debt")?
            .expression_storage();
        assert_eq!(rows, 0);
        assert!(steps > paid);
        paid = steps;
    }
    destination.root = transfer::append(&ast, &mut destination, Ok, |value| Ok(value.clone()))
        .map_err(|error| error.message)?;
    for ordinal in 0..2 {
        destination
            .expression_gate(ordinal)
            .ok_or("replayed gate")?
            .validate(&destination, ordinal)
            .map_err(|error| error.message)?;
    }
    let mut found = report(&fixture("enabled")?, Edition::Rust2024)?;
    let entry = *found.package.units[0]
        .configured
        .ast
        .expression_gate(0)
        .ok_or("actual gate")?;
    found.package.units[0].configured.ast.nodes[entry.node.get() as usize].kind =
        Kind::ConstClosure {
            closure: entry.node,
            gate: 0,
        };
    assert!(super::validate(&found).is_err());
    let mut found = report(&fixture("enabled")?, Edition::Rust2024)?;
    found.package.units[0].source = found.package.units[0].source.physical_copy();
    assert!(super::validate(&found).is_err());
    Ok(())
}
#[test]
fn sparse_const_rows_scale_without_growing_common_records() -> Result {
    use std::fmt::Write;
    let mut previous_steps = 0;
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(const_closures)] fn f(){");
        for _ in 0..width {
            write!(source, "let _=const||7u8;")?;
        }
        source.push('}');
        let found = report(&source, Edition::Rust2024)?;
        let ast = &found.package.units[0].configured.ast;
        let (rows, capacity, steps, bytes) = ast
            .gate_sites
            .as_ref()
            .ok_or("sparse const owner")?
            .expression_storage();
        assert_eq!(rows, width);
        if previous_steps != 0 {
            assert!(steps <= previous_steps * 5);
        }
        previous_steps = steps;
        println!(
            "const closures width={width} rows={rows} capacity={capacity} steps={steps} bytes={bytes} payload={}",
            ast.gate_sites_bytes()
        );
    }
    assert_eq!(size_of::<Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<gate_sites::Expression>(), 32);
    assert_eq!(size_of::<gate_sites::Declarations>(), 88);
    assert_eq!(
        size_of::<Syntax<crate::literal::storage::StoredValue>>(),
        312
    );
    assert_eq!(size_of::<resolve::unit::Unit>(), 688);
    assert_eq!(size_of::<transfer::Extent>(), 104);
    assert_eq!(resolve::unit::Unit::original_layout_bytes(), (160, 44));
    println!(
        "const closure layouts Kind=28 Node=36 Expression=32 GateOwner=88 Syntax=312 Unit=688 Extent=104 Original=160 Saved=44"
    );
    Ok(())
}
