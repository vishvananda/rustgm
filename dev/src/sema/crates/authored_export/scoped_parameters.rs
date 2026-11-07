//! Actual generic ancestry stays shared; lifetime slots retain their own indices.
use super::{DeclarationRef, Declarations, DefId, DefKind, Report, Stage, g, s};
use crate::{
    support::bindings::LangItem,
    types::{Kind, arguments::Argument, parameters::ParameterKind},
};

impl Stage {
    pub(super) fn scopes(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        let count = self.definitions.len();
        self.reserve(count.saturating_mul(
            size_of::<bool>()
                + size_of::<Vec<DefId>>()
                + 8 * size_of::<DefId>()
                + size_of::<Option<u32>>(),
        ))?;
        self.nominal_lifetime_bounds(facts)?;
        let mut active = vec![false; count];
        for (index, enabled) in active.iter_mut().enumerate() {
            self.charge(1)?;
            *enabled = self.definitions[index].is_some()
                && matches!(
                    report.resolution.definitions[index].kind,
                    DefKind::SelfType
                        | DefKind::Trait
                        | DefKind::Function
                        | DefKind::Struct
                        | DefKind::Enum
                        | DefKind::TypeAlias
                );
        }
        let mut children = vec![Vec::new(); count];
        let mut roots = Vec::new();
        for (index, &enabled) in active.iter().enumerate() {
            self.charge(1)?;
            if !enabled {
                continue;
            }
            let id = DefId(u32::try_from(index).map_err(|_| "source scope identity overflow")?);
            if let Some(&parent) = facts.generics.parents.get(&id) {
                if active.get(parent.0 as usize) != Some(&true) {
                    return Err("source generic scope parent is not an actual retained owner");
                }
                children[parent.0 as usize].push(id);
            } else {
                roots.push(id);
            }
        }
        let mut totals = vec![None::<u32>; count];
        let mut pending = roots;
        pending.reverse();
        while let Some(owner) = pending.pop() {
            self.charge(1)?;
            if totals[owner.0 as usize].is_some() {
                return Err("source generic scope repeats an owner");
            }
            let parent = facts.generics.parents.get(&owner).copied();
            let captures = parent
                .map(|parent| {
                    totals[parent.0 as usize].ok_or("source generic scope parent is not completed")
                })
                .transpose()?
                .unwrap_or(0);
            let declaration = self.declaration(owner)?;
            let start =
                u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?;
            let kind = report.resolution.definitions[owner.0 as usize].kind;
            // Version thirteen: a nominal's region slots precede its type slots,
            // in the order of its own template arguments.
            let nominal = self.body_nodes() && matches!(kind, DefKind::Struct | DefKind::Enum);
            if nominal {
                self.nominal_region_slots(facts, owner, declaration, start)?;
            }
            self.type_slots(facts, owner, declaration, kind == DefKind::Trait, nominal)?;
            if !nominal {
                self.region_slots(facts, owner, declaration, start)?;
            }
            let end =
                u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?;
            self.reserve(4 * size_of::<s::ParameterScope>())?;
            let parent = parent.map(|parent| self.declaration(parent)).transpose()?;
            self.rows.scopes.push(s::ParameterScope {
                owner: declaration,
                parent,
                declared: g::Range { start, end },
                captures,
            });
            totals[owner.0 as usize] = Some(
                captures
                    .checked_add(end - start)
                    .ok_or("source scope parameter count overflow")?,
            );
            pending.extend(children[owner.0 as usize].iter().rev().copied());
        }
        for (index, enabled) in active.into_iter().enumerate() {
            self.charge(1)?;
            if enabled && totals[index].is_none() {
                return Err("source generic scope graph is cyclic or incomplete");
            }
        }
        Ok(())
    }

    /// Version thirteen transports a nominal's lifetimes but not its declared
    /// lifetime bounds; each consumer would otherwise lose that requirement.
    fn nominal_lifetime_bounds(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        if !self.body_nodes() {
            return Ok(());
        }
        for &(item, _) in &facts.lifetimes.nominal_bounds {
            self.charge(1)?;
            if self
                .definitions
                .get(item.0 as usize)
                .is_some_and(Option::is_some)
            {
                return Err("source nominal lifetime bounds require their wider profile");
            }
        }
        Ok(())
    }

