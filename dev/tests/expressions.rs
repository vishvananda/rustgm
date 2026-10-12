use rustgm::{
    lex::token::Punctuation as P,
    syntax::{
        ast::{Ast, BlockFlavor, Kind, Node, NodeId, Unary},
        expression::{self, parse},
    },
};

fn ast(source: &str) -> Result<Ast<'_>, String> {
    parse(source).map_err(|e| format!("{e:?}"))
}

fn binary(ast: &Ast<'_>, id: NodeId, op: P) -> Result<(NodeId, NodeId), String> {
    let Kind::Binary {
        op: actual,
        left,
        right,
    } = ast.node(id).kind
    else {
        return Err("expected binary node".into());
    };
    assert_eq!(actual, op);
    Ok((left, right))
}

#[test]
fn hand_checked_precedence_dump() -> Result<(), String> {
    let mut output = Vec::new();
    expression::render::expression("1+2*3", &mut output).map_err(|e| format!("{e:?}"))?;
    assert_eq!(output, b"node 0 0 1 literal 0\nvalue 0 integer 0 1 infer 1\nnode 1 2 3 literal 1\nvalue 1 integer 2 3 infer 2\nnode 2 4 5 literal 2\nvalue 2 integer 4 5 infer 3\nnode 3 2 5 binary * 1 2\nnode 4 0 5 binary + 0 3\nroot 4\neof 5\n");
    Ok(())
}

#[test]
fn associativity_is_structural() -> Result<(), String> {
    let assignment = ast("a=b=c")?;
    let (_, right) = binary(&assignment, assignment.root, P::Equal)?;
    binary(&assignment, right, P::Equal)?;
    let subtraction = ast("9-3-1")?;
    let (left, _) = binary(&subtraction, subtraction.root, P::Minus)?;
    binary(&subtraction, left, P::Minus)?;
    let operators = ast("1 | 2 ^ 3 & 4 << 1 + 2 * 3")?;
    let mut node = operators.root;
    for op in [P::Or, P::Caret, P::And, P::ShiftLeft, P::Plus, P::Star] {
        (_, node) = binary(&operators, node, op)?;
    }
    Ok(())
}

#[test]
fn nested_child_ranges_preserve_argument_order() -> Result<(), String> {
    let ast = ast("f(g(1,2),[3,4],(5,6))")?;
    let Kind::Call { arguments, .. } = ast.node(ast.root).kind else {
        return Err("expected call".into());
    };
    let children = ast.list(arguments);
    assert_eq!(children.len(), 3);
    let Kind::Call {
        arguments: inner, ..
    } = ast.node(children[0]).kind
    else {
        return Err("expected nested call".into());
    };
    assert_eq!(ast.list(inner).len(), 2);
    assert!(matches!(ast.node(children[1]).kind, Kind::Array(_)));
    assert!(matches!(ast.node(children[2]).kind, Kind::Tuple(_)));
    Ok(())
}

#[test]
fn method_and_field_calls_have_distinct_facts() -> Result<(), String> {
    let method = ast("x.f(1)")?;
    assert!(matches!(
        method.node(method.root).kind,
        Kind::MethodCall { .. }
    ));
    let field = ast("(x.f)(1)")?;
    let Kind::Call { callee, .. } = field.node(field.root).kind else {
        return Err("expected call".into());
    };
    let Kind::Group(inner) = field.node(callee).kind else {
        return Err("expected grouped field".into());
    };
    assert!(matches!(field.node(inner).kind, Kind::Field { .. }));
    let tuple = ast("x.0(1)")?;
    assert!(matches!(tuple.node(tuple.root).kind, Kind::Call { .. }));
    Ok(())
}

#[test]
fn tuple_projection_splits_tokens_and_preserves_spans() -> Result<(), String> {
    let source = "x.0.1.2";
    let ast = ast(source)?;
    let mut id = ast.root;
    for expected in ["2", "1", "0"] {
        let Kind::Field { value, name } = ast.node(id).kind else {
            return Err("expected projection".into());
        };
        assert_eq!(&source[name.start as usize..name.end as usize], expected);
        id = value;
    }
    assert!(ast.lookahead_high_water <= 3);
    Ok(())
}

