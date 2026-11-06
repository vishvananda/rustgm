//! Expected results propose only this call's fresh late lifetime arguments.
use crate::support::{hash::HashSet, retained};
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
    payment: Option<retained::Payment<'a>>,
    failure: Option<&'static str>,
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
        payment: None,
        failure: None,
    };
    if !query.run()? {
        query.candidates.regions.clear();
    }
    Ok(query.candidates)
}

/// Validate one actual call opening across every input and result. This is a
/// rigid correspondence, not a source call, region proof or projection inverse.
pub(crate) fn signature_opening<'a>(
    types: &'a Store,
    declared: TypeId,
    opened: TypeId,
    payment: retained::Payment<'a>,
) -> Result<bool, &'static str> {
    use crate::types::Kind;
    let Some(Kind::Function {
        bound_regions: count,
        parameters: left,
        result: lr,
        unsafe_: lu,
        convention: lc,
    }) = types.kind(declared)
    else {
        return Err("call opening lacks its actual declared signature");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters: right,
        result: rr,
        unsafe_: ru,
        convention: rc,
    }) = types.kind(opened)
    else {
        return Ok(false);
    };
    if count > 65_536 {
        return Err("call opening lifetime inventory budget exhausted");
    }
    if lu != ru || lc != rc {
        return Ok(false);
    }
    let left = types.list(left).ok_or("invalid declared call parameters")?;
    let right = types.list(right).ok_or("invalid opened call parameters")?;
    if left.len() != right.len() {
        return Ok(false);
    }
    payment(
        count as usize,
        (count as usize).saturating_mul(size_of::<Option<RegionId>>()),
    )?;
    let mut query = Match {
        types,
        candidates: Candidates {
            regions: vec![None; count as usize],
            steps: count as usize,
        },
        work: Vec::new(),
        seen: HashSet::default(),
        payment: Some(payment),
        failure: None,
    };
    for (&a, &b) in left.iter().zip(right) {
        query
            .child(a, b, 0)
            .map_err(|_| query.failure.unwrap_or("call opening work exhausted"))?;
    }
    query
        .child(lr, rr, 0)
        .map_err(|_| query.failure.unwrap_or("call opening work exhausted"))?;
    query.run()
}

impl Match<'_> {
    fn pay(&mut self, work: usize, bytes: usize) -> Result<(), Error> {
        if let Some(payment) = self.payment
            && let Err(error) = payment(work, bytes)
        {
            self.failure.get_or_insert(error);
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn run(&mut self) -> Result<bool, &'static str> {
        while let Some((actual, expected, depth)) = self.work.pop() {
            self.charge(1).map_err(|_| {
                self.failure
                    .unwrap_or("call result candidate work exhausted")
            })?;
            if actual == expected && self.types.escaping_binders(actual) == Some(0)
                || self.seen.contains(&(actual, expected, depth))
            {
                continue;
            }
            if self.payment.is_some() {
                let growth = retained::Growth::map::<(TypeId, TypeId, u16), ()>(
                    self.seen.len(),
                    self.seen.capacity(),
                    1,
                );
                self.pay(growth.work, growth.bytes)
                    .map_err(|_| self.failure.unwrap_or("call opening storage exhausted"))?;
                self.seen
                    .try_reserve(1)
                    .map_err(|_| "call opening allocation failed")?;
            }
            self.seen.insert((actual, expected, depth));
            if !self.compare(actual, expected, depth).map_err(|_| {
                self.failure
                    .unwrap_or("invalid or exhausted call result candidate")
            })? {
                return Ok(false);
            }
        }
        Ok(true)
    }
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
        self.pay(count, 0)
    }
    fn child(&mut self, actual: TypeId, expected: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        if self.payment.is_some() {
            let growth = retained::Growth::vector::<(TypeId, TypeId, u16)>(
                self.work.len(),
                self.work.capacity(),
                1,
            );
            self.pay(growth.work, growth.bytes)?;
            self.work.try_reserve(1).map_err(|_| Error::Budget)?;
        }
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
