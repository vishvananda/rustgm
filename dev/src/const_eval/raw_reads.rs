//! Raw reads validate existing symbolic storage without inventing byte reinterpretation.
#[cfg(test)]
#[path = "../../tests/invariants/raw_access.rs"]
mod tests;
use super::{
    Failure, Machine,
    memory::{RawAccess, Resolved},
};
use crate::{
    resolve::model::DefId,
    sema::values::{LocationId, Reader},
    types::TypeId,
};
impl Machine<'_, '_> {
    pub(super) fn initialize_storage(
        &mut self,
        requirement: super::Requirement,
    ) -> Result<bool, Failure> {
        if let super::Requirement::InitializerBody(definition) = requirement
            && let Some(&target) = self.bodies.definitions.get(&definition)
        {
            self.push(target, &[], None)?;
            return Ok(true);
        }
        Ok(false)
    }
    pub(super) fn check_static_read(&self, id: DefId) -> Result<(), Failure> {
        let facts = self.evaluator.facts();
        if facts
            .foreign
            .as_deref()
            .is_some_and(|r| r.imports_global(id))
        {
            return Err("constant execution cannot read external static contents".into());
        }
        if facts
            .mutable_statics
            .as_ref()
            .is_some_and(|ids| ids.contains(&id))
        {
            return Err("constant execution cannot read mutable static storage".into());
        }
        Ok(())
    }
    pub(super) fn project_raw_type(&mut self, place: Resolved) -> Result<(), Failure> {
        if place.raw.is_some() {
            self.path(place.address)?;
            let location = place
                .address
                .location()
                .ok_or("constant pointer has no allocation provenance")?;
            if Reader::new(self.evaluator.facts()).location(location)?.ty != place.ty {
                return Err(
                    "raw pointer type reinterpretation in constant execution is not implemented"
                        .into(),
                );
            }
        }
        Ok(())
    }
    fn footprint(&mut self, ty: TypeId) -> Result<crate::sema::values::StorageLayout, Failure> {
        let (layout, work) = self
            .evaluator
            .storage_layout(ty, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        Ok(layout)
    }
    fn storage_offset(&mut self, mut ty: TypeId, path: &[u64]) -> Result<(TypeId, u64), Failure> {
        let mut offset = 0u64;
        for &index in path {
            self.charge(1)?;
            let field = Reader::new(self.evaluator.facts()).projection_layout(ty, index)?;
            offset = offset
                .checked_add(field.offset)
                .ok_or("constant storage offset overflow")?;
            ty = field.ty;
        }
        Ok((ty, offset))
    }
    pub(super) fn check_raw_read(
        &mut self,
        place: Resolved,
        access: RawAccess,
        root: LocationId,
        path: &[u64],
    ) -> Result<(), Failure> {
        self.check_raw_access(place, access, root, path, false)
    }
    pub(super) fn check_raw_write(&mut self, place: Resolved) -> Result<(), Failure> {
        let access = place
            .raw
            .ok_or("constant raw write lacks its access anchor")?;
        let (root, path) = self.path(place.address)?;
        let location = Reader::new(self.evaluator.facts()).location(root)?;
        if matches!(location.origin, crate::sema::values::Origin::Static(_)) {
            return Err("constant execution cannot write static storage".into());
        }
        if place.address.write_permission() != Some(crate::sema::values::WritePermission::Writable)
        {
            let (answer, work) = {
                let facts = self.evaluator.facts();
                let mut query =
                    crate::types::interior_mutability::Query::new(&facts.types, &facts.aggregates);
                let answer = query.contains(location.ty)?;
                (answer, query.work().steps)
            };
            self.charge(work)?;
            return Err(match answer {
                crate::types::interior_mutability::Answer::Ordinary => {
                    "constant raw pointer write is derived from a shared reference"
                }
                crate::types::interior_mutability::Answer::Interior => {
                    "constant shared-derived interior-mutable raw writes are not implemented"
                }
                crate::types::interior_mutability::Answer::Unresolved => {
                    "constant raw write requires completed interior-mutability facts"
                }
            }
            .into());
        }
        if place.union.is_some() {
            return Err("constant raw union field writes are not implemented".into());
        }
        self.check_raw_access(place, access, root, &path, true)
    }
    fn check_raw_access(
        &mut self,
        place: Resolved,
        access: RawAccess,
        root: LocationId,
        path: &[u64],
        write: bool,
    ) -> Result<(), Failure> {
        let root_ty = Reader::new(self.evaluator.facts()).location(root)?.ty;
        let root_layout = self.footprint(root_ty)?;
        let (ty, offset) = self.storage_offset(root_ty, path)?;
        if ty != place.ty {
            return Err(
                "raw pointer type reinterpretation in constant execution is not implemented".into(),
            );
        }
        let target = self.footprint(ty)?;
        if offset
            .checked_add(target.size)
            .is_none_or(|end| end > root_layout.size)
        {
            return Err(if write {
                "constant write is outside allocation bounds"
            } else {
                "constant read is outside allocation bounds"
            }
            .into());
        }
        let location = access
            .location
            .ok_or("constant pointer has no allocation provenance")?;
        let (base, path) = self.path(crate::sema::values::Constant::pointer(location))?;
        if base != root {
            return Err("constant raw target differs from selected allocation".into());
        }
        let (ty, offset) = self.storage_offset(root_ty, &path)?;
        if ty != access.ty {
            return Err(
                "raw pointer type reinterpretation in constant execution is not implemented".into(),
            );
        }
        let target = self.footprint(ty)?;
        if offset
            .checked_add(target.size)
            .is_none_or(|end| end > root_layout.size)
        {
            return Err("constant raw target is outside allocation bounds".into());
        }
        if root_layout.align < target.align || !offset.is_multiple_of(u64::from(target.align)) {
            return Err(if write {
                "constant raw pointer write is unaligned"
            } else {
                "constant raw pointer read is unaligned"
            }
            .into());
        }
        Ok(())
    }
}