    fn type_slots(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        trait_owner: bool,
        nominal: bool,
    ) -> Result<(), &'static str> {
        // A nominal's lifetimes are its leading template arguments. They own
        // the region slots already recorded, so a type slot's canonical index
        // counts type slots only.
        let mut regions = 0_u32;
        for (ordinal, parameter) in facts
            .generics
            .parameters
            .get(&owner)
            .into_iter()
            .flatten()
            .enumerate()
        {
            self.charge(1)?;
            // A version-thirteen trait default is recorded once its trait header
            // has built the Self type (`trait_defaults`).
            if parameter.default.is_some() && !(trait_owner && self.body_nodes()) {
                return Err("source parameter defaults are outside the initial profile");
            }
            if nominal && matches!(parameter.argument, Argument::Lifetime(_)) {
                if u32::try_from(ordinal).ok() != Some(regions) {
                    return Err("source nominal lifetime follows a type slot");
                }
                regions += 1;
                continue;
            }
            let Argument::Type(ty) = parameter.argument else {
                return Err("source const/lifetime generic argument is not a type slot");
            };
            let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                return Err("source generic slot has no actual parameter identity");
            };
            let actual = facts
                .types
                .parameter(id)
                .ok_or("source parameter is unknown")?;
            let ParameterKind::Type { sized } = actual.kind else {
                return Err("source slot kind is not a type parameter");
            };
            if actual.owner.0 != owner.0 {
                return Err("source parameter belongs to another declaration");
            }
            let sized = if sized {
                let actual = facts
                    .source_bindings
                    .as_deref()
                    .ok_or("source slot has no role catalog")?
                    .header_role(LangItem::Sized)?
                    .ok_or("source Sized requirement has no actual declaration")?;
                Some(s::DeclarationAddress {
                    artifact: s::ArtifactRef(0),
                    declaration: self.declaration(actual.definition)?,
                })
            } else {
                None
            };
            let definition = Some(self.declaration(parameter.binding)?);
            let slot = s::SlotRef(
                u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?,
            );
            self.reserve(crate::support::retained::new_map::<
                crate::types::parameters::ParameterId,
                s::SlotRef,
            >(1))?;
            if self.graph.parameters.insert(id, slot).is_some() {
                return Err("source parameter identity has duplicate slot rows");
            }
            self.reserve(4 * size_of::<s::ParameterSlot>())?;
            self.rows.slots.push(s::ParameterSlot {
                owner: declaration,
                ordinal: u32::try_from(ordinal).map_err(|_| "source parameter ordinal overflow")?,
                index: actual
                    .index
                    .checked_sub(regions)
                    .ok_or("source nominal type slot precedes its lifetimes")?,
                definition,
                kind: s::ParameterKind::Type { sized },
                default: None,
            });
        }
        Ok(())
    }

    /// Records each declared default of a trait's own type slots once the trait
    /// header has its Self type. Only that Self is admitted; wider defaults need
    /// a wider consumer.
    pub(super) fn trait_defaults(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        self_type: crate::types::TypeId,
        self_ref: g::TypeRef,
    ) -> Result<(), &'static str> {
        for parameter in facts.generics.parameters.get(&owner).into_iter().flatten() {
            self.charge(1)?;
            let Some(default) = parameter.default else {
                continue;
            };
            if default != Argument::Type(self_type) {
                return Err(
                    "source parameter default other than its trait's Self requires a wider consumer",
                );
            }
            let Argument::Type(ty) = parameter.argument else {
                return Err("source defaulted generic argument is not a type slot");
            };
            let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                return Err("source defaulted slot has no actual parameter identity");
            };
            let slot = self
                .graph
                .parameters
                .get(&id)
                .copied()
                .ok_or("source defaulted parameter has no reserved slot")?;
            let row = self
                .rows
                .slots
                .get_mut(slot.0 as usize)
                .ok_or("source defaulted slot is outside its table")?;
            if row.default.replace(self_ref).is_some() {
                return Err("source parameter default was recorded twice");
            }
        }
        Ok(())
    }

    /// Version thirteen: each declared lifetime of a struct or enum is one
    /// region slot whose index is its position in the nominal's template
    /// arguments, `RegionId::bound(0, index)`. The slot carries no variance or
    /// outlives claim; every consumer re-derives both from the fields.
    fn nominal_region_slots(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        start: u32,
    ) -> Result<(), &'static str> {
        let declared = facts
            .lifetimes
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        let arguments = facts.generics.parameters.get(&owner);
        for (index, &definition) in (0_u32..).zip(declared) {
            self.charge(1)?;
            let region = crate::types::RegionId::bound(
                0,
                u16::try_from(index).map_err(|_| "source nominal lifetime slot overflow")?,
            )?;
            if arguments.is_some_and(|arguments| {
                arguments
                    .get(index as usize)
                    .map(|p| (p.binding, p.argument))
                    != Some((definition, Argument::Lifetime(region)))
            }) {
                return Err("source nominal lifetime changed its template argument");
            }
            let ordinal = u32::try_from(self.rows.slots.len())
                .map_err(|_| "source region slot overflow")?
                .checked_sub(start)
                .ok_or("source region slot changed owner range")?;
            let slot = s::SlotRef(
                u32::try_from(self.rows.slots.len()).map_err(|_| "source region slot overflow")?,
            );
            self.reserve(
                4 * size_of::<s::ParameterSlot>()
                    + crate::support::retained::new_map::<(DeclarationRef, u32), s::SlotRef>(1),
            )?;
            if self
                .inputs
                .regions
                .insert((declaration, index), slot)
                .is_some()
            {
                return Err("source lifetime has duplicate actual slot identities");
            }
            let definition = Some(self.declaration(definition)?);
            self.rows.slots.push(s::ParameterSlot {
                owner: declaration,
                ordinal,
                index,
                definition,
                kind: s::ParameterKind::Region,
                default: None,
            });
        }
        Ok(())
    }

    fn region_slots(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        start: u32,
    ) -> Result<(), &'static str> {
        let captures = facts
            .associated
            .schema(owner)
            .map_or(0, |schema| schema.captures);
        let (first, total) = facts
            .lifetimes
            .functions
            .get(&owner)
            .map_or((0, captures), |scheme| (scheme.captures, scheme.total));
        let named = facts.lifetimes.parameters.get(&owner);
        self.reserve((total as usize).saturating_mul(size_of::<Option<DefId>>()))?;
        let mut bindings = vec![None; total as usize];
        for &definition in named.into_iter().flatten() {
            self.charge(1)?;
            let index = facts
                .lifetimes
                .named
                .get(&definition)
                .copied()
                .and_then(crate::types::RegionId::bound_parts)
                .filter(|&(depth, _)| depth == 0)
                .map(|(_, slot)| usize::from(slot))
                .ok_or("source lifetime binding has no actual original slot")?;
            let binding = bindings
                .get_mut(index)
                .ok_or("source lifetime binding exceeds its header")?;
            if binding.replace(definition).is_some() {
                return Err("source lifetime slot has duplicate declarations");
            }
        }
        for index in first..total {
            self.charge(1)?;
            let definition = bindings[index as usize]
                .map(|definition| self.declaration(definition))
                .transpose()?;
            let ordinal = u32::try_from(self.rows.slots.len())
                .map_err(|_| "source region slot overflow")?
                .checked_sub(start)
                .ok_or("source region slot changed owner range")?;
            self.reserve(4 * size_of::<s::ParameterSlot>())?;
            let slot = s::SlotRef(
                u32::try_from(self.rows.slots.len()).map_err(|_| "source region slot overflow")?,
            );
            self.reserve(crate::support::retained::new_map::<
                (DeclarationRef, u32),
                s::SlotRef,
            >(1))?;
            if self
                .inputs
                .regions
                .insert((declaration, index), slot)
                .is_some()
            {
                return Err("source lifetime has duplicate actual slot identities");
            }
            self.rows.slots.push(s::ParameterSlot {
                owner: declaration,
                ordinal,
                index,
                definition,
                kind: s::ParameterKind::Region,
                default: None,
            });
        }
        Ok(())
    }
}
