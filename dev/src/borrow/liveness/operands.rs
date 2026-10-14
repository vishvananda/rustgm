//! Coerced operands keep their input lifetime needs until a consumer stores them.
use crate::{
    body::ir::{Body, Rvalue, ValueId},
    types::{
        RegionId, Store,
        outlives::{
            LiveRegions, Parts,
            storage::{Lists, Values},
        },
    },
};

/// Values have separate liveness bits even when their requirements are shared.
/// In particular a replacement operand must not acquire the destination local's
/// region before its defining store: that would keep the old value's loans alive.
/// Aggregate/repeat construction composes its operands, including nested coercions.
pub(super) struct Requirements {
    entities: Vec<Option<u32>>,
    lists: Lists,
    pub steps: usize,
}
impl Requirements {
    pub fn new(
        body: &Body,
        types: &Store,
        regions: &mut LiveRegions,
        entities: &[Option<usize>],
        count: usize,
    ) -> Result<Self, &'static str> {
        let mut this = Self {
            entities: vec![None; count],
            lists: Lists::default(),
            steps: 0,
        };
        let mut values = Vec::with_capacity(body.values.len());
        let mut components = Values::default();
        let mut retained = count
            .saturating_mul(size_of::<Option<u32>>())
            .saturating_add(body.values.len().saturating_mul(size_of::<u32>()));
        this.charge(count.saturating_add(body.values.len()))?;
        for (index, value) in body.values.iter().enumerate() {
            let input = |id: ValueId| {
                values
                    .get(id.0 as usize)
                    .copied()
                    .ok_or("operand lifetime input must precede its result")
            };
            let id = match value.kind {
                Rvalue::Constrain(value) | Rvalue::Repeat { value, .. } => input(value)?,
                Rvalue::Aggregate { fields, .. } => {
                    components.clear();
                    for &value in body.arguments(fields) {
                        let id = input(value)?;
                        let ids = this.lists.get(id).ok_or("invalid operand lifetime list")?;
                        let cost = ids.len() + 1;
                        this.steps = this.steps.saturating_add(cost);
                        if this.steps > 32_000_000 {
                            return Err("operand lifetime work budget exceeded");
                        }
                        components.extend(ids);
                    }
                    this.charge(
                        components
                            .parts()
                            .len()
                            .saturating_mul(components.parts().len().max(1).ilog2() as usize + 1),
                    )?;
                    components.canonicalize();
                    this.intern(components.parts(), &mut retained)?
                }
                _ => this.intern(regions.symbolic(types, value.ty)?, &mut retained)?,
            };
            values.push(id);
            if let Some(entity) = entities[index] {
                this.entities[entity] = Some(id);
            }
        }
        Ok(this)
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= 32_000_000)
            .ok_or("operand lifetime work budget exceeded")?;
        Ok(())
    }
    fn intern(&mut self, ids: Parts<'_>, retained: &mut usize) -> Result<u32, &'static str> {
        self.charge(ids.len() + 1)?;
        *retained = retained
            .checked_add(
                ids.len()
                    .saturating_mul(size_of::<RegionId>())
                    .saturating_add(if ids.parameters.is_empty() { 64 } else { 192 }),
            )
            .filter(|n| *n <= 64_000_000)
            .ok_or("operand lifetime storage exceeds 64 MB budget")?;
        self.lists.intern(ids)
    }
    pub fn get(&self, entity: usize) -> Result<Option<Parts<'_>>, &'static str> {
        self.entities
            .get(entity)
            .copied()
            .flatten()
            .map(|id| self.lists.get(id).ok_or("invalid operand lifetime list"))
            .transpose()
    }
}
