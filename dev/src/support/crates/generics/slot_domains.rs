//! One structural scope index keeps inherited slots shared and ownership queries constant.
use super::{ParameterKind, ParameterSlot, SlotRef, SourceRows};
use crate::support::crates::{self, generics::Work, owner_catalog::DeclarationRef};

/// A borrowed kind view is derived from the same complete declaration table.
/// It is not an authenticated frontend mapping or permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    Module,
    Trait,
    SelfType,
    Function,
    TypeParameter,
    RegionParameter,
    ValueParameter,
    Local,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub parent: Option<DeclarationRef>,
}

/// This indexes immutable wire data only; import must authenticate actual kinds.
pub struct ScopeIndex<'a> {
    rows: &'a SourceRows,
    owners: Vec<Option<u32>>,
    totals: Vec<Counts>,
    ends: Vec<u32>,
}
#[derive(Clone, Copy)]
struct Counts {
    all: u32,
    types: u32,
}
impl<'a> ScopeIndex<'a> {
    /// Canonical preorder lets each scope open/close once, without ancestry walks.
    /// # Errors
    /// Rejects unknown/foreign/duplicate bindings, malformed ordering and quotas.
    pub fn new(
        rows: &'a SourceRows,
        definitions: &[Definition],
        retained_bytes: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        preflight(rows, definitions.len(), retained_bytes, work)?;
        let mut owners = vec![None; definitions.len()];
        let mut bindings = vec![None; definitions.len()];
        let mut totals = Vec::<Counts>::with_capacity(rows.scopes.len());
        let mut ends = vec![0; rows.scopes.len()];
        let mut stack = Vec::<u32>::with_capacity(rows.scopes.len());
        let mut cursor = 0;
        for (index, scope) in rows.scopes.iter().enumerate() {
            work.charge(1)?;
            let index = u32::try_from(index).map_err(|_| "source scope index overflow")?;
            let definition = definitions
                .get(scope.owner.0 as usize)
                .ok_or("source parameter scope owner is unknown")?;
            if !matches!(
                definition.kind,
                DefinitionKind::Trait | DefinitionKind::SelfType | DefinitionKind::Function
            ) || owners[scope.owner.0 as usize].is_some()
            {
                return Err("source parameter scope has a duplicate or invalid owner");
            }
            if !valid_parent(scope.owner, definition, scope.parent, definitions) {
                return Err("source parameter scope inherits from a foreign declaration");
            }
            let parent = scope
                .parent
                .map(|parent| {
                    owners
                        .get(parent.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("source parameter scope parent is missing or forward")
                })
                .transpose()?;
            while stack.last().copied() != parent {
                work.charge(1)?;
                let closed = stack
                    .pop()
                    .ok_or("source parameter scope parent is foreign or closed")?;
                ends[closed as usize] = index;
            }
            let captured = parent.map_or(Counts { all: 0, types: 0 }, |parent| {
                totals[parent as usize]
            });
            if scope.captures != captured.all
                || scope.declared.start != cursor
                || scope.declared.end < cursor
                || scope.declared.end as usize > rows.slots.len()
            {
                return Err("source parameter scope changed its exact prefix or declared range");
            }
            let mut types = captured.types;
            for (ordinal, slot) in rows.slots[cursor as usize..scope.declared.end as usize]
                .iter()
                .enumerate()
            {
                work.charge(1)?;
                if slot.owner != scope.owner || slot.ordinal as usize != ordinal {
                    return Err("source parameter slot has a foreign owner, duplicate or gap");
                }
                if matches!(slot.kind, ParameterKind::Type { .. }) && slot.index != slot.ordinal {
                    return Err("source type parameter changed its exact declared index");
                }
                if matches!(slot.kind, ParameterKind::Type { .. }) {
                    types = types
                        .checked_add(1)
                        .ok_or("source type parameter count overflow")?;
                }
                validate_binding(slot, definitions, &mut bindings, work)?;
            }
            let total = captured
                .all
                .checked_add(scope.declared.end - cursor)
                .ok_or("source parameter scope count overflow")?;
            totals.push(Counts { all: total, types });
            owners[scope.owner.0 as usize] = Some(index);
            stack.push(index);
            cursor = scope.declared.end;
        }
        if cursor as usize != rows.slots.len() {
            return Err("source parameter slots have no exact owning scope");
        }
        for closed in stack {
            work.charge(1)?;
            ends[closed as usize] =
                u32::try_from(rows.scopes.len()).map_err(|_| "source scope endpoint overflow")?;
        }
        Ok(Self {
            rows,
            owners,
            totals,
            ends,
        })
    }

    /// A slot can be used only by its exact scope or a retained descendant.
    /// # Errors
    /// Rejects unknown scopes/slots and charges failed queries as well.
    pub fn contains(
        &self,
        owner: DeclarationRef,
        slot: SlotRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(4)?;
        let scope = self
            .owners
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source parameter use has no owner scope")?;
        let slot = self
            .rows
            .slots
            .get(slot.0 as usize)
            .ok_or("source parameter slot is unknown")?;
        let declared = self
            .owners
            .get(slot.owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source parameter slot has no declaring scope")?;
        Ok(declared <= scope && scope < self.ends[declared as usize])
    }

    /// # Errors
    /// Rejects unknown owners and charges successful and failed lookup work.
    pub fn complete_slots(
        &self,
        owner: DeclarationRef,
        work: &mut Work,
    ) -> Result<u32, &'static str> {
        work.charge(2)?;
        self.owners
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .map(|index| self.totals[index as usize].all)
            .ok_or("source parameter count has no owner scope")
    }

    /// Complete type formals include inherited and unused slots, without copying
    /// or walking each owner's ancestry again. Region formals remain distinct.
    /// # Errors
    /// Rejects unknown owners and charges failed queries as well as successes.
    pub fn complete_type_slots(
        &self,
        owner: DeclarationRef,
        work: &mut Work,
    ) -> Result<u32, &'static str> {
        work.charge(2)?;
        self.owners
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .map(|index| self.totals[index as usize].types)
            .ok_or("source type parameter count has no owner scope")
    }

