//! Whether all-zero bytes are a value of a type whose answer follows from
//! its fields alone, as rustc's default zero-initialization check decides it.
use super::{Definitions, Kind as Aggregate};
use crate::types::{Kind, Primitive, Store, TypeId};

impl Definitions {
    /// Numbers, booleans, characters, arrays, tuples and structs of them hold;
    /// a struct holds when each of its fields does. A raw pointer holds when
    /// `thin` says its referent needs no metadata. Every other type, an enum
    /// or a union among them, is left undecided (`false`).
    pub(crate) fn zero_valid(
        &self,
        types: &Store,
        ty: TypeId,
        mut thin: impl FnMut(TypeId) -> Result<bool, &'static str>,
    ) -> Result<bool, &'static str> {
        let mut pending = vec![ty];
        let mut visited = 0_u32;
        while let Some(ty) = pending.pop() {
            visited += 1;
            let fields = match types.kind(ty) {
                Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => continue,
                Some(Kind::Pointer { pointee, .. }) if thin(pointee)? => continue,
                Some(Kind::Array { element, .. }) => {
                    pending.push(element);
                    continue;
                }
                Some(Kind::Tuple(fields)) if visited < 1024 => fields,
                Some(Kind::Adt(..)) if visited < 1024 => {
                    let shape = self.shape(types, ty)?;
                    if !matches!(
                        shape.definition.kind,
                        Aggregate::Struct | Aggregate::PhantomData
                    ) {
                        return Ok(false);
                    }
                    shape
                        .variant(0)
                        .ok_or("asserted struct has no field group")?
                        .fields
                }
                _ => return Ok(false),
            };
            pending.extend_from_slice(
                types
                    .list(fields)
                    .ok_or("asserted aggregate type lost its fields")?,
            );
        }
        Ok(true)
    }
}
