//! A source-free member batch owns validated rows until its destination is borrowed.
use super::{Field, FieldId, Members, VariantId};
use crate::{support::retained, types::ItemId};
use std::sync::Arc;

pub(crate) struct Prepared {
    identity: Arc<()>,
    variants: usize,
    fields: usize,
    rows: Members,
    variant_base: u32,
    field_base: u32,
}
pub(crate) struct Ready<'a> {
    destination: &'a mut Members,
    rows: Members,
    variant_base: u32,
    field_base: u32,
}
impl Members {
    pub(crate) fn prepare_external_batch(
        &mut self,
        rows: Vec<(ItemId, Vec<Field>)>,
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        let fields = rows.iter().map(|(_, fields)| fields.len()).sum::<usize>();
        let count = rows.len();
        pay(
            fields.saturating_add(count).saturating_add(1),
            retained::new_map::<ItemId, Vec<VariantId>>(count)
                .saturating_add(retained::new_map::<(VariantId, super::Name), FieldId>(
                    fields,
                ))
                .saturating_add(
                    fields.saturating_mul(8 * (size_of::<Field>() + size_of::<FieldId>())),
                )
                .saturating_add(count.saturating_mul(8 * size_of::<super::Variant>()))
                .saturating_add(4 * size_of::<usize>()),
        )?;
        let mut growth = retained::Growth::vector::<super::Variant>(
            self.variants.len(),
            self.variants.capacity(),
            count,
        );
        growth.include(retained::Growth::vector::<Field>(
            self.fields.len(),
            self.fields.capacity(),
            fields,
        ));
        growth.include(retained::Growth::vector::<u32>(
            self.field_ordinals.len(),
            self.field_ordinals.capacity(),
            fields,
        ));
        growth.include(retained::Growth::vector::<VariantId>(
            self.field_variants.len(),
            self.field_variants.capacity(),
            fields,
        ));
        growth.include(retained::Growth::map::<ItemId, Vec<VariantId>>(
            self.by_owner.len(),
            self.by_owner.capacity(),
            count,
        ));
        growth.include(retained::Growth::map::<(VariantId, super::Name), FieldId>(
            self.by_name.len(),
            self.by_name.capacity(),
            fields,
        ));
        pay(growth.work, growth.bytes)?;
        let mut staged = Self::default();
        for (owner, fields) in rows {
            if !self.variants(owner).is_empty() {
                return Err("external member batch changed its actual destination owner");
            }
            staged.insert_external(owner, &fields)?;
        }
        let variant_base = u32::try_from(self.variants.len())
            .map_err(|_| "external member variant base overflow")?;
        let field_base =
            u32::try_from(self.fields.len()).map_err(|_| "external member field base overflow")?;
        let identity = Arc::clone(self.import_identity.get_or_insert_with(|| Arc::new(())));
        self.variants
            .try_reserve(count)
            .map_err(|_| "external member variant reservation failed")?;
        self.fields
            .try_reserve(fields)
            .map_err(|_| "external member field reservation failed")?;
        self.field_ordinals
            .try_reserve(fields)
            .map_err(|_| "external member ordinal reservation failed")?;
        self.field_variants
            .try_reserve(fields)
            .map_err(|_| "external member owner reservation failed")?;
        self.by_owner
            .try_reserve(count)
            .map_err(|_| "external member index reservation failed")?;
        self.by_name
            .try_reserve(fields)
            .map_err(|_| "external member name reservation failed")?;
        u32::try_from(self.variants.len().saturating_add(count))
            .map_err(|_| "external member variant range overflow")?;
        u32::try_from(self.fields.len().saturating_add(fields))
            .map_err(|_| "external member field range overflow")?;
        Ok(Prepared {
            identity,
            variants: self.variants.len(),
            fields: self.fields.len(),
            rows: staged,
            variant_base,
            field_base,
        })
    }
}
impl Prepared {
    pub(crate) fn ready(self, destination: &mut Members) -> Result<Ready<'_>, &'static str> {
        if destination
            .import_identity
            .as_ref()
            .is_none_or(|identity| !Arc::ptr_eq(identity, &self.identity))
            || destination.variants.len() != self.variants
            || destination.fields.len() != self.fields
            || self
                .rows
                .by_owner
                .keys()
                .any(|owner| !destination.variants(*owner).is_empty())
        {
            return Err("external member batch has a foreign or stale actual destination");
        }
        Ok(Ready {
            destination,
            rows: self.rows,
            variant_base: self.variant_base,
            field_base: self.field_base,
        })
    }
}
impl Ready<'_> {
    pub(crate) fn install(self) {
        let target = self.destination;
        // Preparation established the u32 endpoints; exclusive ownership keeps them fixed.
        let variant_base = self.variant_base;
        let field_base = self.field_base;
        for mut variant in self.rows.variants {
            variant.start += field_base;
            variant.end += field_base;
            target.variants.push(variant);
        }
        target.fields.extend(self.rows.fields);
        target.field_ordinals.extend(self.rows.field_ordinals);
        target.field_variants.extend(
            self.rows
                .field_variants
                .into_iter()
                .map(|id| VariantId(id.0 + variant_base)),
        );
        for (owner, mut variants) in self.rows.by_owner {
            for variant in &mut variants {
                variant.0 += variant_base;
            }
            target.by_owner.insert(owner, variants);
        }
        for ((variant, name), field) in self.rows.by_name {
            target.by_name.insert(
                (VariantId(variant.0 + variant_base), name),
                FieldId(field.0 + field_base),
            );
        }
    }
}
