//! Actual token/category work is distinct from selected semantic publication.
use super::*;
use crate::{
    expand::crate_syntax,
    literal::Value,
    syntax::{ast::Kind, item::ItemContext, token_tree::Forest},
};
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn input(source: &str) -> Result<Input> {
    let forest = Forest::parse(source).map_err(|error| error.message)?;
    Ok(Input::from_forest(&forest, forest.roots()).map_err(|error| error.message)?)
}
#[test]
fn actual_first_match_wildcard_and_later_predicates_follow_the_cfg_contract() -> Result {
    for source in [
        "all()=>{7u32} all()=>{9u32} _=>{3u32}",
        "any()=>{9u32} _=>{7u32}",
        "_=>{7u32} all()=>{9u32}",
        "_=>{7u32} _=>{9u32}",
    ] {
        let tokens = input(source)?;
        let config = Config::default();
        let selected = select(
            source,
            &tokens,
            &config,
            MacroContext::Expression,
            &crate::syntax::interpolation::EMPTY_INPUTS,
            &|_, _| Ok(()),
        )
        .map_err(|error| error.message)?;
        let configured = crate_syntax::configure(source, selected.ast, &config)
            .map_err(|error| error.message)?;
        let Kind::Literal(index) = configured.ast.node(configured.ast.root).kind else {
            return Err("selected literal missing".into());
        };
        assert!(
            matches!(&configured.ast.literals[index as usize], Value::Integer(value) if value.magnitude==7)
        );
    }
    for source in [
        "any()=>{7u32}",
        "all()=>{7u32} not()=>{9u32}",
        "_=>{7u32} unknown()=>{9u32}",
    ] {
        let tokens = input(source)?;
        assert!(
            select(
                source,
                &tokens,
                &Config::default(),
                MacroContext::Expression,
                &crate::syntax::interpolation::EMPTY_INPUTS,
                &|_, _| Ok(())
            )
            .is_err()
        );
    }
    Ok(())
}
#[test]
fn all_branches_are_parsed_in_the_real_category_without_semantic_execution() -> Result {
    for (source, grammar) in [
        ("all()=>{7u32} _=>{let =9u32;}", MacroContext::Expression),
        ("all()=>{u32} _=>{1+2}", MacroContext::Type),
        (
            "all()=>{fn valid(){}} _=>{fn broken(,){}}",
            MacroContext::Items(ItemContext::Module),
        ),
        ("_=>{7u32} all()=>{let =9u32;}", MacroContext::Expression),
    ] {
        let tokens = input(source)?;
        assert!(
            select(
                source,
                &tokens,
                &Config::default(),
                grammar,
                &crate::syntax::interpolation::EMPTY_INPUTS,
                &|_, _| Ok(())
            )
            .is_err(),
            "{source}"
        );
    }
    let source = "all()=>{7u32} _=>{missing_macro!()}";
    let tokens = input(source)?;
    select(
        source,
        &tokens,
        &Config::default(),
        MacroContext::Expression,
        &crate::syntax::interpolation::EMPTY_INPUTS,
        &|_, _| Ok(()),
    )
    .map_err(|error| error.message)?;
    Ok(())
}
#[test]
fn empty_expression_rejects_while_empty_statements_keep_their_actual_tail_shape() -> Result {
    let source = "all()=>{}";
    let tokens = input(source)?;
    assert!(
        select(
            source,
            &tokens,
            &Config::default(),
            MacroContext::Expression,
            &crate::syntax::interpolation::EMPTY_INPUTS,
            &|_, _| Ok(())
        )
        .is_err()
    );
    let selected = select(
        source,
        &tokens,
        &Config::default(),
        MacroContext::Statements,
        &crate::syntax::interpolation::EMPTY_INPUTS,
        &|_, _| Ok(()),
    )
    .map_err(|error| error.message)?;
    assert!(
        matches!(selected.ast.node(selected.ast.root).kind,Kind::Expansion{children,tail:None} if children.start==children.end)
    );
    let source = "all()=>7u32, _=>9u32,";
    let tokens = input(source)?;
    let expression = select(
        source,
        &tokens,
        &Config::default(),
        MacroContext::Expression,
        &crate::syntax::interpolation::EMPTY_INPUTS,
        &|_, _| Ok(()),
    )
    .map_err(|error| error.message)?;
    assert_eq!(
        expression.parser_calls, 2,
        "the two framing Expr parses are reused"
    );
    let statements = select(
        source,
        &tokens,
        &Config::default(),
        MacroContext::Statements,
        &crate::syntax::interpolation::EMPTY_INPUTS,
        &|_, _| Ok(()),
    )
    .map_err(|error| error.message)?;
    assert_eq!(
        statements.parser_calls, 4,
        "statement category requires its distinct two grammar checks"
    );
    assert!(matches!(
        statements.ast.node(statements.ast.root).kind,
        Kind::Expansion { tail: Some(_), .. }
    ));
    Ok(())
}
#[test]
fn braced_width_pays_linear_owned_work_and_retains_only_one_checked_ast() -> Result {
    use std::fmt::Write;
    let mut previous = None;
    for width in [8_usize, 32, 128] {
        let mut source = String::new();
        for _ in 1..width {
            source.push_str("any()=>{unknown_name} ");
        }
        source.push_str("_=>{7u32}");
        let tokens = input(&source)?;
        let steps = Cell::new(0_usize);
        let bytes = Cell::new(0_usize);
        let charge = |work, storage| {
            steps.set(steps.get().saturating_add(work));
            bytes.set(bytes.get().saturating_add(storage));
            Ok(())
        };
        let selected = select(
            &source,
            &tokens,
            &Config::default(),
            MacroContext::Expression,
            &crate::syntax::interpolation::EMPTY_INPUTS,
            &charge,
        )
        .map_err(|error| error.message)?;
        assert_eq!(selected.parser_calls, width);
        assert_eq!(selected.ast.nodes.len(), 1);
        if let Some((old_work, old_bytes)) = previous {
            assert!(steps.get() <= old_work * 5);
            assert!(bytes.get() <= old_bytes * 5);
        }
        previous = Some((steps.get(), bytes.get()));
        let mut row = String::new();
        write!(
            row,
            "choice width={width} tokens={} calls={} work={} storage-preflight={} selected-nodes={}",
            tokens.tokens.len(),
            selected.parser_calls,
            steps.get(),
            bytes.get(),
            selected.ast.nodes.len()
        )?;
        eprintln!("{row}");
    }
    Ok(())
}
