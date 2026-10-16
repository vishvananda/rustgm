//! Boolean lifetime proofs are checked against independent finite-set models.
use rustgm::{
    borrow::regions::points::{Point, infer},
    types::region_graph::{Error, Graph, Kind, Region},
};

#[test]
fn symbolic_alternatives_match_minimal_set_models() -> Result<(), Error> {
    for assumptions in 0..4 {
        for edges in 0..256 {
            for conjunction in [false, true] {
                let mut graph = Graph::new(10_000);
                let a = graph.region(Kind::Universal { universe: 0 })?;
                let b = graph.region(Kind::Universal { universe: 0 })?;
                let x = graph.region(Kind::Variable { universe: 0 })?;
                let y = graph.region(Kind::Variable { universe: 0 })?;
                let mut sets = [0b1111_u32, 0b0010, 0b0110, 0b1010, 0, 0];
                if assumptions & 1 != 0 {
                    graph.assume(a, b)?;
                    sets[2] |= sets[3];
                }
                if assumptions & 2 != 0 {
                    graph.assume(b, a)?;
                    sets[3] |= sets[2];
                }
                if assumptions == 3 {
                    sets[2] |= sets[3];
                }
                let candidates = [
                    (x, a),
                    (x, b),
                    (y, a),
                    (y, b),
                    (x, y),
                    (y, x),
                    (x, Region::STATIC),
                    (y, Region::LOCAL),
                ];
                let selected: Vec<_> = candidates
                    .into_iter()
                    .enumerate()
                    .filter_map(|(bit, pair)| (edges & (1 << bit) != 0).then_some(pair))
                    .collect();
                for &(longer, shorter) in &selected {
                    graph.require(longer, shorter, 0)?;
                }
                loop {
                    let before = sets;
                    for &(longer, shorter) in &selected {
                        sets[longer.0 as usize] |= before[shorter.0 as usize];
                    }
                    if sets == before {
                        break;
                    }
                }
                let first = graph.verify_outlives(a, x)?;
                let second = graph.verify_outlives(y, x)?;
                let root = if conjunction {
                    graph.verify_all(&[first, second])?
                } else {
                    graph.verify_any(&[first, second])?
                };
                graph.require_verification(root, 73)?;
                let contains = |longer: Region, shorter: Region| {
                    sets[longer.0 as usize] & sets[shorter.0 as usize] == sets[shorter.0 as usize]
                };
                let expected = if conjunction {
                    contains(a, x) && contains(y, x)
                } else {
                    contains(a, x) || contains(y, x)
                };
                let result = graph.solve();
                assert_eq!(
                    result.is_ok(),
                    expected,
                    "assumptions={assumptions}, edges={edges}, all={conjunction}"
                );
                if !expected {
                    assert!(matches!(result, Err(Error::Verification { source: 73 })));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn different_branches_cannot_supply_the_two_halves_of_a_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    let target = graph.region(Kind::Variable { universe: 0 })?;
    graph.require(target, a, 0)?;
    let first = graph.verify_outlives(a, target)?;
    let second = graph.verify_outlives(b, target)?;
    let root = graph.verify_any(&[first, second])?;
    graph.require_verification(root, 17)?;
    let proof = graph.solve()?;
    // Only the first branch proves the universal relation. Giving the second
    // branch all live points cannot compensate for its failed symbolic proof.
    assert!(infer(&proof, 65, [(target, Point(64))], [b], 10_000).is_err());
    infer(&proof, 65, [(target, Point(64))], [a], 10_000)?;
    Ok(())
}

#[test]
fn runtime_alternatives_compare_complete_extents_without_unioning_branches()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [0, 1, 63, 64, 65, 129] {
        let mut graph = Graph::new(1_000);
        let a = graph.region(Kind::Variable { universe: 0 })?;
        let b = graph.region(Kind::Variable { universe: 0 })?;
        let target = graph.region(Kind::Variable { universe: 0 })?;
        let first = graph.verify_outlives(a, target)?;
        let second = graph.verify_outlives(b, target)?;
        let root = graph.verify_any(&[first, second])?;
        graph.require_verification(root, 0)?;
        let proof = graph.solve()?;
        let split = (0..count).map(|p| (if p % 2 == 0 { a } else { b }, Point(p)));
        let split = infer(&proof, count, split, [target], 10_000);
        assert_eq!(split.is_ok(), count <= 1);
        infer(
            &proof,
            count,
            (0..count).map(|p| (a, Point(p))),
            [target],
            10_000,
        )?;
        infer(&proof, count, [], [target, b], 10_000)?;
    }
    Ok(())
}

#[test]
fn shared_deep_verifications_are_bounded_and_iterative() -> Result<(), Error> {
    let mut graph = Graph::new(100_000);
    let region = graph.region(Kind::Variable { universe: 0 })?;
    let mut test = graph.verify_outlives(region, region)?;
    assert_eq!(test, graph.verify_outlives(region, region)?);
    for _ in 0..4096 {
        test = graph.verify_all(&[test, test])?;
    }
    graph.require_verification(test, 0)?;
    let proof = graph.solve()?;
    assert!(proof.verify_regions(|a, b| Ok(a == b), 100_000)? < 100_000);
    assert!(matches!(
        proof.verify_regions(|_, _| Ok(true), 0),
        Err(Error::Budget)
    ));
    let mut graph = Graph::new(100);
    assert!(
        graph
            .verify_outlives(Region::LOCAL, Region::STATIC)
            .is_err()
    );
    assert!(graph.verify_outlives(Region(100), Region::STATIC).is_err());
    let false_ = graph.verify_any(&[])?;
    graph.require_verification(false_, 99)?;
    assert!(matches!(
        graph.solve(),
        Err(Error::Verification { source: 99 })
    ));
    Ok(())
}