    /// Locate an actual inherited type formal without guessing that Self is
    /// argument zero or retaining flattened formal vectors for every owner.
    /// # Errors
    /// Rejects absent, regional or foreign slots and charges failed queries.
    pub fn type_argument_position(
        &self,
        owner: DeclarationRef,
        slot: SlotRef,
        work: &mut Work,
    ) -> Result<u32, &'static str> {
        if !self.contains(owner, slot, work)? {
            return Err("source application formal belongs to another parameter scope");
        }
        work.charge(3)?;
        let slot = &self.rows.slots[slot.0 as usize];
        if !matches!(slot.kind, ParameterKind::Type { .. }) {
            return Err("source type argument maps to a regional formal");
        }
        let scope = self.owners[slot.owner.0 as usize]
            .ok_or("source application formal has no original parameter scope")?;
        let parent = self.rows.scopes[scope as usize].parent;
        let inherited = parent.map_or(Ok(0), |parent| {
            self.owners[parent.0 as usize]
                .map(|index| self.totals[index as usize].types)
                .ok_or("source application formal parent is missing")
        })?;
        let position = inherited
            .checked_add(slot.index)
            .ok_or("source type argument ordinal overflow")?;
        if position >= self.totals[scope as usize].types {
            return Err("source type argument index is outside the complete type formal order");
        }
        Ok(position)
    }

    /// # Errors
    /// Rejects unknown owners rather than inferring inheritance from lexical scope.
    pub fn parent(
        &self,
        owner: DeclarationRef,
        work: &mut Work,
    ) -> Result<Option<DeclarationRef>, &'static str> {
        work.charge(1)?;
        let index = self
            .owners
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source generic parent query has no actual owner scope")?;
        Ok(self.rows.scopes[index as usize].parent)
    }

    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        use crate::support::retained::vector;
        size_of::<Self>()
            .saturating_add(vector(&self.owners))
            .saturating_add(vector(&self.totals))
            .saturating_add(vector(&self.ends))
    }
}

fn valid_parent(
    owner: DeclarationRef,
    definition: &Definition,
    parent: Option<DeclarationRef>,
    definitions: &[Definition],
) -> bool {
    match definition.kind {
        DefinitionKind::SelfType => parent.is_none(),
        DefinitionKind::Trait => parent
            .and_then(|parent| definitions.get(parent.0 as usize))
            .is_some_and(|parent| {
                parent.kind == DefinitionKind::SelfType && parent.parent == Some(owner)
            }),
        DefinitionKind::Function => {
            match definition.parent {
                None => parent.is_none(),
                Some(expected) if parent.is_none() => definitions
                    .get(expected.0 as usize)
                    .is_some_and(|definition| {
                        matches!(
                            definition.kind,
                            DefinitionKind::Module | DefinitionKind::Function
                        )
                    }),
                Some(expected) if parent == Some(expected) => true,
                Some(expected) => parent
                    .and_then(|parent| definitions.get(parent.0 as usize))
                    .is_some_and(|parent| {
                        parent.kind == DefinitionKind::SelfType && parent.parent == Some(expected)
                    }),
            }
        }
        DefinitionKind::Module
        | DefinitionKind::TypeParameter
        | DefinitionKind::RegionParameter
        | DefinitionKind::ValueParameter
        | DefinitionKind::Local => false,
    }
}

fn validate_binding(
    slot: &ParameterSlot,
    definitions: &[Definition],
    bindings: &mut [Option<u32>],
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(binding) = slot.definition else {
        return Ok(());
    };
    work.charge(1)?;
    let definition = definitions
        .get(binding.0 as usize)
        .ok_or("source parameter definition is unknown")?;
    let valid = match slot.kind {
        ParameterKind::Type { sized: None }
            if binding == slot.owner && definition.kind == DefinitionKind::SelfType =>
        {
            definition
                .parent
                .and_then(|parent| definitions.get(parent.0 as usize))
                .is_some_and(|parent| parent.kind == DefinitionKind::Trait)
        }
        ParameterKind::Type { .. } => {
            definition.kind == DefinitionKind::TypeParameter
                && definition.parent == Some(slot.owner)
        }
        ParameterKind::Region => {
            definition.kind == DefinitionKind::RegionParameter
                && definition.parent == Some(slot.owner)
        }
    };
    if !valid || bindings[binding.0 as usize].replace(slot.ordinal).is_some() {
        return Err(
            "source parameter definition has a foreign owner, wrong kind or duplicate binding",
        );
    }
    Ok(())
}

fn preflight(
    rows: &SourceRows,
    declarations: usize,
    retained_bytes: usize,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    if declarations > crates::MAX_FUNCTIONS
        || rows.scopes.len() > declarations
        || rows.slots.len() > crates::MAX_COMPONENTS
    {
        return Err("source parameter scope row budget exceeded");
    }
    let bytes = declarations
        .saturating_mul(2 * size_of::<Option<u32>>())
        .saturating_add(
            rows.scopes
                .len()
                .saturating_mul(size_of::<Counts>() + 2 * size_of::<u32>()),
        )
        .saturating_add(size_of::<ScopeIndex<'_>>());
    if retained_bytes.saturating_add(bytes) > crates::MAX_BYTES {
        return Err("source parameter scope scratch budget exceeded");
    }
    work.charge(
        declarations
            .saturating_mul(2)
            .saturating_add(rows.scopes.len()),
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/owned_header_slots.rs"]
mod tests;
