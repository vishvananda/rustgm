//! Nominal field identities, source provenance and constructor selection.
#[cfg(test)]
#[path = "../../tests/invariants/provenance_members.rs"]
mod tests;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        model::{Access, DefId},
        unit::NodeKey,
    },
    support::names::{NameId, Names},
    syntax::item::Shape,
    types::{ItemId, TypeId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VariantId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructorKind {
    Variant,
    ConstFunction,
}
#[derive(Clone, Copy, Debug)]
pub struct Constructor {
    pub variant: VariantId,
    pub kind: ConstructorKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Selection {
    Tuple(u32),
    Nominal(FieldId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Name {
    /// Identity in `Members::names`, independently interned from lexical bindings.
    Named(NameId),
    Index(u32),
}
#[derive(Clone, Copy)]
pub struct Field {
    pub origin: Origin,
    pub name: Name,
    pub ty: TypeId,
    pub access: Access,
}
#[derive(Clone, Copy)]
pub struct Variant {
    pub owner: ItemId,
    pub ordinal: u32,
    pub origin: Origin,
    pub shape: Shape,
    pub start: u32,
    pub end: u32,
}
/// External members retain their canonical nominal owner, without source syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Source(NodeKey),
    Library(NodeKey),
    External(DefId),
}
impl Origin {
    #[must_use]
    pub const fn diagnostic(self) -> Option<NodeKey> {
        match self {
            Self::Source(key) | Self::Library(key) => Some(key),
            Self::External(_) => None,
        }
    }
    #[must_use]
    pub const fn source(self) -> Option<NodeKey> {
        if let Self::Source(key) = self {
            Some(key)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn external_owner(self) -> Option<DefId> {
        if let Self::External(owner) = self {
            Some(owner)
        } else {
            None
        }
    }
    pub(crate) fn require_diagnostic(self) -> Result<NodeKey, &'static str> {
        self.diagnostic()
            .ok_or("external member has no source diagnostic identity")
    }
}

/// Coordinates belong to one canonical nominal owner; wire remapping replaces it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Identity {
    pub owner: DefId,
    pub variant: u32,
    pub field: u32,
}
#[derive(Default)]
pub struct Members {
    pub names: Names,
    pub constructors: HashMap<DefId, Constructor>,
    pub variants: Vec<Variant>,
    pub fields: Vec<Field>,
    field_ordinals: Vec<u32>,
    field_variants: Vec<VariantId>,
    by_node: HashMap<NodeKey, VariantId>,
    external_open: Option<HashSet<ItemId>>,
    by_owner: HashMap<ItemId, Vec<VariantId>>,
    by_name: HashMap<(VariantId, Name), FieldId>,
}
impl Members {
    /// Unknown external constructors are coverage facts, never storage variants.
    pub(crate) fn mark_external_open(&mut self, owner: ItemId) {
        self.external_open
            .get_or_insert_with(HashSet::default)
            .insert(owner);
    }
    #[must_use]
    pub fn external_open(&self, owner: ItemId) -> bool {
        self.external_open
            .as_ref()
            .is_some_and(|items| items.contains(&owner))
    }
    /// Selected construction storage is independent of promotion eligibility.
    #[must_use]
    pub fn constructor(&self, definition: DefId) -> Option<VariantId> {
        self.constructors.get(&definition).map(|c| c.variant)
    }
    /// Publish one source-ordered field group and its lookup index.
    ///
    /// # Errors
    /// Rejects duplicate fields/variants and exhausted compact identities.
    pub fn insert(
        &mut self,
        owner: ItemId,
        ordinal: u32,
        node: NodeKey,
        shape: Shape,
        fields: &[Field],
    ) -> Result<VariantId, &'static str> {
        self.publish(owner, ordinal, Origin::Source(node), shape, fields)
    }
    /// Register completed library fields without inventing source syntax identities.
    /// # Errors
    /// Rejects duplicate fields and exhausted compact identities.
    pub(crate) fn insert_metadata(
        &mut self,
        owner: ItemId,
        ordinal: u32,
        origin: NodeKey,
        shape: Shape,
        fields: &[Field],
    ) -> Result<VariantId, &'static str> {
        self.publish(owner, ordinal, Origin::Library(origin), shape, fields)
    }
    /// Publish a source-free closed record after external identities are remapped.
    /// # Errors
    /// Rejects foreign field owners, duplicate records and ordinal inconsistencies.
    pub fn insert_external(
        &mut self,
        owner: ItemId,
        fields: &[Field],
    ) -> Result<VariantId, &'static str> {
        let origin = Origin::External(DefId(owner.0));
        if !self.variants(owner).is_empty() || fields.iter().any(|field| field.origin != origin) {
            return Err("external record member owner is inconsistent");
        }
        if fields
            .iter()
            .any(|field| !matches!(field.name, Name::Named(_)))
        {
            return Err("external record members require named fields");
        }
        self.publish(owner, 0, origin, Shape::Record, fields)
    }
    fn publish(
        &mut self,
        owner: ItemId,
        ordinal: u32,
        provenance: Origin,
        shape: Shape,
        fields: &[Field],
    ) -> Result<VariantId, &'static str> {
        if fields.iter().any(|field| match (provenance, field.origin) {
            (Origin::Source(_), Origin::Source(_)) | (Origin::Library(_), Origin::Library(_)) => {
                false
            }
            (Origin::External(owner), Origin::External(field_owner)) => owner != field_owner,
            _ => true,
        }) {
            return Err("aggregate field provenance differs from its owner");
        }
        if provenance
            .source()
            .is_some_and(|node| self.by_node.contains_key(&node))
        {
            return Err("aggregate variant already complete");
        }
        let id = VariantId(u32::try_from(self.variants.len()).map_err(|_| "too many variants")?);
        let start = u32::try_from(self.fields.len()).map_err(|_| "too many fields")?;
        let end = start
            .checked_add(u32::try_from(fields.len()).map_err(|_| "too many fields")?)
            .ok_or("too many fields")?;
        for (offset, field) in (start..end).zip(fields) {
            if self
                .by_name
                .insert((id, field.name), FieldId(offset))
                .is_some()
            {
                self.by_name.retain(|(variant, _), _| *variant != id);
                return Err("duplicate aggregate field");
            }
        }
        self.fields.extend_from_slice(fields);
        self.field_ordinals.extend(0..end - start);
        self.field_variants
            .extend(std::iter::repeat_n(id, fields.len()));
        self.variants.push(Variant {
            owner,
            ordinal,
            origin: provenance,
            shape,
            start,
            end,
        });
        if let Some(source) = provenance.source() {
            self.by_node.insert(source, id);
        }
        self.by_owner.entry(owner).or_default().push(id);
        Ok(id)
    }
    #[must_use]
    pub fn variants(&self, owner: ItemId) -> &[VariantId] {
        self.by_owner.get(&owner).map_or(&[], Vec::as_slice)
    }
    #[must_use]
    pub fn at_node(&self, node: NodeKey) -> Option<VariantId> {
        self.by_node.get(&node).copied()
    }
    #[must_use]
    pub fn ordinal(&self, field: FieldId) -> Option<u32> {
        self.field_ordinals.get(field.0 as usize).copied()
    }
    #[must_use]
    pub fn variant(&self, field: FieldId) -> Option<VariantId> {
        self.field_variants.get(field.0 as usize).copied()
    }
    #[must_use]
    pub fn identity(&self, field: FieldId) -> Option<Identity> {
        let variant = self.variants.get(self.variant(field)?.0 as usize)?;
        Some(Identity {
            owner: DefId(variant.owner.0),
            variant: variant.ordinal,
            field: self.ordinal(field)?,
        })
    }
    #[must_use]
    pub fn field(&self, variant: VariantId, name: Name) -> Option<FieldId> {
        self.by_name.get(&(variant, name)).copied()
    }
    #[must_use]
    pub fn fields(&self, variant: VariantId) -> Option<&[Field]> {
        let variant = self.variants.get(variant.0 as usize)?;
        self.fields
            .get(variant.start as usize..variant.end as usize)
    }
}
