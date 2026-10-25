//! Allocation summaries count owner nodes once, including shared diamonds.
use super::*;
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(count: usize) -> Result<(CrateIdentity, g::Rows)> {
    let mut source = String::new();
    for i in 0..count {
        write!(&mut source, "pub const fn id{i:08}<T:Copy>(x:T)->T{{x}}")?;
    }
    let report = crate::test_source::resolve(&source, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, interface) = crate::lower::instances::build_library_prepared(
        &report,
        &mut facts,
        CrateIdentity::new("graphs", "counter")?,
        prepared,
    )?;
    Ok((
        interface.identity().clone(),
        interface
            .generic_payload()
            .ok_or("missing actual provider rows")?
            .legacy_rows()?
            .clone(),
    ))
}
fn mapped(rows: &g::Rows) -> Result<Mapped> {
    let mut mapped = Mapped {
        nodes: Vec::new(),
        parameters: Vec::new(),
        graphs: (0..rows.declarations.len())
            .map(|_| Graph::default())
            .collect(),
        scalars: 0,
        round: Cell::new(0),
    };
    for &node in &rows.types {
        mapped.record(rows, node)?;
    }
    Ok(mapped)
}
#[test]
fn shared_diamond_bounds_ignore_repeated_edges_and_unrelated_declarations() -> Result {
    let mut observed = None;
    for count in [1, 32, 128] {
        let (owner, mut rows) = fixture(count)?;
        let function = rows.functions[0];
        let before = mapped(&rows)?.environment_bound(&rows, &function)?;
        let first = u32::try_from(rows.binders.len())?;
        let depth = 25;
        for index in 0..depth {
            rows.binders.push(g::Binder {
                owner: function.declaration,
                parent: Some(if index == 0 {
                    function.original_frame
                } else {
                    g::BinderRef(first + index - 1)
                }),
                kind: g::BinderKind::PublicFunction,
                count: 0,
            });
        }
        let leaf = rows.clauses[function.clauses.start as usize].subject;
        let mut child = leaf;
        for index in (0..depth).rev() {
            let start = u32::try_from(rows.roots.len())?;
            rows.roots.extend([child, child]);
            let root = g::TypeRef(u32::try_from(rows.types.len())?);
            rows.types.push(g::Type::Function(g::Signature {
                binder: g::BinderRef(first + index),
                parameters: g::Range {
                    start,
                    end: start + 2,
                },
                result: leaf,
                unsafe_: false,
                convention: g::Convention::Rust,
                variadic: false,
            }));
            child = root;
        }
        rows.clauses[function.clauses.start as usize].subject = child;
        let payload = g::Payload::new(&owner, rows, &mut g::Work::default())?;
        let rows = payload.legacy_rows()?;
        let mapped = mapped(rows)?;
        let bound = mapped.environment_bound(rows, &rows.functions[0])?;
        assert_eq!(
            bound,
            (before.0 + depth as usize, before.1 + 3 * depth as usize)
        );
        assert_eq!(bound, mapped.environment_bound(rows, &rows.functions[0])?);
        if let Some(previous) = observed {
            assert_eq!(bound, previous);
        }
        observed = Some(bound);
        eprintln!(
            "declarations={count} shared_depth={depth} nodes={} edges={}",
            bound.0, bound.1
        );
    }
    Ok(())
}
