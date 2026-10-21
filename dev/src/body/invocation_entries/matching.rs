//! Match body-universal slots without conflating nested callable binders.
use crate::{
    sema::closures::Execution,
    types::{Kind, RegionId, Store, TypeId, region_graph::Error, structural::Compare},
};
use std::collections::{HashMap, HashSet};
struct Matcher<'a> {
    types: &'a Store,
    universals: u32,
    slots: HashMap<u16, RegionId>,
    equalities: Vec<(RegionId, RegionId)>,
    work: Vec<(TypeId, TypeId, u16)>,
    seen: HashSet<(TypeId, TypeId, u16)>,
    steps: usize,
}
pub(super) fn execution(
    types: &Store,
    formal: Execution,
    actual: TypeId,
) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
    let Some(Kind::Function {
        parameters: a,
        result: ar,
        bound_regions: ab,
        unsafe_: au,
        convention: av,
    }) = types.kind(formal.signature)
    else {
        return Err("missing formal closure execution signature");
    };
    let Some(Kind::Function {
        parameters: b,
        result: br,
        bound_regions: 0,
        unsafe_: bu,
        convention: bv,
    }) = types.kind(actual)
    else {
        return Err("missing selected closure execution signature");
    };
    if ab != formal.universals || au != bu || av != bv {
        return Err("closure execution binder or ABI differs from its contract");
    }
    let mut matcher = Matcher::new(types, formal.universals, ar, br);
    if !matcher
        .list(a, b, 0)
        .map_err(|_| "invalid closure execution parameter shape")?
    {
        return Err("closure execution parameter count differs");
    }
    matcher.finish()
}
pub(super) fn instance(
    types: &Store,
    universals: u32,
    formal: TypeId,
    actual: TypeId,
) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
    Matcher::new(types, universals, formal, actual).finish()
}
impl<'a> Matcher<'a> {
    fn new(types: &'a Store, universals: u32, formal: TypeId, actual: TypeId) -> Self {
        Self {
            types,
            universals,
            slots: HashMap::new(),
            equalities: Vec::new(),
            work: vec![(formal, actual, 0)],
            seen: HashSet::new(),
            steps: 0,
        }
    }
    fn finish(mut self) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
        while let Some((a, b, depth)) = self.work.pop() {
            self.charge(1)
                .map_err(|_| "closure execution matching budget exceeded")?;
            if !self.seen.insert((a, b, depth)) {
                continue;
            }
            if !self
                .compare(a, b, depth)
                .map_err(|_| "invalid closure execution matching identity")?
            {
                return Err("selected closure execution differs structurally from source contract");
            }
        }
        Ok(self.equalities)
    }
}
impl<'a> Compare<'a> for Matcher<'a> {
    fn types(&self) -> &'a Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.work.push((a, b, depth));
        Ok(())
    }
    fn regions(&mut self, a: RegionId, b: RegionId, depth: u16) -> Result<bool, Error> {
        self.charge(1)?;
        let Some((owner, slot)) = a.bound_parts() else {
            return Ok(a == b);
        };
        if owner != depth {
            return Ok(a == b);
        }
        if u32::from(slot) >= self.universals {
            return Err(Error::Identity);
        }
        let actual = if let Some((owner, slot)) = b.bound_parts() {
            let Some(owner) = owner.checked_sub(depth) else {
                return Ok(false);
            };
            RegionId::bound(owner, slot).map_err(|_| Error::Identity)?
        } else {
            b
        };
        if let Some(&previous) = self.slots.get(&slot) {
            if previous != actual {
                self.equalities.push((previous, actual));
            }
        } else {
            self.slots.insert(slot, actual);
        }
        Ok(true)
    }
}
