//! Body formation reads the normalized independent input checked for its own declaration.
use super::{DefId, Generics, TypeId};
use crate::support::hash::HashMap;
use crate::types::parameters::clauses::QuantifiedId;
use std::collections::hash_map::Entry;

#[derive(Default)]
pub(super) struct Builtins {
    inputs: HashMap<(DefId, QuantifiedId), Option<TypeId>>,
}
impl Generics {
    pub(in crate::sema) fn record_builtin_formation(
        &mut self,
        owner: DefId,
        clause: QuantifiedId,
        subject: Option<TypeId>,
    ) -> Result<(), &'static str> {
        let inputs = &mut self
            .builtin_formations
            .get_or_insert_with(Default::default)
            .inputs;
        let full = inputs.len() >= 1_000_000;
        match inputs.entry((owner, clause)) {
            Entry::Vacant(slot) if !full => {
                slot.insert(subject);
            }
            Entry::Vacant(_) => return Err("builtin formation storage budget exceeded"),
            Entry::Occupied(slot) if *slot.get() != subject => {
                return Err("builtin declaration formation changed its completed input");
            }
            Entry::Occupied(_) => {}
        }
        Ok(())
    }

    pub(crate) fn builtin_formation_input(
        &self,
        owner: DefId,
        clause: QuantifiedId,
    ) -> Result<Option<TypeId>, &'static str> {
        self.builtin_formations
            .as_ref()
            .and_then(|inputs| inputs.inputs.get(&(owner, clause)))
            .copied()
            .ok_or("body builtin formation requires completed declaration facts")
    }
}
