use rustgm::borrow::regions::{
    graph::{Error, Graph, Kind, Region, Requirement},
    points::{Point, infer},
};

#[test]
fn proofs_retain_direct_requirements_and_separate_declared_assumptions() -> Result<(), Error> {
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    let v = graph.region(Kind::Variable { universe: 0 })?;
    graph.assume(a, b)?;
    graph.require(a, v, 17)?;
    graph.require(v, b, 29)?;
    let proof = graph.solve()?;
    assert_eq!(proof.assumptions(a)?.collect::<Vec<_>>(), [b]);
    assert_eq!(proof.assumptions(v)?.count(), 0);
    assert_eq!(proof.requirements(a)?.count(), 0);
    assert_eq!(
        proof.requirements(v)?.collect::<Vec<_>>(),
        [Requirement {
            longer: a,
            source: 17
        }]
    );
    assert_eq!(
        proof.requirements(b)?.collect::<Vec<_>>(),
        [Requirement {
            longer: v,
            source: 29
        }]
    );
    assert_eq!(proof.kind(v), Some(Kind::Variable { universe: 0 }));
    assert_eq!(proof.kind(Region(u32::MAX)), None);
    assert!(proof.requirements(Region(u32::MAX)).is_err());
    assert!(proof.assumptions(Region(u32::MAX)).is_err());
    Ok(())
}

#[test]
fn sparse_point_propagation_matches_independent_set_closure()
-> Result<(), Box<dyn std::error::Error>> {
    let mut random = 41_u32;
    for _ in 0..128 {
        let mut graph = Graph::new(10_000);
        let mut regions = Vec::new();
        for _ in 0..6 {
            regions.push(graph.region(Kind::Variable { universe: 0 })?);
        }
        let mut edges = Vec::new();
        for longer in 0..6 {
            for shorter in 0..6 {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if random >> 29 == 0 {
                    edges.push((longer, shorter));
                    graph.require(regions[longer], regions[shorter], 0)?;
                }
            }
        }
        let mut sets = [const { std::collections::BTreeSet::new() }; 6];
        let mut seeds = Vec::new();
        for (id, set) in sets.iter_mut().enumerate() {
            for point in [0, 63, 64, 65, 127, 191] {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if random >> 29 < 3 {
                    set.insert(point);
                    seeds.push((regions[id], Point(point)));
                }
            }
        }
        loop {
            let before = sets.clone();
            for &(longer, shorter) in &edges {
                sets[longer].extend(before[shorter].iter().copied());
            }
            if sets == before {
                break;
            }
        }
        let proof = graph.solve()?;
        let live = infer(&proof, 192, seeds, 10_000)?;
        for (id, set) in sets.iter().enumerate() {
            for point in 0..192 {
                assert_eq!(
                    live.contains(regions[id], Point(point))?,
                    set.contains(&point),
                    "{edges:?}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn abstract_local_bounds_never_connect_runtime_lifetimes() -> Result<(), Box<dyn std::error::Error>>
{
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Variable { universe: 0 })?;
    let b = graph.region(Kind::Variable { universe: 0 })?;
    for region in [a, b] {
        graph.require(region, Region::LOCAL, 0)?;
        graph.require(Region::LOCAL, region, 0)?;
    }
    let proof = graph.solve()?;
    let live = infer(&proof, u32::MAX, [(a, Point(u32::MAX - 1))], 100)?;
    assert_eq!(live.mask_count(), 1);
    assert!(live.contains(a, Point(u32::MAX - 1))?);
    assert!(!live.contains(b, Point(u32::MAX - 1))?);
    assert!(!live.contains(a, Point(0))?);
    assert!(live.contains(Region::LOCAL, Point(0)).is_err());
    assert!(live.contains(a, Point(u32::MAX)).is_err());
    for seed in [
        (Region::LOCAL, Point(0)),
        (Region(u32::MAX), Point(0)),
        (a, Point(4)),
    ] {
        assert!(infer(&proof, 4, [seed], 100).is_err());
    }
    assert!(infer(&proof, 4, [(a, Point(0))], 0).is_err());
    Ok(())
}
