//! Closed physical capabilities are memoized only after complete layout publication.
use super::{Failure, Machine, Reader, TypeId};
use crate::sema::values::StoredShape;
impl Machine<'_, '_> {
    pub(super) fn byte_family(&mut self, root: TypeId) -> Result<bool, Failure> {
        let mut pending = vec![(root, false)];
        while let Some((ty, finish)) = pending.pop() {
            self.charge(1)?;
            if self.memory.bytes.families.contains_key(&ty) {
                continue;
            }
            self.footprint(ty)?;
            let shape = Reader::new(self.evaluator.facts()).storage_shape(ty)?;
            let unsupported = matches!(shape, StoredShape::Unsupported);
            let enumeration = matches!(shape, StoredShape::Enumeration { .. });
            let unsupported = unsupported || (enumeration && !self.prepare_byte_enum(ty)?);
            let children = self.byte_children(ty)?;
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
        let shape = Reader::new(self.evaluator.facts()).storage_shape(ty)?;
        if matches!(shape, StoredShape::Enumeration { .. }) {
            let count = Reader::new(self.evaluator.facts())
                .enum_variants(ty)?
                .identities
                .len();
            self.charge(count.saturating_mul(2))?;
        }
        let length = match Reader::new(self.evaluator.facts()).storage_shape(ty)? {
            StoredShape::Array { length: 0, .. }
            | StoredShape::Scalar
            | StoredShape::Word
            | StoredShape::Unsupported => 0,
            StoredShape::Array { .. } => 1,
            StoredShape::Product { fields, .. } => fields.len(),
            StoredShape::Enumeration { .. } => {
                let reader = Reader::new(self.evaluator.facts());
                let stored = reader.enum_variants(ty)?;
                stored.identities.iter().try_fold(0usize, |count, &id| {
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
            StoredShape::Enumeration { .. } => {
                for &id in reader.enum_variants(ty)?.identities {
                    children.extend_from_slice(reader.enum_variant_fields(ty, id)?.0);
                }
            }
        }
        Ok(children)
    }
}
