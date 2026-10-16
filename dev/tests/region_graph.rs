use rustgm::borrow::regions::graph::{Error, Graph, Kind, Region};

fn universal(graph: &mut Graph, universe: u32) -> Result<Region, Error> {
    graph.region(Kind::Universal { universe })
}
fn variable(graph: &mut Graph, universe: u32) -> Result<Region, Error> {
    graph.region(Kind::Variable { universe })
}

#[test]
fn declared_transitive_bounds_justify_conversions_but_obligations_do_not() -> Result<(), Error> {
    for declared in [true, false] {
        let mut graph = Graph::new(1_000);
        let a = universal(&mut graph, 0)?;
        let b = universal(&mut graph, 0)?;
        let c = universal(&mut graph, 0)?;
        let value = variable(&mut graph, 0)?;
        if declared {
            graph.assume(a, b)?;
            graph.assume(b, c)?;
        }
        graph.require(a, value, 10)?;
        graph.require(value, c, 20)?;
        let result = graph.solve();
        assert_eq!(result.is_ok(), declared);
        if !declared {
            assert!(
                matches!(result, Err(Error::Unsatisfied { longer, shorter, .. }) if longer==a && shorter==c)
            );
        }
    }
    Ok(())
}

#[test]
fn local_storage_cannot_escape_even_through_cycles_but_external_reborrows_can() -> Result<(), Error>
{
    for local in [true, false] {
        let mut graph = Graph::new(1_000);
        let input = universal(&mut graph, 0)?;
        let borrow = variable(&mut graph, 0)?;
        let assigned = variable(&mut graph, 0)?;
        graph.require(if local { Region::LOCAL } else { input }, borrow, 1)?;
        graph.require(borrow, assigned, 2)?;
        graph.require(assigned, borrow, 3)?;
        graph.require(assigned, input, 4)?;
        assert_eq!(graph.solve().is_ok(), !local);
    }
    Ok(())
}

#[test]
fn higher_ranked_placeholders_cannot_leak_into_older_variables() -> Result<(), Error> {
    for variable_universe in [0, 1] {
        let mut graph = Graph::new(1_000);
        let early = variable(&mut graph, variable_universe)?;
        let late = universal(&mut graph, 1)?;
        graph.require(late, early, 1)?;
        graph.require(early, late, 2)?;
        assert_eq!(graph.solve().is_ok(), variable_universe == 1);
    }
    let mut graph = Graph::new(1_000);
    let early = variable(&mut graph, 0)?;
    let late = universal(&mut graph, 1)?;
    graph.require(early, late, 1)?;
    assert!(
        graph.solve().is_ok(),
        "an older variable may choose static and shorten"
    );
    let mut graph = Graph::new(1_000);
    let outer = universal(&mut graph, 0)?;
    let inner = universal(&mut graph, 1)?;
    graph.assume(outer, Region::STATIC)?;
    graph.require(outer, inner, 1)?;
    assert!(
        graph.solve().is_ok(),
        "an explicit static bound is sufficient"
    );
    Ok(())
}

