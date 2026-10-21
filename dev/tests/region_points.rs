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
    for case in 0..128 {
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
        let mut full = Vec::new();
        if case % 3 == 0 {
            let id = (random as usize) % regions.len();
            full.push(regions[id]);
            sets[id].extend(0..192);
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
        let mut repeated = seeds.clone();
        repeated.reverse();
        repeated.extend_from_slice(&seeds);
        for seeds in [seeds, repeated] {
            let expanded = seeds.iter().copied().chain(
                full.iter()
                    .flat_map(|&region| (0..192).map(move |p| (region, Point(p)))),
            );
            let original = infer(&proof, 192, expanded, [], 10_000)?;
            let live = infer(&proof, 192, seeds, full.iter().copied(), 10_000)?;
            let mut before: Vec<_> = original.words().collect();
            let after: Vec<_> = live.words().collect();
            assert!(
                after
                    .windows(2)
                    .all(|pair| { (pair[0].0.0, pair[0].1) < (pair[1].0.0, pair[1].1) })
            );
            before.sort_unstable_by_key(|&(region, word, _)| (region.0, word));
            assert_eq!(before, after);
            assert_eq!(live.mask_count(), before.len());
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
    }
    Ok(())
}

#[test]
fn shared_static_extents_fit_the_sparse_propagation_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(10_000);
    let mut regions = Vec::new();
    for _ in 0..128 {
        let region = graph.region(Kind::Variable { universe: 0 })?;
        graph.require(region, Region::STATIC, 0)?;
        regions.push(region);
    }
    let unrelated = graph.region(Kind::Variable { universe: 0 })?;
    let proof = graph.solve()?;
    let seeds = (0..4096).map(|point| (Region::STATIC, Point(point)));
    let live = infer(&proof, 4097, seeds, [], 100_000)?;
    for region in regions.into_iter().chain([Region::STATIC]) {
        for point in [0, 63, 64, 2049, 4095] {
            assert!(live.contains(region, Point(point))?);
        }
        assert!(!live.contains(region, Point(4096))?);
    }
    assert!(!live.contains(unrelated, Point(0))?);
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
    let live = infer(&proof, u32::MAX, [(a, Point(u32::MAX - 1))], [], 100)?;
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
        assert!(infer(&proof, 4, [seed], [], 100).is_err());
    }
    assert!(infer(&proof, 4, [(a, Point(0))], [], 0).is_err());
    let full = infer(&proof, 4, [], [a], 100)?;
    assert!(full.contains(a, Point(3))?);
    assert!(!full.contains(b, Point(0))?);
    for region in [Region::LOCAL, Region(u32::MAX)] {
        assert!(infer(&proof, 4, [], [region], 100).is_err());
        assert!(infer(&proof, 0, [], [region], 100).is_err());
    }
    assert!(infer(&proof, 4, [(a, Point(4))], [a], 100).is_err());
    assert!(infer(&proof, 4, [], [a], 0).is_err());
    Ok(())
}

#[test]
fn full_extent_views_preserve_partial_words_without_using_assumptions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(1_000);
    let a = graph.region(Kind::Universal { universe: 0 })?;
    let b = graph.region(Kind::Universal { universe: 0 })?;
    graph.assume(b, a)?;
    let proof = graph.solve()?;
    for (points, expected) in [
        (0, vec![]),
        (1, vec![1]),
        (63, vec![u64::MAX >> 1]),
        (64, vec![u64::MAX]),
        (65, vec![u64::MAX, 1]),
        (129, vec![u64::MAX, u64::MAX, 1]),
    ] {
        let live = infer(&proof, points, [], [a, a], 100)?;
        let actual: Vec<_> = live.words().collect();
        assert_eq!(actual.len(), expected.len());
        assert_eq!(live.mask_count(), expected.len());
        for ((region, word, bits), (index, expected)) in
            actual.into_iter().zip(expected.iter().enumerate())
        {
            assert_eq!(region, a);
            assert_eq!(word as usize, index);
            assert_eq!(bits, *expected);
        }
        for point in 0..points {
            assert!(live.contains(a, Point(point))?);
            assert!(!live.contains(b, Point(point))?);
            assert!(!live.contains(Region::STATIC, Point(point))?);
        }
        assert!(live.contains(a, Point(points)).is_err());
    }
    Ok(())
}