#[test]
fn statement_boundaries_and_tail_are_explicit() -> Result<(), String> {
    let ast = ast("{ if true {} [1,2] }")?;
    let Kind::Block {
        statements, tail, ..
    } = ast.node(ast.root).kind
    else {
        return Err("expected block".into());
    };
    assert_eq!(ast.list(statements).len(), 1);
    assert!(matches!(
        ast.node(tail.ok_or("missing tail")?).kind,
        Kind::Array(_)
    ));
    assert!(parse("{ if true {} + 1 }").is_err());
    assert!(parse("{ async {} 1 }").is_err());
    let qualified = parse("unsafe {}").map_err(|e| format!("{e:?}"))?;
    assert!(matches!(
        qualified.node(qualified.root).kind,
        Kind::Block {
            flavor: BlockFlavor::Unsafe,
            ..
        }
    ));
    assert_eq!(qualified.node(qualified.root).span.start, 0);
    Ok(())
}

#[test]
fn contextual_raw_name_is_not_always_a_raw_borrow() -> Result<(), String> {
    for (source, expected) in [
        ("&raw", Unary::Borrow),
        ("&raw const x", Unary::RawConst),
        ("&raw mut x", Unary::RawMut),
    ] {
        let ast = ast(source)?;
        let Kind::Unary { op, .. } = ast.node(ast.root).kind else {
            return Err("expected unary".into());
        };
        assert_eq!(op, expected);
    }
    Ok(())
}

#[test]
fn macro_capture_is_only_retained_token_forest() -> Result<(), String> {
    let ordinary = ast("{ let x = 1; x + 2 }")?;
    assert!(ordinary.captures.is_empty());
    let captured = ast("m!(#[cfg(bad())] 1arbitrary [x])")?;
    assert_eq!(captured.captures.len(), 1);
    assert!(captured.literals.is_empty());
    assert!(captured.captures[0].nodes().len() > 5);
    let nested = format!("m!({}x{})", "[".repeat(20_000), "]".repeat(20_000));
    let deep = ast(&nested)?;
    assert_eq!(deep.captures[0].nodes().len(), 20_002);
    assert!(deep.lookahead_high_water <= 3);
    Ok(())
}

#[test]
fn long_left_chains_build_render_and_drop_without_recursion() -> Result<(), String> {
    let text = format!("{}1", "1+".repeat(20_000));
    let ast = ast(&text)?;
    assert_eq!(ast.nodes.len(), 40_001);
    assert!(ast.lookahead_high_water <= 3);
    assert!(size_of::<Node>() <= 56);
    drop(ast);
    expression::render::expression(&text, &mut std::io::sink()).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn recursive_syntax_is_diagnosed_before_stack_exhaustion() {
    for text in [
        format!("{}0{}", "(".repeat(2000), ")".repeat(2000)),
        format!("{}0", "x=".repeat(2000)),
        format!("{{ let x: {}i32 = 0; }}", "&".repeat(2000)),
        format!("{{ let {}x = 0; }}", "&".repeat(2000)),
    ] {
        let result = parse(&text);
        assert!(result.is_err());
    }
}

#[test]
fn parser_errors_are_atomic_in_the_text_view() {
    for text in ["1 +", "{ let x = 1; x x }", "m!([)]", "\"\\q\""] {
        let mut output = Vec::new();
        assert!(expression::render::expression(text, &mut output).is_err());
        assert!(output.is_empty());
    }
}

#[test]
fn let_chains_do_not_admit_let_in_arbitrary_expressions() -> Result<(), String> {
    ast("if let Some(x) = y && x > 0 && let z = x {}")?;
    for text in [
        "if true || let x = 1 {}",
        "if (let x = 1) {}",
        "if !let x = 1 {}",
        "if let x = let y = 1 {}",
    ] {
        assert!(parse(text).is_err(), "{text}");
    }
    Ok(())
}
