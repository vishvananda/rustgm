//! Check zero initialization in the native layout, before executing unsafe source.
use super::{Budget, Layouts, Shape};
use crate::{
    abi::metadata::Metadata,
    types::{Constraint, Kind, Store, TypeId, aggregate::Definitions},
};

impl Layouts {
    pub(crate) fn zero_initialization(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        root: TypeId,
    ) -> Result<bool, &'static str> {
        let mut work = vec![root];
        let mut budget = Budget::new(1_000_000);
        while let Some(ty) = work.pop() {
            budget.charge(&mut self.steps, 1)?;
            let layout = *self
                .get(ty)
                .ok_or("zero initialization needs its physical layout")?;
            if !layout.inhabited {
                return Ok(false);
            }
            let kind = types
                .kind(ty)
                .ok_or("zero initialization has an unknown type")?;
            match kind {
                // The pinned native check accepts inhabited arrays without
                // looking inside their storage. A direct intrinsic call does
                // not read any element or make invalid bytes a valid value.
                Kind::Primitive(_) | Kind::Array { .. } | Kind::FunctionItem { .. } => {}
                Kind::Never | Kind::Reference { .. } | Kind::Function { .. } => return Ok(false),
                Kind::Pointer { pointee, .. } => {
                    match self.cached_pointer_metadata(types, pointee).transpose()? {
                        Some(Metadata::Thin | Metadata::Length) => {}
                        Some(Metadata::VTable(_)) => return Ok(false),
                        _ => return Err("zero initialization requires known raw pointer metadata"),
                    }
                }
                Kind::Pattern { base, constraint } => {
                    if !includes_zero(types, constraint) {
                        return Ok(false);
                    }
                    work.push(base);
                }
                Kind::Opaque(..) => work.push(types.storage_head(ty)?),
                Kind::Closure { environment, .. } => work.push(environment),
                Kind::Tuple(fields) => {
                    append_fields(types, &mut budget, &mut self.steps, &mut work, fields)?;
                }
                Kind::Adt(..) => {
                    if let Some(fields) =
                        self.zero_initialization_fields(types, definitions, ty, &mut budget)?
                    {
                        append_fields(types, &mut budget, &mut self.steps, &mut work, fields)?;
                    } else {
                        return Ok(false);
                    }
                }
                _ => return Err("zero initialization requires a supported concrete layout"),
            }
        }
        Ok(true)
    }

    fn zero_initialization_fields(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        budget: &mut Budget,
    ) -> Result<Option<crate::types::ListId>, &'static str> {
        use crate::types::aggregate::Kind as Aggregate;
        let shape = definitions.shape(types, ty)?;
        match shape.definition.kind {
            Aggregate::Union => Ok(Some(crate::types::ListId(0))),
            Aggregate::Struct | Aggregate::PhantomData => shape
                .variant(0)
                .map(|variant| Some(variant.fields))
                .ok_or("zero initialization lost its physical record fields"),
            Aggregate::Enum => {
                let layout = self
                    .get(ty)
                    .ok_or("zero initialization lost its enum layout")?;
                if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
                    // A directly represented multivariant enum checks its
                    // scalar tag range. Its payload is not part of that range.
                    let (signed, bits) = tag.ty.integer().ok_or("invalid enum zero tag type")?;
                    let sign = 1_u128 << (bits - 1);
                    let layout = *layout;
                    let len = self.variants(&layout).len();
                    budget.charge(&mut self.steps, len)?;
                    let variants = self.variants(&layout);
                    let (mut negative, mut nonnegative) = (false, false);
                    for variant in variants {
                        let value = variant.discriminant;
                        if value == 0 {
                            return Ok(Some(crate::types::ListId(0)));
                        }
                        negative |= signed && value & sign != 0;
                        nonnegative |= !signed || value & sign == 0;
                    }
                    return Ok((negative && nonnegative).then_some(crate::types::ListId(0)));
                }
                self.zero_enum_fields(types, ty)
            }
        }
    }
}

fn includes_zero(types: &Store, constraint: Constraint) -> bool {
    match constraint {
        Constraint::NonNull => false,
        Constraint::ScalarRange(id) => types.range_allows(id, 0),
    }
}

fn append_fields(
    types: &Store,
    budget: &mut Budget,
    steps: &mut usize,
    work: &mut Vec<TypeId>,
    fields: crate::types::ListId,
) -> Result<(), &'static str> {
    let fields = types
        .list(fields)
        .ok_or("zero initialization lost its fields")?;
    budget.charge(steps, fields.len())?;
    work.extend_from_slice(fields);
    Ok(())
}
