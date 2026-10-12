use rustgm::{
    expand::{
        config::Config,
        macros::{
            compile,
            input::Input,
            matcher::{Compiled, Workspace},
            session, transcribe,
        },
        stream::{ContextId, Environment, Source, Stream},
    },
    support::{names::Names, source::FileId},
    syntax::{
        ast::{Kind, MacroContext},
        expression::fragment,
        token_tree::Forest,
    },
};

fn environment<'a>(definition: &'a str, invocation: &'a str) -> Environment<'a> {
    Environment {
        definition: Source {
            file: FileId(0),
            text: definition,
        },
        invocation: Source {
            file: FileId(1),
            text: invocation,
        },
        context: ContextId(2),
        crate_id: 0,
    }
}
fn compile_rules(rules: &str, names: &mut Names) -> Result<Compiled, String> {
    let forest = Forest::parse(rules).map_err(|e| format!("{e:?}"))?;
    compile::compile(rules, &forest, names)
        .map(Compiled::new)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn forwarded_fragments_follow_the_independently_checked_compatibility_matrix() -> Result<(), String>
{
    let cases = [
        ("block", "{1}", "block expr expr_2021 stmt tt"),
        ("expr", "1+2", "expr expr_2021 pat pat_param stmt tt"),
        ("expr_2021", "1+2", "expr expr_2021 pat pat_param stmt tt"),
        (
            "literal",
            "1",
            "expr expr_2021 literal pat pat_param stmt tt",
        ),
        ("pat", "A | B", "pat pat_param tt"),
        ("pat_param", "A", "pat pat_param tt"),
        (
            "path",
            "a::B",
            "expr expr_2021 pat pat_param path ty stmt meta tt",
        ),
        ("ty", "u32", "path ty meta tt"),
        ("stmt", "let x = 1", "stmt tt"),
        ("item", "fn f(){}", "stmt item tt"),
        ("meta", "doc=\"hello\"", "meta tt"),
        ("vis", "pub(crate)", "vis tt"),
        (
            "ident",
            "abc",
            "expr expr_2021 pat pat_param path ty stmt meta ident tt",
        ),
        ("lifetime", "'a", "lifetime tt"),
        (
            "tt",
            "abc",
            "expr expr_2021 pat pat_param path ty stmt meta ident tt",
        ),
    ];
    for (from, sample, accepted) in cases {
        let source = format!("macro_rules! outer {{($x:{from})=>{{$x}}}} outer!({sample});");
        let report =
            session::analyze(&source, &Config::default()).map_err(|e| format!("{from}: {e:?}"))?;
        let call = &report.invocations[0];
        let stream = Stream::transcribe(
            &call.transcription,
            &call.matched,
            &call.input,
            &environment(&source, &source),
        )
        .map_err(|e| format!("{e:?}"))?;
        let splices = stream
            .syntax_inputs(&call.matched, &source)
            .map_err(|e| format!("{e:?}"))?;
        let input = Input {
            tokens: stream.tokens.clone(),
            after: (1..=stream.tokens.len()).collect(),
        };
        for (to, _, _) in cases {
            let rules = format!("{{($x:{to})=>{{}}}}");
            let mut names = Names::default();
            let mut compiled = compile_rules(&rules, &mut names)?;
            let result = compiled.invoke_with_splices(
                &rules,
                &stream.source,
                &input,
                &mut names,
                &mut Workspace::default(),
                &splices,
            );
            let expected = accepted.split_whitespace().any(|kind| kind == to);
            assert_eq!(
                result.is_ok(),
                expected,
                "{from} -> {to}: {:?}",
                result.err()
            );
        }
    }
    Ok(())
}

#[test]
fn forwarding_keeps_opacity_identity_and_precedence_through_nested_calls() -> Result<(), String> {
    let source = "macro_rules! outer {($x:expr)=>{$x}} outer!(1+2);";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let call = &report.invocations[0];
    let stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &environment(source, source),
    )
    .map_err(|e| format!("{e:?}"))?;
    let splices = stream
        .syntax_inputs(&call.matched, source)
        .map_err(|e| format!("{e:?}"))?;
    let input = Input {
        tokens: stream.tokens.clone(),
        after: vec![1],
    };
    let rules = "{(1+2)=>{0}; ($x:tt)=>{$x * 3}}";
    let mut names = Names::default();
    let mut compiled = compile_rules(rules, &mut names)?;
    let matched = compiled
        .invoke_with_splices(
            rules,
            &stream.source,
            &input,
            &mut names,
            &mut Workspace::default(),
            &splices,
        )
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(matched.rule, 1);
    assert_eq!(matched.captures.parser_calls, 0);
    let transcription =
        transcribe::transcribe(&compiled.definition, &matched, &stream.source, &input)
            .map_err(|e| format!("{e:?}"))?;
    let nested = Stream::transcribe(
        &transcription,
        &matched,
        &input,
        &environment(rules, &stream.source),
    )
    .map_err(|e| format!("{e:?}"))?;
    let inputs = nested
        .syntax_inputs_with_parents(&matched, &stream.source, &[], &splices)
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(inputs[0].original, splices[0].original);
    assert_eq!(inputs[0].original_source, source);
    let ast = fragment::parse_expansion(
        &nested.source,
        &nested.tokens,
        MacroContext::Expression,
        &inputs,
    )
    .map_err(|e| format!("{e:?}"))?;
    let Kind::Binary {
        op: rustgm::lex::token::Punctuation::Star,
        left,
        ..
    } = ast.node(ast.root).kind
    else {
        return Err("missing multiplication".into());
    };
    assert!(matches!(
        ast.node(left).kind,
        Kind::Binary {
            op: rustgm::lex::token::Punctuation::Plus,
            ..
        }
    ));
    Ok(())
}