#[test]
fn small_graphs_agree_with_exhaustive_set_inclusion_models() -> Result<(), Error> {
    // Bits represent one common body point, separate caller points, and an
    // additional static-only point. Infer both variable sets independently.
    let mut seed = 7_u32;
    for assumption in 0..3 {
        for _ in 0..256 {
            let mut graph = Graph::new(10_000);
            let a = universal(&mut graph, 0)?;
            let b = universal(&mut graph, 0)?;
            let x = variable(&mut graph, 0)?;
            let y = variable(&mut graph, 0)?;
            let a_bits = match assumption {
                1 => {
                    graph.assume(a, b)?;
                    7
                }
                2 => {
                    graph.assume(a, Region::STATIC)?;
                    15
                }
                _ => 3,
            };
            let b_bits = if assumption == 1 { 3 } else { 5 };
            let regions = [Region::STATIC, Region::LOCAL, a, b, x, y];
            let mut edges = Vec::new();
            for longer in 0..6 {
                for shorter in 0..6 {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    if seed >> 28 == 0 && longer != shorter {
                        edges.push((longer, shorter));
                        graph.require(regions[longer], regions[shorter], 0)?;
                    }
                }
            }
            let models: Vec<_> = (0_u8..16)
                .flat_map(|x| (0_u8..16).map(move |y| [15, 1, a_bits, b_bits, x, y]))
                .filter(|sets| {
                    edges
                        .iter()
                        .all(|&(longer, shorter)| sets[longer] & sets[shorter] == sets[shorter])
                })
                .collect();
            let solved = graph.solve();
            assert_eq!(
                solved.is_ok(),
                !models.is_empty(),
                "assumption={assumption} edges={edges:?}"
            );
            if let Ok(solved) = solved {
                for region in solved.required_static()? {
                    let index = regions
                        .iter()
                        .position(|r| *r == region)
                        .ok_or(Error::Identity)?;
                    assert!(models.iter().all(|sets| sets[index] == 15));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn deep_chains_are_iterative_and_invalid_or_excessive_work_is_diagnosed() -> Result<(), Error> {
    let mut graph = Graph::new(100_000);
    let external = universal(&mut graph, 0)?;
    let mut prior = external;
    for _ in 0..10_000 {
        let next = variable(&mut graph, 0)?;
        graph.require(prior, next, 0)?;
        prior = next;
    }
    graph.require(prior, external, 0)?;
    let solved = graph.solve()?;
    assert!(solved.steps < 30_010);
    assert!(solved.propagated < 10_010);
    let mut graph = Graph::new(8);
    let a = variable(&mut graph, 0)?;
    assert_eq!(graph.assume(a, Region::STATIC), Err(Error::Identity));
    assert_eq!(
        graph.assume(Region::LOCAL, Region::STATIC),
        Err(Error::Identity)
    );
    assert_eq!(graph.require(Region(u32::MAX), a, 0), Err(Error::Identity));
    assert!(matches!(Graph::new(0).solve(), Err(Error::Budget)));
    Ok(())
}

#[test]
fn younger_placeholders_do_not_cover_an_older_body_extent() -> Result<(), Error> {
    for universe in [0, 1] {
        let mut graph = Graph::new(100);
        let placeholder = universal(&mut graph, universe)?;
        let inferred = variable(&mut graph, 0)?;
        graph.require(inferred, Region::LOCAL, 1)?;
        graph.require(placeholder, inferred, 2)?;
        assert_eq!(graph.solve().is_ok(), universe == 0);
    }
    Ok(())
}

#[test]
fn static_evidence_requires_a_proven_lower_bound_and_stays_in_its_graph() -> Result<(), Error> {
    let mut graph = Graph::new(1_000);
    let a = variable(&mut graph, 0)?;
    let b = variable(&mut graph, 0)?;
    let u = universal(&mut graph, 0)?;
    let unrelated = variable(&mut graph, 0)?;
    graph.assume(u, Region::STATIC)?;
    graph.require(a, Region::STATIC, 1)?;
    graph.require(b, a, 2)?;
    graph.require(a, b, 3)?;
    graph.require(u, b, 4)?;
    graph.require(Region::STATIC, unrelated, 5)?;
    let solved = graph.solve()?;
    assert_eq!(solved.required_static()?, [a, b, u]);
    assert_eq!(solved.required_static()?, [a, b, u]);
    let mut other = Graph::new(1_000);
    assert_eq!(variable(&mut other, 0)?, a);
    assert!(other.solve()?.required_static()?.is_empty());
    let mut invalid = Graph::new(1_000);
    let local = variable(&mut invalid, 0)?;
    invalid.require(Region::LOCAL, local, 1)?;
    invalid.require(local, Region::STATIC, 2)?;
    assert!(matches!(invalid.solve(), Err(Error::Unsatisfied { .. })));
    Ok(())
}
