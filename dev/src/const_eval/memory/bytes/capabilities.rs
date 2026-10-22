//! Closed physical capabilities are memoized only after complete layout publication.
use super::{Failure, Machine, Reader, TypeId};
use crate::sema::values::StoredShape;
impl Machine<'_, '_> {
    pub(in crate::const_eval::memory) fn byte_family(
        &mut self,
        root: TypeId,
    ) -> Result<bool, Failure> {
        let mut pending = vec![(root, false)];
        while let Some((ty, finish)) = pending.pop() {
            self.charge(1)?;
            if self.memory.bytes.families.contains_key(&ty) {
                continue;
            }
            self.footprint(ty)?;
            let physical = Reader::new(self.evaluator.facts()).storage_type(ty)?;
            let shape = Reader::new(self.evaluator.facts()).storage_shape(physical)?;
            let unsupported = matches!(shape, StoredShape::Unsupported);
            let enumeration = matches!(shape, StoredShape::Enumeration { .. } | StoredShape::Niche);
            let unsupported = unsupported || (enumeration && !self.prepare_byte_enum(physical)?);
            let children = if physical == ty {
                self.byte_children(physical)?
            } else {
                // Distinct opaque applications share only completed physical
                // family validation, never their logical identity or authority.
                self.charge(1)?;
                self.reserve_cells(1)?;
                vec![physical]
            };
            self.reserve_cells(
                pending
                    .len()
                    .saturating_add(children.len())
                    .saturating_add(1)
                    .saturating_mul(4),
            )?;
            if !finish && !children.is_empty() {
                pending.push((ty, true));
                pending.extend(children.into_iter().rev().map(|ty| (ty, false)));
                continue;
            }
            let accepted = !unsupported
                && children
                    .iter()
                    .all(|child| self.memory.bytes.families.get(child) == Some(&true));
            self.reserve_cells(4)?;
            self.memory.bytes.families.insert(ty, accepted);
        }
        self.memory
            .bytes
            .families
            .get(&root)
            .copied()
            .ok_or("missing completed byte family".into())
    }
    fn byte_children(&mut self, ty: TypeId) -> Result<Vec<TypeId>, Failure> {
        let niche = matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Niche
        );
        if matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Enumeration { .. } | StoredShape::Niche
        ) {
            let reader = Reader::new(self.evaluator.facts());
            let count = if niche {
                2
            } else {
                reader.enum_variants(ty)?.identities.len()
            };
            self.charge(count.saturating_mul(2))?;
        }
        let length = match Reader::new(self.evaluator.facts()).storage_shape(ty)? {
            StoredShape::Array { length: 0, .. }
            | StoredShape::Scalar
            | StoredShape::Word
            | StoredShape::Unsupported => 0,
            StoredShape::Array { .. } => 1,
            StoredShape::Product { fields, .. } => fields.len(),
            StoredShape::Enumeration { .. } | StoredShape::Niche => {
                let reader = Reader::new(self.evaluator.facts());
                let ids;
                let identities = if niche {
                    let niche = reader.enum_niche(ty)?;
                    ids = [niche.empty, niche.payload];
                    &ids[..]
                } else {
                    reader.enum_variants(ty)?.identities
                };
                identities.iter().try_fold(0usize, |count, &id| {
                    count
                        .checked_add(reader.enum_variant_fields(ty, id)?.0.len())
                        .ok_or("enum family field count overflow")
                })?
            }
        };
        self.charge(length)?;
        self.reserve_cells(length)?;
        let mut children = Vec::new();
        children
            .try_reserve_exact(length)
            .map_err(|_| "byte family children allocation failed")?;
        let reader = Reader::new(self.evaluator.facts());
        match reader.storage_shape(ty)? {
            StoredShape::Array { length: 0, .. }
            | StoredShape::Scalar
            | StoredShape::Word
            | StoredShape::Unsupported => {}
            StoredShape::Array { element, .. } => children.push(element),
            StoredShape::Product { fields, .. } => children.extend_from_slice(fields),
            StoredShape::Enumeration { .. } | StoredShape::Niche => {
                if niche {
                    children.push(reader.enum_niche(ty)?.encoding.ty);
                } else {
                    for &id in reader.enum_variants(ty)?.identities {
                        children.extend_from_slice(reader.enum_variant_fields(ty, id)?.0);
                    }
                }
            }
        }
        Ok(children)
    }
}
