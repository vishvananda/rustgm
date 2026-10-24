//! Forest data queries are independent of source proof/execution authority.
use super::{BinderIndex, HeaderIndex, Work, g, s};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn actual() -> Result<s::SourceRows> {
    super::super::tests::actual()
}
fn forest(
    rows: &mut s::SourceRows,
    depth: usize,
    queries: usize,
) -> Result<(g::BinderRef, g::BinderRef, usize)> {
    let header = rows
        .functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("clone_from")
        })
        .ok_or("actual Clone provided header")?;
    let root = header.original.binder;
    let owner = header.declaration;
    let mut parent = root;
    // Typed synthetic Clause rows exercise the data forest only. No Checked,
    // semantic environment, accepted payload or original body is manufactured.
    for _ in 0..depth {
        let reference = g::BinderRef(u32::try_from(rows.binders.len())?);
        rows.binders.push(s::Binder {
            owner,
            parent: Some(parent),
            kind: s::BinderKind::Clause,
            count: 1,
        });
        parent = reference;
    }
    let first = rows.regions.len();
    for _ in 0..queries {
        rows.regions.push(s::Region::Bound {
            binder: parent,
            depth: u16::try_from(depth)?,
            slot: 0,
        });
    }
    Ok((root, parent, first))
}

#[test]
fn sibling_width_preserves_constant_query_work_and_distinct_preorder_intervals() -> Result {
    let original = actual()?;
    for width in [8, 32, 128] {
        let mut rows = original.clone();
        let (root, _, _) = forest(&mut rows, 1, 0)?;
        let owner = rows.binders[root.0 as usize].owner;
        let mut siblings = Vec::new();
        for _ in 0..width {
            let reference = g::BinderRef(u32::try_from(rows.binders.len())?);
            rows.binders.push(s::Binder {
                owner,
                parent: Some(root),
                kind: s::BinderKind::Clause,
                count: 1,
            });
            siblings.push(reference);
        }
        let mut work = Work::default();
        let headers = HeaderIndex::new(&rows, 0, &mut work)?;
        let index = BinderIndex::new(&headers, &mut work)?;
        let before = work.steps;
        for pair in siblings.windows(2) {
            assert!(index.ancestor(root, pair[0], &mut work)?);
            assert!(!index.ancestor(pair[0], pair[1], &mut work)?);
            assert!(!index.ancestor(pair[1], pair[0], &mut work)?);
        }
        assert_eq!(work.steps - before, (width - 1) * 9);
        println!(
            "binder-siblings={width} query-work={} indexed-frames={}",
            work.steps - before,
            rows.binders.len()
        );
    }
    Ok(())
}

#[test]
fn depth_and_query_width_use_one_forest_traversal_without_multiplied_walks() -> Result {
    let original = actual()?;
    let bound = original
        .regions
        .iter()
        .filter(|region| matches!(region, s::Region::Bound { .. }))
        .count();
    let mut observed = [[0_usize; 3]; 3];
    for (d, depth) in [8, 32, 128].into_iter().enumerate() {
        for (q, queries) in [8, 32, 128].into_iter().enumerate() {
            let mut rows = original.clone();
            let (root, leaf, first) = forest(&mut rows, depth, queries)?;
            let mut work = Work::default();
            let headers = HeaderIndex::new(&rows, 0, &mut work)?;
            let before = work.steps;
            let index = BinderIndex::new(&headers, &mut work)?;
            observed[d][q] = work.steps - before;
            assert_eq!(index.queries, bound + queries);
            assert_eq!(index.max_depth, depth + 1);
            assert!(index.ancestor(root, leaf, &mut work)?);
            assert!(!index.ancestor(leaf, root, &mut work)?);
            for region in first..rows.regions.len() {
                assert_eq!(
                    index.target(s::RegionRef(u32::try_from(region)?), &mut work)?,
                    Some(root)
                );
            }
            println!(
                "binder-depth={depth} queries={queries} work={} indexed={} retained={} peak={} scratch-envelope={}",
                observed[d][q],
                index.queries,
                index.bytes,
                index.peak,
                index.peak.saturating_sub(index.bytes)
            );
        }
    }
    for d in 1..3 {
        for q in 1..3 {
            assert_eq!(
                observed[d][q] - observed[d][q - 1],
                observed[0][q] - observed[0][q - 1]
            );
            assert_eq!(
                observed[d][q] - observed[d - 1][q],
                observed[d][0] - observed[d - 1][0]
            );
        }
    }
    Ok(())
}

#[test]
fn malformed_ancestor_depth_slot_cycle_and_foreign_owner_reject_before_publication() -> Result {
    let original = actual()?;
    for case in 0..4 {
        let mut rows = original.clone();
        let (_, leaf, query) = forest(&mut rows, 8, 1)?;
        match case {
            0 => {
                rows.regions[query] = s::Region::Bound {
                    binder: leaf,
                    depth: 100,
                    slot: 0,
                }
            }
            1 => {
                rows.regions[query] = s::Region::Bound {
                    binder: leaf,
                    depth: 0,
                    slot: u32::MAX,
                }
            }
            2 => rows.binders[leaf.0 as usize].parent = Some(leaf),
            _ => {
                let owner = rows.binders[leaf.0 as usize].owner;
                rows.binders[leaf.0 as usize].owner = rows
                    .functions
                    .iter()
                    .find(|header| header.declaration != owner)
                    .ok_or("foreign function")?
                    .declaration;
            }
        }
        let mut work = Work::default();
        let headers = HeaderIndex::new(&rows, 0, &mut work)?;
        let before = work.steps;
        let Err(error) = BinderIndex::new(&headers, &mut work) else {
            return Err(format!("case{case} unexpectedly returned an index").into());
        };
        assert!(work.steps > before);
        println!(
            "binder-opponent={case} attempted={} header-retained={} header-peak={} published=false error={error}",
            work.steps - before,
            headers.retained_bytes(),
            headers.peak_bytes()
        );
    }
    Ok(())
}
