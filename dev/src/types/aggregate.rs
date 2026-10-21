//! Completed nominal shapes are separate from canonical, potentially recursive identity.
mod field_forms;
pub mod finiteness;
mod instances;
mod metadata;
mod packing;
mod preparation;
use super::{ItemId, ListId, Primitive, Store, TypeId};
pub use instances::Shape;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Struct,
    Enum,
    Union,
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldDestruction {
    #[default]
    Fields,
    Suppressed,
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
    pending_discriminants: HashMap<ItemId, std::collections::HashSet<u32>>,
    discriminant_indexing: usize,
    discriminant_publications: usize,
    unions: bool,
    packing_checked: bool,
    projected_templates: bool,
    entries: Vec<Definition>,
    index: HashMap<ItemId, u32>,
    variants: Vec<Variant>,
    instances: Option<Box<instances::Instances>>,
    storage: Option<Box<field_forms::Fields>>,
    metadata_ready: std::collections::HashSet<TypeId>,
    suppressed_fields: std::collections::HashSet<ItemId>,
}
impl Definitions {
    pub(crate) fn mark_discriminants_pending(&mut self, item: ItemId) -> Result<(), &'static str> {
        let definition = self.get(item).ok_or("missing pending enum declaration")?;
        if definition.kind != Kind::Enum || self.pending_discriminants.contains_key(&item) {
            return Err("invalid pending enum discriminants");
        }
        let count = definition.end - definition.start;
        self.discriminant_indexing = self
            .discriminant_indexing
            .checked_add(count as usize)
            .filter(|&work| work <= 1_000_000)
            .ok_or("enum discriminant indexing budget exceeded")?;
        if count != 0 {
            self.pending_discriminants
                .insert(item, (0..count).collect());
        }
        Ok(())
    }
    #[must_use]
    pub(crate) fn discriminants_ready(&self, item: ItemId) -> bool {
        !self.pending_discriminants.contains_key(&item)
    }
    #[must_use]
    pub(crate) fn discriminant_ready(&self, item: ItemId, ordinal: u32) -> bool {
        self.pending_discriminants
            .get(&item)
            .is_none_or(|pending| !pending.contains(&ordinal))
    }
    /// Actual indexed variant ordinals and first completed publications.
    #[must_use]
    pub fn discriminant_work(&self) -> (usize, usize) {
        (self.discriminant_indexing, self.discriminant_publications)
    }
    #[must_use]
    pub const fn has_unions(&self) -> bool {
        self.unions
    }
    /// Declaration policy suppresses implicit field cleanup, preserving an explicit destructor.
    #[must_use]
    pub fn destruction(&self, item: ItemId) -> Option<FieldDestruction> {
        self.get(item)?;
        Some(if self.suppressed_fields.contains(&item) {
            FieldDestruction::Suppressed
        } else {
            FieldDestruction::Fields
        })
    }
    pub(crate) fn suppress_field_destruction(&mut self, item: ItemId) -> Result<(), &'static str> {
        self.get(item)
            .ok_or("missing destruction policy declaration")?;
        if !self.suppressed_fields.insert(item) {
            return Err("field destruction policy is already registered");
        }
        Ok(())
    }
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
        let templates = &self.variants[definition.start as usize..definition.end as usize];
        let instances = self.instances.get_or_insert_with(Default::default);
        if types.region_args(arguments).is_some()
            && !instances.projected_template(types, item, templates)?
        {
            return Ok(());
        }
        instances.demand(types, ty, templates)
    }
    /// Read structural fields without allocating or repeating substitution.
    /// Concrete projected fields require their independently normalized storage.
    /// # Errors
    /// Rejects missing nominal identities or undemanded type/const field instances.
    pub fn shape(&self, types: &Store, ty: TypeId) -> Result<Shape<'_>, &'static str> {
        if let Some(fields) = self.storage.as_ref().and_then(|storage| storage.get(ty)) {
            let raw = self.raw_shape(types, ty)?;
            return Shape::new(raw.definition, self.variants(raw.definition), Some(fields));
        }
        let shape = self.raw_shape(types, ty)?;
        if types.contains_parameters(ty) == Some(false)
            && self
                .instances
                .as_ref()
                .is_some_and(|instances| instances.has_projections(ty))
        {
            return Err("concrete aggregate fields require associated type normalization");
        }
        Ok(shape)
    }
    /// Raw substituted fields remain available for caller-owned formation proofs.
    pub(crate) fn raw_shape(&self, types: &Store, ty: TypeId) -> Result<Shape<'_>, &'static str> {
        let Some(super::Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("aggregate shape requires a nominal type");
        };
        let definition = self
            .get(item)
            .ok_or("missing completed aggregate definition")?;
        let fields = if types.region_args(arguments).is_some()
            && !self
                .instances
                .as_ref()
                .is_some_and(|instances| instances.contains(ty))
        {
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
        if matches!(kind, Kind::Struct | Kind::Union) && variants.len() != 1 {
            return Err("a struct or union has exactly one field group");
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
        self.unions |= kind == Kind::Union;
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
        if let Some(pending) = self.pending_discriminants.get_mut(&item) {
            self.discriminant_publications += usize::from(pending.remove(&ordinal));
            if pending.is_empty() {
                self.pending_discriminants.remove(&item);
            }
        }
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

#[cfg(test)]
#[path = "../../tests/invariants/enum_tag_readiness.rs"]
mod readiness_tests;
