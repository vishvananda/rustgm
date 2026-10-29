//! Selected constant type children retain parsed values, deferred tokens and genuine provenance.
use super::*;
use crate::{
    expand::{config::Config, crate_syntax},
    syntax::{
        ast::TypeFlavor,
        expression,
        interpolation::{Nonterminal, Splice},
    },
};
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub(super) fn selected(source: &str) -> TestResult<(crate::resolve::unit::Unit, NodeId)> {
    typed(source, TypeFlavor::Tuple)
}
pub(super) fn typed(
    source: &str,
    flavor: TypeFlavor,
) -> TestResult<(crate::resolve::unit::Unit, NodeId)> {
    let unit = crate::resolve::unit::Unit::parse(
        crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
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
        .rev()
        .find_map(|(i, n)| {
            matches!(
                n.kind,
                Kind::Type {
                    flavor: actual,
                    ..
                } if actual == flavor
            )
            .then(|| NodeId::new(u32::try_from(i).unwrap_or(u32::MAX)))
        })
        .ok_or("selected field type")?;
    Ok((unit, root))
}
#[test]
fn inline_literals_compound_children_are_selected_paid_and_linear() -> TestResult {
    use std::fmt::Write as _;
    let mut measurements = Vec::new();
    for count in [8, 32, 128] {
        let mut source = String::from("struct Holder((");
        for _ in 0..count {
            source.push_str("[u8;{(1usize+3usize)/2usize}],");
        }
        source.push_str("));");
        let (unit, root) = selected(&source)?;
        let mut row = None;
        for unrelated in [0, 1024] {
            let mut input = String::new();
            for i in 0..unrelated {
                write!(&mut input, "const NOISE{i}:usize={i};")?;
            }
            input.push_str(&source);
            let (noisy, key) = selected(&input)?;
            let work = Cell::new((0_usize, 0_usize));
            let parsed = field(&noisy.configured.ast, key, &|w, b| {
                let (a, c) = work.get();
                work.set((a + w, c + b));
                Ok(())
            })
            .map_err(|e| e.message)?;
            assert_eq!(parsed.ast.literals.len(), 3 * count);
            assert_eq!(parsed.literals.decoded_bytes(), 0);
            assert!(
                parsed
                    .ast
                    .literals
                    .iter()
                    .all(|v| matches!(v, StoredValue::Integer(_)))
            );
            let magnitudes: Vec<_> = parsed
                .ast
                .literals
                .iter()
                .filter_map(|value| {
                    if let StoredValue::Integer(value) = value {
                        Some(value.magnitude)
                    } else {
                        None
                    }
                })
                .collect();
            assert!(
                magnitudes
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .all(|values| *values == [1, 3, 2])
            );
            if let Some(expected) = row {
                assert_eq!(work.get(), expected);
            } else {
                row = Some(work.get());
            }
            println!(
                "count={count} unrelated={unrelated} nodes={} values={} work/bytes={:?}",
                parsed.ast.nodes.len(),
                parsed.ast.literals.len(),
                work.get()
            );
        }
        let parsed = field(&unit.configured.ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
        assert_eq!(parsed.ast.literals.len(), 3 * count);
        measurements.push(row.ok_or("measurement")?);
    }
    assert!(
        measurements[2].0 - measurements[1].0 <= 4 * (measurements[1].0 - measurements[0].0) + 16
    );
    assert!(measurements[2].1 <= 5 * measurements[1].1);
    assert_eq!(size_of::<Node>(), 36);
    Ok(())
}
#[test]
fn literal_pool_indices_failed_growth_and_owned_payload_rejection_are_exact() -> TestResult {
    let source = "struct S(([u8;2],));";
    let (unit, root) = selected(source)?;
    let parsed = field(&unit.configured.ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    assert_eq!(parsed.ast.literals.len(), 1);
    let attempts = Cell::new(0);
    let mut target = Syntax::default();
    assert!(
        derive_constants::literal(
            &parsed.ast,
            0,
            &mut target,
            &|_, b| {
                attempts.set(attempts.get() + 1);
                if b > 0 {
                    Err(error(0, "declared literal growth denial"))
                } else {
                    Ok(())
                }
            },
            0
        )
        .is_err()
    );
    assert!(target.literals.is_empty());
    assert!(attempts.get() >= 2);
    target.literals.reserve(4);
    let warm = Cell::new((0, 0));
    derive_constants::literal(
        &parsed.ast,
        0,
        &mut target,
        &|w, b| {
            let (work, bytes) = warm.get();
            warm.set((work + w, bytes + b));
            Ok(())
        },
        0,
    )
    .map_err(|e| e.message)?;
    assert_eq!(warm.get(), (2, 0));
    assert_eq!(target.literals, parsed.ast.literals);
    assert!(
        derive_constants::literal(&parsed.ast, u32::MAX, &mut target, &|_, _| Ok(()), 0).is_err()
    );
    assert_eq!(target.literals.len(), 1);
    let mut bad = parsed;
    bad.ast.literals[0] =
        StoredValue::String(crate::literal::storage::Bytes::Decoded { start: 0, end: 3 });
    assert!(
        field(&bad.ast, bad.ast.root, &|_, _| Ok(()))
            .err()
            .ok_or("stale byte pool admitted")?
            .message
            .contains("byte-pool owner")
    );
    Ok(())
}
#[test]
fn deferred_field_counts_materialize_only_at_the_actual_graft() -> TestResult {
    let source = "struct S(([u8;2],));";
    let unconfigured = expression::parse_crate(source).map_err(|e| e.message)?;
    let unconfigured = ParsedSyntax::own(source, unconfigured).map_err(|e| e.message)?;
    let root = unconfigured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(i, n)| {
            matches!(
                n.kind,
                Kind::Type {
                    flavor: TypeFlavor::Array,
                    ..
                }
            )
            .then(|| NodeId::new(u32::try_from(i).unwrap_or(u32::MAX)))
        })
        .ok_or("array")?;
    let copied = field(&unconfigured.ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    assert_eq!(copied.ast.pending.literals.len(), 1);
    assert!(copied.ast.literals.is_empty());
    let span = copied.ast.node(copied.ast.root).span;
    let mut ast = crate::syntax::ast::Ast::default();
    Splice {
        fragment: Nonterminal::Syntax {
            kind: expression::fragment::FragmentKind::Type,
            parsed: &copied,
        },
        original_source: source,
        original: span,
        destination: span,
    }
    .import(source, &mut ast)
    .map_err(|e| e.message)?;
    let configured =
        crate_syntax::configure(source, ast, &Config::default()).map_err(|e| e.message)?;
    assert_eq!(configured.ast.literals.len(), 1);
    assert!(
        configured
            .ast
            .nodes
            .iter()
            .all(|n| !matches!(n.kind, Kind::DeferredLiteral(_)))
    );
    Ok(())
}
#[test]
fn pending_payload_is_paid_once_for_the_actual_selected_splice() -> TestResult {
    use crate::expand::builtin::choice::syntax_payload;
    use std::mem::size_of_val;
    let empty = Syntax::<StoredValue>::default();
    assert_eq!(syntax_payload(&empty), 0);
    let source = "#[cfg(all())] struct S([u8;2]);";
    let ast = expression::parse_crate(source).map_err(|e| e.message)?;
    let mut parsed = ParsedSyntax::own(source, ast).map_err(|e| e.message)?;
    assert!(!parsed.ast.pending.owners.is_empty());
    assert_eq!(parsed.ast.pending.literals.len(), 1);
    let before = syntax_payload(&parsed.ast);
    let owners = std::mem::take(&mut parsed.ast.pending.owners);
    let literals = std::mem::take(&mut parsed.ast.pending.literals);
    assert_eq!(
        before - syntax_payload(&parsed.ast),
        size_of_val(owners.as_slice()) + size_of_val(literals.as_slice())
    );
    parsed.ast.pending.owners = owners;
    parsed.ast.pending.literals = literals;
    assert_eq!(syntax_payload(&parsed.ast), before);
    Ok(())
}
#[test]
fn original_stdarch_types_macro_has_a_real_captured_array_literal() -> TestResult {
    let source =
        crate::test_source::fixture("pa41/tests/spec/stdarch-array-derives/macro-input.rs")?;
    let (stream, _registry, parsed) = super::macro_tests::transcribed(&source)?;
    let whole = parsed.ast.node(parsed.ast.root).span;
    let mut ast = crate::syntax::ast::Ast::default();
    Splice {
        fragment: Nonterminal::Syntax {
            kind: expression::fragment::FragmentKind::Item,
            parsed: &parsed,
        },
        original_source: &stream.source,
        original: whole,
        destination: whole,
    }
    .import(&stream.source, &mut ast)
    .map_err(|e| e.message)?;
    let configured =
        crate_syntax::configure(&stream.source, ast, &Config::default()).map_err(|e| e.message)?;
    let stored = ParsedSyntax::own(&stream.source, configured.ast).map_err(|e| e.message)?;
    let root = stored
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(i, n)| {
            matches!(
                n.kind,
                Kind::Type {
                    flavor: TypeFlavor::Array,
                    ..
                }
            )
            .then(|| NodeId::new(u32::try_from(i).unwrap_or(u32::MAX)))
        })
        .ok_or("actual stdarch field array")?;
    let Kind::Type { elements, .. } = stored.ast.node(root).kind else {
        return Err("array".into());
    };
    let [base, count] = stored.ast.list(elements) else {
        return Err("array children".into());
    };
    assert!(matches!(stored.ast.node(*base).kind, Kind::Name(..)));
    let Kind::Literal(index) = stored.ast.node(*count).kind else {
        return Err(format!("actual count kind {:?}", stored.ast.node(*count).kind).into());
    };
    let value = *stored
        .ast
        .literals
        .get(index as usize)
        .ok_or("actual literal pool")?;
    assert!(matches!(value,StoredValue::Integer(n) if n.magnitude==2&&n.suffix.is_none()));
    let copied = field(&stored.ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    assert_eq!(copied.ast.literals, vec![value]);
    let span = stored.ast.node(*count).span;
    assert!(
        stream
            .origins
            .iter()
            .any(|o| o.generated.start <= span.start && o.generated.end >= span.end)
    );
    println!(
        "actual stdarch array count Kind::Literal({index}) span={span:?} value={value:?}; selected_nodes={} pool={}",
        copied.ast.nodes.len(),
        copied.ast.literals.len()
    );
    Ok(())
}
