//! Canonical nullable-enum policy precedes all concrete layout queries.
use super::{Definitions, ItemId, Kind, Representation, Store};
use crate::support::hash::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NullableEnum {
    pub empty: u32,
    pub payload: u32,
    pub field: u32,
}
pub(super) type Policies = HashMap<ItemId, NullableEnum>;
impl Definitions {
    pub(crate) fn nullable_enum(&self, item: ItemId) -> Option<NullableEnum> {
        self.nullable_enums.as_deref()?.get(&item).copied()
    }
    pub(crate) fn register_nullable_enum(
        &mut self,
        types: &Store,
        item: ItemId,
        policy: NullableEnum,
    ) -> Result<(), &'static str> {
        let definition = self.get(item).ok_or("missing nullable enum declaration")?;
        let variants = self.variants(definition);
        if definition.kind != Kind::Enum
            || definition.repr.representation != Representation::Rust
            || definition.repr.tag.is_some()
            || definition.repr.packed.is_some()
            || definition.repr.align.is_some()
            || variants.len() != 2
            || policy.empty == policy.payload
            || policy.field != 0
        {
            return Err("invalid nullable enum declaration policy");
        }
        let empty = variants
            .get(policy.empty as usize)
            .ok_or("missing nullable empty variant")?;
        let payload = variants
            .get(policy.payload as usize)
            .ok_or("missing nullable payload variant")?;
        if types
            .list(empty.fields)
            .is_none_or(|fields| !fields.is_empty())
            || types
                .list(payload.fields)
                .is_none_or(|fields| fields.len() != 1)
        {
            return Err("nullable enum fields differ from their canonical policy");
        }
        let policies = self.nullable_enums.get_or_insert_with(Default::default);
        match policies.entry(item) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(policy);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err("nullable enum policy is already registered");
            }
        }
        Ok(())
    }
}
