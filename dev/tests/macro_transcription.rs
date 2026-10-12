use rustgm::{
    expand::macros::{
        compile,
        input::Input,
        matcher::Compiled,
        transcribe::{self, Origin, Piece},
    },
    support::names::Names,
    syntax::token_tree::Forest,
};

fn transcribe(rules: &str, source: &str) -> Result<(Vec<String>, usize), String> {
    let mut names = Names::default();
    let forest = Forest::parse(rules).map_err(|e| format!("{e:?}"))?;
    let definition = compile::compile(rules, &forest, &mut names).map_err(|e| format!("{e:?}"))?;
    let forest = Forest::parse(source).map_err(|e| format!("{e:?}"))?;
    let input = Input::from_forest(&forest, forest.roots()).map_err(|e| format!("{e:?}"))?;
    let mut compiled = Compiled::new(definition);
    let matched = compiled
        .invoke(rules, source, &input, &mut names)
        .map_err(|e| format!("{e:?}"))?;
    let output = transcribe::transcribe(&compiled.definition, &matched, source, &input)
        .map_err(|e| format!("{e:?}"))?;
    let mut spellings = Vec::new();
    for piece in output.pieces {
        spellings.push(match piece {
            Piece::Token { token, origin } => {
                let bytes = if origin == Origin::Definition {
                    rules
                } else {
                    source
                };
                bytes[token.start as usize..token.end as usize].to_owned()
            }
            Piece::Opaque(id) => format!("opaque{}", id.0),
            Piece::Crate(_) => "$crate".to_owned(),
            Piece::Doc { .. } => "doc-comment".to_owned(),
        });
    }
    Ok((spellings, matched.captures.parser_calls))
}
#[test]
fn repeated_opaque_substitutions_keep_one_parsed_identity() -> Result<(), String> {
    let (pieces, parsed) = transcribe("{($e:expr)=>{($e) + ($e)}}", "1+2")?;
    assert_eq!(pieces, ["(", "opaque0", ")", "+", "(", "opaque0", ")"]);
    assert_eq!(parsed, 1);
    Ok(())
}
#[test]
fn repetition_indices_preserve_empty_nested_sequences_and_outer_scalars() -> Result<(), String> {
    for (rules, source, expected) in [
        (
            "{($([$($i:ident),*]);*)=>{$([$($i);*])*}}",
            "[a,b];[];[c]",
            "[ a ; b ] [ ] [ c ]",
        ),
        (
            "{($outer:ident;$($i:ident),*)=>{$($outer::$i);*}}",
            "m;a,b",
            "m :: a ; m :: b",
        ),
        (
            "{($($i:ident),*;$($j:ident),*)=>{$(($i,$j))*}}",
            "a,b;x,y",
            "( a , x ) ( b , y )",
        ),
        ("{($($i:ident),*)=>{$($i)+}}", "a,b", "a b"),
        ("{($($i:ident),*)=>{$($i)?}}", "a,b", "a b"),
        ("{($($i:ident),*)=>{$($i)*}}", "", ""),
        ("{($($i:ident)?)=>{$($i)?}}", "x", "x"),
    ] {
        let (pieces, _) = transcribe(rules, source)?;
        assert_eq!(pieces.join(" "), expected, "{rules}");
    }
    Ok(())
}
#[test]
fn incorrect_shape_is_an_invocation_time_error() {
    for (rules, source) in [
        ("{($($i:ident),*)=>{$i}}", "a,b"),
        ("{($i:ident)=>{$($i)*}}", "a"),
        ("{($($i:ident),*)=>{$($($i)*)*}}", "a,b"),
        ("{($($i:ident),*)=>{$($i)+}}", ""),
        ("{($($i:ident),*;$($j:ident),*)=>{$(($i,$j))*}}", "a,b;x"),
        ("{()=>{$(x)*}}", ""),
    ] {
        assert!(transcribe(rules, source).is_err(), "{rules}: {source}");
    }
}
#[test]
fn nested_definitions_and_crate_markers_retain_typed_template_tokens() -> Result<(), String> {
    let (pieces, _) = transcribe("{()=>{macro_rules! n {($x:ident)=>{$crate::f($x)}}}}", "")?;
    assert_eq!(
        pieces.join(" "),
        "macro_rules ! n { ( $ x : ident ) => { $crate :: f ( $ x ) } }"
    );
    Ok(())
}
