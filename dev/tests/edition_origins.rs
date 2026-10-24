//! Transcription resolves editions from existing origins without relexing output.
use rustgm::{
    expand::{
        macros::{
            compile,
            input::Input,
            matcher::{Compiled, Matched, Workspace},
            transcribe,
        },
        marks::{ExpansionId, SyntaxContextId},
        stream::{Environment, Location, Source, Stream},
    },
    lex::token::{LexError, Token},
    support::{edition::Edition, names::Names, source::FileId},
    syntax::{
        ast::{Kind, MacroContext},
        edition_context::WithEditions,
        expression::fragment,
        interpolation::{EMPTY_INPUTS, Inputs, Slice},
        item::ItemContext,
        token_tree::{Forest, error},
    },
};

fn expand(
    rules: &str,
    source: &str,
    defining: Edition,
    context: &dyn Inputs,
) -> Result<(Stream, Matched), LexError> {
    let mut names = Names::default();
    let definition = compile::in_edition(rules, &Forest::parse(rules)?, &mut names, defining)?;
    let forest = Forest::parse(source)?;
    let input = Input::from_forest(&forest, forest.roots())?;
    let mut compiled = Compiled::new(definition);
    let matched = compiled.invoke_with_inputs(
        rules,
        source,
        &input,
        &mut names,
        &mut Workspace::default(),
        context,
    )?;
    let transcription = transcribe::transcribe(&compiled.definition, &matched, source, &input)?;
    let stream = Stream::transcribe(
        &transcription,
        &matched,
        &input,
        &Environment {
            definition: Source {
                file: FileId(0),
                text: rules,
            },
            invocation: Source {
                file: FileId(1),
                text: source,
            },
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(2),
            crate_id: 0,
        },
    )?;
    Ok((stream, matched))
}

fn edition(location: Location, defining: Edition, caller: Edition) -> Result<Edition, LexError> {
    match location.file {
        FileId(0) => Ok(defining),
        FileId(1) => Ok(caller),
        _ => Err(error(location.span.start, "unknown original source")),
    }
}

