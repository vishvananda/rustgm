//! Membership must be a proved equality, not an extra universal assumption.
use super::*;
fn universal(graph: &mut Graph) -> Result<Region, Error> {
    graph.region(Kind::Universal { universe: 0 })
}
fn variable(graph: &mut Graph) -> Result<Region, Error> {
    let region = graph.region(Kind::Variable { universe: 0 })?;
    graph.require(region, Region::LOCAL, 0)?;
    Ok(region)
}

#[test]
fn upper_and_lower_bounds_select_a_permitted_name() -> Result<(), Error> {
    for upper in [false, true] {
        let mut graph = Graph::new(10_000);
        let a = universal(&mut graph)?;
        let b = universal(&mut graph)?;
        let hidden = variable(&mut graph)?;
        let choices = graph.member_choices(&[Region::STATIC, a, b])?;
        let member = graph.member(hidden, choices, 7)?;
        if upper {
            graph.require(a, hidden, 8)?;
        } else {
            graph.require(hidden, a, 8)?;
        }
        let proof = graph.solve()?;
        assert_eq!(proof.member_choice(member)?, a);
        assert_eq!(proof.assumption_count(), 0);
        assert!(
            proof
                .requirements(hidden)?
                .any(|r| r.longer == a && r.source == 7)
        );
        assert!(
            proof
                .requirements(a)?
                .any(|r| r.longer == hidden && r.source == 7)
        );
    }
    Ok(())
}

#[test]
fn ordered_branch_inputs_choose_the_shorter_capture() -> Result<(), Error> {
    for ordered in [false, true] {
        let mut graph = Graph::new(10_000);
        let a = universal(&mut graph)?;
        let b = universal(&mut graph)?;
        let hidden = variable(&mut graph)?;
        if ordered {
            graph.assume(a, b)?;
        }
        graph.require(a, hidden, 1)?;
        graph.require(b, hidden, 2)?;
        let choices = graph.member_choices(&[Region::STATIC, a, b])?;
        let member = graph.member(hidden, choices, 3)?;
        if ordered {
            assert_eq!(graph.solve()?.member_choice(member)?, b);
        } else {
            assert!(matches!(
                graph.solve(),
                Err(Error::Membership { source: 3 })
            ));
        }
    }
    Ok(())
}

#[test]
fn ambiguous_choices_and_local_storage_do_not_become_captures() -> Result<(), Error> {
    for local in [false, true] {
        let mut graph = Graph::new(10_000);
        let a = universal(&mut graph)?;
        let b = universal(&mut graph)?;
        let hidden = variable(&mut graph)?;
        if local {
            graph.require(Region::LOCAL, hidden, 1)?;
        }
        let choices = graph.member_choices(&[a, b])?;
        graph.member(hidden, choices, 9)?;
        assert!(matches!(
            graph.solve(),
            Err(Error::Membership { source: 9 })
        ));
    }
    Ok(())
}

#[test]
fn static_lower_bound_and_equivalent_names_are_retained() -> Result<(), Error> {
    for is_static in [false, true] {
        let mut graph = Graph::new(10_000);
        let a = universal(&mut graph)?;
        let b = universal(&mut graph)?;
        graph.assume(a, b)?;
        graph.assume(b, a)?;
        let hidden = variable(&mut graph)?;
        let choices = graph.member_choices(&[b, Region::STATIC])?;
        let member = graph.member(hidden, choices, 1)?;
        if is_static {
            graph.require(hidden, Region::STATIC, 2)?;
        }
        let proof = graph.solve()?;
        assert_eq!(
            proof.member_choice(member)?,
            if is_static { Region::STATIC } else { b }
        );
    }
    Ok(())
}

#[test]
fn cyclic_members_intersect_capture_permissions() -> Result<(), Error> {
    let mut graph = Graph::new(10_000);
    let first = universal(&mut graph)?;
    let shared = universal(&mut graph)?;
    let last = universal(&mut graph)?;
    let left_region = variable(&mut graph)?;
    let right_region = variable(&mut graph)?;
    graph.require(left_region, right_region, 1)?;
    graph.require(right_region, left_region, 2)?;
    let left = graph.member_choices(&[first, shared])?;
    let right = graph.member_choices(&[shared, last])?;
    let mx = graph.member(left_region, left, 3)?;
    let my = graph.member(right_region, right, 4)?;
    let proof = graph.solve()?;
    assert_eq!(proof.member_choice(mx)?, shared);
    assert_eq!(proof.member_choice(my)?, shared);
    Ok(())
}

