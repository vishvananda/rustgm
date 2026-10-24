use rustgm::{
    expand::{
        config::Config,
        macros::session,
        marks::{ExpansionId, SyntaxContextId},
        stream::{Environment, Source, Stream},
    },
    lex::token::{Kind as TokenKind, Punctuation as P},
    support::source::FileId,
    syntax::{
        ast::Kind,
        cursor::TokenPosition,
        expression::fragment::{self, FragmentKind as F},
    },
};
fn environment(source: &str) -> Environment<'_> {
    Environment {
        definition: Source {
            file: FileId(2),
            text: source,
        },
        invocation: Source {
            file: FileId(5),
            text: source,
        },
        context: SyntaxContextId::ROOT,
        introduction: ExpansionId(9),
        crate_id: 0,
    }
}

#[test]
fn metadata_and_visibility_splice_typed_facts_into_generated_items() -> Result<(), String> {
    use rustgm::{
        expand::crate_syntax,
        syntax::{
            ast::MacroContext,
            item::{ItemContext, Visibility},
        },
    };
    for (source, disabled, restricted) in [
        (
            "macro_rules! m {($v:vis,$a:meta)=>{#[$a] $v fn f(){}}} m!(pub(crate),cfg(any()));",
            true,
            true,
        ),
        (
            "macro_rules! m {($v:vis,$a:meta)=>{#[$a] $v fn f(){}}} m!(,doc=\"hello\");",
            false,
            false,
        ),
        (
            "macro_rules! m {($a:meta)=>{#[cfg_attr(all(),$a)] fn f(){}}} m!(doc=\"hello\");",
            false,
            false,
        ),
        (
            "macro_rules! m {($a:meta)=>{#[cfg($a)] fn f(){}}} m!(any());",
            true,
            false,
        ),
    ] {
        let report =
            session::analyze(source, &Config::default()).map_err(|e| format!("{source}: {e:?}"))?;
        let call = &report.invocations[0];
        let stream = Stream::transcribe(
            &call.transcription,
            &call.matched,
            &call.input,
            &environment(source),
        )
        .map_err(|e| format!("{e:?}"))?;
        let splices = stream
            .syntax_inputs(&call.matched, source)
            .map_err(|e| format!("{e:?}"))?;
        let ast = fragment::parse_expansion(
            &stream.source,
            &stream.tokens,
            MacroContext::Items(ItemContext::Module),
            &splices,
        )
        .map_err(|e| format!("{source}: {e:?}"))?;
        assert_eq!(
            matches!(ast.items[0].visibility, Visibility::Restricted(_)),
            restricted
        );
        let configured = crate_syntax::configure(&stream.source, ast, &Config::default())
            .map_err(|e| format!("{source}: {e:?}"))?;
        assert_eq!(configured.owners[0].enabled, !disabled);
    }
    Ok(())
}
#[test]
fn matching_transcription_and_parsing_preserve_opaque_precedence() -> Result<(), String> {
    let source = "macro_rules! m {($e:expr)=>{$e * 3}} fn f(){m!(1+2);} ";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let call = &report.invocations[0];
    let stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &environment(source),
    )
    .map_err(|e| format!("{e:?}"))?;
    let splices = stream
        .syntax_inputs(&call.matched, source)
        .map_err(|e| format!("{e:?}"))?;
    let parsed = fragment::parse_with_splices(
        &stream.source,
        &stream.tokens,
        F::Expression,
        TokenPosition::default(),
        &splices,
    )
    .map_err(|e| format!("{e:?}"))?;
    let Kind::Binary {
        op: P::Star, left, ..
    } = parsed.ast.node(parsed.ast.root).kind
    else {
        return Err("missing multiply root".into());
    };
    assert!(matches!(
        parsed.ast.node(left).kind,
        Kind::Binary { op: P::Plus, .. }
    ));
    assert_eq!(call.matched.captures.parser_calls, 1);
    let first = stream.origin(0).ok_or("missing capture origin")?;
    assert_eq!(first.original.file, FileId(5));
    assert_eq!(first.context, None);
    assert_eq!(first.introduction, ExpansionId(0));
    let star = stream
        .origin(stream.tokens[1].start)
        .ok_or("missing template origin")?;
    assert_eq!(star.original.file, FileId(2));
    assert_eq!(star.context, Some(SyntaxContextId::ROOT));
    assert_eq!(star.introduction, ExpansionId(9));
    assert_eq!(parsed.position.index, stream.tokens.len());
    Ok(())
}
#[test]
fn template_doc_tokens_are_created_once_with_definition_context() -> Result<(), String> {
    let source = "macro_rules! m {()=>{/// hello\nfn f(){}}} m!();";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let call = &report.invocations[0];
    let stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &environment(source),
    )
    .map_err(|e| format!("{e:?}"))?;
    let splices = stream
        .syntax_inputs(&call.matched, source)
        .map_err(|e| format!("{e:?}"))?;
    let parsed = fragment::parse_with_splices(
        &stream.source,
        &stream.tokens,
        F::Item,
        TokenPosition::default(),
        &splices,
    )
    .map_err(|e| format!("{e:?}"))?;
    assert!(matches!(
        parsed.ast.node(parsed.ast.root).kind,
        Kind::Attributed { .. }
    ));
    assert_eq!(parsed.ast.attributes.len(), 1);
    assert!(
        stream
            .origins
            .iter()
            .all(|o| o.original.file == FileId(2) && o.context == Some(SyntaxContextId::ROOT))
    );
    assert_eq!(parsed.position.index, stream.tokens.len());
    Ok(())
}
#[test]
fn crate_markers_have_a_typed_identity_distinct_from_identifier_spelling() -> Result<(), String> {
    let source = "macro_rules! m {()=>{$crate::value}} fn f(){m!();}";
    let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let call = &report.invocations[0];
    let stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &environment(source),
    )
    .map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        stream.tokens[0].kind,
        TokenKind::crate_root(0).map_err(|e| format!("{e:?}"))?
    );
    assert_eq!(stream.tokens[1].kind, TokenKind::Punctuation(P::ColonColon));
    let ast = fragment::parse_expansion(
        &stream.source,
        &stream.tokens,
        rustgm::syntax::ast::MacroContext::Expression,
        &[],
    )
    .map_err(|e| format!("{e:?}"))?;
    assert!(ast.nodes.iter().any(|n| n.kind == Kind::CrateRoot(0)));
    Ok(())
}

