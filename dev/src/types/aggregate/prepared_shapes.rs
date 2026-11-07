//! Prepared ordinary external structs retain the exact shape destination frontier.
use super::{Definitions, ItemId, Kind, Repr, Variant};
use crate::{
    support::retained,
    types::{ListId, arena::Identity},
};
/// One imported enum shape and its re-derived nullable layout policy.
pub(crate) struct ImportedEnum {
    pub item: ItemId,
    pub repr: Repr,
    pub variants: Vec<Variant>,
    pub nullable: Option<super::nullable::NullableEnum>,
}
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
    /// Imported enums keep their validated variant order and completed
    /// discriminants; no source discriminant query is pending. A re-derived
    /// nullable policy is staged with its shape and installed with it.
    pub(crate) fn prepare_imported_nominals(
        &mut self,
        types: &crate::types::Store,
        structs: &[(ItemId, Repr, ListId)],
        rows: &[ImportedEnum],
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        let variants = rows
            .iter()
            .map(|row| row.variants.len())
            .sum::<usize>()
            .saturating_add(structs.len());
        let nullable = rows.iter().filter(|row| row.nullable.is_some()).count();
        if nullable != 0 {
            let mut growth = self.nullable_publication_growth();
            growth.work = growth.work.saturating_mul(nullable);
            growth.bytes = growth.bytes.saturating_mul(nullable);
            pay(growth.work, growth.bytes)?;
            self.nullable_enums
                .get_or_insert_with(Default::default)
                .try_reserve(nullable)
                .map_err(|_| "external enum nullable policy reservation failed")?;
        }
        let count = rows.len().saturating_add(structs.len());
        self.prepare_staged(count, variants, pay, |staged, destination| {
            stage_structs(staged, destination, structs)?;
            for row in rows {
                if destination.get(row.item).is_some()
                    || destination.nullable_enum(row.item).is_some()
                    || row.variants.is_empty()
                {
                    return Err("external enum batch changed its actual destination owner");
                }
                staged.insert(row.item, Kind::Enum, row.repr, &row.variants)?;
                if let Some(policy) = row.nullable {
                    staged.register_nullable_enum(types, row.item, policy, None)?;
                }
            }
            Ok(())
        })
    }
    fn prepare_staged(
        &mut self,
        count: usize,
        variants: usize,
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
        stage: impl FnOnce(&mut Self, &Self) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        pay(
            count
                .saturating_mul(4)
                .saturating_add(variants)
                .saturating_add(1),
            retained::new_map::<ItemId, u32>(count)
                .saturating_add(count.saturating_mul(8 * size_of::<super::Definition>()))
                .saturating_add(variants.saturating_mul(8 * size_of::<Variant>()))
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
            variants,
        ));
        growth.include(retained::Growth::map::<ItemId, u32>(
            self.index.len(),
            self.index.capacity(),
            count,
        ));
        pay(growth.work, growth.bytes)?;
        let mut staged = Self::default();
        stage(&mut staged, self)?;
        let entry_base =
            u32::try_from(self.entries.len()).map_err(|_| "external struct entry base overflow")?;
        let variant_base = u32::try_from(self.variants.len())
            .map_err(|_| "external struct variant base overflow")?;
        let identity = self.arena_identity();
        self.entries
            .try_reserve(count)
            .map_err(|_| "external struct entry reservation failed")?;
        self.variants
            .try_reserve(variants)
            .map_err(|_| "external struct variant reservation failed")?;
        self.index
            .try_reserve(count)
            .map_err(|_| "external struct index reservation failed")?;
        u32::try_from(self.entries.len().saturating_add(count))
            .map_err(|_| "external struct entry range overflow")?;
        u32::try_from(self.variants.len().saturating_add(variants))
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
fn stage_structs(
    staged: &mut Definitions,
    destination: &Definitions,
    rows: &[(ItemId, Repr, ListId)],
) -> Result<(), &'static str> {
    for &(item, repr, fields) in rows {
        if destination.get(item).is_some() {
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
    Ok(())
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
        if let Some(policies) = self.rows.nullable_enums {
            // Preparation reserved one destination slot per staged policy.
            target
                .nullable_enums
                .get_or_insert_with(Default::default)
                .extend(*policies);
        }
        for (item, index) in self.rows.index {
            target.index.insert(item, index + entry_base);
        }
        target.packing_checked = false;
    }
}