#[test]
fn another_members_growth_can_resolve_an_ambiguous_domain() -> Result<(), Error> {
    let mut graph = Graph::new(10_000);
    let a = universal(&mut graph)?;
    let b = universal(&mut graph)?;
    let x = variable(&mut graph)?;
    let y = variable(&mut graph)?;
    graph.require(x, y, 1)?;
    let left = graph.member_choices(&[Region::STATIC, a, b])?;
    let right = graph.member_choices(&[a])?;
    let mx = graph.member(x, left, 2)?;
    let my = graph.member(y, right, 3)?;
    let proof = graph.solve()?;
    assert_eq!(proof.member_choice(mx)?, a);
    assert_eq!(proof.member_choice(my)?, a);
    Ok(())
}

#[test]
fn quantified_placeholders_are_never_exported_as_capture_choices() -> Result<(), Error> {
    let mut graph = Graph::new(10_000);
    let local = graph.region(Kind::Universal { universe: 1 })?;
    assert!(matches!(
        graph.member_choices(&[local]),
        Err(Error::Identity)
    ));
    assert!(matches!(
        graph.member_choices(&[Region::LOCAL]),
        Err(Error::Identity)
    ));
    let hidden = variable(&mut graph)?;
    graph.require(hidden, local, 1)?;
    let choices = graph.member_choices(&[Region::STATIC])?;
    let member = graph.member(hidden, choices, 2)?;
    assert_eq!(graph.solve()?.member_choice(member)?, Region::STATIC);
    Ok(())
}

#[test]
fn independent_members_share_wide_capture_lists() -> Result<(), Error> {
    for count in [16, 64, 256, 1024] {
        let mut graph = Graph::new(100_000);
        let mut captures = vec![Region::STATIC];
        for _ in 0..count {
            captures.push(universal(&mut graph)?);
        }
        let choices = graph.member_choices(&captures)?;
        let mut members = Vec::new();
        for &capture in &captures[1..] {
            let hidden = variable(&mut graph)?;
            graph.require(capture, hidden, 1)?;
            members.push(graph.member(hidden, choices, 2)?);
        }
        let proof = graph.solve()?;
        for (&member, &capture) in members.iter().zip(&captures[1..]) {
            assert_eq!(proof.member_choice(member)?, capture);
        }
        println!(
            "members={count} regions={} obligations={} steps={}",
            proof.regions, proof.obligations, proof.steps
        );
        assert!(
            proof.steps < 65 * count + 100,
            "{} steps for {count} captures",
            proof.steps
        );
    }
    Ok(())
}

#[test]
fn a_shared_variable_tail_is_walked_once_per_rigid_bound() -> Result<(), Error> {
    for count in [16, 64, 256, 1024] {
        let mut graph = Graph::new(100_000);
        let capture = universal(&mut graph)?;
        let choices = graph.member_choices(&[Region::STATIC, capture])?;
        let mut previous = capture;
        let mut members = Vec::new();
        for _ in 0..count {
            let hidden = variable(&mut graph)?;
            graph.require(previous, hidden, 1)?;
            members.push(graph.member(hidden, choices, 2)?);
            previous = hidden;
        }
        let proof = graph.solve()?;
        for member in members {
            assert_eq!(proof.member_choice(member)?, capture);
        }
        println!(
            "members={count} regions={} obligations={} steps={}",
            proof.regions, proof.obligations, proof.steps
        );
        assert!(
            proof.steps < 65 * count + 100,
            "{} steps for {count} links",
            proof.steps
        );
    }
    Ok(())
}

#[test]
fn impossible_or_exhausted_choices_do_not_return_partial_proofs() -> Result<(), Error> {
    for budget in [80, 10_000] {
        let mut graph = Graph::new(budget);
        let capture = universal(&mut graph)?;
        let hidden = variable(&mut graph)?;
        let choices = graph.member_choices(&[capture])?;
        graph.member(hidden, choices, 19)?;
        if budget == 80 {
            for _ in 0..5 {
                variable(&mut graph)?;
            }
            assert!(matches!(graph.solve(), Err(Error::Budget)));
        } else {
            graph.require(hidden, Region::STATIC, 20)?;
            assert!(matches!(
                graph.solve(),
                Err(Error::Membership { source: 19 })
            ));
        }
    }
    Ok(())
}

