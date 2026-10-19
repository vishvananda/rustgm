//! Mutual lifetime entailment is independent of inference obligations.
use super::super::{Error, Graph, Kind, Region};

#[test]
fn components_follow_only_assumptions_and_include_implicit_static_edges() -> Result<(), Error> {
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    let c = graph.region(Kind::Universal { universe: 1 })?;
    let d = graph.region(Kind::Variable { universe: 0 })?;
    graph.assume(a, b)?;
    graph.assume(b, a)?;
    graph.assume(c, a)?;
    graph.require(a, c, 0)?;
    graph.require(d, a, 0)?;
    graph.require(a, d, 0)?;
    let classes = graph.assumption_classes()?;
    assert_eq!(classes[a.0 as usize], classes[b.0 as usize]);
    assert_ne!(classes[a.0 as usize], classes[c.0 as usize]);
    assert_ne!(classes[a.0 as usize], classes[d.0 as usize]);
    graph.assume(a, Region::STATIC)?;
    let classes = graph.assumption_classes()?;
    for region in [a, b, c] {
        assert_eq!(classes[region.0 as usize], Region::STATIC);
    }
    assert_ne!(classes[d.0 as usize], Region::STATIC);
    Ok(())
}

#[test]
fn established_identity_requires_proof_and_never_identifies_local_borrow_extents()
-> Result<(), Error> {
    for related in [false, true] {
        let mut graph = Graph::new(1_000);
        let a = graph.region(Kind::Variable { universe: 0 })?;
        let b = graph.region(Kind::Variable { universe: 0 })?;
        for region in [a, b] {
            graph.require(region, Region::LOCAL, 0)?;
            graph.require(Region::LOCAL, region, 0)?;
        }
        if related {
            graph.require(a, b, 0)?;
            graph.require(b, a, 0)?;
        }
        assert_ne!(
            graph.assumption_classes()?[a.0 as usize],
            graph.assumption_classes()?[b.0 as usize]
        );
        let solution = graph.solve()?;
        let classes = solution.established_classes()?;
        assert_eq!(classes[a.0 as usize] == classes[b.0 as usize], related);
    }
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    graph.require(a, b, 0)?;
    graph.require(b, a, 0)?;
    assert!(
        graph.solve().is_err(),
        "unentailed named equalities cannot publish a solution"
    );
    Ok(())
}

#[test]
fn long_cycles_fit_a_linear_work_budget_without_recursive_traversal() -> Result<(), Error> {
    for count in [16, 128, 1_024, 16_384] {
        let mut graph = Graph::new(count * 4 + 10);
        let regions = (0..count)
            .map(|_| graph.region(Kind::Universal { universe: 0 }))
            .collect::<Result<Vec<_>, _>>()?;
        for pair in regions.windows(2) {
            graph.assume(pair[0], pair[1])?;
        }
        graph.assume(regions[count - 1], regions[0])?;
        let classes = graph.assumption_classes()?;
        assert_eq!(classes[0], Region::STATIC);
        assert!(regions.iter().all(|r| classes[r.0 as usize] == regions[0]));
    }
    Ok(())
}

#[test]
fn components_agree_with_exhaustive_transitive_closure() -> Result<(), Error> {
    for edges in 0_u32..512 {
        let mut graph = Graph::new(1_000);
        let mut reach = [[false; 4]; 4];
        let mut regions = vec![Region::STATIC];
        for _ in 0..3 {
            regions.push(graph.region(Kind::Universal { universe: 0 })?);
        }
        for (a, row) in reach.iter_mut().enumerate() {
            for (b, cell) in row.iter_mut().enumerate() {
                *cell = a == 0 || a == b;
                if a != 0 && b != a && edges & (1 << ((a - 1) * 3 + b - usize::from(b > a))) != 0 {
                    graph.assume(regions[a], regions[b])?;
                    *cell = true;
                }
            }
        }
        for via in 0..4 {
            for a in 0..4 {
                for b in 0..4 {
                    reach[a][b] |= reach[a][via] && reach[via][b];
                }
            }
        }
        let classes = graph.assumption_classes()?;
        for a in 0..4 {
            for b in 0..4 {
                assert_eq!(
                    classes[regions[a].0 as usize] == classes[regions[b].0 as usize],
                    reach[a][b] && reach[b][a]
                );
            }
        }
    }
    Ok(())
}
