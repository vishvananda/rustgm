//! Demanded outlives contracts index the crate's explicit nominal bounds once.
use super::*;
use crate::{resolve::model::DefKind, types::ItemId};
use std::{cell::Cell, fmt::Write};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn nominal(report: &Report, name: &str) -> TestResult<ItemId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Struct && report.package.names.get(definition.name) == name
        })
        .ok_or("declared nominal")?;
    Ok(ItemId(u32::try_from(index)?))
}

fn sorted(pairs: Option<&[crate::types::outlives::Pair]>) -> TestResult<Vec<(u32, u32)>> {
    let mut pairs: Vec<_> = pairs
        .ok_or("demanded requirements")?
        .iter()
        .map(|(longer, shorter)| (longer.0, shorter.0))
        .collect();
    pairs.sort_unstable();
    Ok(pairs)
}

/// The paid work of demanding the outlives contracts of `count` independent
/// nominals, then of the first bounded one, in a crate that declares `bounds`
/// explicit nominal bounds that no field implies. `restart` forgets the index
/// before every demand, as each demand regrouped every bound before the
/// repair. Every demanded contract equals the crate-wide inference's.
fn demanded_work(count: usize, bounds: usize, restart: bool) -> TestResult<usize> {
    let mut source = String::new();
    for index in 0..bounds {
        write!(source, "struct B{index}<'a,'b:'a>{{a:&'a u8,b:&'b u8}}")?;
    }
    for index in 0..count {
        write!(source, "struct L{index}<'a>{{value:&'a u8}}")?;
    }
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    assert_eq!(facts.lifetimes.nominal_bounds.len(), bounds);
    let full = std::mem::take(&mut facts.lifetimes.nominal_requirements);
    let work = Cell::new(0_usize);
    let pay = |cost: usize, _bytes: usize| {
        work.set(work.get().saturating_add(cost));
        Ok::<(), &'static str>(())
    };
    let mut items = (0..count)
        .map(|index| nominal(&report, &format!("L{index}")))
        .collect::<TestResult<Vec<_>>>()?;
    items.push(nominal(&report, "B0")?);
    for &item in &items {
        if restart {
            facts.lifetimes.explicit_bounds = None;
        }
        let predicates = &facts.generics.predicates;
        facts.lifetimes.demand_reached_outlives(
            &mut facts.types,
            &facts.aggregates,
            item,
            |owner| {
                predicates
                    .get(&DefId(owner.0))
                    .into_iter()
                    .flatten()
                    .map(|predicate| predicate.value)
            },
            &pay,
        )?;
        assert_eq!(
            sorted(facts.lifetimes.nominal_requirements.get(item))?,
            sorted(full.get(item))?
        );
    }
    assert!(!sorted(full.get(nominal(&report, "B0")?))?.is_empty());
    Ok(work.get())
}

#[test]
fn demanded_outlives_contracts_index_each_explicit_bound_once() -> TestResult {
    let grid = |count, bounds| demanded_work(count, bounds, false);
    // Indexing the bounds costs the same whichever number of nominals is
    // demanded, and each demanded nominal costs the same whatever the number
    // of bounds: the work is a sum, not a product.
    let narrow = [grid(64, 64)?, grid(64, 256)?, grid(64, 512)?];
    let wide = [grid(512, 64)?, grid(512, 256)?, grid(512, 512)?];
    let demands = [narrow[0], grid(256, 64)?, wide[0]];
    eprintln!("explicit bounds narrow={narrow:?} wide={wide:?} demands={demands:?}");
    for (wide_cost, narrow_cost) in wide.iter().zip(&narrow).skip(1) {
        assert_eq!(wide_cost - wide[0], narrow_cost - narrow[0]);
    }
    assert!((narrow[2] - narrow[1]) * 192 < (narrow[1] - narrow[0]) * 256 * 3 / 2);
    assert!((demands[2] - demands[1]) * 192 < (demands[1] - demands[0]) * 256 * 3 / 2);
    let restarted = demanded_work(512, 512, true)?;
    eprintln!("explicit bounds restarted(512, 512)={restarted}");
    // Each later demand regroups all 512 bounds again.
    assert!(restarted >= wide[2] + 512 * 512);
    Ok(())
}
