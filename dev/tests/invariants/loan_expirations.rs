use super::super::{Mode, State};
use super::{Expirations, Loan, Point, Set, points};
use crate::{
    body::ir::ValueId,
    borrow::{
        places::Id,
        regions::graph::{Graph, Kind, Region},
    },
};
use std::collections::BTreeSet;

fn loans(
    regions: impl IntoIterator<Item = Region>,
) -> Result<Vec<Loan>, std::num::TryFromIntError> {
    regions
        .into_iter()
        .enumerate()
        .map(|(id, region)| {
            Ok(Loan {
                place: Id(id),
                region,
                mode: Mode::Reserved,
                value: ValueId(u32::try_from(id)?),
            })
        })
        .collect()
}

#[test]
fn expiration_events_match_independent_point_sets_across_jumps_and_regeneration()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(10_000);
    let mut regions = vec![Region::STATIC];
    for _ in 0..5 {
        regions.push(graph.region(Kind::Variable { universe: 0 })?);
    }
    let sets: Vec<BTreeSet<u32>> = vec![
        (0..257).collect(),
        (0..128).collect(),
        (63..193).collect(),
        [0, 62, 63, 64, 65, 127, 128, 191, 192, 255, 256]
            .into_iter()
            .collect(),
        (0..257).filter(|point| point % 3 != 0).collect(),
        BTreeSet::new(),
    ];
    let seeds = sets.iter().enumerate().skip(1).flat_map(|(id, set)| {
        let region = regions[id];
        set.iter().map(move |&point| (region, Point(point)))
    });
    let live = points::infer(&graph.solve()?, 257, seeds, [Region::STATIC], 10_000)?;
    let loans = loans((0..193).map(|loan| regions[loan % regions.len()]))?;
    let index = Expirations::new(&loans, &live)?;
    let mut state = Set::new(loans.len().div_ceil(64));
    let mut expected = vec![(false, false); loans.len()];
    let path = (0..257)
        .chain(90..137)
        .chain([2, 1, 256, 64, 64, 65, 129, 0]);
    for (step, point) in path.enumerate() {
        if step % 41 == 0 {
            // A new incoming join is not validated by the previous cursor, even
            // when its numeric point happens to be the consecutive point.
            let incoming: Vec<_> = state.words().to_vec();
            state.copy_from(&incoming);
            assert!(state.valid_at.is_none());
        }
        index.prune(&mut state, Point(point), &loans, &live)?;
        for (loan, (present, active)) in expected.iter_mut().enumerate() {
            if !sets[loan % regions.len()].contains(&point) {
                *present = false;
                *active = false;
            }
            let word = state.words()[loan / 64];
            assert_eq!(
                word.live & (1 << (loan % 64)) != 0,
                *present,
                "{step}/{loan}"
            );
            assert_eq!(
                word.active & (1 << (loan % 64)) != 0,
                *active,
                "{step}/{loan}"
            );
        }
        for offset in 0..7 {
            let loan = (step * 17 + offset * 31) % loans.len();
            let inside = sets[loan % regions.len()].contains(&point);
            state.generate(loan, inside);
            expected[loan] = (true, false);
            if offset % 2 == 0 {
                state.activate(loan);
                expected[loan].1 = true;
            }
        }
    }
    Ok(())
}

#[test]
fn continuous_regions_pay_for_words_and_exits_instead_of_loans_times_points()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [8_u32, 32, 128, 512] {
        let mut graph = Graph::new(10_000);
        let region = graph.region(Kind::Variable { universe: 0 })?;
        let points = width * 8 + 1;
        let live = points::infer(
            &graph.solve()?,
            points,
            (0..points - 1).map(|point| (region, Point(point))),
            [],
            100_000,
        )?;
        let loans = loans(std::iter::repeat_n(region, width as usize))?;
        let index = Expirations::new(&loans, &live)?;
        assert_eq!(index.event_points(), 1);
        assert_eq!(index.entries, (width as usize).div_ceil(64));
        let mut state = Set::new(loans.len().div_ceil(64));
        let mut work = index.steps;
        work += index.prune(&mut state, Point(0), &loans, &live)?;
        for loan in 0..loans.len() {
            state.generate(loan, true);
            state.activate(loan);
        }
        for point in 1..points {
            work += index.prune(&mut state, Point(point), &loans, &live)?;
            let present = state
                .words()
                .iter()
                .map(|word| word.live.count_ones())
                .sum::<u32>();
            assert_eq!(present, if point + 1 == points { 0 } else { width });
        }
        assert!(work <= width as usize * 11, "{width}: {work}");
        assert!(
            state
                .words()
                .iter()
                .all(|word| word.live == 0 && word.active == 0)
        );
    }
    Ok(())
}

#[test]
fn full_extents_and_final_points_do_not_create_expiration_events()
-> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new(1_000);
    let region = graph.region(Kind::Variable { universe: 0 })?;
    let proof = graph.solve()?;
    let loans = loans([Region::STATIC, region])?;
    for count in [1, 63, 64, 65, 129, u32::MAX] {
        let live = points::infer(
            &proof,
            count,
            [(region, Point(count - 1))],
            [Region::STATIC],
            1_000,
        )?;
        let index = Expirations::new(&loans, &live)?;
        assert!(index.events.is_empty());
        let empty = Expirations::new(&[], &live)?;
        assert!(empty.events.is_empty());
        assert_eq!(empty.steps, 0);
        let mut state = Set::new(1);
        state.copy_from(&[State { live: 3, active: 2 }]);
        index.prune(&mut state, Point(count - 1), &loans, &live)?;
        assert_eq!((state.words()[0].live, state.words()[0].active), (3, 2));
    }
    Ok(())
}
