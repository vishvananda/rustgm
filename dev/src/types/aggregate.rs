//! Completed nominal shapes are separate from canonical, potentially recursive identity.
pub mod finiteness;
mod instances;
mod packing;
mod preparation;
use super::{ItemId, ListId, Primitive, Store, TypeId};
pub use instances::Shape;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Struct,
    Enum,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Representation {
    #[default]
    Rust,
    C,
    Transparent,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Repr {
    pub representation: Representation,
    pub packed: Option<u32>,
    pub align: Option<u32>,
    pub tag: Option<Primitive>,
}
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub fields: ListId,
    /// Target-width two's-complement bits; the semantic checker validates its range.
    pub discriminant: u128,
}
#[derive(Clone, Copy, Debug)]
pub struct Definition {
    pub item: ItemId,
    pub kind: Kind,
    pub repr: Repr,
    pub start: u32,
    pub end: u32,
}
#[derive(Default)]
pub struct Definitions {
    packing_checked: bool,
    entries: Vec<Definition>,
    index: HashMap<ItemId, u32>,
    variants: Vec<Variant>,
    instances: Option<Box<instances::Instances>>,
}
impl Definitions {
    /// Instantiate immediate fields from completed templates, keyed by the full
    /// canonical nominal type. This does not expand referents or prove layout.
    /// Parameter/default/bound validation belongs to declaration semantics.
    /// # Errors
    /// Rejects missing templates, invalid arguments and bounded substitution work.
    pub fn instantiate(&mut self, types: &mut Store, ty: TypeId) -> Result<(), &'static str> {
        let Some(super::Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("field instantiation requires a nominal type");
        };
        let definition = *self.get(item).ok_or("missing aggregate field template")?;
        if types.region_args(arguments).is_some() {
            return Ok(());
        }
        let templates = &self.variants[definition.start as usize..definition.end as usize];
        self.instances
            .get_or_insert_with(Default::default)
            .demand(types, ty, templates)
    }
    /// Read structural fields without allocating or repeating substitution.
    /// Lifetime-only shapes retain the established declaration-template path.
    /// # Errors
    /// Rejects missing nominal identities or undemanded type/const field instances.
    pub fn shape(&self, types: &Store, ty: TypeId) -> Result<Shape<'_>, &'static str> {
        let Some(super::Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("aggregate shape requires a nominal type");
        };
        let definition = self
            .get(item)
            .ok_or("missing completed aggregate definition")?;
        let fields = if types.region_args(arguments).is_some() {
            None
        } else {
            Some(
                self.instances
                    .as_ref()
                    .ok_or("generic aggregate requires instantiated fields")?
                    .get(ty)?,
            )
        };
        Shape::new(definition, self.variants(definition), fields)
    }
    #[must_use]
    pub fn instantiation_steps(&self) -> usize {
        self.instances
            .as_ref()
            .map_or(0, |instances| instances.steps)
    }
    /// Publish a completed shape once, retaining declaration/variant ordering.
    ///
    /// # Errors
    /// Rejects duplicate publication, invalid struct arity and compact ID overflow.
    pub fn insert(
        &mut self,
        item: ItemId,
        kind: Kind,
        repr: Repr,
        variants: &[Variant],
    ) -> Result<(), &'static str> {
        if self.index.contains_key(&item) {
            return Err("aggregate definition already complete");
        }
        if kind == Kind::Struct && variants.len() != 1 {
            return Err("a struct has exactly one field group");
        }
        let id = u32::try_from(self.entries.len()).map_err(|_| "too many aggregate definitions")?;
        let start =
            u32::try_from(self.variants.len()).map_err(|_| "too many aggregate variants")?;
        let end = self
            .variants
            .len()
            .checked_add(variants.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("too many aggregate variants")?;
        self.variants.extend_from_slice(variants);
        self.packing_checked = false;
        self.entries.push(Definition {
            item,
            kind,
            repr,
            start,
            end,
        });
        self.index.insert(item, id);
        Ok(())
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&Definition> {
        self.entries.get(*self.index.get(&item)? as usize)
    }
    #[must_use]
    pub fn variants(&self, definition: &Definition) -> &[Variant] {
        &self.variants[definition.start as usize..definition.end as usize]
    }
    /// Finish discriminants before a layout cache observes these definitions.
    ///
    /// # Errors
    /// Rejects an unknown owner or variant ordinal.
    pub fn set_discriminant(
        &mut self,
        item: ItemId,
        ordinal: u32,
        bits: u128,
    ) -> Result<(), &'static str> {
        let definition = self.get(item).ok_or("unknown discriminant owner")?;
        if ordinal >= definition.end - definition.start {
            return Err("invalid discriminant variant");
        }
        let index = definition.start + ordinal;
        self.variants[index as usize].discriminant = bits;
        Ok(())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Definition> {
        self.entries.iter()
    }
}

impl<'a> IntoIterator for &'a Definitions {
    type Item = &'a Definition;
    type IntoIter = std::slice::Iter<'a, Definition>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
