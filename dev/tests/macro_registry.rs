use rustgm::{
    expand::{
        config::Config,
        macros::{
            compile,
            input::Input,
            matcher::{Compiled, Workspace},
            session, transcribe,
        },
        registry::Registry,
        stream::{ContextId, Environment, Source, Stream},
    },
    lex::token::Punctuation as P,
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
fn next_call(
    original: &str,
    current: &Stream,
    rules: &str,
    registry: &mut Registry,
) -> Result<Stream, String> {
    let view = registry.view(|file| (file == FileId(0)).then_some(original));
    let ast = fragment::parse_expansion_with_inputs(
        &current.source,
        &current.tokens,
        MacroContext::Expression,
        &view,
    )
    .map_err(|e| format!("{e:?}"))?;
    let Kind::Macro { capture, .. } = ast.node(ast.root).kind else {
        return Err("missing nested macro".into());
    };
    let forest = &ast.captures[capture as usize];
    let input = Input::from_forest(forest, forest.children(0).ok_or("missing argument group")?)
        .map_err(|e| format!("{e:?}"))?;
    let mut names = Names::default();
    let definition = compile::compile(
        rules,
        &Forest::parse(rules).map_err(|e| format!("{e:?}"))?,
        &mut names,
    )
    .map_err(|e| format!("{e:?}"))?;
    let mut compiled = Compiled::new(definition);
    let mut matched = compiled
        .invoke_with_inputs(
            rules,
            &current.source,
            &input,
            &mut names,
            &mut Workspace::default(),
            &view,
        )
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(matched.captures.parser_calls, 0);
    let transcription =
        transcribe::transcribe(&compiled.definition, &matched, &current.source, &input)
            .map_err(|e| format!("{e:?}"))?;
    let mut stream = Stream::transcribe(
        &transcription,
        &matched,
        &input,
        &environment(rules, &current.source),
    )
    .map_err(|e| format!("{e:?}"))?;
    registry
        .adopt(
            &mut stream,
            &mut matched,
            FileId(1),
            &current.source,
            &input,
        )
        .map_err(|e| format!("{e:?}"))?;
    Ok(stream)
}

#[test]
fn persistent_handles_survive_generated_syntax_and_repeated_forwarding() -> Result<(), String> {
    let source = "macro_rules! outer {($x:expr)=>{middle!($x)}} fn f(){outer!(1+2);}";
    let mut report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let mut call = report.invocations.remove(0);
    let mut stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &environment(source, source),
    )
    .map_err(|e| format!("{e:?}"))?;
    let mut registry = Registry::default();
    registry
        .adopt(
            &mut stream,
            &mut call.matched,
            FileId(0),
            source,
            &call.input,
        )
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.token_count(), 3);
    assert!(
        registry
            .adopt(
                &mut stream,
                &mut call.matched,
                FileId(0),
                source,
                &call.input
            )
            .is_err()
    );
    assert!(stream.syntax_inputs(&call.matched, source).is_err());
    drop(report);
    drop(call);
    let middle = next_call(
        source,
        &stream,
        "{($x:expr)=>{inner!($x,$x)}}",
        &mut registry,
    )?;
    drop(stream);
    let final_stream = next_call(
        source,
        &middle,
        "{($a:expr,$b:expr)=>{$a * $b}}",
        &mut registry,
    )?;
    drop(middle);
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.token_count(), 3);
    let ast = fragment::parse_expansion_with_inputs(
        &final_stream.source,
        &final_stream.tokens,
        MacroContext::Expression,
        &registry.view(|file| (file == FileId(0)).then_some(source)),
    )
    .map_err(|e| format!("{e:?}"))?;
    let Kind::Binary {
        op: P::Star,
        left,
        right,
    } = ast.node(ast.root).kind
    else {
        return Err("missing multiply".into());
    };
    assert_ne!(left, right);
    for node in [left, right] {
        assert!(matches!(
            ast.node(node).kind,
            Kind::Binary { op: P::Plus, .. }
        ));
    }
    Ok(())
}
