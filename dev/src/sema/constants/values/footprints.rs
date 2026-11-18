//! Raw access footprints reuse canonical ABI records after bounded readiness checks.
use super::{Evaluator, Failure, Reader};
use crate::{
    abi::layout::{PENDING_STORAGE_FIELDS, STORAGE_WORK_LIMIT, Shape, StorageReadiness},
    types::{Kind, TypeId},
};

#[derive(Clone, Copy)]
pub(crate) struct StorageLayout {
    pub size: u64,
    pub align: u32,
}
#[derive(Clone, Copy)]
pub(crate) enum ProjectionStorage {
    Product,
    UnionField {
        field: crate::sema::members::Selection,
    },
}
#[derive(Clone, Copy)]
pub(crate) struct ProjectionLayout {
    pub ty: TypeId,
    pub offset: u64,
    pub storage: ProjectionStorage,
}
impl Evaluator<'_> {
    fn storage_readiness(&mut self, ty: TypeId, limit: usize) -> Result<usize, Failure> {
        let facts = &mut *self.queries.typing.facts;
        let mut readiness = StorageReadiness::new(ty);
        let mut work = 0;
        let mut prepared = None;
        while let Some(pending) = readiness.advance(
            &facts.layouts,
            &facts.types,
            &facts.aggregates,
            &mut work,
            limit,
        )? {
            let Some(Kind::Adt(owner, _)) = facts.types.kind(pending) else {
                return Err("missing nominal storage identity".into());
            };
            // Instantiation can finish a published field template. A second
            // request for the same instance still needs semantic normalization.
            if prepared == Some(pending) || facts.aggregates.get(owner).is_none() {
                return Err(PENDING_STORAGE_FIELDS.into());
            }
            if work >= limit {
                return Err(STORAGE_WORK_LIMIT.into());
            }
            let before = facts.aggregates.instantiation_steps();
            facts.aggregates.instantiate(&mut facts.types, pending)?;
            work = work
                .checked_add(
                    facts
                        .aggregates
                        .instantiation_steps()
                        .saturating_sub(before),
                )
                .filter(|&cost| cost <= limit)
                .ok_or(STORAGE_WORK_LIMIT)?;
            prepared = Some(pending);
        }
        Ok(work)
    }

    /// Pending definitions are checked before invoking the stable ABI cache.
    /// No failed layout is cached merely because a field has not been published.
    pub(crate) fn storage_layout(
        &mut self,
        ty: TypeId,
        limit: usize,
    ) -> Result<(StorageLayout, usize), Failure> {
        self.storage_footprint::<true>(ty, limit)
    }
    /// Address arithmetic needs layout, without asserting that a value inhabits it.
    pub(crate) fn address_layout(
        &mut self,
        ty: TypeId,
        limit: usize,
    ) -> Result<(StorageLayout, usize), Failure> {
        self.storage_footprint::<false>(ty, limit)
    }
    fn storage_footprint<const INHABITED: bool>(
        &mut self,
        ty: TypeId,
        limit: usize,
    ) -> Result<(StorageLayout, usize), Failure> {
        let mut work = self.storage_readiness(ty, limit)?;
        let facts = &mut *self.queries.typing.facts;
        let before = facts.layouts.steps;
        let id = facts
            .layouts
            .demand_bounded(
                &facts.types,
                &facts.aggregates,
                ty,
                limit.saturating_sub(work),
            )
            .map_err(|error| Failure::Invalid(error.message))?;
        work = work
            .checked_add(facts.layouts.steps - before)
            .filter(|n| *n <= limit)
            .ok_or(STORAGE_WORK_LIMIT)?;
        let layout = facts
            .layouts
            .at(id)
            .ok_or("missing prepared storage layout")?;
        if INHABITED && !layout.inhabited {
            return Err("constant read requires inhabited storage".into());
        }
        Ok((
            StorageLayout {
                size: layout.size,
                align: layout.align,
            },
            work,
        ))
    }
}
impl Reader<'_> {
    /// Follow an ordinal in retained physical storage, never a reconstructed layout.
    pub(crate) fn projection_layout(
        self,
        ty: TypeId,
        index: u64,
    ) -> Result<ProjectionLayout, &'static str> {
        let layout = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing prepared projection layout")?;
        match layout.shape {
            Shape::Array {
                element,
                stride,
                length,
            } => {
                if index >= length {
                    return Err("constant index is out of bounds");
                }
                Ok(ProjectionLayout {
                    ty: element,
                    storage: ProjectionStorage::Product,
                    offset: stride
                        .checked_mul(index)
                        .ok_or("constant storage offset overflow")?,
                })
            }
            Shape::Aggregate { .. } => self.aggregate_projection(ty, index, layout),
            Shape::Niche { .. } => self.niche_projection(ty, index),
            _ => Err("constant projection requires aggregate storage"),
        }
    }
    fn aggregate_projection(
        self,
        ty: TypeId,
        index: u64,
        layout: &crate::abi::layout::Layout,
    ) -> Result<ProjectionLayout, &'static str> {
        let union = match self.facts.types.kind(ty) {
            Some(Kind::Adt(item, _)) => match self.facts.aggregates.get(item).map(|d| d.kind) {
                Some(crate::types::aggregate::Kind::Struct) => None,
                Some(crate::types::aggregate::Kind::Union) => Some(item),
                Some(crate::types::aggregate::Kind::Enum) => {
                    return self.tagless_projection(ty, index);
                }
                _ => {
                    return Err("raw constant enum or union field access is not implemented");
                }
            },
            _ => None,
        };
        let [variant] = self.facts.layouts.variants(layout) else {
            return Err("constant storage requires one product variant");
        };
        let index = usize::try_from(index).map_err(|_| "constant field index exceeds host size")?;
        let owner = ty;
        let ty = self
            .facts
            .types
            .list(variant.fields)
            .and_then(|fields| fields.get(index))
            .copied()
            .ok_or("constant field is out of bounds")?;
        let offset = *self
            .facts
            .layouts
            .offsets(variant)
            .get(index)
            .ok_or("missing prepared field offset")?;
        let storage = if let Some(item) = union {
            if offset != 0
                || !self
                    .facts
                    .constant_images
                    .as_ref()
                    .is_some_and(|arena| arena.families.contains_key(&owner))
            {
                return Err("raw union field access requires a prepared scalar image family");
            }
            let [member] = self.facts.members.variants(item) else {
                return Err("union projection requires one declaration field group");
            };
            let member = self
                .facts
                .members
                .variants
                .get(member.0 as usize)
                .ok_or("missing union projection field group")?;
            let field = u32::try_from(index)
                .ok()
                .and_then(|index| member.start.checked_add(index))
                .filter(|&field| member.owner == item && field < member.end)
                .ok_or("union projection field has a different owner")?;
            ProjectionStorage::UnionField {
                field: crate::sema::members::Selection::Nominal(crate::sema::members::FieldId(
                    field,
                )),
            }
        } else {
            ProjectionStorage::Product
        };
        Ok(ProjectionLayout {
            ty,
            offset,
            storage,
        })
    }
}