#[test]
fn explicit_full_extents_do_not_retain_a_region_by_word_matrix()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(100_000);
    let mut regions = Vec::new();
    for _ in 0..4096 {
        let region = graph.region(Kind::Variable { universe: 0 })?;
        graph.require(region, Region::STATIC, 0)?;
        regions.push(region);
    }
    let unrelated = graph.region(Kind::Variable { universe: 0 })?;
    let proof = graph.solve()?;
    let live = infer(
        &proof,
        65_537,
        [(unrelated, Point(31))],
        [Region::STATIC],
        100_000,
    )?;
    assert_eq!(live.mask_count(), 4097 * 1025 + 1);
    for region in regions.into_iter().chain([Region::STATIC]) {
        for point in [0, 63, 64, 65_535, 65_536] {
            assert!(live.contains(region, Point(point))?);
        }
    }
    assert!(live.contains(unrelated, Point(31))?);
    assert!(!live.contains(unrelated, Point(30))?);
    Ok(())
}

#[test]
fn cyclic_extents_propagate_complete_words_with_output_bounded_work()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [8_u32, 32, 128, 512, 1024] {
        let mut graph = Graph::new(2_000_000);
        let regions = (0..count)
            .map(|_| graph.region(Kind::Variable { universe: 0 }))
            .collect::<Result<Vec<_>, _>>()?;
        for index in 0..count as usize {
            let next = (index + 1) % regions.len();
            graph.require(regions[next], regions[index], 0)?;
            // Repeated source obligations retain distinct diagnostic identities
            // but do not require repeated propagation between runtime components.
            graph.require(regions[next], regions[index], 1)?;
        }
        let unrelated = graph.region(Kind::Variable { universe: 0 })?;
        let proof = graph.solve()?;
        let words = count as usize * count.div_ceil(64) as usize;
        let limit = 16 * (3 * count as usize + words) + 100;
        let live = infer(
            &proof,
            count,
            regions.iter().copied().zip((0..count).map(Point)),
            [],
            limit,
        )?;
        assert_eq!(live.mask_count(), words);
        assert!(live.steps <= limit);
        for region in regions {
            for point in [0, count / 2, count - 1] {
                assert!(live.contains(region, Point(point))?);
            }
        }
        assert!(!live.contains(unrelated, Point(0))?);
        assert!(!live.contains(Region::STATIC, Point(0))?);
    }
    Ok(())
}

#[test]
fn joined_acyclic_extents_send_each_complete_word_once() -> Result<(), Box<dyn std::error::Error>> {
    for count in [8_u32, 32, 128, 512] {
        let mut graph = Graph::new(2_000_000);
        let regions = (0..count)
            .map(|_| graph.region(Kind::Variable { universe: 0 }))
            .collect::<Result<Vec<_>, _>>()?;
        for index in 0..regions.len() {
            for distance in [1, 2] {
                if let Some(&longer) = regions.get(index + distance) {
                    graph.require(longer, regions[index], 0)?;
                }
            }
        }
        let proof = graph.solve()?;
        let words = (1..=count).map(|n| n.div_ceil(64) as usize).sum::<usize>();
        let limit = 16 * (3 * count as usize + words) + 100;
        let seeds: Vec<_> = regions.iter().copied().zip((0..count).map(Point)).collect();
        for seeds in [seeds.clone(), seeds.into_iter().rev().collect()] {
            let live = infer(&proof, count, seeds, [], limit)?;
            assert_eq!(live.mask_count(), words);
            assert!(live.steps <= limit);
            for (index, region) in regions.iter().copied().enumerate() {
                let end = u32::try_from(index)?;
                for point in [0, end / 2, end] {
                    assert!(live.contains(region, Point(point))?);
                }
                if end + 1 < count {
                    assert!(!live.contains(region, Point(end + 1))?);
                }
            }
        }
    }
    Ok(())
}
