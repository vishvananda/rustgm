//! Canonical non-type names carry their actual store's never-reused identity.
use crate::{
    support::names::{NameId, Names as Arena},
    types::Store,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureId {
    arena: u32,
    slot: NameId,
}
impl std::fmt::Debug for FeatureId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("FeatureId")
    }
}
#[derive(Default)]
pub(super) struct Names {
    arena: u32,
    names: Arena,
}
static NEXT_ARENA: AtomicU32 = AtomicU32::new(1);

impl Store {
    /// Canonical names are representation, never enabling or caller evidence.
    /// # Errors
    /// Rejects invalid names, exhausted identities and unpaid cold allocation.
    pub fn intern_feature_charged(
        &mut self,
        spelling: &str,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<FeatureId, &'static str> {
        pay(spelling.len().saturating_add(1), 0)?;
        if spelling.is_empty()
            || !spelling
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || spelling.as_bytes()[0].is_ascii_digit()
        {
            return Err("feature name requires a canonical ASCII identifier");
        }
        if self.existing_arena_identity().is_none() {
            pay(1, crate::types::arena::Identity::allocation_bytes())?;
            self.arena_identity();
        }
        let arguments = self.arguments.as_ref();
        let clauses = arguments.and_then(|storage| storage.clauses.as_ref());
        let needed = usize::from(arguments.is_none())
            .saturating_mul(size_of::<crate::types::arguments::Storage>())
            .saturating_add(
                usize::from(clauses.is_none()).saturating_mul(size_of::<super::clauses::Storage>()),
            );
        pay(1, needed)?;
        if clauses.is_none_or(|storage| storage.features.is_none()) {
            pay(1, size_of::<Names>())?;
            let arena = NEXT_ARENA
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                    next.checked_add(1)
                })
                .map_err(|_| "feature arena identity space exhausted")?;
            let storage = self
                .arguments
                .get_or_insert_with(Default::default)
                .clauses
                .get_or_insert_with(Default::default);
            storage.features = Some(Box::new(Names {
                arena,
                names: Arena::default(),
            }));
        }
        let storage = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.clauses.as_mut())
            .and_then(|storage| storage.features.as_mut())
            .ok_or("feature name arena publication is missing")?;
        let slot = storage
            .names
            .intern_exact_paid(spelling, &mut |growth| pay(growth.work, growth.bytes))?;
        Ok(FeatureId {
            arena: storage.arena,
            slot,
        })
    }

    /// Borrow the exact current-store spelling. Foreign IDs never alias a slot.
    #[must_use]
    pub fn feature_name(&self, id: FeatureId) -> Option<&str> {
        let storage = self
            .arguments
            .as_ref()?
            .clauses
            .as_ref()?
            .features
            .as_ref()?;
        (id.arena == storage.arena && (id.slot.0 as usize) < storage.names.len())
            .then(|| storage.names.get(id.slot))
    }
}
