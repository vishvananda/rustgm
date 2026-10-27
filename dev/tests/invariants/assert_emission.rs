//! Actual conditional AST assembly keeps ordinary calls and lazy message tokens.
use super::*;
use crate::{
    expand::{
        builtin::delegation::Site,
        marks::{ExpansionId, Marks, Transparency},
        registry::Registry,
        stream::Location,
    },
    support::edition::Edition,
    syntax::{
        ast::{BlockFlavor, Unary},
        edition_context::WithEditions,
    },
};
fn generated(
    text: &str,
    charge: &Charge<'_>,
) -> std::result::Result<(crate::expand::builtin::delegation::Delegated, usize), LexError> {
    let input = input(text)?;
    let registry = Registry::default();
    let mut marks = Marks::default();
    let context = marks
        .apply(
            SyntaxContextId::ROOT,
            ExpansionId(1),
            Transparency::Opaque,
            SyntaxContextId::ROOT,
        )
        .map_err(|message| error(0, message))?;
    let view = registry.view(|_| Some(text));
    let inputs = WithEditions::new(&view, |_| Ok(Edition::Rust2024));
    super::super::build(
        &super::super::Request {
            site: Site {
                source: Source {
                    file: FileId(3),
                    text,
                },
                original: Location {
                    file: FileId(3),
                    span: Span {
                        start: 0,
                        end: u32::try_from(text.len())
                            .map_err(|_| error(0, "test input too long"))?,
                    },
                },
                introduction: ExpansionId(1),
                call_context: context,
                path_context: context,
                grammar: crate::syntax::ast::MacroContext::Expression,
            },
            edition: Edition::Rust2024,
            input: &input,
            registry: &registry,
            inputs: &inputs,
            config: &Config::default(),
            charge,
        },
        |_| Ok((SyntaxContextId::ROOT, false)),
    )
}
#[test]
fn default_and_custom_assertions_build_once_with_real_paths_and_lazy_suffixes() -> Result {
    for text in ["first()", "first(),", "first(), \"{}\", message()"] {
        let (generated, calls) = generated(text, &|_, _| Ok(()))?;
        assert_eq!(calls, 1);
        let ast = &generated.syntax.ast;
        let Kind::If {
            condition,
            then,
            otherwise: None,
        } = ast.node(ast.root).kind
        else {
            return Err(error(0, "missing real conditional"));
        };
        assert!(matches!(
            ast.node(condition).kind,
            Kind::Unary { op: Unary::Not, .. }
        ));
        let Kind::Block {
            flavor: BlockFlavor::Plain,
            tail: Some(failure),
            ..
        } = ast.node(then).kind
        else {
            return Err(error(0, "missing conditional panic tail"));
        };
        let names: Vec<_> = ast
            .nodes
            .iter()
            .filter_map(|node| match node.kind {
                Kind::Name(span, _) => generated
                    .stream
                    .source
                    .get(span.start as usize..span.end as usize),
                _ => None,
            })
            .collect();
        assert_eq!(names.iter().filter(|&&name| name == "first").count(), 1);
        assert!(
            ast.nodes
                .iter()
                .any(|node| matches!(node.kind, Kind::CrateRoot(0)))
        );
        if text.contains("message") {
            assert!(names.contains(&"panic_2021"));
            assert!(
                !names.contains(&"message"),
                "message must remain unparsed for the real panic macro"
            );
            assert!(matches!(ast.node(failure).kind, Kind::Macro { .. }));
            assert!(generated.stream.registered);
        } else {
            assert!(names.contains(&"panicking") && names.contains(&"panic"));
            assert!(matches!(ast.node(failure).kind, Kind::Call { .. }));
            assert!(ast.literals.iter().any(|value|matches!(value,crate::literal::Value::String(bytes) if bytes.as_ref()==b"assertion failed: first()")));
        }
    }
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn assert_emission_payment_failure_keeps_source_and_registry_unpublished() -> Result {
    let text = "{ let x=1u8; x==2u8 }, \"{}\", message()";
    let attempts = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    generated(text, &|work, storage| {
        attempts.set(attempts.get() + work);
        bytes.set(bytes.get() + storage);
        Ok(())
    })?;
    let required = attempts.get();
    assert!(required > 1 && bytes.get() > 0);
    for limit in [0, 1, required / 2, required - 1] {
        let paid = Cell::new(0usize);
        assert!(
            generated(text, &|work, _| {
                paid.set(paid.get() + work);
                if paid.get() > limit {
                    Err(error(0, "test exhausted Assert budget"))
                } else {
                    Ok(())
                }
            })
            .is_err(),
            "{limit}"
        );
        assert!(paid.get() > limit);
    }
    Ok(())
}
