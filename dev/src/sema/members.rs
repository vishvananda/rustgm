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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldId(pub u32);
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
#[derive(Default)]
pub struct Members {
    pub names: Names,
    pub constructors: HashMap<DefId, VariantId>,
    pub variants: Vec<Variant>,
    pub fields: Vec<Field>,
    by_node: HashMap<NodeKey, VariantId>,
    by_name: HashMap<(VariantId, Name), FieldId>,
}
impl Members {
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
        if self.by_node.contains_key(&node) {
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
        self.variants.push(Variant {
            owner,
            ordinal,
            node,
            shape,
            start,
            end,
        });
        self.by_node.insert(node, id);
        Ok(id)
    }
    #[must_use]
    pub fn at_node(&self, node: NodeKey) -> Option<VariantId> {
        self.by_node.get(&node).copied()
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
