//! Shared typed storage loading preserves static readiness and evaluation context.
use super::{Constant, Failure, LocationId, Machine, Origin, Range, Reader, TypeId};
use crate::const_eval::static_reads::ReadPurpose;
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn read_storage(
        &mut self,
        root: LocationId,
        path: &[u64],
        range: Option<Range>,
        leaf: TypeId,
        purpose: ReadPurpose,
    ) -> Result<Option<Constant>, Failure> {
        let value = if self.memory.roots.contains_key(&root) {
            if let Some(range) = range {
                self.read_range(root, path, range)?
            } else {
                self.read_cell(root, path)?
            }
        } else {
            let location = Reader::new(self.evaluator.facts()).location(root)?;
            let mut value = if let Origin::Static(id) = location.origin {
                self.check_static_read(id)?;
                self.check_static_value_read(location.ty, leaf, purpose)?;
                let Some(value) = Reader::new(self.evaluator.facts()).definition(id) else {
                    self.waiting = Some(crate::const_eval::Requirement::InitializerBody(id));
                    return Ok(None);
                };
                value
            } else {
                location
                    .value
                    .ok_or("constant allocation has no completed contents")?
            };
            for &index in path {
                self.charge(1)?;
                value = self.evaluator.field(value, index)?;
            }
            if let Some(range) = range {
                self.charge(1)?;
                self.evaluator
                    .sequence_range(value, range.from, range.length)?
            } else {
                value
            }
        };
        Ok(Some(value))
    }
}
