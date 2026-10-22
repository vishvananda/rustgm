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
            let length = match shape {
                StoredShape::Array { length: 0, .. }
                | StoredShape::Scalar
                | StoredShape::Word
                | StoredShape::Enumeration { .. }
                | StoredShape::Unsupported => 0,
                StoredShape::Array { .. } => 1,
                StoredShape::Product { fields, .. } => fields.len(),
            };
            let unsupported = unsupported || (enumeration && !self.prepare_byte_enum(ty)?);
            self.reserve_cells(
                pending
                    .len()
                    .saturating_add(length)
                    .saturating_add(1)
                    .saturating_mul(4),
            )?;
            self.charge(length)?;
            let children = match Reader::new(self.evaluator.facts()).storage_shape(ty)? {
                StoredShape::Array { length: 0, .. }
                | StoredShape::Scalar
                | StoredShape::Word
                | StoredShape::Enumeration { .. }
                | StoredShape::Unsupported => Vec::new(),
                StoredShape::Array { element, .. } => vec![element],
                StoredShape::Product { fields, .. } => fields.to_vec(),
            };
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
}
