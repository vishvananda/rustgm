//! Prepared ordinary external structs retain the exact shape destination frontier.
use super::{Definitions, ItemId, Kind, Repr, Variant};
use crate::{
    support::retained,
    types::{ListId, arena::Identity},
};
pub(crate) struct Prepared {
    identity: Identity,
    entries: usize,
    variants: usize,
    rows: Definitions,
    entry_base: u32,
    variant_base: u32,
}
pub(crate) struct Ready<'a> {
    destination: &'a mut Definitions,
    rows: Definitions,
    entry_base: u32,
    variant_base: u32,
}
impl Definitions {
    pub(crate) fn prepare_imported_structs(
        &mut self,
        rows: &[(ItemId, Repr, ListId)],
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        let count = rows.len();
        pay(
            count.saturating_mul(4).saturating_add(1),
            retained::new_map::<ItemId, u32>(count)
                .saturating_add(
                    count.saturating_mul(
                        8 * (size_of::<super::Definition>() + size_of::<Variant>()),
                    ),
                )
                .saturating_add(Identity::allocation_bytes()),
        )?;
        let mut growth = retained::Growth::vector::<super::Definition>(
            self.entries.len(),
            self.entries.capacity(),
            count,
        );
        growth.include(retained::Growth::vector::<Variant>(
            self.variants.len(),
            self.variants.capacity(),
            count,
        ));
        growth.include(retained::Growth::map::<ItemId, u32>(
            self.index.len(),
            self.index.capacity(),
            count,
        ));
        pay(growth.work, growth.bytes)?;
        let mut staged = Self::default();
        for &(item, repr, fields) in rows {
            if self.get(item).is_some() {
                return Err("external struct batch changed its actual destination owner");
            }
            staged.insert(
                item,
                Kind::Struct,
                repr,
                &[Variant {
                    fields,
                    discriminant: 0,
                }],
            )?;
        }
        let entry_base =
            u32::try_from(self.entries.len()).map_err(|_| "external struct entry base overflow")?;
        let variant_base = u32::try_from(self.variants.len())
            .map_err(|_| "external struct variant base overflow")?;
        let identity = self.arena_identity();
        self.entries
            .try_reserve(count)
            .map_err(|_| "external struct entry reservation failed")?;
        self.variants
            .try_reserve(count)
            .map_err(|_| "external struct variant reservation failed")?;
        self.index
            .try_reserve(count)
            .map_err(|_| "external struct index reservation failed")?;
        u32::try_from(self.entries.len().saturating_add(count))
            .map_err(|_| "external struct entry range overflow")?;
        u32::try_from(self.variants.len().saturating_add(count))
            .map_err(|_| "external struct variant range overflow")?;
        Ok(Prepared {
            identity,
            entries: self.entries.len(),
            variants: self.variants.len(),
            rows: staged,
            entry_base,
            variant_base,
        })
    }
}
impl Prepared {
    pub(crate) fn ready(self, destination: &mut Definitions) -> Result<Ready<'_>, &'static str> {
        if !destination.has_identity(&self.identity)
            || destination.entries.len() != self.entries
            || destination.variants.len() != self.variants
            || self
                .rows
                .entries
                .iter()
                .any(|row| destination.get(row.item).is_some())
        {
            return Err("external struct batch has a foreign or stale actual destination");
        }
        Ok(Ready {
            destination,
            rows: self.rows,
            entry_base: self.entry_base,
            variant_base: self.variant_base,
        })
    }
}
impl Ready<'_> {
    pub(crate) fn install(self) {
        let target = self.destination;
        let entry_base = self.entry_base;
        let variant_base = self.variant_base;
        for mut row in self.rows.entries {
            row.start += variant_base;
            row.end += variant_base;
            target.entries.push(row);
        }
        target.variants.extend(self.rows.variants);
        for (item, index) in self.rows.index {
            target.index.insert(item, index + entry_base);
        }
        target.packing_checked = false;
    }
}
