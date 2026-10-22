//! Source dependency aliases close before external initializer arguments are substituted.
use super::{Checker, DefId, Error, NodeKey};
use crate::support::hash::HashMap;
use crate::{
    sema::{model::Declarations, values::Dependency},
    types::{
        RegionId, Store,
        substitute::{Bound, RegionEquations},
    },
};
use std::ops::Range;
const LIMIT: usize = 64 * 1024 * 1024;
const WORK: usize = 1_000_000;

pub(super) struct Frontier {
    body: NodeKey,
    owner: Option<DefId>,
    position: usize,
}
pub(super) struct Appended {
    body: NodeKey,
    owner: Option<DefId>,
    range: Range<usize>,
}
impl Checker<'_> {
    pub(super) fn dependency_frontier(&self) -> Frontier {
        let owner = self
            .state
            .environment_owner
            .filter(|_| !self.state.in_function);
        Frontier {
            body: self.state.body,
            owner,
            position: owner
                .and_then(|id| self.facts.constant_dependencies.get(&id))
                .map_or(0, Vec::len),
        }
    }
    pub(super) fn dependency_append(&self, first: &Frontier) -> Result<Appended, Error> {
        let last = self.dependency_frontier();
        if first.body != last.body || first.owner != last.owner || first.position > last.position {
            return Err(self.error(
                self.state.body.node,
                "constant dependency append owner changed",
            ));
        }
        Ok(Appended {
            body: first.body,
            owner: first.owner,
            range: first.position..last.position,
        })
    }
    pub(super) fn canonicalize_dependencies(
        &mut self,
        appended: [Appended; 2],
        equations: [Range<usize>; 2],
    ) -> Result<(), Error> {
        let source = self.state.body;
        if appended
            .iter()
            .any(|row| row.body != source || row.owner != appended[0].owner)
            || appended[0].range.end > appended[1].range.start
        {
            return Err(self.error(
                source.node,
                "constant dependency publication has foreign ownership",
            ));
        }
        let Some(owner) = appended[0].owner else {
            return Ok(());
        };
        let [first, second] = appended;
        let result = canonicalize(
            self.facts,
            owner,
            source,
            self.state.body_binders,
            [first.range, second.range],
            equations,
        );
        result.map_err(|message| self.error(source.node, message))
    }
}

fn canonicalize(
    facts: &mut Declarations,
    owner: DefId,
    source: NodeKey,
    universals: u32,
    ranges: [Range<usize>; 2],
    equations: [Range<usize>; 2],
) -> Result<(), &'static str> {
    let count = ranges.iter().try_fold(0_usize, |n, range| {
        range
            .end
            .checked_sub(range.start)
            .and_then(|length| n.checked_add(length))
            .ok_or("constant dependency publication range is reversed")
    })?;
    if count == 0 {
        return Ok(());
    }
    let scratch = count.saturating_mul(size_of::<Dependency>());
    if count > WORK || scratch > LIMIT {
        return Err("constant dependency publication storage budget exceeded");
    }
    let rows = facts
        .constant_dependencies
        .get(&owner)
        .ok_or("constant dependency publication lost its owner")?;
    let mut work = count;
    let needed = needs_regions(facts, rows, &ranges, &mut work);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    if !needed? {
        return Ok(());
    }
    let mut selected = Vec::with_capacity(count);
    for range in &ranges {
        selected.extend_from_slice(
            rows.get(range.clone())
                .ok_or("constant dependency publication range is stale")?,
        );
    }
    let scratch = crate::support::retained::vector(&selected);
    let mut work = 0;
    let result = (|| {
        let mut fold = source_folder(
            facts, source, universals, &selected, equations, scratch, &mut work,
        )?;
        let result: Result<(), &'static str> = (|| {
            for row in &mut selected {
                *row = apply(&mut fold, &mut facts.types, *row)?;
            }
            Ok(())
        })();
        work = work.saturating_add(fold.steps);
        result
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result?;
    let rows = facts
        .constant_dependencies
        .get_mut(&owner)
        .ok_or("constant dependency publication lost its owner")?;
    let mut selected = selected.into_iter();
    for range in ranges {
        for row in rows
            .get_mut(range)
            .ok_or("constant dependency publication became stale")?
        {
            *row = selected
                .next()
                .ok_or("constant dependency publication lost a row")?;
        }
    }
    Ok(())
}
fn needs_regions(
    facts: &Declarations,
    rows: &[Dependency],
    ranges: &[Range<usize>; 2],
    work: &mut usize,
) -> Result<bool, &'static str> {
    use crate::types::arguments::Argument;
    let mut needed = false;
    for range in ranges {
        for row in rows
            .get(range.clone())
            .ok_or("constant dependency publication range is stale")?
        {
            match *row {
                Dependency::Constant(instance) => {
                    for argument in facts
                        .types
                        .arguments(instance.arguments)
                        .ok_or("constant dependency has invalid arguments")?
                    {
                        *work = work.saturating_add(1);
                        needed |= match argument {
                            Argument::Lifetime(region) => region != RegionId::STATIC,
                            Argument::Type(ty) => facts
                                .types
                                .needs_region_context(ty)
                                .ok_or("constant dependency has an invalid type")?,
                            Argument::Const(_) => false,
                        };
                    }
                }
                Dependency::Callable(ty) => {
                    needed |= facts
                        .types
                        .needs_region_context(ty)
                        .ok_or("callable dependency has an invalid type")?;
                }
            }
            if *work > WORK {
                return Err("constant dependency publication work budget exceeded");
            }
        }
    }
    Ok(needed)
}

