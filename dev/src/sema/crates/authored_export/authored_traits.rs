//! Complete validated source headers retain exact Self syntax and attribute claims.
mod supertrait_sources;
use super::{
    DeclarationRef, Declarations, DefKind, OwnerKind, Report, Stage, canonical_nodes::Context, g, s,
};
use crate::{
    resolve::bindings::{HeaderAttribute, Reservations},
    sema::compiler_items::headers::Trait,
    support::bindings::LangItem,
    types::{ItemId, Kind},
};

/// Admission precedes any declaration/constant row serialization.
pub(super) fn require_range_transport(facts: &Declarations) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source header graph has no role catalog")?;
    if [LangItem::RangeMin, LangItem::RangeMax, LangItem::RangeSub]
        .into_iter()
        .any(|role| catalog.language(role).is_some())
    {
        return Err("source range helper contract transport is not implemented");
    }
    Ok(())
}

impl Stage {
    pub(super) fn traits(
        &mut self,
        report: &Report,
        facts: &Declarations,
        ordinary_traits: bool,
    ) -> Result<(), &'static str> {
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source header graph has no role catalog")?;
        self.charge(1)?;
        if catalog.language(LangItem::UnsafeUnpin).is_some() {
            return Err("source UnsafeUnpin contract transport is not implemented");
        }
        if catalog.language(LangItem::ConstParamTy).is_some() {
            return Err("source ConstParamTy contract transport is not implemented");
        }
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("source role reservations are absent")?;
        let start =
            u32::try_from(self.rows.roles.len()).map_err(|_| "source role table overflow")?;
        let mut emitted = if ordinary_traits {
            self.reserve(report.resolution.definitions.len())?;
            vec![false; report.resolution.definitions.len()]
        } else {
            Vec::new()
        };
        for role in LangItem::ALL {
            self.charge(1)?;
            let Some(declaration) = catalog.header_role(role)? else {
                continue;
            };
            if matches!(
                role,
                LangItem::PointeeTrait
                    | LangItem::MetadataType
                    | LangItem::DynMetadata
                    | LangItem::PhantomData
            ) {
                return Err("source Pointee contract transport is not implemented");
            }
            if role == LangItem::Send {
                return Err("source Send diagnostic contract transport is not implemented");
            }
            let address = self.address(declaration.definition)?;
            self.reserve(4 * size_of::<s::RoleBinding>())?;
            self.rows.roles.push(s::RoleBinding {
                role,
                declaration: address,
            });
            match report.resolution.definitions[declaration.definition.0 as usize].kind {
                DefKind::Trait => {
                    let (header, literal) = Trait::source_with_bounds(report, facts, declaration)?;
                    self.trait_header(facts, &header, reserved, literal)?;
                    if let Some(slot) = emitted.get_mut(declaration.definition.0 as usize) {
                        *slot = true;
                    }
                }
                DefKind::Function if role == LangItem::CloneFn => {}
                _ => return Err("source role form is outside the complete first transport family"),
            }
        }
        self.rows.policy.roles = g::Range {
            start,
            end: u32::try_from(self.rows.roles.len()).map_err(|_| "source role table overflow")?,
        };
        if ordinary_traits {
            for owner in facts.associated.owners() {
                self.charge(1)?;
                let OwnerKind::Trait { definition, .. } = owner.kind else {
                    continue;
                };
                let slot = emitted
                    .get_mut(definition.0 as usize)
                    .ok_or("ordinary trait owner is outside its actual declaration arena")?;
                if !*slot {
                    let (header, literal) = Trait::source_owner_with_bounds(report, facts, owner)?;
                    self.trait_header(facts, &header, reserved, literal)?;
                    *slot = true;
                }
            }
        }
        let expected = facts
            .associated
            .owners()
            .iter()
            .filter(|owner| matches!(owner.kind, OwnerKind::Trait { .. }))
            .count();
        self.charge(facts.associated.owners().len())?;
        if self.rows.traits.len() != expected {
            return Err("source trait graph contains an unsupported unvalidated declaration");
        }
        Ok(())
    }

    fn trait_header(
        &mut self,
        facts: &Declarations,
        header: &Trait<'_>,
        reserved: &Reservations,
        literal: &[crate::syntax::ast::NodeId],
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let canonical = facts
            .generics
            .traits
            .as_deref()
            .and_then(|traits| traits.header(ItemId(header.definition.0)))
            .ok_or("source trait has no complete canonical header")?;
        if !canonical.results.is_empty() {
            return Err("source associated result transport is not implemented");
        }
        let inventory =
            supertrait_sources::Inventory::new(self, facts, header, canonical, literal)?;
        let prefix = inventory.prefix;
        let declaration = self.declaration(header.definition)?;
        let self_definition = self.declaration(header.self_definition)?;
        let context = self.frame(
            declaration,
            None,
            s::BinderKind::DeclarationHeader,
            header.captures,
        )?;
        let self_type = self.ty(facts, header.self_type, context)?;
        let Kind::Parameter(parameter) = facts
            .types
            .kind(header.self_type)
            .ok_or("source trait Self type is unknown")?
        else {
            return Err("source trait lost its actual Self parameter");
        };
        let self_slot = self
            .graph
            .parameters
            .get(&parameter)
            .copied()
            .ok_or("source trait Self slot was not reserved")?;
        let (inputs, written_supertraits) =
            self.trait_inputs(facts, header, context, self_slot, inventory)?;
        let region_bounds = self.region_bounds(&canonical.regions, context)?;
        let start =
            u32::try_from(self.rows.members.len()).map_err(|_| "source member table overflow")?;
        self.reserve(
            header
                .members
                .len()
                .saturating_mul(4 * size_of::<DeclarationRef>()),
        )?;
        for &member in header.members {
            let actual = self.declaration(member)?;
            self.rows.members.push(actual);
        }
        let members = g::Range {
            start,
            end: u32::try_from(self.rows.members.len())
                .map_err(|_| "source member table overflow")?,
        };
        let flags = flags(header, reserved)?;
        self.reserve(4 * size_of::<s::TraitHeader>())?;
        self.rows.traits.push(s::TraitHeader {
            declaration,
            scope: declaration,
            self_definition,
            self_type,
            flags,
            region_captures: header.captures,
            region_bounds,
            inputs,
            canonical_inputs: prefix.canonical,
            declared_inputs: prefix.declared,
            written_parameters: u32::try_from(header.written.parameters)
                .map_err(|_| "source written parameter count overflow")?,
            written_supertraits,
            members,
        });
        Ok(())
    }

    fn trait_inputs(
        &mut self,
        facts: &Declarations,
        header: &Trait<'_>,
        context: Context,
        self_slot: s::SlotRef,
        mut inventory: supertrait_sources::Inventory<'_>,
    ) -> Result<(g::Range, g::Range), &'static str> {
        let start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source clause table overflow")?;
        for (ordinal, input) in header.inputs.iter().enumerate() {
            self.charge(1)?;
            let is_self = inventory.is_self(facts, ordinal)?;
            let index = self.clause(facts, input, context, is_self.then_some(self_slot))?;
            inventory.record(self, facts, ordinal, input, index, is_self)?;
        }
        let inputs = g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source clause table overflow")?,
        };
        Ok((inputs, inventory.finish(self)?))
    }

    pub(super) fn region_bounds(
        &mut self,
        bounds: &[(crate::types::RegionId, crate::types::RegionId)],
        context: Context,
    ) -> Result<g::Range, &'static str> {
        self.reserve(bounds.len().saturating_mul(4 * size_of::<s::RegionBound>()))?;
        let start = u32::try_from(self.rows.region_bounds.len())
            .map_err(|_| "source region bound table overflow")?;
        for &(longer, shorter) in bounds {
            self.charge(1)?;
            let longer = self.region(longer, context)?;
            let shorter = self.region(shorter, context)?;
            self.rows
                .region_bounds
                .push(s::RegionBound { longer, shorter });
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.region_bounds.len())
                .map_err(|_| "source region bound table overflow")?,
        })
    }
}

fn flags(header: &Trait<'_>, reserved: &Reservations) -> Result<s::TraitFlags, &'static str> {
    if header.attributes.contains(HeaderAttribute::Marker) {
        return Err("source marker trait contract transport is not implemented");
    }
    let mut bits = (u16::from(header.flags.unsafe_) * s::TraitFlags::UNSAFE)
        | (u16::from(header.flags.constant) * s::TraitFlags::CONST)
        | (u16::from(header.flags.auto_) * s::TraitFlags::AUTO)
        | (u16::from(reserved.denies_implementation(header.definition)) * s::TraitFlags::DENY_IMPL);
    for (attribute, flag) in [
        (HeaderAttribute::Fundamental, s::TraitFlags::FUNDAMENTAL),
        (
            HeaderAttribute::Specialization,
            s::TraitFlags::SPECIALIZATION,
        ),
        (HeaderAttribute::Coinductive, s::TraitFlags::COINDUCTIVE),
        (
            HeaderAttribute::DynIncompatible,
            s::TraitFlags::DYN_INCOMPATIBLE,
        ),
    ] {
        if header.attributes.contains(attribute) {
            bits |= flag;
        }
    }
    s::TraitFlags::from_bits(bits)
}
