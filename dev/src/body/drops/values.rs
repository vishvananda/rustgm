//! Ownership storage for destructible rvalues, separate from borrowed-place uses.
use crate::body::{
    ir::{Temporary, ValueId},
    scopes::Lifetime,
    storage::Location,
};
use std::collections::HashMap;

/// Returning transfers the result before the callee's local cleanup starts.
#[derive(Clone, Copy)]
pub struct Release {
    pub value: ValueId,
    pub location: Location,
    pub exits_before: u32,
}
#[derive(Default)]
pub struct Facts {
    pub created: Vec<Temporary>,
    pub returned: Vec<Release>,
    index: HashMap<ValueId, usize>,
}
impl Facts {
    #[must_use]
    pub fn creation(&self, value: ValueId) -> Option<&Temporary> {
        self.index.get(&value).map(|&index| &self.created[index])
    }
    pub(crate) fn insert(&mut self, temporary: Temporary) -> Result<(), &'static str> {
        if self
            .index
            .insert(temporary.value, self.created.len())
            .is_some()
        {
            return Err("duplicate owned value creation");
        }
        self.created.push(temporary);
        Ok(())
    }
    pub(crate) fn forward(&mut self, value: ValueId, lifetime: Lifetime) {
        if let Some(&index) = self.index.get(&value) {
            self.created[index].lifetime = lifetime;
        }
    }
}