#[test]
fn literal_and_transparent_captured_names_keep_their_source_editions() -> Result<(), LexError> {
    for defining in [Edition::Rust2021, Edition::Rust2024] {
        for caller in [Edition::Rust2021, Edition::Rust2024] {
            for (rules, source, accepted) in [
                ("{()=>{fn gen(){}}}", "", defining == Edition::Rust2021),
                (
                    "{($name:ident)=>{fn $name(){}}}",
                    "gen",
                    caller == Edition::Rust2021,
                ),
                ("{($name:ident)=>{fn $name(){}}}", "r#gen", true),
            ] {
                let (stream, _) = expand(rules, source, defining, &caller)?;
                let context = WithEditions::new(&EMPTY_INPUTS, |token| {
                    edition(stream.token_origin(token)?, defining, caller)
                });
                let result = fragment::parse_expansion_with_inputs(
                    &stream.source,
                    &stream.tokens,
                    MacroContext::Items(ItemContext::Module),
                    &context,
                );
                assert_eq!(
                    result.is_ok(),
                    accepted,
                    "{defining:?}/{caller:?}: {rules}: {:?}",
                    result.err()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn generated_expression_keeps_defining_edition_at_the_next_matcher() -> Result<(), LexError> {
    let outer = "{()=>{target!(gen())}}";
    let target = "{($e:expr)=>{$e}}";
    for defining in [Edition::Rust2021, Edition::Rust2024] {
        let (stream, _) = expand(outer, "", defining, &Edition::Rust2024)?;
        let context = WithEditions::new(&EMPTY_INPUTS, |token| {
            edition(stream.token_origin(token)?, defining, Edition::Rust2024)
        });
        let ast = fragment::parse_expansion_with_inputs(
            &stream.source,
            &stream.tokens,
            MacroContext::Expression,
            &context,
        )?;
        let Kind::Macro { capture, .. } = ast.node(ast.root).kind else {
            return Err(error(0, "missing generated macro call"));
        };
        let forest = &ast.captures[capture as usize];
        let range = forest
            .children(0)
            .ok_or_else(|| error(0, "missing macro argument group"))?;
        let input = Input::from_forest(forest, range)?;
        for target_edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut names = Names::default();
            let definition =
                compile::in_edition(target, &Forest::parse(target)?, &mut names, target_edition)?;
            let result = Compiled::new(definition).invoke_with_inputs(
                target,
                &stream.source,
                &input,
                &mut names,
                &mut Workspace::default(),
                &context,
            );
            assert_eq!(result.is_ok(), defining == Edition::Rust2021);
        }
    }
    Ok(())
}

#[test]
fn opaque_captured_expression_is_not_reparsed_under_output_edition() -> Result<(), LexError> {
    let source = "gen()";
    let (stream, matched) = expand(
        "{($e:expr)=>{$e}}",
        source,
        Edition::Rust2024,
        &Edition::Rust2021,
    )?;
    assert_eq!(matched.captures.parser_calls, 1);
    let splices = stream.syntax_inputs(&matched, source)?;
    let slice = Slice(&splices);
    let context = WithEditions::new(&slice, |token| {
        Err(error(token.start, "opaque syntax must not be reparsed"))
    });
    fragment::parse_expansion_with_inputs(
        &stream.source,
        &stream.tokens,
        MacroContext::Expression,
        &context,
    )?;
    let rules = "{($e:expr)=>{$e}}";
    let mut names = Names::default();
    let definition = compile::compile(rules, &Forest::parse(rules)?, &mut names)?;
    let input = Input {
        tokens: stream.tokens.clone(),
        after: vec![1],
    };
    let forwarded = Compiled::new(definition).invoke_with_inputs(
        rules,
        &stream.source,
        &input,
        &mut names,
        &mut Workspace::default(),
        &context,
    )?;
    assert_eq!(forwarded.captures.parser_calls, 0);
    Ok(())
}

#[test]
fn opaque_attribute_paths_reuse_checked_names_without_reassigning_their_edition()
-> Result<(), LexError> {
    for source in ["gen", "r#gen", "module::gen"] {
        let (stream, matched) = expand(
            "{($p:path)=>{#[$p] fn hidden(){}}}",
            source,
            Edition::Rust2024,
            &Edition::Rust2021,
        )?;
        let splices = stream.syntax_inputs(&matched, source)?;
        let slice = Slice(&splices);
        let context = WithEditions::new(&slice, |token| {
            Err(error(
                token.start,
                "checked path cannot be reassigned an edition",
            ))
        });
        fragment::parse_expansion_with_inputs(
            &stream.source,
            &stream.tokens,
            MacroContext::Items(ItemContext::Module),
            &context,
        )?;
    }
    Ok(())
}

#[test]
fn opaque_path_admission_is_local_to_its_span() -> Result<(), LexError> {
    let source = "gen";
    let (stream, matched) = expand(
        "{($p:path)=>{#[$p] #[gen] fn hidden(){}}}",
        source,
        Edition::Rust2024,
        &Edition::Rust2021,
    )?;
    let splices = stream.syntax_inputs(&matched, source)?;
    let slice = Slice(&splices);
    let context = WithEditions::new(&slice, |_| Ok(Edition::Rust2024));
    assert!(
        fragment::parse_expansion_with_inputs(
            &stream.source,
            &stream.tokens,
            MacroContext::Items(ItemContext::Module),
            &context
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn opaque_path_conversion_to_metadata_keeps_parsed_identifier_validity() -> Result<(), LexError> {
    let source = "module::gen";
    let (stream, matched) = expand(
        "{($p:path)=>{$p}}",
        source,
        Edition::Rust2024,
        &Edition::Rust2021,
    )?;
    let splices = stream.syntax_inputs(&matched, source)?;
    let slice = Slice(&splices);
    let context = WithEditions::new(&slice, |token| {
        Err(error(
            token.start,
            "opaque path metadata must reuse identifier validity",
        ))
    });
    fragment::metadata_with_inputs(
        &stream.source,
        &stream.tokens,
        rustgm::syntax::cursor::TokenPosition::default(),
        &context,
    )?;
    Ok(())
}

#[test]
fn missing_and_partial_token_origins_fail_closed() -> Result<(), LexError> {
    let (mut stream, _) = expand("{()=>{gen}}", "", Edition::Rust2021, &Edition::Rust2024)?;
    let token = *stream
        .tokens
        .first()
        .ok_or_else(|| error(0, "missing generated token"))?;
    assert_eq!(stream.token_origin(token)?.file, FileId(0));
    assert!(
        stream
            .token_origin(Token {
                end: token.end - 1,
                ..token
            })
            .is_err()
    );
    assert!(
        stream
            .token_origin(Token {
                start: token.start + 1,
                ..token
            })
            .is_err()
    );
    stream.origins.clear();
    let context = WithEditions::new(&EMPTY_INPUTS, |token| {
        edition(
            stream.token_origin(token)?,
            Edition::Rust2021,
            Edition::Rust2024,
        )
    });
    assert!(
        fragment::parse_expansion_with_inputs(
            &stream.source,
            &stream.tokens,
            MacroContext::Expression,
            &context
        )
        .is_err()
    );
    Ok(())
}
