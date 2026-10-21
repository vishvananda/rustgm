//! Nominal field identities, source provenance and constructor selection.
use crate::{
    resolve::{
        model::{Access, DefId},
        unit::NodeKey,
    },
    support::names::{NameId, Names},
    syntax::item::Shape,
    types::{ItemId, TypeId},
};
use std::collections::HashMap;

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
    pub node: NodeKey,
    pub name: Name,
    pub ty: TypeId,
    pub access: Access,
}
#[derive(Clone, Copy)]
pub struct Variant {
    pub owner: ItemId,
    pub ordinal: u32,
    pub node: NodeKey,
    pub shape: Shape,
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy)]
enum Provenance {
    Source(NodeKey),
    Metadata(NodeKey),
}
impl Provenance {
    fn origin(self) -> NodeKey {
        match self {
            Self::Source(key) | Self::Metadata(key) => key,
        }
    }
    fn source(self) -> Option<NodeKey> {
        if let Self::Source(key) = self {
            Some(key)
        } else {
            None
        }
    }
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
    by_owner: HashMap<ItemId, Vec<VariantId>>,
    by_name: HashMap<(VariantId, Name), FieldId>,
}
impl Members {
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
        self.publish(owner, ordinal, Provenance::Source(node), shape, fields)
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
        self.publish(owner, ordinal, Provenance::Metadata(origin), shape, fields)
    }
    fn publish(
        &mut self,
        owner: ItemId,
        ordinal: u32,
        provenance: Provenance,
        shape: Shape,
        fields: &[Field],
    ) -> Result<VariantId, &'static str> {
        let node = provenance.origin();
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
            node,
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
