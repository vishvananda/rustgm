//! A union ordinal selects aliased Image bytes, never an independent product cell.
use super::{Failure, Machine, Resolved, UnionField};
use crate::{
    sema::values::{Origin, ProjectionStorage, Reader},
    types::{Kind, aggregate},
};
#[derive(Clone, Copy)]
pub(in crate::const_eval) enum StorageTarget {
    Product,
    UnionScalar { field: UnionField, projected: bool },
}
impl StorageTarget {
    pub(in crate::const_eval) fn path_len(self, total: usize) -> Result<usize, Failure> {
        if matches!(
            self,
            Self::UnionScalar {
                projected: true,
                ..
            }
        ) {
            total
                .checked_sub(1)
                .ok_or("constant union field address lacks its projection".into())
        } else {
            Ok(total)
        }
    }
    pub(in crate::const_eval) const fn field(self) -> Option<UnionField> {
        match self {
            Self::Product => None,
            Self::UnionScalar { field, .. } => Some(field),
        }
    }
}
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn storage_target(
        &self,
        place: Resolved,
    ) -> Result<StorageTarget, Failure> {
        if let Some(field) = place.union {
            return Ok(StorageTarget::UnionScalar {
                field,
                projected: false,
            });
        }
        let reader = Reader::new(self.evaluator.facts());
        let location = reader.location(
            place
                .address
                .location()
                .ok_or("constant pointer lacks allocation provenance")?,
        )?;
        let Origin::Projection { base, index } = location.origin else {
            return Ok(StorageTarget::Product);
        };
        let owner = reader.location(base)?.ty;
        let facts = self.evaluator.facts();
        if !matches!(facts.types.kind(owner), Some(Kind::Adt(item, _)) if facts.aggregates.get(item).is_some_and(|d| d.kind == aggregate::Kind::Union))
        {
            return Ok(StorageTarget::Product);
        }
        let projection = reader.projection_layout(owner, index)?;
        if projection.ty != location.ty {
            return Err("constant union projection differs from its declared field type".into());
        }
        match projection.storage {
            ProjectionStorage::UnionScalar { field } => Ok(StorageTarget::UnionScalar {
                field: UnionField { owner, field },
                projected: true,
            }),
            ProjectionStorage::Product => {
                Err("constant union projection lacks its scalar image capability".into())
            }
        }
    }
}
