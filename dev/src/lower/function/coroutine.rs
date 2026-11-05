//! Physical frame access consumes only an already proved generated future body.
use super::{Address, Lower, Projected, Representation, Selected};
use crate::{
    body::ir::{LocalId, Place, ProjectionId},
    lowir::vocabulary::Type,
};

impl Lower<'_, '_, '_> {
    pub(super) fn project_future(
        &mut self,
        id: ProjectionId,
        base: Option<Address>,
    ) -> Result<(), &'static str> {
        let offset = self.future_offset(id)?;
        let projection = &self.body.projections[id.0 as usize];
        let retained = self.context.layout(projection.ty)?.size != 0
            || self.addresses.needs(Place::Projection(id));
        let address = if retained {
            Some(self.offset(
                base.ok_or("future field has no actual frame storage")?,
                offset,
            )?)
        } else {
            None
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }

    pub(super) fn future_offset(&mut self, id: ProjectionId) -> Result<u64, &'static str> {
        let projection = self
            .body
            .projections
            .get(id.0 as usize)
            .ok_or("future native field lost its actual projection")?;
        let index = crate::body::coroutines::field(self.context.types, self.body, projection)?;
        let layout = self.context.layout(self.place_type(projection.base)?)?;
        let [variant] = self.context.layouts.variants(&layout) else {
            return Err("future native field lost its actual single storage layout");
        };
        let fields = self
            .context
            .types
            .list(variant.fields)
            .ok_or("future native field lost its actual layout fields")?;
        if fields.get(index as usize).copied() != Some(projection.ty) {
            return Err("future native field differs from its actual typed layout");
        }
        self.context
            .layouts
            .offsets(variant)
            .get(index as usize)
            .copied()
            .ok_or("future native field lost its actual byte offset")
    }

    pub(super) fn future_take(
        &mut self,
        id: ProjectionId,
        target: LocalId,
    ) -> Result<(), &'static str> {
        let projection =
            crate::body::coroutines::transfer(self.context.types, self.body, id, target)?;
        let representation = self.context.representation(projection.ty)?;
        let input = match representation {
            Representation::Scalar(Type::Void) => Selected::Unit,
            Representation::Scalar(ty) => {
                let source = self
                    .place(Place::Projection(id))?
                    .ok_or("future scalar transfer lost its actual source storage")?;
                Selected::Runtime(self.builder.load(ty, source.operand)?)
            }
            Representation::Object(_) | Representation::ScalarObject(_) => Selected::Object(
                self.place(Place::Projection(id))?
                    .ok_or("future object transfer lost its actual source storage")?,
            ),
        };
        let destination = self.place(Place::Local(target))?;
        self.write(destination, input, projection.ty)
    }
}
