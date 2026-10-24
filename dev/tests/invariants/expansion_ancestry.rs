//! Compare actual ancestry against a parent-walk oracle, including query work.
use super::*;
type TestResult = Result<(), &'static str>;

fn linear(parents: &[u32], ancestor: u32, mut context: u32) -> bool {
    if ancestor as usize >= parents.len() || context as usize >= parents.len() {
        return false;
    }
    while context != ancestor && context != 0 {
        context = parents[context as usize];
    }
    context == ancestor
}

#[test]
fn root_ancestry_is_allocation_free_and_records_stay_compact() -> TestResult {
    let mut ancestry = Ancestry::default();
    assert_eq!(size_of::<Ancestry>(), size_of::<Vec<ExpansionId>>());
    assert_eq!(size_of::<Node>(), 12);
    assert_eq!(ancestry.nodes.capacity(), 0);
    assert!(ancestry.ancestor(ExpansionId(0), ExpansionId(0)));
    assert!(!ancestry.ancestor(ExpansionId(1), ExpansionId(1)));
    ancestry.append(ExpansionId(1), ExpansionId(0))?;
    assert_eq!(ancestry.nodes.len(), 1);
    assert!(ancestry.ancestor(ExpansionId(0), ExpansionId(1)));
    assert!(!ancestry.ancestor(ExpansionId(1), ExpansionId(0)));
    Ok(())
}

#[test]
fn branches_match_the_independent_parent_walk_for_every_pair() -> TestResult {
    for shape in 0..4 {
        let mut ancestry = Ancestry::default();
        let mut parents = vec![0];
        let mut noise = 47_u32;
        for id in 1..=256 {
            noise = noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let parent = match shape {
                0 => id - 1,
                1 => 0,
                2 => (id - 1) / 2,
                _ => noise % id,
            };
            ancestry.append(ExpansionId(id), ExpansionId(parent))?;
            parents.push(parent);
        }
        for ancestor in 0..=257 {
            for context in 0..=257 {
                assert_eq!(
                    ancestry.ancestor(ExpansionId(ancestor), ExpansionId(context)),
                    linear(&parents, ancestor, context),
                    "shape={shape} ancestor={ancestor} context={context}",
                );
            }
        }
    }
    Ok(())
}

#[test]
fn rejected_publication_does_not_change_existing_ancestry() -> TestResult {
    let mut ancestry = Ancestry::default();
    for id in 1..=64 {
        ancestry.append(ExpansionId(id), ExpansionId(id - 1))?;
    }
    let capacity = ancestry.nodes.capacity();
    for (id, parent) in [(0, 0), (64, 0), (66, 0), (65, 65), (65, u32::MAX)] {
        assert!(
            ancestry
                .append(ExpansionId(id), ExpansionId(parent))
                .is_err()
        );
        assert_eq!(ancestry.nodes.len(), 64);
        assert_eq!(ancestry.nodes.capacity(), capacity);
        assert!(ancestry.ancestor(ExpansionId(1), ExpansionId(64)));
        assert!(!ancestry.ancestor(ExpansionId(65), ExpansionId(65)));
    }
    ancestry.append(ExpansionId(65), ExpansionId(1))?;
    assert!(ancestry.ancestor(ExpansionId(1), ExpansionId(65)));
    assert!(!ancestry.ancestor(ExpansionId(2), ExpansionId(65)));
    Ok(())
}

fn observations(ancestry: &Ancestry, depth: u32, queries: u32) -> Vec<usize> {
    (0..queries)
        .map(|query| {
            let target = (query * 131 + queries) % (depth + 1);
            let mut steps = 0;
            assert!(ancestry.reaches(ExpansionId(target), ExpansionId(depth), || steps += 1));
            assert!(steps <= 3 * (u32::BITS - depth.leading_zeros()) as usize + 2);
            steps
        })
        .collect()
}

#[test]
fn depth_and_query_growth_do_not_multiply_parent_walks() -> TestResult {
    for depth in [8, 32, 128, 512, 2_048, 8_192] {
        let mut ancestry = Ancestry::default();
        for id in 1..=depth {
            ancestry.append(ExpansionId(id), ExpansionId(id - 1))?;
        }
        for queries in [8, 32, 128] {
            let cold = observations(&ancestry, depth, queries);
            assert_eq!(observations(&ancestry, depth, queries), cold);
            eprintln!(
                "depth={depth} queries={queries} steps={} records={}",
                cold.iter().sum::<usize>(),
                ancestry.nodes.len()
            );
        }
        let before = observations(&ancestry, depth, 128);
        for id in depth + 1..=depth + 1_024 {
            ancestry.append(ExpansionId(id), ExpansionId(0))?;
        }
        assert_eq!(observations(&ancestry, depth, 128), before);
        assert_eq!(ancestry.nodes.len(), (depth + 1_024) as usize);
    }
    Ok(())
}