fn source_folder(
    facts: &mut Declarations,
    source: NodeKey,
    universals: u32,
    selected: &[Dependency],
    ranges: [Range<usize>; 2],
    scratch: usize,
    work: &mut usize,
) -> Result<Bound, &'static str> {
    let mut observer = Bound::observe_budgeted_regions(WORK, LIMIT.saturating_sub(scratch))?;
    let outcome: Result<(), &'static str> = (|| {
        for &dependency in selected {
            apply(&mut observer, &mut facts.types, dependency)?;
        }
        Ok(())
    })();
    *work = work.saturating_add(observer.steps);
    outcome?;
    let (bytes, visits) = observer.retained_allocation();
    *work = work.saturating_add(visits);
    let n = ranges.iter().try_fold(0_usize, |count, range| {
        range
            .end
            .checked_sub(range.start)
            .and_then(|length| count.checked_add(length))
            .ok_or("constant dependency equality range is reversed")
    })?;
    let extra = Bound::projection_closure_preflight(n, 0);
    let map_count = observer.observed_regions().count();
    *work = work.saturating_add(map_count);
    let map_bytes = crate::support::retained::new_map::<RegionId, RegionId>(map_count);
    if scratch
        .saturating_add(bytes)
        .saturating_add(extra)
        .saturating_add(map_bytes)
        > LIMIT
    {
        return Err("constant dependency alias storage budget exceeded");
    }
    let mut closure = published_equations(facts, source, ranges, work)?;
    let mut replacements = HashMap::with_capacity_and_hasher(
        map_count,
        crate::support::hash::BuildFxHasher::default(),
    );
    for region in observer.observed_regions() {
        *work = work.saturating_add(2);
        original(facts, source, universals, region)?;
        let anchor = closure.canonical_anchor(region, work)?;
        original(facts, source, universals, anchor)?;
        replacements.insert(region, anchor);
    }
    drop(observer);
    drop(closure);
    Bound::restore_budgeted_regions(
        replacements,
        WORK.saturating_sub(*work),
        LIMIT.saturating_sub(scratch),
    )
}
fn published_equations(
    facts: &Declarations,
    source: NodeKey,
    ranges: [Range<usize>; 2],
    work: &mut usize,
) -> Result<RegionEquations, &'static str> {
    if ranges[0].end > ranges[1].start {
        return Err("constant dependency equality ranges overlap");
    }
    let mut closure = RegionEquations::default();
    for range in ranges {
        let slice = facts
            .region_equalities
            .get(range)
            .ok_or("constant dependency equality range is stale")?;
        *work = work.saturating_add(slice.len());
        if slice.iter().any(|row| row.body != source) {
            return Err("constant dependency equality range has a foreign owner");
        }
        closure.extend_counted(slice.iter().map(|row| (row.left, row.right)), work)?;
    }
    Ok(closure)
}
fn original(
    facts: &Declarations,
    source: NodeKey,
    universals: u32,
    region: RegionId,
) -> Result<(), &'static str> {
    if region == RegionId::STATIC {
        return Ok(());
    }
    if let Some((depth, slot)) = region.bound_parts() {
        return if depth == 0 && u32::from(slot) < universals {
            Ok(())
        } else {
            Err("constant dependency has a foreign binder")
        };
    }
    if facts
        .body_regions
        .get(region.0 as usize - 1)
        .is_some_and(|row| row.body == source)
    {
        Ok(())
    } else {
        Err("constant dependency has a foreign source identity")
    }
}
fn apply(fold: &mut Bound, store: &mut Store, row: Dependency) -> Result<Dependency, &'static str> {
    Ok(match row {
        Dependency::Constant(mut instance) => {
            instance.arguments = fold.apply_arguments(store, instance.arguments)?;
            Dependency::Constant(instance)
        }
        Dependency::Callable(ty) => Dependency::Callable(fold.apply(store, ty)?),
    })
}

#[cfg(test)]
#[path = "../../../tests/invariants/dependency_scopes.rs"]
mod tests;
