//! Raw access footprints reuse canonical ABI records after bounded readiness checks.
use super::{Evaluator, Failure, Reader};
use crate::{
    abi::layout::Shape,
    types::{Kind, TypeId, aggregate::ShapeReadiness},
};
use std::collections::HashSet;

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
    fn storage_readiness(&self, ty: TypeId, limit: usize) -> Result<usize, Failure> {
        if self.facts().layouts.get(ty).is_some() {
            return (limit != 0)
                .then_some(1)
                .ok_or("constant storage layout work budget exceeded".into());
        }
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        let mut work = 0usize;
        while let Some(next) = pending.pop() {
            work = work
                .checked_add(1)
                .filter(|n| *n <= limit)
                .ok_or("constant storage layout work budget exceeded")?;
            let facts = self.facts();
            if facts.layouts.get(next).is_some() || !seen.insert(next) {
                continue;
            }
            let fields = match facts.types.kind(next).ok_or("missing storage type")? {
                Kind::Tuple(fields) => Some(fields),
                Kind::Opaque(..) => {
                    pending.push(facts.types.storage_head(next)?);
                    None
                }
                Kind::Parameter(_) | Kind::Projection(_) | Kind::ArrayTerm { .. } => {
                    return Err("constant storage layout requires a closed normalized type".into());
                }
                Kind::Array { element, .. } => {
                    pending.push(element);
                    None
                }
                Kind::Closure { environment, .. } => {
                    pending.push(environment);
                    None
                }
                Kind::Adt(item, _) => {
                    let ShapeReadiness::Ready(shape) =
                        facts.aggregates.try_shape(&facts.types, next)?
                    else {
                        return Err("constant storage layout requires completed fields".into());
                    };
                    if shape.definition.kind == crate::types::aggregate::Kind::Enum
                        && !facts.aggregates.discriminants_ready(item)
                    {
                        return Err(
                            "constant storage layout requires completed enum discriminants".into(),
                        );
                    }
                    for variant in shape.variants() {
                        let fields = facts
                            .types
                            .list(variant.fields)
                            .ok_or("missing storage fields")?;
                        if work
                            .saturating_add(pending.len())
                            .saturating_add(fields.len())
                            > limit
                        {
                            return Err("constant storage layout work budget exceeded".into());
                        }
                        pending.extend(fields.iter().copied());
                    }
                    None
                }
                _ => None,
            };
            if let Some(fields) = fields {
                let fields = facts.types.list(fields).ok_or("missing storage fields")?;
                if work
                    .saturating_add(pending.len())
                    .saturating_add(fields.len())
                    > limit
                {
                    return Err("constant storage layout work budget exceeded".into());
                }
                pending.extend(fields.iter().copied());
            }
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
            .demand(&facts.types, &facts.aggregates, ty)
            .map_err(|error| Failure::Invalid(error.message))?;
        work = work
            .checked_add(facts.layouts.steps - before)
            .filter(|n| *n <= limit)
            .ok_or("constant storage layout work budget exceeded")?;
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
            Kind::Adt(item, _) => {
                let definition = self
                    .facts
                    .aggregates
                    .get(item)
                    .ok_or("missing completed stored nominal")?;
                let union = match definition.kind {
                    crate::types::aggregate::Kind::Struct => false,
                    crate::types::aggregate::Kind::Union => true,
                    crate::types::aggregate::Kind::Enum => {
                        if definition.repr.tag.is_none()
                            && definition.repr.representation
                                == crate::types::aggregate::Representation::Rust
                            && matches!(layout.shape, Shape::Aggregate { tag: None, .. })
                            && let Some(shape) = self.tagless_enumeration(ty)?
                        {
                            return Ok(shape);
                        }
                        return Self::enumeration_shape(definition, layout);
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
