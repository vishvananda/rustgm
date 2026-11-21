//! Whether all-zero bytes are a value of a type whose answer follows from
//! its fields alone, as rustc's default zero-initialization check decides it.
use super::{Definitions, Kind as Aggregate};
use crate::types::{Kind, ListId, Primitive, Store, TypeId};

pub(crate) enum ZeroFields {
    Valid,
    Visit(ListId),
    Unproven,
}

impl Definitions {
    /// Walk the structural parts of a zero value. Raw pointers and the
    /// selected fields of enums require facts from the caller's physical view.
    pub(crate) fn zero_valid(
        &self,
        types: &Store,
        ty: TypeId,
        mut physical: impl FnMut(TypeId) -> Result<ZeroFields, &'static str>,
    ) -> Result<bool, &'static str> {
        let mut pending = vec![ty];
        let mut visited = 0_u32;
        while let Some(ty) = pending.pop() {
            visited += 1;
            let fields = match types.kind(ty) {
                Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => continue,
                Some(Kind::Pointer { .. }) => physical(ty)?,
                Some(Kind::Array { element, .. }) => {
                    pending.push(element);
                    continue;
                }
                Some(Kind::Tuple(fields)) if visited < 1024 => ZeroFields::Visit(fields),
                Some(Kind::Adt(..)) if visited < 1024 => {
                    let shape = self.shape(types, ty)?;
                    match shape.definition.kind {
                        Aggregate::Struct | Aggregate::PhantomData => ZeroFields::Visit(
                            shape
                                .variant(0)
                                .ok_or("asserted struct has no field group")?
                                .fields,
                        ),
                        Aggregate::Enum => physical(ty)?,
                        // rustc's default check never looks inside a union:
                        // all-zero bytes are a value of every union.
                        Aggregate::Union => ZeroFields::Valid,
                    }
                }
                _ => return Ok(false),
            };
            let fields = match fields {
                ZeroFields::Valid => continue,
                ZeroFields::Visit(fields) => fields,
                ZeroFields::Unproven => return Ok(false),
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
