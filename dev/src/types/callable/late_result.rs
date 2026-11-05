//! Expected results propose only this call's fresh late lifetime arguments.
use crate::support::hash::HashSet;
use crate::types::{RegionId, Store, TypeId, region_graph::Error, structural::Compare};

pub(crate) struct Candidates {
    pub regions: Vec<Option<RegionId>>,
    pub steps: usize,
}
struct Match<'a> {
    types: &'a Store,
    candidates: Candidates,
    work: Vec<(TypeId, TypeId, u16)>,
    seen: HashSet<(TypeId, TypeId, u16)>,
}

/// A complete rigid match supplies a sufficient candidate instantiation, not
/// an inverse law for a projection. The call must still prove every real input,
/// result, formation, region and loan condition. No type/const input is inferred.
pub(crate) fn candidates(
    types: &Store,
    result: TypeId,
    expected: TypeId,
    count: u32,
) -> Result<Candidates, &'static str> {
    if count > 65_536 {
        return Err("call result lifetime candidate budget exhausted");
    }
    let mut query = Match {
        types,
        candidates: Candidates {
            regions: vec![None; count as usize],
            steps: count as usize,
        },
        work: vec![(result, expected, 0)],
        seen: HashSet::default(),
    };
    while let Some((actual, expected, depth)) = query.work.pop() {
        query
            .charge(1)
            .map_err(|_| "call result candidate work exhausted")?;
        if actual == expected && types.escaping_binders(actual) == Some(0)
            || !query.seen.insert((actual, expected, depth))
        {
            continue;
        }
        if !query
            .compare(actual, expected, depth)
            .map_err(|_| "invalid or exhausted call result candidate")?
        {
            query.candidates.regions.clear();
            break;
        }
    }
    Ok(query.candidates)
}

impl<'a> Compare<'a> for Match<'a> {
    fn types(&self) -> &'a Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.candidates.steps = self.candidates.steps.saturating_add(count);
        if self.candidates.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, actual: TypeId, expected: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.work.push((actual, expected, depth));
        Ok(())
    }
    fn regions(&mut self, actual: RegionId, expected: RegionId, depth: u16) -> Result<bool, Error> {
        let Some((binder, slot)) = actual.bound_parts() else {
            return Ok(actual == expected);
        };
        if binder < depth {
            return Ok(actual == expected);
        }
        if binder > depth {
            return Ok(RegionId::bound(binder - 1, slot).map_err(|_| Error::Identity)? == expected);
        }
        let expected = match expected.bound_parts() {
            Some((binder, _)) if binder < depth => return Ok(false),
            Some((binder, slot)) => {
                RegionId::bound(binder - depth, slot).map_err(|_| Error::Identity)?
            }
            None => expected,
        };
        let candidate = self
            .candidates
            .regions
            .get_mut(usize::from(slot))
            .ok_or(Error::Identity)?;
        if candidate.is_some_and(|previous| previous != expected) {
            return Ok(false);
        }
        *candidate = Some(expected);
        Ok(true)
    }
}
