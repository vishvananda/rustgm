//! Validate the selected referent through typed array and record address paths.
use super::super::{StaticRef, Storage, StoredProjection};
use crate::support::crates::{Lifetime, Nominals, TypeNode, TypeRef};

impl Storage {
    pub(super) fn validate_projection(
        &self,
        ty: TypeRef,
        target: StaticRef,
        path: &[StoredProjection],
        types: &Nominals,
    ) -> Result<(), &'static str> {
        let Some(TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent,
        }) = types.types().node(ty)
        else {
            return Err("metadata projected reference requires a static shared type");
        };
        if !path
            .iter()
            .any(|p| matches!(p, StoredProjection::Element(_)))
        {
            return Err("metadata element path requires an array projection");
        }
        let mut selected = self
            .statics
            .get(target.0 as usize)
            .ok_or("unknown metadata projected static target")?
            .ty();
        for step in path {
            selected = match (*step, types.types().node(selected)) {
                (StoredProjection::Field(index), Some(TypeNode::Record(record))) => types.records()
                    [usize::from(record.0)]
                .fields()
                .get(index as usize)
                .ok_or("metadata field path exceeds its record")?
                .ty(),
                (StoredProjection::Element(index), Some(TypeNode::Array { element, length }))
                    if index < length =>
                {
                    element
                }
                _ => return Err("metadata projection does not match its storage type"),
            };
        }
        if selected != referent {
            return Err("metadata projected reference has a different referent");
        }
        Ok(())
    }
}
