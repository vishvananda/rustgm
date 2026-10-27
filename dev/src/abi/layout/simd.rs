//! Vector alignment keeps the existing nominal array field and object offsets.
use super::{FieldStorage, Layout, Layouts, MAX_SIZE, Range, Shape, index, niches::Budget, padded};
use crate::types::{
    Store, TypeId,
    aggregate::{self, Definitions, SimdShape},
};

impl Layouts {
    fn simd_lane(
        &mut self,
        mut ty: TypeId,
        types: &Store,
        definitions: &Definitions,
        budget: &mut Budget,
    ) -> Result<Layout, &'static str> {
        loop {
            budget.charge(&mut self.steps, 1)?;
            let layout = *self.get(ty).ok_or("missing SIMD lane layout")?;
            if matches!(layout.shape, Shape::Scalar) && layout.size != 0 {
                return Ok(layout);
            }
            match types.kind(ty) {
                Some(crate::types::Kind::Tuple(_)) => {}
                Some(crate::types::Kind::Adt(item, _)) => {
                    let definition = definitions
                        .get(item)
                        .ok_or("missing SIMD lane declaration")?;
                    if definition.kind != aggregate::Kind::Struct
                        || !matches!(
                            definition.repr.representation,
                            aggregate::Representation::Rust
                                | aggregate::Representation::Transparent
                        )
                    {
                        return Err("SIMD lane does not have a primitive scalar representation");
                    }
                }
                _ => return Err("SIMD lane does not have a primitive scalar representation"),
            }
            let [variant] = self.variants(&layout) else {
                return Err("SIMD lane does not have a primitive scalar representation");
            };
            let variant = *variant;
            let fields = types
                .list(variant.fields)
                .ok_or("missing SIMD lane fields")?;
            let mut principal = None;
            for (index, &field) in fields.iter().enumerate() {
                budget.charge(&mut self.steps, 1)?;
                let offset = *self
                    .offsets(&variant)
                    .get(index)
                    .ok_or("missing SIMD lane field offset")?;
                let child = self.get(field).ok_or("missing SIMD lane field layout")?;
                if child.size == 0 {
                    continue;
                }
                if offset != 0
                    || child.size != layout.size
                    || child.align != layout.align
                    || principal.replace(field).is_some()
                {
                    return Err("SIMD lane does not have a primitive scalar representation");
                }
            }
            ty = principal.ok_or("SIMD lane does not have a primitive scalar representation")?;
        }
    }
    pub(super) fn simd_layout(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        shape: aggregate::Shape<'_>,
        budget: &mut Budget,
    ) -> Result<Layout, &'static str> {
        let SimdShape::Ready { lane, count } = definitions.simd_shape(types, ty)? else {
            return Err("SIMD layout requires completed array fields");
        };
        let lane = self.simd_lane(lane, types, definitions, budget)?;
        let bytes = lane
            .size
            .checked_mul(count)
            .filter(|&size| size <= MAX_SIZE)
            .ok_or("SIMD size exceeds target object limit")?;
        let repr = shape.definition.repr;
        let align = if repr.packed.is_some() {
            1_u64 << bytes.trailing_zeros()
        } else {
            bytes
                .checked_next_power_of_two()
                .ok_or("SIMD alignment overflow")?
        };
        let align = u32::try_from(align).map_err(|_| "SIMD alignment exceeds target limit")?;
        let variant = shape.variant(0).ok_or("SIMD has no field group")?;
        let start = index(self.variants.len())?;
        let layout = self.fields(
            types,
            variant.fields,
            aggregate::Repr::default(),
            FieldStorage::Product {
                base: 0,
                discriminant: variant.discriminant,
            },
            definitions.packing_checked(),
        )?;
        Ok(Layout {
            size: padded(bytes, align)?,
            align,
            contains_explicit_align: repr.align.is_some(),
            shape: Shape::Aggregate {
                variants: Range {
                    start,
                    end: index(self.variants.len())?,
                },
                tag: None,
            },
            ..layout
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/simd_representation.rs"]
mod tests;
