//! SIMD is a checked representation of the genuine single-array nominal field.
use super::{Definition, Definitions, Representation, ShapeReadiness};
use crate::types::{Kind, ListId, Store, TypeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SimdShape {
    NotSimd,
    Pending,
    Ready { lane: TypeId, count: u64 },
}
fn array(types: &Store, fields: ListId) -> Result<(TypeId, Option<u64>), &'static str> {
    let [field] = types.list(fields).ok_or("missing SIMD fields")? else {
        return Err("SIMD representation requires exactly one array field");
    };
    let (lane, count) = match types.kind(*field) {
        Some(Kind::Array { element, length }) => (element, Some(length)),
        Some(Kind::ArrayTerm { element, .. }) => (element, None),
        _ => return Err("SIMD representation requires an array field"),
    };
    if count.is_some_and(|count| count == 0 || count > 32768) {
        return Err("SIMD lane count must be between 1 and 32768");
    }
    Ok((lane, count))
}
impl Definitions {
    pub(crate) fn validate_simd_declaration(
        &self,
        types: &Store,
        definition: &Definition,
    ) -> Result<(), &'static str> {
        if definition.repr.representation != Representation::Simd {
            return Ok(());
        }
        if definition.kind != super::Kind::Struct {
            return Err("SIMD representation requires a struct");
        }
        let [variant] = self.variants(definition) else {
            return Err("SIMD representation requires one field group");
        };
        let (lane, _) = array(types, variant.fields)?;
        match types.kind(lane) {
            Some(Kind::Parameter(_) | Kind::Pointer { .. }) => Ok(()),
            Some(Kind::Primitive(p)) if p.integer().is_some() || p.is_float() => Ok(()),
            _ => Err("SIMD declaration lane must be an integer, float, raw pointer or parameter"),
        }
    }
    /// Read only actual prepared fields; this query neither substitutes nor grants features.
    pub(crate) fn simd_shape(&self, types: &Store, ty: TypeId) -> Result<SimdShape, &'static str> {
        let Some(kind) = types.kind(ty) else {
            return Err("missing SIMD query type");
        };
        let Kind::Adt(item, _) = kind else {
            return Ok(
                if matches!(
                    kind,
                    Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..)
                ) {
                    SimdShape::Pending
                } else {
                    SimdShape::NotSimd
                },
            );
        };
        let Some(definition) = self.get(item) else {
            return Ok(SimdShape::Pending);
        };
        if definition.repr.representation != Representation::Simd {
            return Ok(SimdShape::NotSimd);
        }
        let ShapeReadiness::Ready(shape) = self.try_shape(types, ty)? else {
            return Ok(SimdShape::Pending);
        };
        let variant = shape.variant(0).ok_or("SIMD has no field group")?;
        let (lane, count) = array(types, variant.fields)?;
        if types.contains_parameters(lane) != Some(false)
            || types.contains_projections(lane) != Some(false)
        {
            return Ok(SimdShape::Pending);
        }
        Ok(match count {
            Some(count) => SimdShape::Ready { lane, count },
            None => SimdShape::Pending,
        })
    }
}