/// Completed physical storage facts; unsupported families remain distinct from unreadiness.
#[derive(Clone, Copy)]
pub(crate) enum StoredShape<'a> {
    Scalar,
    Word,
    Enumeration {
        tag: crate::abi::layout::Tag,
    },
    Niche,
    Array {
        element: TypeId,
        stride: u64,
        length: u64,
    },
    Product {
        fields: &'a [TypeId],
        variant: Option<crate::sema::members::VariantId>,
        union: bool,
    },
    Unsupported,
}
impl<'a> Reader<'a> {
    pub(crate) fn storage_shape(self, ty: TypeId) -> Result<StoredShape<'a>, &'static str> {
        let layout = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing prepared storage shape")?;
        match self
            .facts
            .types
            .kind(ty)
            .ok_or("missing stored shape type")?
        {
            Kind::Primitive(p) if p != crate::types::Primitive::Str => {
                if !matches!(layout.shape, Shape::Scalar) {
                    return Err("primitive differs from retained storage shape");
                }
                Ok(StoredShape::Scalar)
            }
            Kind::Pattern { base, .. } => self.storage_shape(base),
            Kind::Reference { .. } | Kind::Pointer { .. } | Kind::Function { .. } => {
                self.word_shape(ty)?;
                Ok(StoredShape::Word)
            }
            Kind::Array { element, length } => {
                let Shape::Array {
                    element: actual,
                    stride,
                    length: actual_length,
                } = layout.shape
                else {
                    return Err("array differs from retained storage shape");
                };
                if actual != element || actual_length != length {
                    return Err("array storage shape differs from its canonical type");
                }
                Ok(StoredShape::Array {
                    element,
                    stride,
                    length,
                })
            }
            Kind::Tuple(_) => self.stored_product(layout, None, false),
            Kind::Closure { environment, .. } => self.stored_closure(environment, layout),
            Kind::Adt(item, _) => {
                let definition = self
                    .facts
                    .aggregates
                    .get(item)
                    .ok_or("missing completed stored nominal")?;
                let union = match definition.kind {
                    crate::types::aggregate::Kind::Struct
                    | crate::types::aggregate::Kind::PhantomData => false,
                    crate::types::aggregate::Kind::Union => true,
                    crate::types::aggregate::Kind::Enum => {
                        if matches!(layout.shape, Shape::Niche { .. }) {
                            self.enum_niche(ty)?;
                            return Ok(StoredShape::Niche);
                        }
                        if definition.repr.tag.is_none()
                            && definition.repr.representation
                                == crate::types::aggregate::Representation::Rust
                            && matches!(layout.shape, Shape::Aggregate { tag: None, .. })
                            && let Some(shape) = self.tagless_enumeration(ty)?
                        {
                            return Ok(shape);
                        }
                        let fallback = self
                            .facts
                            .aggregates
                            .nullable_enum(item)
                            .and_then(|policy| policy.fallback_tag);
                        return Self::enumeration_shape(definition, layout, fallback);
                    }
                };
                let [id] = self.facts.members.variants(item) else {
                    return Err("stored nominal requires one declaration variant");
                };
                let variant = self
                    .facts
                    .members
                    .variants
                    .get(id.0 as usize)
                    .ok_or("missing stored declaration variant")?;
                if variant.owner != item || variant.ordinal != 0 {
                    return Err("stored declaration variant has a different owner");
                }
                self.stored_product(layout, Some(*id), union)
            }
            _ => Ok(StoredShape::Unsupported),
        }
    }
    pub(super) fn stored_product(
        self,
        layout: &crate::abi::layout::Layout,
        variant: Option<crate::sema::members::VariantId>,
        union: bool,
    ) -> Result<StoredShape<'a>, &'static str> {
        if !matches!(layout.shape, Shape::Aggregate { tag: None, .. }) {
            return Err("stored product has an unexpected layout tag");
        }
        let [stored] = self.facts.layouts.variants(layout) else {
            return Err("stored product requires one physical variant");
        };
        let fields = self
            .facts
            .types
            .list(stored.fields)
            .ok_or("missing stored product fields")?;
        if self.facts.layouts.offsets(stored).len() != fields.len() {
            return Err("stored product offsets differ from its fields");
        }
        if let Some(id) = variant {
            let declaration = self
                .facts
                .members
                .variants
                .get(id.0 as usize)
                .ok_or("missing stored variant declaration")?;
            if declaration
                .end
                .checked_sub(declaration.start)
                .and_then(|n| usize::try_from(n).ok())
                != Some(fields.len())
            {
                return Err("stored product differs from declaration field count");
            }
        }
        Ok(StoredShape::Product {
            fields,
            variant,
            union,
        })
    }
}
