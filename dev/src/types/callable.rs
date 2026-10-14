//! Function-item templates preserve formal early arguments across inferred uses.
use super::parameters::ParameterId;
use super::{ItemId, Kind, RegionId, Store, TypeId, outlives::Pair};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Template {
    /// Early parameters live outside the callable's late binder: depth one.
    pub signature: TypeId,
    pub parameters: u32,
    start: usize,
    end: usize,
    generics_start: usize,
    generics_end: usize,
}
/// Completed declaration facts. A query borrows one fixed table and type store.
#[derive(Default)]
pub struct Templates {
    entries: HashMap<ItemId, Template>,
    bounds: Vec<Pair>,
    generics: Vec<ParameterId>,
}
impl Templates {
    /// Publish a generic callable once. Bound pairs use early parameter indices
    /// at depth zero; the signature retains its own late-bound depth zero.
    ///
    /// # Errors
    /// Rejects repeated publication, malformed templates and excessive storage.
    pub fn insert(
        &mut self,
        types: &Store,
        item: ItemId,
        signature: TypeId,
        parameters: u32,
        bounds: &[Pair],
    ) -> Result<(), &'static str> {
        self.insert_generic(types, item, signature, parameters, bounds, &[])
    }
    /// Publish the compact type/const parameter order separately from early and
    /// late lifetimes. Parameter identities retain their declaration indices.
    /// # Errors
    /// Rejects invalid/repeated templates, foreign parameters and storage limits.
    pub fn insert_generic(
        &mut self,
        types: &Store,
        item: ItemId,
        signature: TypeId,
        parameters: u32,
        bounds: &[Pair],
        generics: &[ParameterId],
    ) -> Result<(), &'static str> {
        if self.entries.contains_key(&item)
            || (parameters == 0 && generics.is_empty())
            || parameters > 65_536
            || !matches!(types.kind(signature), Some(Kind::Function { .. }))
            || types
                .escaping_binders(signature)
                .is_none_or(|depth| depth > 1)
        {
            return Err("invalid generic callable template");
        }
        let mut uses = super::bound_uses::Uses::default();
        uses.collect(types, signature)?;
        if uses
            .indices
            .iter()
            .any(|index| u32::from(*index) >= parameters)
        {
            return Err("callable template references a missing early parameter");
        }
        if self
            .bounds
            .len()
            .saturating_add(bounds.len())
            .saturating_add(self.entries.len())
            .saturating_add(self.generics.len())
            .saturating_add(generics.len())
            >= 1_000_000
        {
            return Err("callable template storage budget exceeded");
        }
        let valid = |region: RegionId| {
            region == RegionId::STATIC
                || region
                    .bound_parts()
                    .is_some_and(|(depth, index)| depth == 0 && u32::from(index) < parameters)
        };
        if bounds.iter().any(|(a, b)| !valid(*a) || !valid(*b)) {
            return Err("invalid callable template bound");
        }
        let mut previous = None;
        for parameter in generics {
            let parameter = types
                .parameter(*parameter)
                .ok_or("invalid callable generic parameter")?;
            if parameter.owner != item || previous.is_some_and(|index| index >= parameter.index) {
                return Err(
                    "callable generic parameters must belong to this owner in declaration order",
                );
            }
            previous = Some(parameter.index);
        }
        let start = self.bounds.len();
        let generics_start = self.generics.len();
        self.generics.extend_from_slice(generics);
        self.bounds.extend_from_slice(bounds);
        self.entries.insert(
            item,
            Template {
                signature,
                parameters,
                start,
                end: self.bounds.len(),
                generics_start,
                generics_end: self.generics.len(),
            },
        );
        Ok(())
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<Template> {
        self.entries.get(&item).copied()
    }
    #[must_use]
    pub fn bounds(&self, item: ItemId) -> Option<&[Pair]> {
        let template = self.entries.get(&item)?;
        Some(&self.bounds[template.start..template.end])
    }
    #[must_use]
    pub fn generic_parameters(&self, item: ItemId) -> Option<&[ParameterId]> {
        let template = self.entries.get(&item)?;
        Some(&self.generics[template.generics_start..template.generics_end])
    }
}
