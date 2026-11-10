//! Sparse alias propagation must preserve full-solver failures and richer consumers.
use super::*;
use crate::types::region_graph::Kind;
fn graph(edges: u8, assumptions: u8, younger: bool) -> Result<Graph, Error> {
    let mut graph = Graph::new(100_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal {
        universe: u32::from(younger),
    })?;
    let x = graph.region(Kind::Variable { universe: 0 })?;
    let y = graph.region(Kind::Variable {
        universe: u32::from(younger),
    })?;
    for (index, (longer, shorter)) in [(a, b), (b, a), (Region::STATIC, a)]
        .into_iter()
        .enumerate()
    {
        if assumptions & (1 << index) != 0 {
            graph.assume(longer, shorter)?;
        }
    }
    for (index, (longer, shorter)) in [
        (a, x),
        (x, b),
        (b, y),
        (y, a),
        (x, y),
        (y, x),
        (Region::LOCAL, x),
        (x, Region::STATIC),
    ]
    .into_iter()
    .enumerate()
    {
        if edges & (1 << index) != 0 {
            graph.require(
                longer,
                shorter,
                u32::try_from(index).map_err(|_| Error::Budget)?,
            )?;
        }
    }
    Ok(graph)
}
#[test]
fn sparse_alias_checks_preserve_constraints_across_cycles_and_universes() -> Result<(), Error> {
    for younger in [false, true] {
        for assumptions in 0..8 {
            for edges in 0..=255 {
                let full = graph(edges, assumptions, younger)?.solve().map(|_| ());
                let sparse = graph(edges, assumptions, younger)?
                    .check_aliases()
                    .map(|_| ());
                assert_eq!(
                    sparse, full,
                    "edges={edges} assumptions={assumptions} younger={younger}"
                );
            }
        }
    }
    Ok(())
}
fn rich(member: bool, paid: bool) -> Result<Graph, Error> {
    let mut graph = Graph::new(100_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    let x = graph.region(Kind::Variable { universe: 0 })?;
    graph.require(a, x, 1)?;
    graph.require(x, b, 2)?;
    if paid {
        graph.assume(a, b)?;
    }
    if member {
        let choices = graph.member_choices(&[a, b])?;
        graph.member(x, choices, 3)?;
    } else {
        let test = graph.verify_outlives(a, b)?;
        graph.require_verification(test, 3)?;
    }
    Ok(graph)
}
#[test]
fn richer_alias_graphs_keep_the_complete_observer_solver() -> Result<(), Error> {
    for member in [false, true] {
        for paid in [false, true] {
            let full = rich(member, paid)?.solve().map(|solution| solution.steps);
            assert_eq!(rich(member, paid)?.check_aliases(), full);
        }
    }
    let mut invalid = Graph::new(10);
    assert_eq!(
        invalid.require(Region(999), Region::STATIC, 0),
        Err(Error::Identity)
    );
    // The real scan is bounded even when there are no obligation seeds.
    let mut small = Graph::new(8);
    for _ in 0..4 {
        small.region(Kind::Universal { universe: 0 })?;
    }
    let variable = small.region(Kind::Variable { universe: 0 })?;
    small.require(variable, Region::LOCAL, 0)?;
    assert_eq!(small.check_aliases(), Err(Error::Budget));
    Ok(())
}
fn wide(width: usize) -> Result<Graph, Error> {
    let mut graph = Graph::new(100_000);
    let first = graph.region(Kind::Universal { universe: 0 })?;
    for _ in 1..width {
        graph.region(Kind::Universal { universe: 0 })?;
    }
    let alias = graph.region(Kind::Variable { universe: 0 })?;
    graph.require(alias, Region::LOCAL, 0)?;
    graph.require(first, alias, 1)?;
    graph.require(alias, first, 2)?;
    Ok(graph)
}
#[test]
fn alias_frontier_storage_is_independent_of_unused_width_and_linear_in_contexts()
-> Result<(), Error> {
    for width in [8, 32, 128] {
        let graph = wide(width)?;
        let mut sparse = Solver::new(&graph);
        sparse.seed_alias_frontier()?;
        assert_eq!(sparse.reached.len(), 2);
        let sparse_capacity = sparse.reached.capacity();
        let mut full = Solver::new(&graph);
        full.seed_rigids()?;
        assert_eq!(full.reached.len(), width + 2);
        assert!(full.reached.capacity() > sparse_capacity);
        let single = wide(width)?.check_aliases()?;
        for contexts in [1, 8, 32, 128] {
            let mut work = 0;
            for _ in 0..contexts {
                work += wide(width)?.check_aliases()?;
            }
            assert_eq!(work, single * contexts);
            assert!(work <= (width + 24) * contexts);
            eprintln!(
                "alias width={width} contexts={contexts} work={work} sparse_seeds={} dense_seeds={} sparse_capacity={sparse_capacity}",
                2 * contexts,
                (width + 2) * contexts
            );
        }
    }
    Ok(())
}

#[test]
fn rejected_alias_checks_pay_completed_work_independent_of_unused_allowance() -> Result<(), Error> {
    let mut expected = None;
    for limit in [1024, 32_768, 1_000_000] {
        let mut graph = Graph::new(limit);
        let first = graph.region(Kind::Universal { universe: 0 })?;
        let second = graph.region(Kind::Universal { universe: 0 })?;
        graph.require(first, second, 7)?;
        let checked = graph.check_aliases_counted();
        assert!(matches!(
            checked.result,
            Err(Error::Unsatisfied { source: 7, .. })
        ));
        assert!(!checked.conservative);
        assert!(checked.steps > 0 && checked.steps < 64);
        if let Some(expected) = expected {
            assert_eq!(checked.steps, expected);
        }
        expected = Some(checked.steps);
    }
    for member in [false, true] {
        let checked = rich(member, false)?.check_aliases_counted();
        assert!(checked.result.is_err());
        assert!(checked.conservative);
        assert_eq!(checked.steps, 100_000);
    }
    Ok(())
}
