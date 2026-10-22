//! Cold sequence projections retain the original storage and normalized absolute bounds.
use super::{Id, Place, Pointer, ProjectionId, References, Value};
#[derive(Clone, Copy)]
pub(super) struct Subsequence {
    pub base: Id,
    pub from: u64,
    pub length: u64,
}
#[derive(Clone, Copy)]
pub(in crate::sema::constants) enum ProjectionView {
    Field { base: Id, index: u64 },
    Sequence { base: Id, from: u64, length: u64 },
    Byte { base: Id, offset: u64 },
}
impl References {
    pub(in crate::sema::constants) fn projection(
        &self,
        place: Place,
    ) -> Result<Option<ProjectionView>, &'static str> {
        place
            .projection
            .map(|id| match id {
                ProjectionId::Field(id) => self
                    .projections
                    .get(id as usize)
                    .map(|p| ProjectionView::Field {
                        base: p.base,
                        index: p.index,
                    })
                    .ok_or("missing constant projection"),
                ProjectionId::Sequence(id) => self
                    .ranges
                    .as_ref()
                    .and_then(|r| r.get(id as usize))
                    .map(|p| ProjectionView::Sequence {
                        base: p.base,
                        from: p.from,
                        length: p.length,
                    })
                    .ok_or("missing constant sequence projection"),
                ProjectionId::Byte(id) => self
                    .projections
                    .get(id as usize)
                    .map(|p| ProjectionView::Byte {
                        base: p.base,
                        offset: p.index,
                    })
                    .ok_or("missing constant byte projection"),
            })
            .transpose()
    }
    pub(in crate::sema::constants) fn byte_projection(
        &mut self,
        mut base: Pointer,
        mut offset: u64,
        mut place: Place,
    ) -> Result<Value, &'static str> {
        let parent = self.get(Value::Reference(base))?;
        if let Some(ProjectionView::Byte {
            base: root,
            offset: prior,
        }) = self.projection(parent)?
        {
            offset = offset
                .checked_add(prior)
                .ok_or("constant byte projection offset overflow")?;
            base.place = root;
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        let projection = ProjectionId::Byte(
            u32::try_from(self.projections.len())
                .map_err(|_| "too many constant byte projections")?,
        );
        place.projection = Some(projection);
        self.projections.push(super::Projection {
            base: base.place,
            index: offset,
        });
        self.places.push(place);
        Ok(Value::Reference(Pointer {
            place: id,
            metadata: None,
            write: base.write,
        }))
    }
    pub(in crate::sema::constants) fn subsequence(
        &mut self,
        mut base: Pointer,
        mut from: u64,
        length: u64,
        mut place: Place,
    ) -> Result<Value, &'static str> {
        let original = self.get(Value::Reference(base))?;
        if let Some(ProjectionView::Sequence {
            base: root,
            from: offset,
            length: bound,
        }) = self.projection(original)?
        {
            if from.checked_add(length).is_none_or(|end| end > bound) {
                return Err("constant range is out of bounds");
            }
            from = from
                .checked_add(offset)
                .ok_or("constant range offset overflow")?;
            base.place = root;
        }
        // Includes geometric capacity slack for the cold range arena and its Place.
        if self.ranges.as_ref().map_or(0, Vec::len).saturating_add(2) > 64 * 1024 * 1024 / 256 {
            return Err("constant range storage limit exceeded");
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        let ranges = self.ranges.get_or_insert_with(Vec::new);
        let projection = ProjectionId::Sequence(
            u32::try_from(ranges.len()).map_err(|_| "too many constant ranges")?,
        );
        ranges.push(Subsequence {
            base: base.place,
            from,
            length,
        });
        place.projection = Some(projection);
        self.places.push(place);
        Ok(Value::Reference(Pointer {
            place: id,
            metadata: None,
            write: base.write,
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rest_projection_layout_keeps_ordinary_place_density() {
        use std::mem::size_of;
        assert_eq!(size_of::<super::super::Place>(), 64);
        assert_eq!(size_of::<super::super::Pointer>(), 24);
        assert_eq!(size_of::<super::super::Projection>(), 16);
        assert_eq!(size_of::<super::Subsequence>(), 24);
        assert_eq!(size_of::<Option<super::super::ProjectionId>>(), 8);
        assert_eq!(size_of::<crate::sema::values::Constant>(), 32);
        assert_eq!(size_of::<crate::sema::values::Location>(), 64);
        assert_eq!(size_of::<crate::sema::values::Origin>(), 24);
        assert_eq!(size_of::<crate::sema::values::View>(), 48);
        let references = super::super::References::default();
        assert!(references.ranges.is_none());
    }
}
