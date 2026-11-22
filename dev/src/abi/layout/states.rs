//! Each layout query caches only the states whose types it touches.
use super::{Budget, State, TypeId};
use crate::support::hash::HashMap;

#[derive(Default)]
pub(super) struct States {
    // Keep the layout cache small until the first query.
    entries: Option<Box<HashMap<TypeId, State>>>,
}

impl States {
    pub(super) fn get(&self, ty: TypeId) -> Option<&State> {
        self.entries.as_ref()?.get(&ty)
    }

    pub(super) fn get_mut(&mut self, ty: TypeId) -> Option<&mut State> {
        self.entries.as_mut()?.get_mut(&ty)
    }

    pub(super) fn begin(&mut self, ty: TypeId, budget: &mut Budget) -> Result<(), &'static str> {
        let entries = self.entries.get_or_insert_with(Box::default);
        entries.try_reserve(1).map_err(|_| {
            budget.exhausted = true;
            "layout state allocation failed"
        })?;
        entries.insert(ty, State::Active);
        Ok(())
    }

    pub(super) fn remove(&mut self, ty: TypeId) {
        if let Some(entries) = &mut self.entries {
            entries.remove(&ty);
        }
    }
}
