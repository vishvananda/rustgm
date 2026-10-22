//! Prepare just the storage-tail instances needed to represent a pointer.
#[cfg(test)]
#[path = "../../../tests/invariants/metadata_preparation.rs"]
mod tests;
use super::Definitions;
use crate::support::hash::HashSet;
use crate::types::{ItemId, Kind, Store, TypeId};

/// Physical preparation never treats provisional Sized properties as storage facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataPreparation {
    Ready,
    Fields(ItemId),
    Unresolved(TypeId),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Readiness {
    Ordinary,
    Physical,
}
impl Definitions {
    /// Prefix layouts and pointer referents are not metadata dependencies.
    /// A false result requests caller-owned normalization of a projected tail
    /// or concrete substitution of a rigid parameter; it is never cached.
    /// # Errors
    /// Rejects missing fields, invalid types, cyclic tails and exhausted work.
    pub fn prepare_metadata(
        &mut self,
        types: &mut Store,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        match self.prepare_metadata_tail(types, ty, Readiness::Ordinary)? {
            MetadataPreparation::Ready => Ok(true),
            MetadataPreparation::Unresolved(_) => Ok(false),
            MetadataPreparation::Fields(_) => Err("missing aggregate field template"),
        }
    }
    /// Prepare immutable physical tail facts, yielding exact missing declarations.
    /// # Errors
    /// Rejects invalid types, cyclic tails and exhausted preparation work.
    pub fn prepare_physical_metadata(
        &mut self,
        types: &mut Store,
        ty: TypeId,
    ) -> Result<MetadataPreparation, &'static str> {
        self.prepare_metadata_tail(types, ty, Readiness::Physical)
    }
    fn prepare_metadata_tail(
        &mut self,
        types: &mut Store,
        mut ty: TypeId,
        mode: Readiness,
    ) -> Result<MetadataPreparation, &'static str> {
        let mut seen = HashSet::default();
        loop {
            self.instances
                .get_or_insert_with(Default::default)
                .charge(1)?;
            if (mode == Readiness::Ordinary && types.is_sized(ty) == Some(true))
                || self.metadata_ready.get(&ty).is_some_and(|&ready| {
                    mode == Readiness::Ordinary || ready == Readiness::Physical
                })
            {
                break;
            }
            if !seen.insert(ty) {
                return Err("recursive pointer metadata tail");
            }
            let fields = match types.kind(ty).ok_or("invalid metadata preparation type")? {
                Kind::Adt(item, _) => {
                    if self.get(item).is_none() && mode == Readiness::Physical {
                        return Ok(MetadataPreparation::Fields(item));
                    }
                    self.instantiate(types, ty)?;
                    // Concrete associated fields need their original semantic
                    // proof before the physical query observes a normal form.
                    if self.has_projected_fields(ty)
                        && self.storage.as_ref().is_none_or(|s| s.get(ty).is_none())
                    {
                        return Ok(MetadataPreparation::Unresolved(ty));
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
                Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => {
                    return Ok(MetadataPreparation::Unresolved(ty));
                }
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
        for ty in seen {
            self.metadata_ready
                .entry(ty)
                .and_modify(|ready| {
                    if mode == Readiness::Physical {
                        *ready = mode;
                    }
                })
                .or_insert(mode);
        }
        Ok(MetadataPreparation::Ready)
    }
}
