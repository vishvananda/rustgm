//! Primary Rust syntax rules and the retained independent Assert-message evidence.
use super::*;
use crate::{
    expand::{builtin::assertion, macros::input::Input, stream::Source},
    support::source::FileId,
    syntax::{
        cursor::TokenPosition,
        expression::{self, fragment},
        interpolation::EMPTY_INPUTS,
        token_tree::Forest,
    },
};
use std::{cell::Cell, fmt::Write};
type TestResult = Result<(), LexError>;
fn write(source: &str) -> Result<String, LexError> {
    let forest = Forest::parse(source)?;
    let input = Input::from_forest(&forest, forest.roots())?;
    let parsed = fragment::parse_with_inputs(
        source,
        &input.tokens,
        fragment::FragmentKind::Expression,
        TokenPosition::default(),
        &EMPTY_INPUTS,
    )?;
    assert_eq!(parsed.position.index, input.tokens.len(), "source={source}");
    assert_eq!(parsed.position.offset, 0, "source={source}");
    expression(
        source,
        &parsed.ast,
        parsed.ast.root,
        &EMPTY_INPUTS,
        &|_, _| Ok(()),
    )
}
#[test]
fn parsed_expression_messages_follow_the_actual_ast_and_forwarded_condition() -> TestResult {
    for (source, expected) in [
        ("2usize==3usize", "2usize == 3usize"),
        ("{false}", "{ false }"),
        ("1+2==4", "1 + 2 == 4"),
        ("(1+2)*3==8", "(1 + 2) * 3 == 8"),
        ("f(a,b).field[0]", "f(a, b).field[0]"),
        (
            "if true {false}else{true}",
            "if true { false } else { true }",
        ),
        ("{let x=1u8;x==2u8}", "{ let x = 1u8; x == 2u8 }"),
        ("S{field:1,..base}", "S { field: 1, ..base }"),
        ("S{x}", "S { x }"),
        ("|x:u8| x+1", "|x: u8| x + 1"),
        ("x as u8", "x as u8"),
        ("(x as u8)<2", "(x as u8) < 2"),
        ("f::<u8>(1)", "f::<u8>(1)"),
    ] {
        let actual = write(source);
        assert!(actual.is_ok(), "source={source} error={actual:?}");
        assert_eq!(actual?, expected, "source={source}");
    }
    let source = "2usize == 3usize,";
    let forest = Forest::parse(source)?;
    let input = Input::from_forest(&forest, forest.roots())?;
    let prepared = assertion::prepare(
        Source {
            file: FileId(0),
            text: source,
        },
        &input,
        &EMPTY_INPUTS,
        &|_, _| Ok(()),
    )?;
    assert_eq!(
        prepared.default_message(&EMPTY_INPUTS, &|_, _| Ok(()))?,
        "assertion failed: 2usize == 3usize"
    );
    let source = "true, \"{}\", argument()";
    let forest = Forest::parse(source)?;
    let input = Input::from_forest(&forest, forest.roots())?;
    let prepared = assertion::prepare(
        Source {
            file: FileId(0),
            text: source,
        },
        &input,
        &EMPTY_INPUTS,
        &|_, _| Ok(()),
    )?;
    assert!(
        prepared
            .default_message(&EMPTY_INPUTS, &|_, _| Ok(()))
            .is_err()
    );
    assert!(crate::expand::builtin::Builtin::Assert.executable());
    Ok(())
}
#[test]
fn expression_writer_work_is_selected_bounded_and_layout_storage_is_cold() -> TestResult {
    let mut observed = Vec::new();
    for width in [8usize, 32, 128] {
        let source = std::iter::repeat_n("value", width)
            .collect::<Vec<_>>()
            .join(" + ");
        let ast = expression::parse(&source)?;
        let work = Cell::new(0usize);
        let bytes = Cell::new(0usize);
        let output = expression(&source, &ast, ast.root, &EMPTY_INPUTS, &|steps, storage| {
            work.set(work.get().saturating_add(steps));
            bytes.set(bytes.get().saturating_add(storage));
            Ok(())
        })?;
        assert_eq!(
            output
                .split_whitespace()
                .filter(|word| *word == "value")
                .count(),
            width
        );
        assert!(work.get() <= width * 1024);
        assert!(bytes.get() <= width * 8192);
        observed.push((width, work.get(), bytes.get()));
    }
    let mut ordinary_work = None;
    for width in [0usize, 32, 128] {
        let mut source = "fn chosen(){let _=2usize==3usize;}".to_owned();
        for index in 0..width {
            write!(&mut source, "fn unrelated{index}(){{}} ")
                .map_err(|_| error(0, "test source formatting failed"))?;
        }
        let ast = expression::parse_crate(&source)?;
        let root = ast
            .nodes
            .iter()
            .position(|node| matches!(node.kind, Kind::Binary { .. }))
            .and_then(|index| u32::try_from(index).ok())
            .map(NodeId::new)
            .ok_or_else(|| error(0, "missing selected actual expression"))?;
        let work = Cell::new(0usize);
        let output = expression(&source, &ast, root, &EMPTY_INPUTS, &|steps, _| {
            work.set(work.get() + steps);
            Ok(())
        })?;
        assert_eq!(output, "2usize == 3usize");
        if let Some(expected) = ordinary_work {
            assert_eq!(work.get(), expected);
        } else {
            ordinary_work = Some(work.get());
        }
    }
    let source = "1 + 2";
    let ast = expression::parse(source)?;
    let attempts = Cell::new(0usize);
    assert!(
        expression(source, &ast, ast.root, &EMPTY_INPUTS, &|steps, _| {
            attempts.set(attempts.get() + steps);
            Err(error(0, "test exhausted writer budget"))
        })
        .is_err()
    );
    assert!(attempts.get() > 0);
    let mut cyclic = expression::parse("(true)")?;
    cyclic.nodes[cyclic.root.get() as usize].kind = Kind::Group(cyclic.root);
    assert!(
        expression(
            "(true)",
            &cyclic,
            cyclic.root,
            &EMPTY_INPUTS,
            &|_, _| Ok(())
        )
        .is_err()
    );
    eprintln!(
        "assert writer widths={observed:?} Action={} document={} Package={} Node={} Item={}",
        size_of::<Action<'_>>(),
        size_of::<layout::Token<'_>>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::syntax::item::Item>()
    );
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::syntax::item::Item>(), 64);
    Ok(())
}
