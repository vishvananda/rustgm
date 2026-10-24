//! Pure typed construction checks token/spacing preservation, not role authority.
use super::*;
use crate::{resolve::unit::FileId, syntax::token_tree::Forest};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn delegation_preserves_classified_input_spacing_and_never_parses_a_source_image() -> TestResult {
    let text = "tag[nested]{tokens}";
    let forest = Forest::parse(text)?;
    let input = Input::from_forest(&forest, forest.roots())?;
    let registry = Registry::default();
    let site = Site {
        source: Source {
            file: FileId(0),
            text,
        },
        original: Location {
            file: FileId(0),
            span: Span {
                start: 0,
                end: u32::try_from(text.len())?,
            },
        },
        introduction: ExpansionId(1),
        call_context: SyntaxContextId::ROOT,
        path_context: SyntaxContextId::ROOT,
        grammar: MacroContext::Expression,
    };
    let generated = build(
        &site,
        Handler::Panic2021,
        &input,
        &registry,
        &Config::default(),
        |token| Ok((SyntaxContextId::ROOT, token.end as usize != text.len())),
    )?;
    assert!(generated.stream.registered);
    assert!(generated.stream.opaque.is_empty());
    assert_eq!(registry.len(), 0);
    assert_eq!(generated.stream.tokens.len(), input.tokens.len() + 2);
    for (before, after) in input.tokens.iter().zip(&generated.stream.tokens[1..]) {
        assert_eq!(before.kind, after.kind);
        assert_eq!(
            text.get(before.start as usize..before.end as usize),
            generated
                .stream
                .source
                .get(after.start as usize..after.end as usize)
        );
        let origin = generated
            .stream
            .origin(after.start)
            .ok_or("actual forwarded origin")?;
        assert_eq!(
            origin.original.span,
            Span {
                start: before.start,
                end: before.end
            }
        );
        assert_eq!(origin.joint, Some(before.end as usize != text.len()));
        assert_eq!(origin.context, Some(SyntaxContextId::ROOT));
        assert_eq!(origin.introduction, ExpansionId(0));
    }
    assert!(matches!(
        generated.syntax.ast.nodes[0].kind,
        Kind::CrateRoot(0)
    ));
    assert!(matches!(
        generated.syntax.ast.node(generated.syntax.ast.root).kind,
        Kind::Macro { .. }
    ));
    assert!(generated.payload_bytes() > 0);
    Ok(())
}