#[test]
fn membership_matches_independent_finite_order_models() -> Result<(), Error> {
    let mut seed = 29_u32;
    for _ in 0..512 {
        let mut graph = Graph::new(100_000);
        let nodes = [
            Region::STATIC,
            universal(&mut graph)?,
            universal(&mut graph)?,
            universal(&mut graph)?,
        ];
        let hidden = variable(&mut graph)?;
        let mut order = [[false; 4]; 4];
        order[0].fill(true);
        for (index, row) in order.iter_mut().enumerate() {
            row[index] = true;
        }
        for longer in 1..4 {
            for shorter in 1..4 {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if seed >> 29 == 0 {
                    graph.assume(nodes[longer], nodes[shorter])?;
                    order[longer][shorter] = true;
                }
            }
        }
        // This tiny oracle uses a matrix closure, independent of the compiler's
        // sparse propagation, SCCs, canonical sets and minimum worklist.
        for middle in 0..4 {
            for longer in 0..4 {
                for shorter in 0..4 {
                    order[longer][shorter] |= order[longer][middle] && order[middle][shorter];
                }
            }
        }
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let upper = (seed >> 16) as usize & 15;
        let lower = (seed >> 20) as usize & 15;
        let permitted = (seed >> 24) as usize & 15;
        let choices: Vec<_> = nodes
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| (permitted & (1 << i) != 0).then_some(r))
            .collect();
        let choices = graph.member_choices(&choices)?;
        let member = graph.member(hidden, choices, 1)?;
        for (index, &region) in nodes.iter().enumerate() {
            if upper & (1 << index) != 0 {
                graph.require(region, hidden, 2)?;
            }
            if lower & (1 << index) != 0 {
                graph.require(hidden, region, 3)?;
            }
        }
        let eligible: Vec<_> = (0..4)
            .filter(|&candidate| {
                permitted & (1 << candidate) != 0
                    && (0..4).all(|bound| {
                        (upper & (1 << bound) == 0 || order[bound][candidate])
                            && (lower & (1 << bound) == 0 || order[candidate][bound])
                    })
            })
            .collect();
        let ordered: Vec<_> = eligible
            .iter()
            .copied()
            .filter(|&candidate| {
                eligible
                    .iter()
                    .all(|&other| order[other][candidate] || order[candidate][other])
            })
            .collect();
        let minima: Vec<_> = ordered
            .iter()
            .copied()
            .filter(|&candidate| ordered.iter().all(|&other| order[other][candidate]))
            .collect();
        match graph.solve() {
            Ok(proof) => {
                let selected = proof.member_choice(member)?;
                assert!(
                    minima.iter().any(|&i| nodes[i] == selected),
                    "unjustified selection {selected:?} with {order:?}"
                );
            }
            Err(Error::Membership { .. } | Error::Unsatisfied { .. }) => assert!(
                minima.is_empty(),
                "missed solution {minima:?} with {order:?}"
            ),
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[test]
fn incomparable_captures_can_select_static() -> Result<(), Error> {
    let mut graph = Graph::new(10_000);
    let a = universal(&mut graph)?;
    let b = universal(&mut graph)?;
    let hidden = variable(&mut graph)?;
    let choices = graph.member_choices(&[Region::STATIC, a, b])?;
    let member = graph.member(hidden, choices, 1)?;
    assert_eq!(graph.solve()?.member_choice(member)?, Region::STATIC);
    Ok(())
}

#[test]
fn dependency_order_precedes_conservative_selection() -> Result<(), Error> {
    for reversed in [false, true] {
        let mut graph = Graph::new(10_000);
        let first = universal(&mut graph)?;
        let second = universal(&mut graph)?;
        let parent = variable(&mut graph)?;
        let child = variable(&mut graph)?;
        graph.require(parent, child, 1)?;
        let broad = graph.member_choices(&[Region::STATIC, first, second])?;
        let narrow = graph.member_choices(&[first])?;
        let (parent_member, child_member) = if reversed {
            let child_member = graph.member(child, narrow, 3)?;
            (graph.member(parent, broad, 2)?, child_member)
        } else {
            let parent_member = graph.member(parent, broad, 2)?;
            (parent_member, graph.member(child, narrow, 3)?)
        };
        let proof = graph.solve()?;
        assert_eq!(proof.member_choice(parent_member)?, first);
        assert_eq!(proof.member_choice(child_member)?, first);
    }
    Ok(())
}

#[test]
fn ordered_candidates_above_an_incomparable_pair_share_reachability() -> Result<(), Error> {
    for count in [16, 64, 256, 1024] {
        let mut graph = Graph::new(100_000);
        let left = universal(&mut graph)?;
        let right = universal(&mut graph)?;
        let junction = universal(&mut graph)?;
        graph.assume(junction, left)?;
        graph.assume(junction, right)?;
        let mut choices = vec![Region::STATIC, left, right, junction];
        let mut previous = junction;
        for _ in 0..count {
            let region = universal(&mut graph)?;
            graph.assume(region, previous)?;
            choices.push(region);
            previous = region;
        }
        let hidden = variable(&mut graph)?;
        let choices = graph.member_choices(&choices)?;
        let member = graph.member(hidden, choices, 1)?;
        let proof = graph.solve()?;
        assert_eq!(proof.member_choice(member)?, junction);
        println!("chain-with-fork={count} steps={}", proof.steps);
        assert!(proof.steps < 65 * count + 200);
    }
    Ok(())
}
