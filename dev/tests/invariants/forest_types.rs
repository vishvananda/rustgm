//! Only selected macro forests move; opaque payloads keep their real registry owner.
use super::*;
use crate::{
    expand::{
        config::Config,
        macros::session,
        marks::{ExpansionId, SyntaxContextId},
        registry::Registry,
        stream::{Environment, Source, Stream},
    },
    support::source::FileId,
    syntax::{
        expression::fragment::{self, FragmentKind},
        interpolation::{Inputs, Nonterminal, Splice},
        item::ItemContext,
        token_tree::{Span, TreeKind},
    },
    test_source,
};
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/derive-type-macros/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
struct Captured {
    source: String,
    stream: Stream,
    registry: Registry,
    parsed: ParsedSyntax,
    root: NodeId,
}
fn captured() -> TestResult<Captured> {
    let source = input("original-macro")?;
    let mut report = session::analyze(&source, &Config::default()).map_err(|e| e.message)?;
    assert_eq!(report.invocations.len(), 1);
    let mut call = report.invocations.remove(0);
    let mut stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &Environment {
            definition: Source {
                file: FileId(0),
                text: &source,
            },
            invocation: Source {
                file: FileId(0),
                text: &source,
            },
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(1),
            crate_id: 0,
            edition: crate::support::edition::Edition::Rust2024,
        },
        &report.names,
    )
    .map_err(|e| e.message)?;
    let mut registry = Registry::default();
    registry
        .adopt(
            &mut stream,
            &mut call.matched,
            FileId(0),
            &source,
            &call.input,
        )
        .map_err(|e| e.message)?;
    let ast = fragment::parse_expansion_with_inputs(
        &stream.source,
        &stream.tokens,
        MacroContext::Items(ItemContext::Module),
        &registry.view(|file| (file == FileId(0)).then_some(source.as_str())),
    )
    .map_err(|e| e.message)?;
    let parsed = ParsedSyntax::own(&stream.source, ast).map_err(|e| e.message)?;
    let root = parsed
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            matches!(
                node.kind,
                Kind::Macro {
                    context: MacroContext::Type,
                    ..
                }
            )
            .then(|| NodeId::new(u32::try_from(index).unwrap_or(u32::MAX)))
        })
        .ok_or("actual original type macro")?;
    Ok(Captured {
        source,
        stream,
        registry,
        parsed,
        root,
    })
}
#[test]
fn original_opaque_pattern_macro_keeps_capture_identity_through_fresh_grafts() -> TestResult {
    let actual = captured()?;
    let copied = field(&actual.parsed.ast, actual.root, &|_, _| Ok(())).map_err(|e| e.message)?;
    assert_eq!(copied.ast.nodes.len(), 2);
    assert_eq!(copied.ast.captures.len(), 1);
    assert!(copied.ast.literals.is_empty());
    assert_eq!(copied.literals.decoded_bytes(), 0);
    let original = actual.parsed.ast.node(actual.root).span;
    let spelling = &actual.stream.source[original.start as usize..original.end as usize];
    let destination = format!("first {spelling} second {spelling}");
    let mut target = crate::syntax::ast::Ast::default();
    let mut roots = Vec::new();
    for start in [6, 6 + spelling.len() + 8] {
        let splice = Splice {
            fragment: Nonterminal::Syntax {
                kind: FragmentKind::Type,
                parsed: &copied,
            },
            original_source: &actual.stream.source,
            original,
            destination: Span {
                start: u32::try_from(start)?,
                end: u32::try_from(start + spelling.len())?,
            },
        };
        roots.push(
            splice
                .import(&destination, &mut target)
                .map_err(|e| e.message)?,
        );
    }
    assert_ne!(roots[0], roots[1]);
    assert_eq!(target.captures.len(), 2);
    for (ordinal, root) in roots.into_iter().enumerate() {
        let Kind::Macro {
            path,
            capture,
            context: MacroContext::Type,
        } = target.node(root).kind
        else {
            return Err("lost actual type macro".into());
        };
        assert_eq!(capture as usize, ordinal);
        assert!(matches!(target.node(path).kind, Kind::Name(_, None)));
        let forest = &target.captures[capture as usize];
        let token = forest
            .nodes()
            .iter()
            .find_map(|tree| match tree.kind {
                TreeKind::Token(kind @ crate::lex::token::Kind::Interpolated(_)) => {
                    Some(crate::lex::token::Token {
                        kind,
                        start: tree.span.start,
                        end: tree.span.end,
                    })
                }
                _ => None,
            })
            .ok_or("persistent opaque range")?;
        let view = actual
            .registry
            .view(|file| (file == FileId(0)).then_some(actual.source.as_str()));
        let opaque = view
            .get(token)
            .map_err(|e| e.message)?
            .ok_or("actual registry payload")?;
        let Nonterminal::Syntax { parsed, .. } = opaque.fragment else {
            return Err("actual pattern payload".into());
        };
        assert!(matches!(
            parsed.ast.node(parsed.ast.root).kind,
            Kind::PatternRange {
                start: Some(_),
                end: Some(_),
                inclusive: true
            }
        ));
        assert!(actual.registry.view(|_| None).get(token).is_err());
        assert_eq!(
            opaque.destination,
            Span {
                start: token.start,
                end: token.end
            }
        );
    }
    Ok(())
}
#[test]
fn selected_capture_context_bounds_and_failed_payments_never_publish_a_copy() -> TestResult {
    let actual = captured()?;
    let source = &actual.parsed.ast;
    let before = (source.nodes.len(), source.captures.len());
    for limit in 1..=3 {
        let attempts = Cell::new(0);
        let denied = |_, _| {
            attempts.set(attempts.get() + 1);
            if attempts.get() == limit {
                Err(error(0, "denied selected capture"))
            } else {
                Ok(())
            }
        };
        assert!(field(source, actual.root, &denied).is_err());
        assert_eq!(attempts.get(), limit);
        assert_eq!((source.nodes.len(), source.captures.len()), before);
    }
    for mode in 0..4 {
        let mut owned = field(source, actual.root, &|_, _| Ok(())).map_err(|e| e.message)?;
        let root = owned.ast.root;
        let Kind::Macro { path, .. } = owned.ast.node(root).kind else {
            return Err("macro".into());
        };
        match mode {
            0 => {
                if let Kind::Macro { capture, .. } = &mut owned.ast.nodes[root.get() as usize].kind
                {
                    *capture = u32::MAX;
                }
            }
            1 => {
                if let Kind::Macro { context, .. } = &mut owned.ast.nodes[root.get() as usize].kind
                {
                    *context = MacroContext::Expression;
                }
            }
            2 => {
                if let Kind::Macro { path, .. } = &mut owned.ast.nodes[root.get() as usize].kind {
                    *path = NodeId::new(u32::MAX - 1);
                }
            }
            _ => {
                owned.ast.nodes[root.get() as usize].kind = Kind::TypePattern {
                    base: path,
                    constraint: crate::syntax::ast::TypeConstraint::NonNull,
                }
            }
        }
        assert!(
            field(&owned.ast, root, &|_, _| Ok(())).is_err(),
            "mode={mode}"
        );
    }
    let copied = field(source, actual.root, &|_, _| Ok(())).map_err(|e| e.message)?;
    let original = source.node(actual.root).span;
    let spelling = &actual.stream.source[original.start as usize..original.end as usize];
    let mut target = crate::syntax::ast::Ast::default();
    let bad = Splice {
        fragment: Nonterminal::Syntax {
            kind: FragmentKind::Type,
            parsed: &copied,
        },
        original_source: &actual.stream.source,
        original: Span {
            end: original.end - 1,
            ..original
        },
        destination: Span {
            start: 0,
            end: u32::try_from(spelling.len() - 1)?,
        },
    };
    assert!(bad.import(spelling, &mut target).is_err());
    assert!(target.nodes.is_empty() && target.captures.is_empty());
    Ok(())
}
#[test]
fn selected_forest_work_is_linear_and_does_not_copy_unrelated_source_arenas() -> TestResult {
    let mut previous = None;
    for count in [8, 32, 128] {
        let source = format!("struct Holder(macro_ty!(({})));", "u32,".repeat(count));
        let unit = crate::resolve::unit::Unit::parse(
            crate::support::source::SourceFile::new(source.into_bytes())?,
            &Config::default(),
            &mut crate::support::names::Names::default(),
        )
        .map_err(|e| e.message)?;
        let root = unit
            .configured
            .ast
            .nodes
            .iter()
            .enumerate()
            .find_map(|(i, n)| {
                matches!(n.kind, Kind::Macro { .. })
                    .then(|| NodeId::new(u32::try_from(i).unwrap_or(u32::MAX)))
            })
            .ok_or("actual macro")?;
        let usage = Cell::new((0_usize, 0_usize));
        let copy = field(&unit.configured.ast, root, &|work, bytes| {
            let old = usage.get();
            usage.set((old.0 + work, old.1 + bytes));
            Ok(())
        })
        .map_err(|e| e.message)?;
        assert_eq!(copy.ast.nodes.len(), 2);
        assert_eq!(
            copy.ast.captures[0].nodes(),
            unit.configured.ast.captures[0].nodes()
        );
        if let Some((work, bytes)) = previous {
            assert!(usage.get().0 <= work * 5);
            assert!(usage.get().1 <= bytes * 5);
        }
        previous = Some(usage.get());
        eprintln!("type macro tokens={count} copywork/bytes={:?}", usage.get());
    }
    assert_eq!(size_of::<Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
