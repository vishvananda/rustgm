//! Prepare just the storage-tail instances needed to represent a pointer.
use super::Definitions;
use crate::types::{Kind, Store, TypeId};
use std::collections::HashSet;

impl Definitions {
    /// Prefix layouts and pointer referents are not metadata dependencies.
    /// A false result requests caller-owned normalization of a projected tail
    /// or concrete substitution of a rigid parameter; it is never cached.
    /// # Errors
    /// Rejects missing fields, invalid types, cyclic tails and exhausted work.
    pub fn prepare_metadata(
        &mut self,
        types: &mut Store,
        mut ty: TypeId,
    ) -> Result<bool, &'static str> {
        let mut seen = HashSet::new();
        loop {
            self.instances
                .get_or_insert_with(Default::default)
                .charge(1)?;
            if types.is_sized(ty) == Some(true) || self.metadata_ready.contains(&ty) {
                break;
            }
            if !seen.insert(ty) {
                return Err("recursive pointer metadata tail");
            }
            let fields = match types.kind(ty).ok_or("invalid metadata preparation type")? {
                Kind::Adt(..) => {
                    self.instantiate(types, ty)?;
                    // Concrete associated fields need their original semantic
                    // proof before the physical query observes a normal form.
                    if self.has_projected_fields(ty)
                        && self.storage.as_ref().is_none_or(|s| s.get(ty).is_none())
                    {
                        return Ok(false);
                    }
                    let shape = self.shape(types, ty)?;
                    if shape.definition.kind == super::Kind::Enum {
                        break;
                    }
                    shape
                        .variants()
                        .next()
                        .ok_or("missing metadata field group")?
                        .fields
                }
                Kind::Tuple(fields) => fields,
                Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => return Ok(false),
                _ => break,
            };
            let Some(&tail) = types
                .list(fields)
                .ok_or("missing metadata tail fields")?
                .last()
            else {
                break;
            };
            ty = tail;
        }
        self.metadata_ready.extend(seen);
        Ok(true)
    }
}
