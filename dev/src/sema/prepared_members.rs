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
/// Batch dimensions: owners, field groups, fields and constructor names.
#[derive(Clone, Copy)]
struct Counts {
    owners: usize,
    variants: usize,
    fields: usize,
    constructors: usize,
}
impl Members {
    /// Imported records and enums share one prepared batch; every enum owner
    /// publishes its complete ordered variants and constructor names.
    pub(crate) fn prepare_external_nominals(
        &mut self,
        records: Vec<(ItemId, Vec<Field>)>,
        enums: Vec<(ItemId, Vec<super::ExternalVariant>)>,
        pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        let variants = enums
            .iter()
            .map(|(_, variants)| variants.len())
            .sum::<usize>();
        let counts = Counts {
            owners: records.len().saturating_add(enums.len()),
            variants: variants.saturating_add(records.len()),
            fields: records
                .iter()
                .map(|(_, fields)| fields.len())
                .chain(
                    enums
                        .iter()
                        .flat_map(|(_, variants)| variants.iter().map(|v| v.fields.len())),
                )
                .sum::<usize>(),
            constructors: variants,
        };
        self.prepare_staged(counts, pay, |staged, destination| {
            for (owner, fields) in records {
                if !destination.variants(owner).is_empty() {
                    return Err("external member batch changed its actual destination owner");
                }
                staged.insert_external(owner, &fields)?;
            }
            for (owner, variants) in enums {
                if !destination.variants(owner).is_empty()
                    || variants
                        .iter()
                        .any(|variant| destination.constructors.contains_key(&variant.constructor))
                {
                    return Err("external enum batch changed its actual destination owner");
                }
                staged.insert_external_variants(owner, &variants)?;
            }
            Ok(())
        })
    }
    /// The retained growth of every destination table one batch extends.
    fn staging_growth(&self, counts: &Counts) -> retained::Growth {
        let &Counts {
            owners,
            variants,
            fields,
            constructors,
        } = counts;
        let mut growth = retained::Growth::vector::<super::Variant>(
            self.variants.len(),
            self.variants.capacity(),
            variants,
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
            owners,
        ));
        growth.include(retained::Growth::map::<(VariantId, super::Name), FieldId>(
            self.by_name.len(),
            self.by_name.capacity(),
            fields,
        ));
        growth.include(retained::Growth::map::<
            crate::resolve::model::DefId,
            super::Constructor,
        >(
            self.constructors.len(),
            self.constructors.capacity(),
            constructors,
        ));
        growth
    }
    fn prepare_staged(
        &mut self,
        counts: Counts,
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
        stage: impl FnOnce(&mut Self, &Self) -> Result<(), &'static str>,
    ) -> Result<Prepared, &'static str> {
        let Counts {
            owners,
            variants,
            fields,
            constructors,
        } = counts;
        pay(
            fields
                .saturating_add(variants)
                .saturating_add(constructors)
                .saturating_add(1),
            retained::new_map::<ItemId, Vec<VariantId>>(owners)
                .saturating_add(retained::new_map::<(VariantId, super::Name), FieldId>(
                    fields,
                ))
                .saturating_add(retained::new_map::<
                    crate::resolve::model::DefId,
                    super::Constructor,
                >(constructors))
                .saturating_add(
                    fields.saturating_mul(8 * (size_of::<Field>() + size_of::<FieldId>())),
                )
                .saturating_add(variants.saturating_mul(8 * size_of::<super::Variant>()))
                .saturating_add(4 * size_of::<usize>()),
        )?;
        let growth = self.staging_growth(&counts);
        pay(growth.work, growth.bytes)?;
        let mut staged = Self::default();
        stage(&mut staged, self)?;
        let variant_base = u32::try_from(self.variants.len())
            .map_err(|_| "external member variant base overflow")?;
        let field_base =
            u32::try_from(self.fields.len()).map_err(|_| "external member field base overflow")?;
        let identity = Arc::clone(self.import_identity.get_or_insert_with(|| Arc::new(())));
        self.variants
            .try_reserve(variants)
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
            .try_reserve(owners)
            .map_err(|_| "external member index reservation failed")?;
        self.by_name
            .try_reserve(fields)
            .map_err(|_| "external member name reservation failed")?;
        self.constructors
            .try_reserve(constructors)
            .map_err(|_| "external member constructor reservation failed")?;
        u32::try_from(self.variants.len().saturating_add(variants))
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
            || self
                .rows
                .constructors
                .keys()
                .any(|name| destination.constructors.contains_key(name))
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
        for (name, mut constructor) in self.rows.constructors {
            constructor.variant.0 += variant_base;
            target.constructors.insert(name, constructor);
        }
        for ((variant, name), field) in self.rows.by_name {
            target.by_name.insert(
                (VariantId(variant.0 + variant_base), name),
                FieldId(field.0 + field_base),
            );
        }
    }
}
