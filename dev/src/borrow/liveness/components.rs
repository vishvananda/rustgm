//! Immutable component sets are shared by independent value and cleanup entities.
use crate::{
    support::{intern_slice::Slices, retained},
    types::{RegionId, TypeId, outlives::Parts, parameters::ParameterId},
};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Component {
    Region(RegionId),
    Parameter(ParameterId),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Value(TypeId),
    Destructor(TypeId),
    Operand(u32),
}

#[derive(Default)]
pub(super) struct Components {
    pub entities: Vec<u32>,
    pub sets: Slices<u32>,
    pub values: Vec<Component>,
}

#[derive(Default)]
pub(super) struct Builder {
    components: Components,
    index: HashMap<Component, u32>,
    queries: HashMap<Key, u32>,
    scratch: Vec<u32>,
    pub steps: usize,
}
impl Builder {
    pub fn get(&self, key: Key) -> Option<u32> {
        self.queries.get(&key).copied()
    }
    pub fn remember(&mut self, key: Key, set: u32) -> Result<(), &'static str> {
        self.charge(1)?;
        self.queries.insert(key, set);
        self.check_storage()
    }
    pub fn intern(&mut self, parts: Parts<'_>) -> Result<u32, &'static str> {
        self.charge(parts.len().saturating_mul(3).saturating_add(1))?;
        if parts.len() > 2_000_000 {
            return Err("liveness component set exceeds storage budget");
        }
        self.scratch.clear();
        for component in parts
            .regions
            .iter()
            .copied()
            .map(Component::Region)
            .chain(parts.parameters.iter().copied().map(Component::Parameter))
        {
            let id = if let Some(&id) = self.index.get(&component) {
                id
            } else {
                let id = u32::try_from(self.components.values.len())
                    .map_err(|_| "too many liveness components")?;
                self.components.values.push(component);
                self.index.insert(component, id);
                id
            };
            self.scratch.push(id);
        }
        let result = self.components.sets.intern(&self.scratch)?;
        self.check_storage()?;
        Ok(result)
    }
    pub fn entity(&mut self, set: u32) -> Result<(), &'static str> {
        self.charge(1)?;
        self.components.entities.push(set);
        self.check_storage()
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= super::LIMIT)
            .ok_or("liveness component work budget exceeded")?;
        Ok(())
    }
    fn check_storage(&self) -> Result<(), &'static str> {
        // Include the later count/state vectors as well as construction scratch.
        let bytes = retained::sum([
            retained::vector(&self.components.entities).saturating_mul(2),
            retained::vector(&self.components.values).saturating_mul(2),
            self.components.sets.allocation().bytes,
            retained::map(&self.index),
            retained::map(&self.queries),
            retained::vector(&self.scratch),
        ]);
        if bytes > 64_000_000 {
            return Err("liveness components exceed 64 MB budget");
        }
        Ok(())
    }
    pub fn finish(self) -> Components {
        self.components
    }
}
