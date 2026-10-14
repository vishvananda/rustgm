//! Canonical component sets retain compact lifetime-only identities on the ordinary path.
use super::{ParameterId, Parts, RegionId};
use crate::support::intern_slice::Slices;
use std::collections::HashMap;

const SYMBOLIC: u32 = 1 << 31;

#[derive(Default)]
struct Symbolic {
    parameters: Slices<ParameterId>,
    entries: Vec<(u32, u32)>,
    index: HashMap<(u32, u32), u32>,
}
#[derive(Default)]
pub(crate) struct Lists {
    regions: Slices<RegionId>,
    symbolic: Option<Box<Symbolic>>,
}
impl Lists {
    pub(crate) fn intern(&mut self, parts: Parts<'_>) -> Result<u32, &'static str> {
        let regions = self.regions.intern(parts.regions)?;
        if regions >= SYMBOLIC {
            return Err("too many lifetime component lists");
        }
        if parts.parameters.is_empty() {
            return Ok(regions);
        }
        let symbolic = self.symbolic.get_or_insert_with(Default::default);
        let parameters = symbolic.parameters.intern(parts.parameters)?;
        let key = (regions, parameters);
        if let Some(id) = symbolic.index.get(&key) {
            return Ok(*id | SYMBOLIC);
        }
        let id = u32::try_from(symbolic.entries.len())
            .ok()
            .filter(|id| *id < SYMBOLIC)
            .ok_or("too many symbolic component lists")?;
        symbolic.entries.push(key);
        symbolic.index.insert(key, id);
        Ok(id | SYMBOLIC)
    }
    pub(crate) fn get(&self, id: u32) -> Option<Parts<'_>> {
        let (regions, parameters) = if id & SYMBOLIC == 0 {
            (id, &[][..])
        } else {
            let symbolic = self.symbolic.as_ref()?;
            let &(regions, parameters) = symbolic.entries.get((id & !SYMBOLIC) as usize)?;
            (regions, symbolic.parameters.get(parameters)?)
        };
        Some(Parts {
            regions: self.regions.get(regions)?,
            parameters,
        })
    }
}

#[derive(Default)]
pub(crate) struct Values {
    pub(crate) regions: Vec<RegionId>,
    pub(crate) parameters: Vec<ParameterId>,
}
impl Values {
    pub(crate) fn parts(&self) -> Parts<'_> {
        Parts {
            regions: &self.regions,
            parameters: &self.parameters,
        }
    }
    pub(crate) fn clear(&mut self) {
        self.regions.clear();
        self.parameters.clear();
    }
    pub(crate) fn extend(&mut self, parts: Parts<'_>) {
        self.regions.extend_from_slice(parts.regions);
        self.parameters.extend_from_slice(parts.parameters);
    }
    pub(crate) fn canonicalize(&mut self) {
        self.regions.retain(|r| *r != RegionId::STATIC);
        self.regions.sort_unstable_by_key(|r| r.0);
        self.regions.dedup();
        self.parameters.sort_unstable_by_key(|p| p.0);
        self.parameters.dedup();
    }
}
