//! Concrete field normal forms are structural data, separate from their raw proof roots.
use super::{Definitions, ListId, ShapeReadiness, Store, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Fields {
    entries: HashMap<TypeId, Vec<ListId>>,
    retained: usize,
}
impl Fields {
    pub(super) fn get(&self, ty: TypeId) -> Option<&[ListId]> {
        self.entries.get(&ty).map(Vec::as_slice)
    }
}
impl Definitions {
    /// Read available fields without starting substitution or semantic work.
    /// Pending fields are distinct from an empty shape and from malformed IDs.
    /// Discriminants do not participate in structural storage readiness.
    /// # Errors
    /// Preserves malformed identities and failed field instantiations.
    pub fn try_shape(&self, types: &Store, ty: TypeId) -> Result<ShapeReadiness<'_>, &'static str> {
        let Some(crate::types::Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("aggregate shape requires a nominal type");
        };
        types
            .arguments(arguments)
            .ok_or("invalid aggregate arguments")?;
        let Some(definition) = self.get(item) else {
            return Ok(ShapeReadiness::Pending);
        };
        if types.region_args(arguments).is_none()
            && self.instances.as_ref().is_none_or(|i| !i.contains(ty))
        {
            return Ok(ShapeReadiness::Pending);
        }
        if types.contains_parameters(ty) == Some(false)
            && self.storage.as_ref().is_none_or(|s| s.get(ty).is_none())
        {
            if self.has_projected_fields(ty) {
                return Ok(ShapeReadiness::Pending);
            }
            // An undemanded region-only shape may still contain a closed
            // associated projection. Inspect typed flags, never diagnostic text.
            for variant in self.variants(definition) {
                for &field in types
                    .list(variant.fields)
                    .ok_or("invalid aggregate fields")?
                {
                    if types
                        .contains_projections(field)
                        .ok_or("invalid aggregate field type")?
                    {
                        return Ok(ShapeReadiness::Pending);
                    }
                }
            }
        }
        self.shape(types, ty).map(ShapeReadiness::Ready)
    }
    /// Index typed field flags as each declaration publishes its field lists.
    /// Signature-only associated items do not require storage graph preparation.
    pub(crate) fn index_field_projections(
        &mut self,
        types: &Store,
        fields: ListId,
    ) -> Result<(), &'static str> {
        let fields = types.list(fields).ok_or("invalid storage field template")?;
        self.projected_templates |= fields
            .iter()
            .any(|&ty| types.contains_projections(ty) == Some(true));
        Ok(())
    }
    pub(crate) fn has_projected_templates(&self) -> bool {
        self.projected_templates
    }

    /// Immediate raw fields after instantiation; this flag carries no proof.
    pub(crate) fn has_projected_fields(&self, ty: TypeId) -> bool {
        self.instances
            .as_ref()
            .is_some_and(|instances| instances.has_projections(ty))
    }
    /// Publish independently normalized concrete fields once. Raw substituted
    /// fields remain the proof roots for every caller, including erased inputs.
    pub(crate) fn complete_storage(
        &mut self,
        types: &Store,
        ty: TypeId,
        fields: Vec<ListId>,
    ) -> Result<(), &'static str> {
        if types.contains_parameters(ty) != Some(false) {
            return Err("normalized storage requires a concrete nominal identity");
        }
        let raw = self.raw_shape(types, ty)?;
        if raw.variants().len() != fields.len() {
            return Err("normalized storage changed variant coverage");
        }
        for (variant, &list) in raw.variants().zip(&fields) {
            let original = types
                .list(variant.fields)
                .ok_or("missing raw storage fields")?;
            let normalized = types
                .list(list)
                .ok_or("missing normalized storage fields")?;
            if original.len() != normalized.len()
                || normalized.iter().any(|&field| {
                    types.contains_parameters(field) != Some(false)
                        || types.contains_projections(field) != Some(false)
                })
            {
                return Err("normalized storage retains symbolic fields or changed their arity");
            }
        }
        let storage = self.storage.get_or_insert_with(Default::default);
        if let Some(existing) = storage.get(ty) {
            return if existing == fields {
                Ok(())
            } else {
                Err("independent storage normalization changed a completed shape")
            };
        }
        storage.retained = storage
            .retained
            .checked_add(
                fields
                    .len()
                    .saturating_mul(size_of::<ListId>())
                    .saturating_add(64),
            )
            .filter(|bytes| *bytes <= 64_000_000)
            .ok_or("normalized storage exceeds 64 MB budget")?;
        storage.entries.insert(ty, fields);
        Ok(())
    }
}
