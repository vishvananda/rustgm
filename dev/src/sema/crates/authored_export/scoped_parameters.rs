//! Actual generic ancestry stays shared; lifetime slots retain their own indices.
use super::{DeclarationRef, Declarations, DefId, DefKind, Report, Stage, g, s};
use crate::{
    support::{bindings::LangItem, retained},
    types::{
        Kind,
        arguments::{Argument, ConstKind},
        parameters::ParameterKind,
    },
};
use std::num::NonZeroU32;

impl Stage {
    pub(super) fn scopes(
        &mut self,
        report: &Report,
        facts: &Declarations,
        admitted: &[(DefId, s::DefinitionKind)],
    ) -> Result<(), &'static str> {
        self.nominal_lifetime_bounds(facts)?;
        let index_bytes = self
            .definitions
            .len()
            .saturating_mul(size_of::<Option<NonZeroU32>>());
        let owner_bytes = admitted.len().saturating_mul(size_of::<DeclarationRef>());
        self.reserve(index_bytes.saturating_add(owner_bytes))?;
        let mut indices = vec![None; self.definitions.len()];
        let mut owners = Vec::with_capacity(admitted.len());
        let mut slots = ScopeSlots::default();
        // Admission assigns each declaration its position in this inventory.
        for (declaration, &(owner, kind)) in admitted.iter().enumerate() {
            self.charge(1)?;
            if self.has_scope(report, facts, owner, kind) {
                let declaration = DeclarationRef(
                    u32::try_from(declaration)
                        .map_err(|_| "source scope owner exceeds its declaration range")?,
                );
                let position = u32::try_from(owners.len().saturating_add(1))
                    .ok()
                    .and_then(NonZeroU32::new)
                    .ok_or("source scope position exceeds its compact range")?;
                if indices[owner.0 as usize].replace(position).is_some() {
                    return Err("source generic scope repeats an owner");
                }
                owners.push(declaration);
                slots.include(facts, owner);
            }
        }
        let owner_count = owners.len();
        let forest_bytes = owner_count.saturating_mul(
            2 * size_of::<Option<NonZeroU32>>()
                + size_of::<(NonZeroU32, u32, Option<NonZeroU32>)>(),
        );
        self.reserve(owner_count.saturating_mul(size_of::<s::ParameterScope>()))?;
        self.reserve(forest_bytes)?;
        self.rows.scopes.reserve_exact(owner_count);
        let mut first_child = vec![None; owner_count];
        let mut next_sibling = vec![None; owner_count];
        let mut pending = Vec::with_capacity(owner_count);
        // Prepending while visiting increasing IDs keeps siblings in stack order.
        for &declaration in &owners {
            self.charge(1)?;
            let owner = admitted[declaration.0 as usize].0;
            let position = indices[owner.0 as usize].ok_or("source scope lost its position")?;
            if let Some(&parent) = facts.generics.parents.get(&owner) {
                let parent = indices
                    .get(parent.0 as usize)
                    .copied()
                    .flatten()
                    .ok_or("source generic scope parent is not an actual retained owner")?;
                next_sibling[scope_index(position)] =
                    first_child[scope_index(parent)].replace(position);
            } else {
                pending.push((position, 0_u32, None));
            }
        }
        drop(indices);
        self.release_temporary(index_bytes)?;
        self.reserve_scope_slots(&slots)?;
        pending.reverse();
        while let Some((position, captures, parent)) = pending.pop() {
            self.charge(1)?;
            let index = scope_index(position);
            let declaration = owners[index];
            let owner = admitted[declaration.0 as usize].0;
            let declared = self.declared_scope_slots(
                facts,
                owner,
                declaration,
                report.resolution.definitions[owner.0 as usize].kind,
            )?;
            let parent = parent.map(|position| owners[scope_index(position)]);
            self.rows.scopes.push(s::ParameterScope {
                owner: declaration,
                parent,
                declared,
                captures,
            });
            let total = captures
                .checked_add(declared.end - declared.start)
                .ok_or("source scope parameter count overflow")?;
            let mut child = first_child[index];
            while let Some(child_position) = child {
                pending.push((child_position, total, Some(position)));
                child = next_sibling[scope_index(child_position)];
            }
        }
        if self.rows.scopes.len() != owner_count {
            return Err("source generic scope graph is cyclic or incomplete");
        }
        drop((pending, first_child, next_sibling, owners));
        self.release_temporary(owner_bytes.saturating_add(forest_bytes))
    }

    fn reserve_scope_slots(&mut self, slots: &ScopeSlots) -> Result<(), &'static str> {
        use crate::types::parameters::ParameterId;
        let count = slots.parameters.saturating_add(slots.regions);
        self.reserve(count.saturating_mul(size_of::<s::ParameterSlot>()))?;
        self.rows
            .slots
            .try_reserve_exact(count)
            .map_err(|_| "source generic slot column allocation failed")?;

        let parameters = retained::new_map::<ParameterId, s::SlotRef>(slots.parameters);
        self.reserve(parameters)?;
        self.graph
            .parameters
            .try_reserve(slots.parameters)
            .map_err(|_| "source generic parameter index allocation failed")?;
        let retained = retained::map(&self.graph.parameters);
        self.release_temporary(
            parameters
                .checked_sub(retained)
                .ok_or("source generic parameter index exceeded its preflight")?,
        )?;

        let regions = retained::new_map::<(DeclarationRef, u32), s::SlotRef>(slots.regions);
        self.reserve(regions)?;
        self.inputs
            .regions
            .try_reserve(slots.regions)
            .map_err(|_| "source generic region index allocation failed")?;
        let retained = retained::map(&self.inputs.regions);
        self.release_temporary(
            regions
                .checked_sub(retained)
                .ok_or("source generic region index exceeded its preflight")?,
        )
    }

    fn emit_scope_slot(&mut self, row: s::ParameterSlot) -> Result<(), &'static str> {
        if self.rows.slots.len() == self.rows.slots.capacity() {
            return Err("source generic slot column exceeded its preflight");
        }
        self.rows.slots.push(row);
        Ok(())
    }

    fn has_scope(
        &self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
        kind: s::DefinitionKind,
    ) -> bool {
        matches!(
            kind,
            s::DefinitionKind::SelfType
                | s::DefinitionKind::Trait
                | s::DefinitionKind::TraitAlias
                | s::DefinitionKind::Function
                | s::DefinitionKind::Struct
                | s::DefinitionKind::Enum
                | s::DefinitionKind::AssociatedType
        ) || kind == s::DefinitionKind::Constant && self.scoped_constant(report, facts, owner)
            || kind == s::DefinitionKind::TypeAlias
                && facts
                    .generics
                    .parameters
                    .get(&owner)
                    .is_some_and(|parameters| !parameters.is_empty())
    }

    fn declared_scope_slots(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        kind: DefKind,
    ) -> Result<g::Range, &'static str> {
        let start =
            u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?;
        if kind == DefKind::Const {
            return Ok(g::Range { start, end: start });
        }
        // Written lifetimes lead template arguments. Associated families add
        // their own written lifetimes after the parent binder's captures.
        let family = self
            .declaration_policies()
            .then(|| facts.associated.families.get(&owner))
            .flatten();
        let leading = if self.body_nodes() && matches!(kind, DefKind::Struct | DefKind::Enum)
            || self.declaration_policies() && (kind == DefKind::Trait || family.is_some())
        {
            let captures = family.map_or(0, |actual| actual.captures);
            self.leading_region_slots(facts, owner, declaration, start, captures)?;
            Leading::Declaration
        } else if self.body_nodes()
            && kind == DefKind::SelfType
            && implementation_owner(facts, owner, self.declaration_policies())
        {
            Leading::Implementation
        } else {
            Leading::None
        };
        let defaults = kind == DefKind::Trait && self.body_nodes()
            || matches!(kind, DefKind::Struct | DefKind::Enum) && self.declaration_policies();
        self.type_slots(facts, owner, declaration, defaults, leading)?;
        let trait_self = self.declaration_policies()
            && kind == DefKind::SelfType
            && facts.associated.trait_owner(owner).is_some();
        // A plain associated result uses its parent's binder. Its captured
        // regions already belong to that retained parent scope.
        let associated_result = self.declaration_policies()
            && kind == DefKind::TypeAlias
            && facts.associated.owner(owner).is_some();
        if leading != Leading::Declaration && !trait_self && !associated_result {
            self.region_slots(facts, owner, declaration, start)?;
        }
        let end = u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?;
        Ok(g::Range { start, end })
    }

    fn scoped_constant(&self, report: &Report, facts: &Declarations, definition: DefId) -> bool {
        self.provided_constant(report, facts, definition)
            || self.authored_required_constant(report, facts, definition)
                && !self.range_required_constant(facts, definition)
    }

    /// Older nominal profiles carry declared lifetimes without outlives rows.
    fn nominal_lifetime_bounds(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        if !self.body_nodes() || self.declaration_policies() {
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
        defaults: bool,
        leading: Leading,
    ) -> Result<(), &'static str> {
        // Nominals and traits put their lifetime arguments first. Their
        // wire indices count type slots separately. An impl's leading lifetimes are
        // its captured region slots, recorded after its types, so its type
        // slot's ordinal and index both count type slots only.
        let mixed_arguments = leading != Leading::None;
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
            // Each header records its explicit defaults in the actual declaration binder.
            if parameter.default.is_some() && !defaults {
                return Err("source parameter defaults are outside the initial profile");
            }
            if mixed_arguments && matches!(parameter.argument, Argument::Lifetime(_)) {
                if u32::try_from(ordinal).ok() != Some(regions) {
                    return Err("source lifetime template argument follows a type slot");
                }
                regions += 1;
                continue;
            }
            // Version thirteen: a const parameter is a slot in the type index
            // space, recorded with its primitive type.
            let (id, constant) = match parameter.argument {
                Argument::Type(ty) => {
                    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                        return Err("source generic slot has no actual parameter identity");
                    };
                    (id, false)
                }
                Argument::Const(term) if self.body_nodes() => {
                    let Some(ConstKind::Parameter(id)) = facts.types.const_kind(term) else {
                        return Err("source generic slot has no actual parameter identity");
                    };
                    (id, true)
                }
                _ => return Err("source const/lifetime generic argument is not a type slot"),
            };
            let actual = facts
                .types
                .parameter(id)
                .ok_or("source parameter is unknown")?;
            if actual.owner.0 != owner.0 {
                return Err("source parameter belongs to another declaration");
            }
            let sized = match actual.kind {
                ParameterKind::Type { sized } if !constant => sized,
                ParameterKind::Const { ty } if constant => {
                    let context = super::canonical_nodes::Context {
                        owner: declaration,
                        frame: None,
                        body: None,
                    };
                    let ty = self.ty(facts, ty, context)?;
                    if self.declaration_policies() {
                        s::const_formal_type(&self.rows.types, ty)?;
                    } else {
                        s::const_parameter_type(&self.rows.types, ty)?;
                    }
                    self.push_slot(
                        parameter.binding,
                        id,
                        declaration,
                        (ordinal, regions, leading),
                        actual.index,
                        s::ParameterKind::Const { ty },
                    )?;
                    continue;
                }
                _ => return Err("source slot kind is not a type parameter"),
            };
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
            self.push_slot(
                parameter.binding,
                id,
                declaration,
                (ordinal, regions, leading),
                actual.index,
                s::ParameterKind::Type { sized },
            )?;
        }
        Ok(())
    }

    /// One type or const slot, in the type index space after any leading
    /// lifetimes.
    fn push_slot(
        &mut self,
        binding: DefId,
        id: crate::types::parameters::ParameterId,
        declaration: DeclarationRef,
        (ordinal, regions, leading): (usize, u32, Leading),
        index: u32,
        kind: s::ParameterKind,
    ) -> Result<(), &'static str> {
        let definition = Some(self.declaration(binding)?);
        let slot = s::SlotRef(
            u32::try_from(self.rows.slots.len()).map_err(|_| "source slot table overflow")?,
        );
        if self.graph.parameters.insert(id, slot).is_some() {
            return Err("source parameter identity has duplicate slot rows");
        }
        let ordinal = u32::try_from(ordinal).map_err(|_| "source parameter ordinal overflow")?;
        self.emit_scope_slot(s::ParameterSlot {
            owner: declaration,
            ordinal: if leading == Leading::Implementation {
                ordinal - regions
            } else {
                ordinal
            },
            index: index
                .checked_sub(regions)
                .ok_or("source nominal type slot precedes its lifetimes")?,
            definition,
            kind,
            default: None,
        })
    }

    /// Each written lifetime is one leading template argument and one bound
    /// region at the same index. Its slot grants no variance or outlives fact.
    fn leading_region_slots(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        start: u32,
        captures: u32,
    ) -> Result<(), &'static str> {
        let declared = facts
            .lifetimes
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        let arguments = facts.generics.parameters.get(&owner);
        for (offset, &definition) in (0_u32..).zip(declared) {
            self.charge(1)?;
            let index = captures
                .checked_add(offset)
                .ok_or("source lifetime slot overflow")?;
            let region = crate::types::RegionId::bound(
                0,
                u16::try_from(index).map_err(|_| "source nominal lifetime slot overflow")?,
            )?;
            if arguments.is_some_and(|arguments| {
                arguments
                    .get(offset as usize)
                    .map(|p| (p.binding, p.argument))
                    != Some((definition, Argument::Lifetime(region)))
            }) {
                return Err("source lifetime changed its template argument");
            }
            let ordinal = u32::try_from(self.rows.slots.len())
                .map_err(|_| "source region slot overflow")?
                .checked_sub(start)
                .ok_or("source region slot changed owner range")?;
            let slot = s::SlotRef(
                u32::try_from(self.rows.slots.len()).map_err(|_| "source region slot overflow")?,
            );
            if self
                .inputs
                .regions
                .insert((declaration, index), slot)
                .is_some()
            {
                return Err("source lifetime has duplicate actual slot identities");
            }
            let definition = Some(self.declaration(definition)?);
            self.emit_scope_slot(s::ParameterSlot {
                owner: declaration,
                ordinal,
                index,
                definition,
                kind: s::ParameterKind::Region,
                default: None,
            })?;
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
        let (first, total) = region_interval(facts, owner);
        let named = facts.lifetimes.parameters.get(&owner);
        let binding_bytes = (total as usize).saturating_mul(size_of::<Option<DefId>>());
        if binding_bytes != 0 {
            self.reserve(binding_bytes)?;
        }
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
            let slot = s::SlotRef(
                u32::try_from(self.rows.slots.len()).map_err(|_| "source region slot overflow")?,
            );
            if self
                .inputs
                .regions
                .insert((declaration, index), slot)
                .is_some()
            {
                return Err("source lifetime has duplicate actual slot identities");
            }
            self.emit_scope_slot(s::ParameterSlot {
                owner: declaration,
                ordinal,
                index,
                definition,
                kind: s::ParameterKind::Region,
                default: None,
            })?;
        }
        drop(bindings);
        if binding_bytes != 0 {
            self.release_temporary(binding_bytes)?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct ScopeSlots {
    parameters: usize,
    regions: usize,
}
impl ScopeSlots {
    fn include(&mut self, facts: &Declarations, owner: DefId) {
        let parameters = facts.generics.parameters.get(&owner).map_or(0, Vec::len);
        let named = facts.lifetimes.parameters.get(&owner).map_or(0, Vec::len);
        let (first, total) = region_interval(facts, owner);
        self.parameters = self.parameters.saturating_add(parameters);
        self.regions = self
            .regions
            .saturating_add(named.max(total.saturating_sub(first) as usize));
    }
}

fn region_interval(facts: &Declarations, owner: DefId) -> (u32, u32) {
    let captures = facts
        .associated
        .schema(owner)
        .map_or(0, |schema| schema.captures);
    facts
        .lifetimes
        .functions
        .get(&owner)
        .map_or((0, captures), |scheme| (scheme.captures, scheme.total))
}

fn scope_index(position: NonZeroU32) -> usize {
    (position.get() - 1) as usize
}

/// Where an owner's lifetimes stand among its template arguments.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Leading {
    /// The type-argument vector owns no lifetimes.
    None,
    /// Leading written lifetimes of a nominal, trait or associated family.
    Declaration,
    /// Captures of an inherent impl in v13 or a trait impl in v14.
    Implementation,
}

/// An actual implementation Self owner, with the profile's trait-impl fence.
pub(super) fn implementation_owner(facts: &Declarations, owner: DefId, trait_impls: bool) -> bool {
    facts
        .associated
        .owner(owner)
        .and_then(|id| facts.associated.get(id))
        .is_some_and(|actual| {
            actual.self_definition == owner
                && matches!(
                    actual.kind,
                    crate::sema::associated::OwnerKind::Impl { trait_type, .. }
                        if trait_impls || trait_type.is_none()
                )
        })
}
