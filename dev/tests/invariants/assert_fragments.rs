//! A genuine registry-owned expression is relocated exactly once on each demand.
use super::*;
use crate::{
    expand::{
        macros::session,
        marks::ExpansionId,
        registry::Registry,
        stream::{Environment, Stream},
    },
    syntax::ast::{MacroContext, NodeId},
};
fn capture() -> std::result::Result<(&'static str, Registry, Stream), LexError> {
    let source = "macro_rules! outer {($x:expr)=>{actual!($x)}} fn f(){outer!(1 + 2 == 4);}";
    let mut report = session::analyze(source, &Config::default())?;
    let mut call = report.invocations.remove(0);
    let mut stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &Environment {
            definition: Source {
                file: FileId(0),
                text: source,
            },
            invocation: Source {
                file: FileId(0),
                text: source,
            },
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(1),
            crate_id: 0,
            edition: crate::support::edition::Edition::Rust2024,
        },
        &report.names,
    )?;
    let mut registry = Registry::default();
    registry.adopt(
        &mut stream,
        &mut call.matched,
        FileId(0),
        source,
        &call.input,
    )?;
    assert_eq!(registry.len(), 1);
    Ok((source, registry, stream))
}

pub(super) fn relocated_and_forwarded() -> Result {
    let (source, registry, stream) = capture()?;
    let view = registry.view(|id| (id == FileId(0)).then_some(source));
    let parsed = fragment::parse_expansion_with_inputs(
        &stream.source,
        &stream.tokens,
        MacroContext::Expression,
        &view,
    )?;
    let Kind::Macro { capture, .. } = parsed.node(parsed.root).kind else {
        return Err(error(0, "expected real transcribed macro"));
    };
    let forest = &parsed.captures[capture as usize];
    let arguments = forest
        .children(0)
        .ok_or_else(|| error(0, "expected argument group"))?;
    let input = Input::from_forest(forest, arguments)?;
    let prepared = prepare(
        Source {
            file: FileId(1),
            text: &stream.source,
        },
        &input,
        &view,
        &|_, _| Ok(()),
    )?;
    let mut first =
        prepared.consume("first ", |_| Ok((SyntaxContextId::ROOT, false)), &|_, _| {
            Ok(())
        })?;
    let span = first.ast.node(first.ast.root).span;
    assert_eq!(span.start, 6);
    assert_eq!(
        &first.stream.source[span.start as usize..span.end as usize],
        "1 + 2 == 4"
    );
    registry.forward(&mut first.stream)?;
    let input = Input {
        tokens: first.stream.tokens.clone(),
        after: vec![1],
    };
    let view = registry.view(|id| (id == FileId(0)).then_some(source));
    let prepared = prepare(
        Source {
            file: FileId(2),
            text: &first.stream.source,
        },
        &input,
        &view,
        &|_, _| Ok(()),
    )?;
    let count = prepared.ast.nodes.len();
    let mut second = prepared.consume(
        "second-prefix ",
        |_| Ok((SyntaxContextId::ROOT, true)),
        &|_, _| Ok(()),
    )?;
    let span = second.ast.node(second.ast.root).span;
    assert_eq!(span.start, 14);
    assert_eq!(
        &second.stream.source[span.start as usize..span.end as usize],
        "1 + 2 == 4"
    );
    assert_eq!(second.ast.nodes.len(), count);
    registry.forward(&mut second.stream)?;
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.token_count(), 5);
    assert_eq!(second.stream.opaque.len(), 1);
    assert_eq!(second.stream.origins[0].context, None);
    assert_eq!(second.stream.origins[0].joint, Some(true));
    let mut prepared = prepare(
        Source {
            file: FileId(2),
            text: &first.stream.source,
        },
        &input,
        &view,
        &|_, _| Ok(()),
    )?;
    prepared.ast.nodes[NodeId::new(0).get() as usize].span.start = 0;
    let origins = Cell::new(0);
    assert!(
        prepared
            .consume(
                "",
                |_| {
                    origins.set(origins.get() + 1);
                    Ok((SyntaxContextId::ROOT, false))
                },
                &|_, _| Ok(())
            )
            .is_err()
    );
    assert_eq!(
        origins.get(),
        0,
        "all typed spans reject before output/origin production"
    );
    assert_eq!(registry.len(), 1);
    Ok(())
}
