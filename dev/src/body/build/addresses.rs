//! Raw place addresses require neither alignment promises nor a persistent loan.
use super::{Builder, Error, NodeId, Place, Rvalue, Type, ValueId};
use crate::body::ir::ProjectionKind;

impl Builder<'_> {
    pub(super) fn finish_address(&mut self, node: NodeId) -> Result<ValueId, Error> {
        let place = self
            .pop_place(node)?
            .ok_or_else(|| self.error(node, "missing raw address place"))?;
        let ty = self.ty(node)?;
        let Some(Type::Pointer { pointee, .. }) = self.facts.types.kind(ty) else {
            return Err(self.error(node, "raw address requires a pointer type"));
        };
        if pointee != self.place_type(node, place)? {
            return Err(self.error(node, "raw address differs from its selected place"));
        }
        self.address_place(node, place)?;
        self.evaluate(node, ty, Rvalue::Address(place))
    }
    fn address_place(&self, node: NodeId, mut place: Place) -> Result<(), Error> {
        for _ in 0..1_000_000 {
            match place {
                Place::Temporary(_) => {
                    return Err(self.error(node, "cannot take raw address of a temporary"));
                }
                Place::Local(_) | Place::Static(_) => return Ok(()),
                Place::Projection(id) => {
                    let projection = &self.body.projections[id.0 as usize];
                    if matches!(projection.kind, ProjectionKind::Dereference { .. }) {
                        return Ok(());
                    }
                    place = projection.base;
                }
            }
        }
        Err(self.error(node, "raw address place work budget exceeded"))
    }
}
