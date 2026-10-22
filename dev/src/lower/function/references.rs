//! Borrowed temporary addresses are materialized where their values dominate uses.
use super::{Address, Lower, Representation, Selected};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::ir::{self, Action, Place, Rvalue, ValueId},
    lowir::{
        program::{MemorySpan, Operand},
        vocabulary::Type,
    },
    types::TypeId,
};

#[derive(Default)]
pub(super) struct Storage {
    places: HashSet<Place>,
    pub temporaries: HashMap<ValueId, Address>,
}
impl Storage {
    pub fn collect(body: &ir::Body) -> Result<Self, &'static str> {
        let mut result = Self::default();
        for value in &body.values {
            if let Rvalue::Borrow { place, .. } | Rvalue::Address(place) = value.kind {
                result.mark(body, place)?;
            }
        }
        for projection in &body.projections {
            if projection.kind.is_dereference() {
                result.mark(body, projection.base)?;
            }
        }
        if !result.places.is_empty() {
            for statement in &body.statements {
                if let Action::BindView { place, .. } = statement.action {
                    result.mark(body, place)?;
                }
            }
        }
        Ok(result)
    }
    pub(super) fn mark(&mut self, body: &ir::Body, mut place: Place) -> Result<(), &'static str> {
        while self.places.insert(place) {
            let Place::Projection(id) = place else {
                break;
            };
            let projection = body
                .projections
                .get(id.0 as usize)
                .ok_or("missing borrowed projection")?;
            if matches!(projection.base, Place::Projection(base) if base.0 >= id.0) {
                return Err("borrowed projection does not have an earlier base");
            }
            place = projection.base;
        }
        Ok(())
    }
    pub fn needs(&self, place: Place) -> bool {
        !self.places.is_empty() && self.places.contains(&place)
    }
}
impl Lower<'_, '_, '_> {
    pub(super) fn zero_backing(&mut self, ty: TypeId) -> Result<Address, &'static str> {
        let layout = self.context.layout(ty)?;
        if layout.size != 0 {
            return Err("nonzero value requested zero-sized backing");
        }
        let slot = self.builder.object_slot(MemorySpan {
            bytes: u64::from(layout.align),
            align: layout.align,
        })?;
        Ok(Address {
            operand: Operand::Slot(slot),
            align: layout.align,
        })
    }
    pub(super) fn materialize(&mut self, id: ValueId, value: Selected) -> Result<(), &'static str> {
        if !self.addresses.needs(Place::Temporary(id))
            || self
                .body
                .promotions
                .as_ref()
                .is_some_and(|p| p.get(id).is_some())
        {
            return Ok(());
        }
        let ty = self.body.values[id.0 as usize].ty;
        let address = match value {
            Selected::Object(address) => address,
            Selected::Unit | Selected::Item(_) => self.zero_backing(ty)?,
            Selected::Runtime(_) => {
                let address = self.allocate(ty)?.ok_or("borrowed scalar has no storage")?;
                self.write(Some(address), value, ty)?;
                address
            }
        };
        if self.addresses.temporaries.insert(id, address).is_some() {
            return Err("borrowed temporary evaluated twice in the same body");
        }
        Ok(())
    }
    pub(super) fn constrain(
        &mut self,
        operand: ValueId,
        target_ty: TypeId,
        target: Representation,
    ) -> Result<Selected, &'static str> {
        let source = self
            .context
            .representation(self.body.values[operand.0 as usize].ty)?;
        let same = match (source, target) {
            (Representation::Scalar(a), Representation::Scalar(b)) => a == b,
            (Representation::Object(a), Representation::Object(b))
            | (Representation::PointerObject(a), Representation::PointerObject(b)) => {
                a.bytes == b.bytes && a.align == b.align
            }
            _ => false,
        };
        if !same {
            return self.unsize(operand, target_ty);
        }
        self.value(operand)
    }
    pub(super) fn dereference(
        &mut self,
        base: Address,
        ty: TypeId,
    ) -> Result<Address, &'static str> {
        let align = self
            .context
            .layouts
            .reference_align(self.context.types, self.context.aggregates, ty)
            .map_err(|e| e.message)?;
        let value = self.builder.load(Type::Ptr, base.operand)?;
        Ok(Address {
            operand: value.operand,
            align,
        })
    }
}
