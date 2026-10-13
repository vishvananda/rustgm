//! Static pattern paths share offset recipes, but form addresses in their use blocks.
use super::{Lower, Projected};
use crate::body::ir::{Place, ProjectionId, ProjectionKind};
use crate::types::TypeId;
use std::num::NonZeroU32;

#[derive(Clone, Copy)]
pub(super) struct Recipe {
    anchor: Place,
    offset: u64,
}
#[derive(Clone, Copy)]
pub(super) enum Step {
    Offset(u64),
    Dereference(TypeId),
}
/// A persistent prefix; ordinary offset-only recipes retain their compact shape.
#[derive(Clone, Copy)]
pub(super) struct Dereference {
    parent: Option<NonZeroU32>,
    offset: u64,
    ty: TypeId,
}
impl Lower<'_, '_> {
    pub(super) fn select(&mut self, id: ProjectionId, anchor: Place) -> Result<(), &'static str> {
        self.selection_work.clear();
        let mut place = Place::Projection(id);
        let mut offset = 0_u64;
        let mut parent = None;
        while place != anchor {
            let Place::Projection(current) = place else {
                return Err("pattern path does not reach its scrutinee");
            };
            if let Some(recipe) = self.selections.get(&current) {
                if recipe.anchor != anchor {
                    return Err("pattern path changed scrutinee");
                }
                offset = recipe.offset;
                parent = self.selection_dereferences.get(&current).copied();
                break;
            }
            let projection = &self.body.projections[current.0 as usize];
            let layout = self.context.layout(self.place_type(projection.base)?)?;
            let leaf = self.context.layout(projection.ty)?;
            let step = match projection.kind {
                ProjectionKind::Dereference { .. } => Step::Dereference(projection.ty),
                ProjectionKind::Field(field) => {
                    Step::Offset(self.selected_field_offset(&layout, field)?)
                }
                ProjectionKind::ConstantIndex(index) => {
                    Step::Offset(Self::pattern_byte_offset(&layout, &leaf, index, None)?)
                }
                ProjectionKind::Subarray { start, end } => {
                    Step::Offset(Self::pattern_byte_offset(&layout, &leaf, start, Some(end))?)
                }
                ProjectionKind::Index { .. } | ProjectionKind::SliceIndex { .. } => {
                    return Err("dynamic indexing must precede the pattern path");
                }
            };
            self.selection_work.push((current, step));
            place = projection.base;
        }
        while let Some((current, step)) = self.selection_work.pop() {
            match step {
                Step::Offset(delta) => {
                    offset = offset
                        .checked_add(delta)
                        .ok_or("pattern path offset overflow")?;
                }
                Step::Dereference(ty) => {
                    let id = self
                        .dereferences
                        .len()
                        .checked_add(1)
                        .and_then(|n| u32::try_from(n).ok())
                        .and_then(NonZeroU32::new)
                        .ok_or("too many pattern dereference recipes")?;
                    self.dereferences.push(Dereference { parent, offset, ty });
                    parent = Some(id);
                    offset = 0;
                }
            }
            self.selections.insert(current, Recipe { anchor, offset });
            if let Some(id) = parent {
                self.selection_dereferences.insert(current, id);
            }
        }
        let leaf = self
            .context
            .layout(self.body.projections[id.0 as usize].ty)?;
        let address = if leaf.size == 0 && !self.addresses.needs(Place::Projection(id)) {
            None
        } else {
            let mut base = self
                .place(anchor)?
                .ok_or("pattern selection has no input storage")?;
            self.address_work.clear();
            while let Some(id) = parent {
                let index = id.get() as usize - 1;
                let step = self
                    .dereferences
                    .get(index)
                    .ok_or("missing pattern dereference recipe")?;
                self.address_work.push(index);
                parent = step.parent;
            }
            while let Some(index) = self.address_work.pop() {
                let step = self.dereferences[index];
                base = self.offset(base, step.offset)?;
                base = self.dereference(base, step.ty)?;
            }
            Some(self.offset(base, offset)?)
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
}
