//! Typed field instances share declaration identity and discriminants.
use super::{Definition, Variant};
use crate::types::{Kind, ListId, Store, TypeId, substitute::Bound};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct Range {
    start: u32,
    end: u32,
}
#[derive(Default)]
pub(super) struct Instances {
    index: HashMap<TypeId, Result<Range, &'static str>>,
    projected_templates: HashMap<crate::types::ItemId, bool>,
    projections: HashSet<TypeId>,
    fields: Vec<ListId>,
    pub prepared: HashMap<(TypeId, bool), Result<(), &'static str>>,
    pub steps: usize,
    retained: usize,
}
impl Instances {
    pub(super) fn has_projections(&self, ty: TypeId) -> bool {
        self.projections.contains(&ty)
    }
    pub(super) fn projected_template(
        &mut self,
        types: &Store,
        item: crate::types::ItemId,
        templates: &[Variant],
    ) -> Result<bool, &'static str> {
        if let Some(&projected) = self.projected_templates.get(&item) {
            return Ok(projected);
        }
        let mut projected = false;
        for template in templates {
            let fields = types
                .list(template.fields)
                .ok_or("invalid field template")?;
            self.charge(fields.len().saturating_add(1))?;
            projected |= fields
                .iter()
                .any(|&field| types.contains_projections(field) == Some(true));
        }
        self.retain(64)?;
        self.projected_templates.insert(item, projected);
        Ok(projected)
    }
    pub(super) fn contains(&self, ty: TypeId) -> bool {
        self.index.contains_key(&ty)
    }
    pub(super) fn get(&self, ty: TypeId) -> Result<&[ListId], &'static str> {
        let range = *self
            .index
            .get(&ty)
            .ok_or("generic aggregate requires instantiated fields")?;
        let range = range?;
        self.fields
            .get(range.start as usize..range.end as usize)
            .ok_or("invalid instantiated field extent")
    }
    pub(super) fn demand(
        &mut self,
        types: &mut Store,
        ty: TypeId,
        templates: &[Variant],
    ) -> Result<(), &'static str> {
        if let Some(result) = self.index.get(&ty) {
            return result.map(|_| ());
        }
        self.charge(1)?;
        self.retain(64)?;
        let result = self.instantiate(types, ty, templates);
        self.index.insert(ty, result);
        result.map(|_| ())
    }
    pub(super) fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= 4_000_000)
            .ok_or("aggregate instantiation work budget exceeded")?;
        Ok(())
    }
    pub(super) fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(bytes)
            .filter(|n| *n <= 64_000_000)
            .ok_or("aggregate instantiation exceeds 64 MB budget")?;
        Ok(())
    }
    fn instantiate(
        &mut self,
        types: &mut Store,
        ty: TypeId,
        templates: &[Variant],
    ) -> Result<Range, &'static str> {
        self.charge(templates.len())?;
        self.retain(templates.len().saturating_mul(size_of::<ListId>()))?;
        let Some(Kind::Adt(owner, arguments)) = types.kind(ty) else {
            return Err("invalid field instance type");
        };
        let mut substitution = Bound::joint(types, owner, arguments)?;
        let mut variants = Vec::with_capacity(templates.len());
        let mut fields = Vec::new();
        for template in templates {
            let count = types
                .list(template.fields)
                .ok_or("invalid aggregate field template")?
                .len();
            self.charge(count)?;
            fields.clear();
            for index in 0..count {
                let field = types
                    .list(template.fields)
                    .ok_or("invalid aggregate field template")?[index];
                let before = substitution.steps;
                let result = substitution.apply(types, field);
                self.charge(substitution.steps.saturating_sub(before))?;
                let result = result?;
                if types.contains_projections(result) == Some(true) {
                    self.projections.insert(ty);
                }
                fields.push(result);
            }
            variants.push(types.intern_list(&fields)?);
        }
        let start =
            u32::try_from(self.fields.len()).map_err(|_| "too many instantiated field groups")?;
        let end = self
            .fields
            .len()
            .checked_add(variants.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("too many instantiated field groups")?;
        self.fields.extend(variants);
        Ok(Range { start, end })
    }
}

/// Borrowed structural view of a particular nominal use. Substituted field lists
/// are distinct from tags/representation, which remain on the declaration.
#[derive(Clone, Copy)]
pub struct Shape<'a> {
    pub definition: &'a Definition,
    variants: &'a [Variant],
    fields: Option<&'a [ListId]>,
}
impl<'a> Shape<'a> {
    pub(super) fn new(
        definition: &'a Definition,
        variants: &'a [Variant],
        fields: Option<&'a [ListId]>,
    ) -> Result<Self, &'static str> {
        if fields.is_some_and(|fields| fields.len() != variants.len()) {
            return Err("instantiated aggregate has different variant coverage");
        }
        Ok(Self {
            definition,
            variants,
            fields,
        })
    }
    #[must_use]
    pub fn variant(self, index: usize) -> Option<Variant> {
        let mut variant = *self.variants.get(index)?;
        if let Some(fields) = self.fields {
            variant.fields = *fields.get(index)?;
        }
        Some(variant)
    }
    #[must_use]
    pub fn variants(
        self,
    ) -> impl ExactSizeIterator<Item = Variant> + DoubleEndedIterator + Clone + 'a {
        self.variants
            .iter()
            .enumerate()
            .map(move |(index, variant)| Variant {
                fields: self.fields.map_or(variant.fields, |fields| fields[index]),
                ..*variant
            })
    }
}