#[test]
fn whole_expansions_obey_expression_statement_and_item_contexts() -> Result<(), String> {
    use rustgm::syntax::{ast::MacroContext as C, item::ItemContext};
    for (source, context, accepted) in [
        (
            "macro_rules! m {()=>{1;2}} fn f(){m!();}",
            C::Expression,
            false,
        ),
        (
            "macro_rules! m {()=>{1;2}} fn f(){m!();}",
            C::Statements,
            true,
        ),
        (
            "macro_rules! m {()=>{let x=1;x}} fn f(){m!();}",
            C::Statements,
            true,
        ),
        (
            "macro_rules! m {()=>{struct S; fn f(){}}} m!();",
            C::Items(ItemContext::Module),
            true,
        ),
        (
            "macro_rules! m {()=>{struct S;}} m!();",
            C::Items(ItemContext::Trait),
            false,
        ),
        (
            "macro_rules! m {()=>{fn f();}} trait T{m!();}",
            C::Items(ItemContext::Trait),
            true,
        ),
        (
            "macro_rules! m {()=>{}} m!();",
            C::Items(ItemContext::Module),
            true,
        ),
    ] {
        let report = session::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
        let call = &report.invocations[0];
        let stream = Stream::transcribe(
            &call.transcription,
            &call.matched,
            &call.input,
            &environment(source),
        )
        .map_err(|e| format!("{e:?}"))?;
        let splices = stream
            .syntax_inputs(&call.matched, source)
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            fragment::parse_expansion(&stream.source, &stream.tokens, context, &splices).is_ok(),
            accepted,
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn opaque_statements_preserve_complete_let_statements_and_expression_tails() -> Result<(), String> {
    use rustgm::syntax::ast::MacroContext;
    for (argument, tail) in [("let x=1", false), ("3", true)] {
        let source = format!("macro_rules! m {{($s:stmt)=>{{$s $s}}}} fn f(){{m!({argument});}}");
        let report = session::analyze(&source, &Config::default()).map_err(|e| format!("{e:?}"))?;
        let call = &report.invocations[0];
        let stream = Stream::transcribe(
            &call.transcription,
            &call.matched,
            &call.input,
            &environment(&source),
        )
        .map_err(|e| format!("{e:?}"))?;
        let splices = stream
            .syntax_inputs(&call.matched, &source)
            .map_err(|e| format!("{e:?}"))?;
        let ast = fragment::parse_expansion(
            &stream.source,
            &stream.tokens,
            MacroContext::Statements,
            &splices,
        )
        .map_err(|e| format!("{e:?}"))?;
        let Kind::Expansion { tail: value, .. } = ast.node(ast.root).kind else {
            return Err("expected statement expansion root".into());
        };
        assert_eq!(value.is_some(), tail);
    }
    Ok(())
}

#[test]
fn source_macro_nodes_record_the_grammar_that_must_consume_their_expansions() -> Result<(), String>
{
    use rustgm::syntax::{ast::MacroContext as C, expression, item::ItemContext};
    let source =
        "m!(); type T=m!(); fn f(){let m!()=0;m!();} fn g(){m!{}} fn h(){m!()} trait T{m!();}";
    let ast = expression::parse_crate(source).map_err(|e| format!("{e:?}"))?;
    let contexts: Vec<_> = ast
        .nodes
        .iter()
        .filter_map(|n| {
            if let Kind::Macro { context, .. } = n.kind {
                Some(context)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        contexts,
        [
            C::Items(ItemContext::Module),
            C::Type,
            C::Pattern,
            C::Statements,
            C::Statements,
            C::Expression,
            C::Items(ItemContext::Trait)
        ]
    );
    Ok(())
}
