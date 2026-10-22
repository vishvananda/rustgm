//! Raw reads validate existing symbolic storage without inventing byte reinterpretation.
#[cfg(test)]
#[path = "../../tests/invariants/mutable_refs.rs"]
mod mutable_tests;
#[cfg(test)]
#[path = "../../tests/invariants/reconstructed_refs.rs"]
mod reference_tests;
#[cfg(test)]
#[path = "../../tests/invariants/raw_access.rs"]
mod tests;
use super::{
    Failure, Machine,
    memory::{RawAccess, RawAlignment, Resolved},
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
            self.path(place.address)?.fields_only()?;
            let location = place
                .address
                .location()
                .ok_or("constant pointer has no allocation provenance")?;
            if !Reader::new(self.evaluator.facts()).same_storage_type(
                Reader::new(self.evaluator.facts()).location(location)?.ty,
                place.ty,
            ) {
                return Err(
                    "raw pointer type reinterpretation in constant execution is not implemented"
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub(super) fn footprint(
        &mut self,
        ty: TypeId,
    ) -> Result<crate::sema::values::StorageLayout, Failure> {
        let (layout, work) = self
            .evaluator
            .storage_layout(ty, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        Ok(layout)
    }
    pub(super) fn storage_offset(
        &mut self,
        ty: TypeId,
        path: &[u64],
    ) -> Result<(TypeId, u64), Failure> {
        let (ty, offset, _) = self.storage_coordinates::<false>(ty, path)?;
        Ok((ty, offset))
    }
    pub(super) fn storage_coordinates<const IMAGE: bool>(
        &mut self,
        mut ty: TypeId,
        path: &[u64],
    ) -> Result<(TypeId, u64, Option<super::memory::ImagePath>), Failure> {
        let mut offset = 0u64;
        let mut image = None;
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            let field = Reader::new(self.evaluator.facts()).projection_layout(ty, index)?;
            match field.storage {
                crate::sema::values::ProjectionStorage::Product => {}
                crate::sema::values::ProjectionStorage::UnionField { .. } => {
                    if IMAGE && image.is_none() {
                        image = Some(super::memory::ImagePath {
                            owner: ty,
                            prefix: position,
                            offset,
                        });
                    }
                    if field.offset != 0 {
                        return Err("constant union projection has a nonzero offset".into());
                    }
                }
            }
            offset = offset
                .checked_add(field.offset)
                .ok_or("constant storage offset overflow")?;
            ty = field.ty;
        }
        Ok((ty, offset, image))
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
    pub(super) fn shared_raw_reference(
        &mut self,
        place: Resolved,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        self.raw_reference(place, false)
    }
    pub(super) fn mutable_raw_reference(
        &mut self,
        place: Resolved,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        self.raw_reference(place, true)
    }
    /// The initial reference profile requires completed, fully initialized storage.
    fn raw_reference(
        &mut self,
        place: Resolved,
        mutable: bool,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        if place.raw.is_none() {
            return Err("constant raw shared reference lacks its access anchor".into());
        }
        self.checked_reference(place, mutable)
    }
    pub(super) fn union_reference(
        &mut self,
        place: Resolved,
        mutable: bool,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        let field = place
            .union
            .ok_or("constant union reference lacks a field capability")?;
        let (address, work) = self.evaluator.union_address(
            place.address,
            field.owner,
            field.field,
            self.limits.steps.saturating_sub(self.work.steps),
        )?;
        self.charge(work)?;
        self.checked_reference(
            Resolved {
                address,
                union: None,
                ..place
            },
            mutable,
        )
    }
    fn checked_reference(
        &mut self,
        place: Resolved,
        mutable: bool,
    ) -> Result<Option<crate::sema::values::Constant>, Failure> {
        let (root, path) = self.path(place.address)?.fields_only()?;
        let location = Reader::new(self.evaluator.facts()).location(root)?;
        if mutable {
            if matches!(location.origin, crate::sema::values::Origin::Static(_)) {
                return Err(
                    "constant mutable raw references to static storage are not implemented".into(),
                );
            }
            if place.address.write_permission()
                != Some(crate::sema::values::WritePermission::Writable)
            {
                return Err("constant mutable raw references from shared-derived pointers are not implemented".into());
            }
        }
        if let crate::sema::values::Origin::Static(id) = location.origin
            && self.check_static_read(id).is_err()
        {
            // Reference formation does not imply a contents read. Those legal
            // static-reference profiles need a separate deferred validity path.
            return Err("constant raw shared references to mutable or external static storage are not implemented".into());
        }
        if let crate::sema::values::Origin::Static(id) = location.origin
            && self.active_initializers.contains(&id)
        {
            return Err(
                "constant raw shared references to cyclic static storage are not implemented"
                    .into(),
            );
        }
        let root_type = location.ty;
        let root_layout = self.footprint(root_type)?;
        let (ty, offset) = self.storage_offset(root_type, &path)?;
        if !Reader::new(self.evaluator.facts()).same_storage_type(ty, place.ty) {
            return Err(
                "raw pointer type reinterpretation in constant execution is not implemented".into(),
            );
        }
        let leaf = self.footprint(ty)?;
        if offset
            .checked_add(leaf.size)
            .is_none_or(|end| end > root_layout.size)
        {
            return Err("constant reference is outside allocation bounds".into());
        }
        // A packed raw base may permit a by-value field read. Creating a
        // reference additionally requires the actual selected leaf's alignment.
        if root_layout.align < leaf.align || !offset.is_multiple_of(u64::from(leaf.align)) {
            return Err(if mutable {
                "constant raw mutable reference is unaligned"
            } else {
                "constant raw shared reference is unaligned"
            }
            .into());
        }
        if self
            .read_for(place, super::static_reads::ReadPurpose::ReferenceValidation)?
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(if mutable {
            place.address
        } else {
            place.address.restrict_shared()
        }))
    }
    pub(super) fn check_raw_write(&mut self, place: Resolved) -> Result<(), Failure> {
        let access = place
            .raw
            .ok_or("constant raw write lacks its access anchor")?;
        let (root, path) = self.path(place.address)?.fields_only()?;
        self.check_raw_write_authority(place.address, root)?;
        self.check_raw_access(
            Resolved {
                ty: place.union.map_or(place.ty, |union| union.owner),
                union: None,
                ..place
            },
            access,
            root,
            &path,
            true,
        )
    }
    pub(super) fn check_raw_write_authority(
        &mut self,
        address: crate::sema::values::Constant,
        root: LocationId,
    ) -> Result<(), Failure> {
        let location = Reader::new(self.evaluator.facts()).location(root)?;
        if matches!(location.origin, crate::sema::values::Origin::Static(_)) {
            return Err("constant execution cannot write static storage".into());
        }
        if address.write_permission() != Some(crate::sema::values::WritePermission::Writable) {
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
        Ok(())
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
        if !Reader::new(self.evaluator.facts()).same_storage_type(ty, place.ty) {
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
        let (base, path) = self
            .path(crate::sema::values::Constant::pointer(location))?
            .fields_only()?;
        if base != root {
            return Err("constant raw target differs from selected allocation".into());
        }
        let (ty, offset) = self.storage_offset(root_ty, &path)?;
        if !Reader::new(self.evaluator.facts()).same_storage_type(ty, access.ty) {
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
        if access.alignment == RawAlignment::Required
            && (root_layout.align < target.align || !offset.is_multiple_of(u64::from(target.align)))
        {
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
