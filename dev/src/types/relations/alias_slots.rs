//! Compiler-created body aliases retain ordered declaration slots, not proof equalities.
use crate::{support::hash::HashMap, types::RegionId};

pub(crate) struct BodyAliases {
    ordered: Vec<RegionId>,
    slots: HashMap<RegionId, u16>,
}
impl BodyAliases {
    pub(crate) fn new(ordered: Vec<RegionId>) -> Result<Self, &'static str> {
        let mut slots = HashMap::default();
        for (slot, &alias) in ordered.iter().enumerate() {
            let slot = u16::try_from(slot).map_err(|_| "too many body alias slots")?;
            if alias == RegionId::STATIC
                || alias.bound_parts().is_some()
                || slots.insert(alias, slot).is_some()
            {
                return Err("invalid or repeated body universal alias");
            }
        }
        Ok(Self { ordered, slots })
    }

    pub(crate) fn len(&self) -> usize {
        self.ordered.len()
    }

    pub(crate) fn slot(&self, alias: RegionId) -> Option<u16> {
        self.slots.get(&alias).copied()
    }

    pub(crate) fn ordered(&self) -> &[RegionId] {
        &self.ordered
    }

    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.ordered);
        total.map(&self.slots);
        total
    }
}
