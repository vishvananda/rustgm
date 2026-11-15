//! Typed loads, stores and value moves through the ordinary snapshot paths.
use super::{
    super::{Address, Lower, Representation, Selected},
    Call,
};
use crate::{lowir::vocabulary::Type, types::TypeId};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_memory(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        match (call.name, call.inputs) {
            ("read_via_copy", &[pointer]) => {
                let Some((pointer, target)) = self.thin_input(pointer)? else {
                    return Ok(None);
                };
                if target != call.result {
                    return Ok(None);
                }
                let source = Address {
                    operand: pointer.operand,
                    align: self.context.layout(target)?.align,
                };
                self.loaded(source, target).map(Some)
            }
            ("write_via_move", &[pointer, value]) => {
                let Some((pointer, target)) = self.thin_input(pointer)? else {
                    return Ok(None);
                };
                if target != self.input_type(value)? || call.result != TypeId::UNIT {
                    return Ok(None);
                }
                let selected = self.value(value)?;
                if matches!(selected, Selected::Runtime(_) | Selected::Object(_)) {
                    let destination = Address {
                        operand: pointer.operand,
                        align: self.context.layout(target)?.align,
                    };
                    self.write(Some(destination), selected, target)?;
                }
                Ok(Some(Selected::Unit))
            }
            ("black_box", &[value]) => {
                if self.input_type(value)? != call.result {
                    return Ok(None);
                }
                // The identity; an object result owns its own snapshot.
                match self.value(value)? {
                    Selected::Object(source) => self.loaded(source, call.result).map(Some),
                    selected => Ok(Some(selected)),
                }
            }
            _ => Ok(None),
        }
    }
    /// A fresh snapshot of the value of type `ty` stored at `source`.
    fn loaded(&mut self, source: Address, ty: TypeId) -> Result<Selected, &'static str> {
        match self.context.representation(ty)? {
            Representation::Scalar(Type::Void) => Ok(Selected::Unit),
            Representation::Scalar(scalar) => self
                .builder
                .load(scalar, source.operand)
                .map(Selected::Runtime),
            Representation::Object(_) | Representation::ScalarObject(_) => {
                let destination = self
                    .allocate(ty)?
                    .ok_or("loaded object value has no snapshot storage")?;
                self.write(Some(destination), Selected::Object(source), ty)?;
                Ok(Selected::Object(destination))
            }
        }
    }
}
