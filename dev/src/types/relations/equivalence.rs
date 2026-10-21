//! Named lifetime equivalence from caller assumptions or a completed body proof.
use super::{Constraints, Error, Graph, Identities, Region, RegionId, RegionKind, Solution};
use crate::types::substitute;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct RegionEqualities {
    representatives: HashMap<RegionId, RegionId>,
}
impl RegionEqualities {
    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.representatives);
        total
    }
}
impl Constraints<'_> {
    pub(crate) fn assumed_equalities(&self) -> Result<RegionEqualities, Error> {
        let binder = self.binders.first().ok_or(Error::Identity)?;
        let mut names = vec![(RegionId::STATIC, Region::STATIC)];
        for (index, &region) in self.arguments[binder.start..binder.end].iter().enumerate() {
            let index = u16::try_from(index).map_err(|_| Error::Budget)?;
            names.push((
                RegionId::bound(0, index).map_err(|_| Error::Identity)?,
                region,
            ));
        }
        RegionEqualities::from_graph(&self.graph, &names)
    }
}
impl RegionEqualities {
    fn from_graph(graph: &Graph, names: &[(RegionId, Region)]) -> Result<Self, Error> {
        let classes = graph.assumption_classes()?;
        Ok(Self::from_classes(&classes, names))
    }
    pub(crate) fn proven(solution: &Solution, identities: &Identities) -> Result<Self, Error> {
        let mut names = vec![(RegionId::STATIC, Region::STATIC)];
        for (slot, region) in identities.universals().enumerate() {
            let slot = u16::try_from(slot).map_err(|_| Error::Budget)?;
            names.push((
                RegionId::bound(0, slot).map_err(|_| Error::Identity)?,
                region,
            ));
        }
        names.extend(identities.variables());
        Ok(Self::from_classes(&solution.established_classes()?, &names))
    }
    fn from_classes(classes: &[Region], names: &[(RegionId, Region)]) -> Self {
        let mut representatives = HashMap::<Region, RegionId>::new();
        for &(name, region) in names {
            representatives
                .entry(classes[region.0 as usize])
                .and_modify(|old| {
                    if name.0 < old.0 {
                        *old = name;
                    }
                })
                .or_insert(name);
        }
        let mut result = Self::default();
        for &(name, region) in names {
            let representative = representatives[&classes[region.0 as usize]];
            if representative != name {
                result.representatives.insert(name, representative);
            }
        }
        result
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.representatives.is_empty()
    }
    /// Only retained environment identities are canonicalized. Substitution may
    /// relocate those captures; newly introduced nested locals remain distinct.
    pub(crate) fn canonical(&self, region: RegionId, depth: u16) -> Result<RegionId, Error> {
        let key = match region.bound_parts() {
            Some((binder, slot)) if binder >= depth => {
                RegionId::bound(binder - depth, slot).map_err(|_| Error::Identity)?
            }
            Some(_) => return Ok(region),
            None => region,
        };
        let Some(&canonical) = self.representatives.get(&key) else {
            return Ok(region);
        };
        match canonical.bound_parts() {
            Some((binder, slot)) => {
                RegionId::bound(binder.checked_add(depth).ok_or(Error::Budget)?, slot)
                    .map_err(|_| Error::Identity)
            }
            None => Ok(canonical),
        }
    }
    /// These aliases are created by the body checker for its own universals,
    /// never inferred from arbitrary free-variable obligations.
    pub(crate) fn with_aliases(mut self, aliases: &[RegionId]) -> Result<Self, Error> {
        for (index, &alias) in aliases.iter().enumerate() {
            if alias == RegionId::STATIC || alias.bound_parts().is_some() {
                return Err(Error::Identity);
            }
            let bound = RegionId::bound(0, u16::try_from(index).map_err(|_| Error::Budget)?)
                .map_err(|_| Error::Identity)?;
            let canonical = self.canonical(bound, 0)?;
            if self.representatives.insert(alias, canonical).is_some() {
                return Err(Error::Identity);
            }
        }
        Ok(self)
    }
    pub(crate) fn substitute(&self, fold: &mut substitute::Bound) -> Result<Self, &'static str> {
        let mut graph = Graph::new(1_000_000);
        let mut names = HashMap::from([(RegionId::STATIC, Region::STATIC)]);
        let mut pairs: Vec<_> = self.representatives.iter().map(|(&a, &b)| (a, b)).collect();
        pairs.sort_unstable_by_key(|&(a, _)| a.0);
        for (a, b) in pairs {
            let a = fold.region(a, 0)?;
            let b = fold.region(b, 0)?;
            let mut intern = |name| -> Result<Region, Error> {
                if let Some(&region) = names.get(&name) {
                    return Ok(region);
                }
                let region = graph.region(RegionKind::Universal { universe: 0 })?;
                names.insert(name, region);
                Ok(region)
            };
            let a = intern(a).map_err(|_| "mapped lifetime equality storage exceeded")?;
            let b = intern(b).map_err(|_| "mapped lifetime equality storage exceeded")?;
            graph
                .assume(a, b)
                .map_err(|_| "invalid mapped lifetime equality")?;
            graph
                .assume(b, a)
                .map_err(|_| "invalid mapped lifetime equality")?;
        }
        Self::from_graph(&graph, &names.into_iter().collect::<Vec<_>>())
            .map_err(|_| "mapped lifetime equality work exceeded")
    }
}
