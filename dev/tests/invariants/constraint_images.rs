//! Equality nodes retain their selected subtree, original names and paid children.
use super::*;
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn parsed(count: usize) -> TestResult<(crate::resolve::unit::Unit, NodeId)> {
    use std::fmt::Write as _;
    let mut source = String::from("struct Holder<T>(Carrier<");
    for index in 0..count {
        write!(&mut source, "Item{index}=Wrap<T>,")?;
    }
    source.push_str(">);");
    let unit = crate::resolve::unit::Unit::parse(
        crate::support::source::SourceFile::new(source.into_bytes())?,
        &crate::expand::config::Config::default(),
        &mut crate::support::names::Names::default(),
    )
    .map_err(|error| error.message)?;
    let root = unit
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            let Kind::Applied { base, .. } = node.kind else {
                return None;
            };
            let span = unit.configured.ast.node(base).span;
            (unit
                .source
                .text()
                .get(span.start as usize..span.end as usize)
                == Some("Carrier"))
            .then(|| NodeId::new(u32::try_from(index).unwrap_or(u32::MAX)))
        })
        .ok_or("actual Carrier application")?;
    Ok((unit, root))
}

#[test]
fn associated_equalities_keep_node_identities_and_paid_failure_linear() -> TestResult {
    let mut previous = None;
    for count in [8, 32, 128] {
        let (unit, root) = parsed(count)?;
        let source = &unit.configured.ast;
        let usage = Cell::new((0_usize, 0_usize));
        let charge = |work: usize, bytes: usize| {
            let old = usage.get();
            usage.set((old.0 + work, old.1 + bytes));
            Ok(())
        };
        let copied = field(source, root, &charge).map_err(|error| error.message)?;
        assert_eq!(
            copied
                .ast
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, Kind::Constraint { .. }))
                .count(),
            count
        );
        let span = source.node(root).span;
        let expected: Vec<_> = source
            .nodes
            .iter()
            .filter_map(|node| match node.kind {
                Kind::Name(span, name)
                    if span.start >= source.node(root).span.start
                        && span.end <= source.node(root).span.end =>
                {
                    Some((span, name))
                }
                _ => None,
            })
            .collect();
        let actual: Vec<_> = copied
            .ast
            .nodes
            .iter()
            .filter_map(|node| match node.kind {
                Kind::Name(span, name) => Some((span, name)),
                _ => None,
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(copied.ast.node(copied.ast.root).span, span);
        for node in &copied.ast.nodes {
            if let Kind::Constraint {
                target,
                value,
                bounds,
            } = node.kind
            {
                assert!(matches!(copied.ast.node(target).kind, Kind::Name(..)));
                assert!(value.is_some());
                assert_eq!(bounds.start, bounds.end);
            }
        }
        let paid = usage.get();
        if let Some((work, bytes)) = previous {
            assert!(paid.0 <= work * 5);
            assert!(paid.1 <= bytes * 5);
        }
        previous = Some(paid);
        failed_copies(source, root, &charge)?;
        eprintln!(
            "copied equalities={count} nodes={} work/bytes={paid:?}",
            copied.ast.nodes.len()
        );
    }
    assert_eq!(size_of::<Node>(), 36);
    Ok(())
}

fn failed_copies(source: &Syntax<StoredValue>, root: NodeId, charge: &Charge<'_>) -> TestResult {
    for limit in [1, 6] {
        let attempts = Cell::new(0_usize);
        let denied = |_: usize, _: usize| {
            attempts.set(attempts.get() + 1);
            if attempts.get() == limit {
                Err(error(0, "denied copied equality work"))
            } else {
                Ok(())
            }
        };
        assert!(field(source, root, &denied).is_err());
        assert_eq!(attempts.get(), limit);
    }
    let mut malformed = field(source, root, charge).map_err(|error| error.message)?;
    let node = malformed
        .ast
        .nodes
        .iter_mut()
        .find(|node| matches!(node.kind, Kind::Constraint { .. }))
        .ok_or("constraint")?;
    if let Kind::Constraint { value, .. } = &mut node.kind {
        *value = Some(NodeId::new(u32::MAX - 1));
    }
    assert!(field(&malformed.ast, malformed.ast.root, charge).is_err());
    Ok(())
}
