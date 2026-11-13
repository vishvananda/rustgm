//! Canonical nullable-enum policy precedes all concrete layout queries.
use super::{Definitions, ItemId, Kind, Representation, Store};
use crate::support::hash::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NullableEnum {
    pub empty: u32,
    pub payload: u32,
    pub field: u32,
    pub fallback_tag: Option<crate::types::Primitive>,
    pub companion: Companion,
    /// The source profile admits bool's unused byte value; course layouts do not.
    pub bool_niche: bool,
}
/// What the variant beside the niche carrier may hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Companion {
    /// `empty` has no fields, as `None` has none.
    Fieldless,
    /// Both variants have one field. At each instantiation the variant whose
    /// field is zero-sized is empty and the other carries the niche, as rustc
    /// niche-fills `Result<(), NonNull<T>>` and `Result<NonNull<T>, ()>`. With
    /// neither, the enum keeps its tagged layout.
    ZeroSized,
}
impl NullableEnum {
    /// Whether `empty` and `payload` are a layout orientation this policy admits.
    pub(crate) fn admits(self, empty: u32, payload: u32) -> bool {
        (empty, payload) == (self.empty, self.payload)
            || (self.companion == Companion::ZeroSized
                && (payload, empty) == (self.empty, self.payload))
    }
}
pub(super) type Policies = HashMap<ItemId, NullableEnum>;
impl Definitions {
    pub(crate) fn nullable_publication_growth(&self) -> crate::support::retained::Growth {
        let mut cost = self.nullable_enums.as_deref().map_or_else(
            || crate::support::retained::Growth {
                work: 0,
                bytes: size_of::<Policies>(),
            },
            |_| crate::support::retained::Growth::default(),
        );
        let (length, capacity) = self
            .nullable_enums
            .as_deref()
            .map_or((0, 0), |p| (p.len(), p.capacity()));
        cost.include(
            crate::support::retained::Growth::map::<ItemId, NullableEnum>(length, capacity, 1),
        );
        cost
    }
    pub(crate) fn nullable_storage_bytes(&self) -> usize {
        self.nullable_enums.as_deref().map_or(0, |p| {
            size_of::<Policies>() + crate::support::retained::map(p)
        })
    }
    pub(crate) fn nullable_enum(&self, item: ItemId) -> Option<NullableEnum> {
        self.nullable_enums.as_deref()?.get(&item).copied()
    }
    pub(crate) fn register_nullable_enum(
        &mut self,
        types: &Store,
        item: ItemId,
        policy: NullableEnum,
        storage_limit: Option<usize>,
    ) -> Result<(), &'static str> {
        let definition = self.get(item).ok_or("missing nullable enum declaration")?;
        let variants = self.variants(definition);
        if definition.kind != Kind::Enum
            || definition.repr.representation != Representation::Rust
            || definition.repr.tag.is_some()
            || definition.repr.packed.is_some()
            || definition.repr.align.is_some()
            || variants.len() != 2
            || policy
                .fallback_tag
                .is_some_and(|tag| tag.integer().is_none())
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
        let empty_fields = match policy.companion {
            Companion::Fieldless => 0,
            Companion::ZeroSized => 1,
        };
        if types
            .list(empty.fields)
            .is_none_or(|fields| fields.len() != empty_fields)
            || types
                .list(payload.fields)
                .is_none_or(|fields| fields.len() != 1)
        {
            return Err("nullable enum fields differ from their canonical policy");
        }
        let policies = self.nullable_enums.get_or_insert_with(Default::default);
        if policies.contains_key(&item) {
            return Err("nullable enum policy is already registered");
        }
        policies
            .try_reserve(1)
            .map_err(|_| "nullable policy allocation failed")?;
        if storage_limit.is_some_and(|limit| {
            size_of::<Policies>() + crate::support::retained::map(policies) > limit
        }) {
            return Err("nullable policy allocation exceeds prepaid storage");
        }
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
