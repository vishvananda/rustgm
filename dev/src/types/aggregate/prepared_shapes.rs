//! Prepared ordinary external structs retain the exact shape destination frontier.
use super::{Definitions, ItemId, Kind, NominalPolicy, Repr, SharedMutation, Variant};
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ImportedPolicy {
    #[default]
    Ordinary,
    Phantom,
    Union,
    SharedMutation,
    SuppressDestruction,
    DanglingPointers,
}
#[derive(Clone, Copy)]
pub(crate) struct ImportedStruct {
    pub item: ItemId,
    pub repr: Repr,
    pub fields: ListId,
    pub policy: ImportedPolicy,
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
    /// nullable policy is staged with its shape and installed with it. The one
    /// struct named by the packet's `PhantomData` role, re-proved later, is
    /// staged with its logical argument.
    pub(crate) fn prepare_imported_nominals(
        &mut self,
        types: &crate::types::Store,
        structs: &[ImportedStruct],
        rows: &[ImportedEnum],
        limits: &[(ItemId, u64)],
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
        let limit_bytes = self.reserve_imported_simd_limits(limits.len(), &mut pay)?;
        self.reserve_imported_policies(structs, &mut pay)?;
        self.prepare_staged(count, variants, pay, |staged, destination| {
            stage_structs(staged, destination, types, structs)?;
            for &(item, limit) in limits {
                staged.register_simd_limit(item, limit, limit_bytes)?;
            }
            for row in rows {
                if destination.get(row.item).is_some()
                    || destination.nullable_enum(row.item).is_some()
                {
                    return Err("external enum batch changed its actual destination owner");
                }
                staged.insert(row.item, Kind::Enum, row.repr, &row.variants)?;
                for variant in &row.variants {
                    staged.index_field_projections(types, variant.fields)?;
                }
                if let Some(policy) = row.nullable {
                    staged.register_nullable_enum(types, row.item, policy, None)?;
                }
            }
            Ok(())
        })
    }
    fn reserve_imported_policies(
        &mut self,
        structs: &[ImportedStruct],
        pay: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        pay(structs.len(), 0)?;
        let (interior, nominal) = structs
            .iter()
            .fold((0, 0), |(interior, nominal), row| match row.policy {
                ImportedPolicy::SharedMutation => (interior + 1, nominal),
                ImportedPolicy::Union
                | ImportedPolicy::SuppressDestruction
                | ImportedPolicy::DanglingPointers => (interior, nominal + 1),
                _ => (interior, nominal),
            });
        let mut growth = retained::Growth::map::<ItemId, ()>(
            self.interior_storage.len(),
            self.interior_storage.capacity(),
            interior,
        );
        growth.include(retained::Growth::map::<ItemId, NominalPolicy>(
            self.nominal_policies.len(),
            self.nominal_policies.capacity(),
            nominal,
        ));
        pay(
            growth.work,
            growth
                .bytes
                .saturating_add(retained::new_map::<ItemId, ()>(interior))
                .saturating_add(retained::new_map::<ItemId, NominalPolicy>(nominal)),
        )?;
        self.interior_storage
            .try_reserve(interior)
            .map_err(|_| "external interior-storage policy reservation failed")?;
        self.nominal_policies
            .try_reserve(nominal)
            .map_err(|_| "external wrapper policy reservation failed")
    }
    fn reserve_imported_simd_limits(
        &mut self,
        count: usize,
        pay: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<usize, &'static str> {
        if count == 0 {
            return Ok(0);
        }
        let limit_bytes = std::mem::size_of::<super::lane_limits::Limits>()
            + retained::new_map::<ItemId, u64>(count);
        let (len, capacity) = self
            .simd_limits
            .as_deref()
            .map_or((0, 0), |limits| (limits.len(), limits.capacity()));
        let growth = retained::Growth::map::<ItemId, u64>(len, capacity, count);
        let owner = if self.simd_limits.is_none() {
            std::mem::size_of::<super::lane_limits::Limits>()
        } else {
            0
        };
        pay(
            count.saturating_add(growth.work),
            limit_bytes
                .saturating_add(growth.bytes)
                .saturating_add(owner),
        )?;
        self.simd_limits
            .get_or_insert_default()
            .try_reserve(count)
            .map_err(|_| "external SIMD limit reservation failed")?;
        Ok(limit_bytes)
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
    types: &crate::types::Store,
    rows: &[ImportedStruct],
) -> Result<(), &'static str> {
    for &ImportedStruct {
        item,
        repr,
        fields,
        policy,
    } in rows
    {
        if destination.get(item).is_some() {
            return Err("external struct batch changed its actual destination owner");
        }
        staged.insert_with_mutation(
            item,
            match policy {
                ImportedPolicy::Phantom => Kind::PhantomData,
                ImportedPolicy::Union => Kind::Union,
                _ => Kind::Struct,
            },
            repr,
            &[Variant {
                fields,
                discriminant: 0,
            }],
            if policy == ImportedPolicy::SharedMutation {
                SharedMutation::Interior
            } else {
                SharedMutation::Ordinary
            },
        )?;
        staged.index_field_projections(types, fields)?;
        match policy {
            ImportedPolicy::Union => staged.suppress_field_destruction(item)?,
            ImportedPolicy::SuppressDestruction => staged.suppress_destruction(item)?,
            ImportedPolicy::DanglingPointers => staged.permit_dangling_pointers(item)?,
            _ => {}
        }
        if repr.representation == super::Representation::Simd {
            let shape = staged
                .get(item)
                .ok_or("imported SIMD struct lost its staged definition")?;
            staged.validate_simd_declaration(types, shape)?;
        }
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
        target.unions |= self.rows.has_unions();
        target.projected_templates |= self.rows.projected_templates;
        if self.rows.has_indirect_arguments() {
            target.argument_policy = super::ArgumentPassing::Indirect;
        }
        let entry_base = self.entry_base;
        let variant_base = self.variant_base;
        for mut row in self.rows.entries {
            row.start += variant_base;
            row.end += variant_base;
            target.entries.push(row);
        }
        target.variants.extend(self.rows.variants);
        target.interior_storage.extend(self.rows.interior_storage);
        target.nominal_policies.extend(self.rows.nominal_policies);
        if let Some(policies) = self.rows.nullable_enums {
            // Preparation reserved one destination slot per staged policy.
            target
                .nullable_enums
                .get_or_insert_with(Default::default)
                .extend(*policies);
        }
        if let Some(limits) = self.rows.simd_limits {
            // Preparation prepaid and reserved one destination slot per limit.
            target.simd_limits.get_or_insert_default().extend(*limits);
        }
        for (item, index) in self.rows.index {
            target.index.insert(item, index + entry_base);
        }
        target.packing_checked = false;
    }
}
