//! Independent finite-set models distinguish forall-local witnesses from captures.
use super::*;

#[test]
fn a_private_intersection_does_not_force_a_capture_static() -> Result<(), Error> {
    for requires_static in [false, true] {
        let mut graph = Graph::new(1_000);
        let capture = graph.region(Kind::Universal { universe: 0 })?;
        let local = graph.region(Kind::Universal { universe: 1 })?;
        let witness = graph.region(Kind::Variable { universe: 1 })?;
        graph.require(capture, witness, 1)?;
        graph.require(
            if requires_static { witness } else { local },
            if requires_static { local } else { witness },
            2,
        )?;
        let expected = if requires_static {
            vec![(capture, Region::STATIC)]
        } else {
            Vec::new()
        };
        assert_eq!(graph.outer_conditions()?, expected);
        assert_eq!(graph.solve().is_ok(), !requires_static);
    }
    Ok(())
}

#[test]
fn unrelated_local_universals_cannot_be_exported_as_assumptions() -> Result<(), Error> {
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 1 })?;
    let b = graph.region(Kind::Universal { universe: 1 })?;
    graph.require(a, b, 7)?;
    assert!(matches!(
        graph.outer_conditions(),
        Err(Error::Unsatisfied { source: 7, .. })
    ));
    Ok(())
}

fn included(longer: u8, shorter: u8) -> bool {
    longer & shorter == shorter
}

#[test]
fn eliminated_conditions_match_forall_exists_set_models() -> Result<(), Error> {
    // Root lifetimes contain the body point. A quantified local and its private
    // witness range over all subsets, including empty. The witness may depend
    // on that local, while the two root assignments remain fixed.
    let mut seed = 17_u32;
    for _ in 0..512 {
        let mut graph = Graph::new(10_000);
        let nodes = [
            graph.region(Kind::Universal { universe: 0 })?,
            graph.region(Kind::Universal { universe: 0 })?,
            graph.region(Kind::Universal { universe: 1 })?,
            graph.region(Kind::Variable { universe: 1 })?,
            Region::STATIC,
        ];
        let mut edges = Vec::new();
        for _ in 0..4 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let a = (seed as usize >> 8) % nodes.len();
            let b = (seed as usize >> 16) % nodes.len();
            edges.push((a, b));
            graph.require(nodes[a], nodes[b], 1)?;
        }
        let conditions = match graph.outer_conditions() {
            Ok(values) => {
                assert!(
                    values
                        .iter()
                        .all(|&(a, b)| [nodes[0], nodes[1], Region::STATIC].contains(&a)
                            && [nodes[0], nodes[1], Region::STATIC].contains(&b))
                );
                Some(values)
            }
            Err(Error::Unsatisfied { .. }) => None,
            Err(error) => return Err(error),
        };
        for a in [1, 3, 5, 7] {
            for b in [1, 3, 5, 7] {
                let actual = forall_exists(&edges, [a, b]);
                let summarized = conditions.as_ref().is_some_and(|conditions| {
                    outer_model(conditions, [nodes[0], nodes[1]], [a, b])
                });
                assert_eq!(summarized, actual, "edges {edges:?}, a={a}, b={b}");
            }
        }
    }
    Ok(())
}

fn forall_exists(edges: &[(usize, usize)], roots: [u8; 2]) -> bool {
    (0..8).all(|q| {
        (0..8).any(|w| {
            let sets = [roots[0], roots[1], q, w, 7];
            edges
                .iter()
                .all(|&(left, right)| included(sets[left], sets[right]))
        })
    })
}

fn outer_model(conditions: &[(Region, Region)], roots: [Region; 2], values: [u8; 2]) -> bool {
    let set = |region| {
        if region == roots[0] {
            values[0]
        } else if region == roots[1] {
            values[1]
        } else {
            7
        }
    };
    conditions
        .iter()
        .all(|&(left, right)| included(set(left), set(right)))
}
