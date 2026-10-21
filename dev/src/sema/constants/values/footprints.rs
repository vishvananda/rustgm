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
pub(crate) struct ProjectionLayout {
    pub ty: TypeId,
    pub offset: u64,
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
        if !layout.inhabited {
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
                    offset: stride
                        .checked_mul(index)
                        .ok_or("constant storage offset overflow")?,
                })
            }
            Shape::Aggregate { .. } => {
                if matches!(self.facts.types.kind(ty), Some(Kind::Adt(item, _)) if self.facts.aggregates.get(item).is_none_or(|d| d.kind != crate::types::aggregate::Kind::Struct))
                {
                    return Err("raw constant enum or union field access is not implemented");
                }
                let [variant] = self.facts.layouts.variants(layout) else {
                    return Err("constant storage requires one product variant");
                };
                let index =
                    usize::try_from(index).map_err(|_| "constant field index exceeds host size")?;
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
                Ok(ProjectionLayout { ty, offset })
            }
            _ => Err("constant projection requires aggregate storage"),
        }
    }
}
