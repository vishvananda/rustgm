//! Pattern recipes share static offsets and dynamic prefixes, evaluated in each use block.
use super::{Lower, Projected};
use crate::body::ir::{Place, ProjectionId, ProjectionKind};
use crate::types::Kind;
use std::num::NonZeroU32;

#[derive(Clone, Copy)]
pub(super) struct Recipe {
    anchor: Place,
    offset: u64,
}
#[derive(Clone, Copy)]
pub(super) enum Step {
    Offset(u64),
    Dynamic(ProjectionId),
}
/// A shared dynamic prefix; ordinary offset-only recipes retain their compact shape.
#[derive(Clone, Copy)]
pub(super) struct Dynamic {
    parent: Option<NonZeroU32>,
    offset: u64,
    projection: ProjectionId,
}
impl Lower<'_, '_> {
    fn selection_step(&mut self, id: ProjectionId) -> Result<Step, &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        match projection.kind {
            ProjectionKind::Dereference { .. }
            | ProjectionKind::RawDereference { .. }
            | ProjectionKind::FromEnd { .. }
            | ProjectionKind::Subslice { .. } => Ok(Step::Dynamic(id)),
            ProjectionKind::Field(field) => {
                let (_, offset) = self
                    .context
                    .place_field(self.place_type(projection.base)?, field)?;
                Ok(match offset {
                    crate::abi::tails::Offset::Fixed(offset) => Step::Offset(offset),
                    crate::abi::tails::Offset::Aligned { .. } => Step::Dynamic(id),
                })
            }
            ProjectionKind::ConstantIndex(index) => {
                let ty = self.place_type(projection.base)?;
                let leaf = self.context.layout(projection.ty)?;
                let offset = if matches!(self.context.types.kind(ty), Some(Kind::Slice(_))) {
                    index
                        .checked_mul(leaf.size)
                        .ok_or("slice pattern offset overflow")?
                } else {
                    let layout = self.context.layout(ty)?;
                    Self::pattern_byte_offset(&layout, &leaf, index, None)?
                };
                Ok(Step::Offset(offset))
            }
            ProjectionKind::Subarray { start, end } => {
                let layout = self.context.layout(self.place_type(projection.base)?)?;
                let leaf = self.context.layout(projection.ty)?;
                Ok(Step::Offset(Self::pattern_byte_offset(
                    &layout,
                    &leaf,
                    start,
                    Some(end),
                )?))
            }
            ProjectionKind::Index { .. }
            | ProjectionKind::IndexParameter { .. }
            | ProjectionKind::SliceIndex { .. } => {
                Err("dynamic indexing must precede the pattern path")
            }
        }
    }
    fn selection_recipe(
        &mut self,
        id: ProjectionId,
        anchor: Place,
    ) -> Result<(Recipe, Option<NonZeroU32>), &'static str> {
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
                parent = self.selection_paths.get(&current).copied();
                break;
            }
            let step = self.selection_step(current)?;
            self.selection_work.push((current, step));
            place = self.body.projections[current.0 as usize].base;
        }
        while let Some((current, step)) = self.selection_work.pop() {
            match step {
                Step::Offset(delta) => {
                    offset = offset
                        .checked_add(delta)
                        .ok_or("pattern path offset overflow")?;
                }
                Step::Dynamic(projection) => {
                    let id = self
                        .selection_steps
                        .len()
                        .checked_add(1)
                        .and_then(|n| u32::try_from(n).ok())
                        .and_then(NonZeroU32::new)
                        .ok_or("too many dynamic pattern recipes")?;
                    self.selection_steps.push(Dynamic {
                        parent,
                        offset,
                        projection,
                    });
                    parent = Some(id);
                    offset = 0;
                }
            }
            self.selections.insert(current, Recipe { anchor, offset });
            if let Some(id) = parent {
                self.selection_paths.insert(current, id);
            }
        }
        Ok((Recipe { anchor, offset }, parent))
    }
    pub(super) fn select(&mut self, id: ProjectionId, anchor: Place) -> Result<(), &'static str> {
        let (recipe, mut parent) = self.selection_recipe(id, anchor)?;
        let ty = self.body.projections[id.0 as usize].ty;
        let wide = self.context.wide_referent(ty)?;
        if !wide
            && self.context.layout(ty)?.size == 0
            && !self.addresses.needs(Place::Projection(id))
        {
            self.projections[id.0 as usize] = Projected::Ready(None);
            return Ok(());
        }
        let mut base = self
            .place(anchor)?
            .ok_or("pattern selection has no input storage")?;
        let mut metadata = if self.context.wide_referent(self.place_type(anchor)?)? {
            Some(self.metadata(anchor)?)
        } else {
            None
        };
        self.address_work.clear();
        while let Some(id) = parent {
            let index = id.get() as usize - 1;
            let step = self
                .selection_steps
                .get(index)
                .ok_or("missing dynamic pattern recipe")?;
            self.address_work.push(index);
            parent = step.parent;
        }
        while let Some(index) = self.address_work.pop() {
            let step = self.selection_steps[index];
            base = self.offset(base, step.offset)?;
            (base, metadata) = self.dynamic_selection(step.projection, base, metadata)?;
        }
        let address = self.offset(base, recipe.offset)?;
        if wide {
            self.place_metadata
                .insert(id, metadata.ok_or("unsized selection lacks metadata")?);
        }
        self.projections[id.0 as usize] = Projected::Ready(Some(address));
        Ok(())
    }
}
