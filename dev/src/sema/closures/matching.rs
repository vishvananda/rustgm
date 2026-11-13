//! Match body-universal slots without conflating nested callable binders.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::closures::Execution,
    types::{Kind, RegionId, Store, TypeId, region_graph::Error, structural::Compare},
};
struct Matcher<'a> {
    types: &'a Store,
    universals: u32,
    slots: HashMap<u16, RegionId>,
    equalities: Vec<(RegionId, RegionId)>,
    work: Vec<(TypeId, TypeId, u16)>,
    seen: HashSet<(TypeId, TypeId, u16)>,
    steps: usize,
}
pub(crate) fn execution(
    types: &Store,
    formal: Execution,
    actual: TypeId,
) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
    execution_bindings(types, formal, actual).map(|bindings| bindings.equalities)
}
pub(crate) fn execution_bindings(
    types: &Store,
    formal: Execution,
    actual: TypeId,
) -> Result<Bindings, &'static str> {
    signature_bindings(types, formal.signature, formal.universals, actual, 0)
        .map(|(bindings, _)| bindings)
}
pub(crate) fn signature_bindings(
    types: &Store,
    formal: TypeId,
    universals: u32,
    actual: TypeId,
    public: u32,
) -> Result<(Bindings, usize), &'static str> {
    let Some(Kind::Function {
        parameters: a,
        result: ar,
        bound_regions: ab,
        unsafe_: au,
        convention: av,
    }) = types.kind(formal)
    else {
        return Err("missing formal closure execution signature");
    };
    let Some(Kind::Function {
        parameters: b,
        result: br,
        bound_regions: bb,
        unsafe_: bu,
        convention: bv,
    }) = types.kind(actual)
    else {
        return Err("missing selected closure execution signature");
    };
    if ab != universals || bb != public || au != bu || av != bv {
        return Err("closure execution binder or ABI differs from its contract");
    }
    let mut matcher = Matcher::new(types, universals, ar, br);
    if !matcher
        .list(a, b, 0)
        .map_err(|_| "invalid closure execution parameter shape")?
    {
        return Err("closure execution parameter count differs");
    }
    matcher.bindings_counted()
}
pub(crate) fn instance(
    types: &Store,
    universals: u32,
    formal: TypeId,
    actual: TypeId,
) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
    Matcher::new(types, universals, formal, actual).finish()
}
pub(crate) struct Bindings {
    pub(crate) universals: u32,
    pub slots: HashMap<u16, RegionId>,
    pub equalities: Vec<(RegionId, RegionId)>,
}
#[cfg(test)]
pub(crate) struct Prefix {
    pub region: RegionId,
    pub universals: u32,
    pub equalities: Vec<(RegionId, RegionId)>,
}
pub(crate) struct Prefixes {
    pub regions: Vec<RegionId>,
    pub universals: u32,
    pub equalities: Vec<(RegionId, RegionId)>,
}
pub(crate) fn bind_instance(
    types: &Store,
    universals: u32,
    formal: TypeId,
    actual: TypeId,
) -> Result<Bindings, &'static str> {
    Matcher::new(types, universals, formal, actual).bindings()
}
pub(crate) fn bind_instance_counted(
    types: &Store,
    universals: u32,
    formal: TypeId,
    actual: TypeId,
) -> Result<(Bindings, usize), &'static str> {
    Matcher::new(types, universals, formal, actual).bindings_counted()
}
impl Bindings {
    #[cfg(test)]
    pub(crate) fn prefix(self, actual: RegionId) -> Result<Prefix, &'static str> {
        let mut prefix = self.prefixes(&[actual])?;
        Ok(Prefix {
            region: prefix.regions.pop().ok_or("missing adapter prefix")?,
            universals: prefix.universals,
            equalities: prefix.equalities,
        })
    }
    pub(crate) fn prefixes(self, actual: &[RegionId]) -> Result<Prefixes, &'static str> {
        if actual.len().saturating_add(self.slots.len()) > 1_000_000 {
            return Err("adapter prefix mapping exceeds its work budget");
        }
        let mut images = HashMap::<RegionId, u16>::with_capacity_and_hasher(
            self.slots.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for (slot, region) in self.slots {
            images
                .entry(region)
                .and_modify(|old| *old = (*old).min(slot))
                .or_insert(slot);
        }
        let mut universals = self.universals;
        let mut regions = Vec::with_capacity(actual.len());
        for &region in actual {
            if region == RegionId::STATIC {
                regions.push(region);
                continue;
            }
            let slot = if let Some(&slot) = images.get(&region) {
                slot
            } else {
                let slot =
                    u16::try_from(universals).map_err(|_| "too many adapter prefix lifetimes")?;
                universals = universals
                    .checked_add(1)
                    .ok_or("adapter lifetime overflow")?;
                images.insert(region, slot);
                slot
            };
            regions.push(RegionId::bound(0, slot)?);
        }
        Ok(Prefixes {
            regions,
            universals,
            equalities: self.equalities,
        })
    }
}
impl<'a> Matcher<'a> {
    fn new(types: &'a Store, universals: u32, formal: TypeId, actual: TypeId) -> Self {
        Self {
            types,
            universals,
            slots: HashMap::default(),
            equalities: Vec::new(),
            work: vec![(formal, actual, 0)],
            seen: HashSet::default(),
            steps: 0,
        }
    }
    fn finish(self) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
        self.bindings().map(|bindings| bindings.equalities)
    }
    fn bindings(self) -> Result<Bindings, &'static str> {
        self.bindings_counted().map(|(bindings, _)| bindings)
    }
    fn bindings_counted(mut self) -> Result<(Bindings, usize), &'static str> {
        while let Some((a, b, depth)) = self.work.pop() {
            self.charge(1)
                .map_err(|_| "closure execution matching budget exceeded")?;
            let a = self.types.storage_head(a)?;
            let b = self.types.storage_head(b)?;
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
        Ok((
            Bindings {
                universals: self.universals,
                slots: self.slots,
                equalities: self.equalities,
            },
            self.steps,
        ))
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
