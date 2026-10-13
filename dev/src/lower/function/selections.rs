//! Static pattern paths share offset recipes, but form addresses in their use blocks.
use super::{Lower, Projected};
use crate::body::ir::{Place, ProjectionId, ProjectionKind};

#[derive(Clone, Copy)]
pub(super) struct Recipe {
    anchor: Place,
    offset: u64,
}
impl Lower<'_, '_> {
    pub(super) fn select(&mut self, id: ProjectionId, anchor: Place) -> Result<(), &'static str> {
        self.selection_work.clear();
        let mut place = Place::Projection(id);
        let mut offset = 0_u64;
        while place != anchor {
            let Place::Projection(current) = place else {
                return Err("pattern path does not reach its scrutinee");
            };
            if let Some(recipe) = self.selections.get(&current) {
                if recipe.anchor != anchor {
                    return Err("pattern path changed scrutinee");
                }
                offset = recipe.offset;
                break;
            }
            let projection = &self.body.projections[current.0 as usize];
            let layout = self.context.layout(self.place_type(projection.base)?)?;
            let leaf = self.context.layout(projection.ty)?;
            let delta = match projection.kind {
                ProjectionKind::Field(field) => self.selected_field_offset(&layout, field)?,
                ProjectionKind::ConstantIndex(index) => {
                    Self::pattern_byte_offset(&layout, &leaf, index, None)?
                }
                ProjectionKind::Subarray { start, end } => {
                    Self::pattern_byte_offset(&layout, &leaf, start, Some(end))?
                }
                ProjectionKind::Index { .. } => {
                    return Err("dynamic indexing must precede the pattern path");
                }
            };
            self.selection_work.push((current, delta));
            place = projection.base;
        }
        while let Some((current, delta)) = self.selection_work.pop() {
            offset = offset
                .checked_add(delta)
                .ok_or("pattern path offset overflow")?;
            self.selections.insert(current, Recipe { anchor, offset });
        }
        let leaf = self
            .context
            .layout(self.body.projections[id.0 as usize].ty)?;
        let address = if leaf.size == 0 {
            None
        } else {
            let base = self
                .place(anchor)?
                .ok_or("pattern selection has no input storage")?;
            Some(self.offset(base, offset)?)
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
}
