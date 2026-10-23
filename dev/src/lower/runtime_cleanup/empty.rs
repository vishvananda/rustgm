//! Only a completed empty inventory can demand exact no-cleanup answers.
use super::{NeedsDrop, State, Store, TypeId};
use crate::types::aggregate::Definitions;

impl State<'_> {
    pub(super) fn dropless(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        element: TypeId,
        complete_empty: bool,
        limit: usize,
    ) -> Result<(), &'static str> {
        if !std::ptr::eq(self.types, types) || !std::ptr::eq(self.definitions, definitions) {
            return Err("source drop glue query belongs to another structural inventory");
        }
        if !complete_empty {
            return Err("source drop glue lacks its proved destruction recipes");
        }
        if self.empty.is_none() {
            self.empty = Some(Box::new(NeedsDrop::concrete_bounded(
                std::iter::empty(),
                limit,
            )?));
        }
        if self
            .empty
            .as_deref_mut()
            .ok_or("source drop glue query disappeared")?
            .demand_concrete_bounded(types, definitions, element, limit)?
        {
            return Err("source drop glue cannot omit required cleanup");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/empty_cleanup.rs"]
mod tests;
