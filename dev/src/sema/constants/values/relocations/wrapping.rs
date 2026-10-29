//! Wrapping words retain provenance without demanding an in-bounds address or a value.
use super::super::{Constant, Evaluator, Failure, Reader, Value, references};
use crate::types::TypeId;
impl Evaluator<'_> {
    pub(crate) fn wrapping_byte_projection(
        &mut self,
        value: Constant,
        displacement: u64,
        ty: TypeId,
        work: &mut usize,
        limit: usize,
    ) -> Result<Constant, Failure> {
        *work = work.saturating_add(1);
        if *work > limit {
            return Err("wrapping pointer work budget exhausted".into());
        }
        if self.facts().types.is_sized(ty) != Some(true) {
            return Err("wrapping pointer requires its completed sized element".into());
        }
        match value.0 {
            Value::Address {
                bits,
                metadata: None,
            } => Ok(Constant::raw_address(bits.wrapping_add(displacement))),
            Value::Reference(pointer) if pointer.metadata.is_none() => {
                if displacement == 0 {
                    return Ok(value);
                }
                let address = self.word_address_charged(value, limit, work)?;
                // Flatten to the original root: later wrapping may return in bounds.
                // An ordinary dereference still checks that root's lifetime and bytes.
                Reader::new(self.facts()).location(address.root)?;
                let mut base = references::Pointer::new(address.root.0);
                base.write = address.write;
                let result = self
                    .queries
                    .typing
                    .facts
                    .constant_references
                    .byte_projection(
                        base,
                        address.offset.wrapping_add(displacement),
                        references::Place {
                            value: None,
                            ty,
                            access: references::Access::Owned,
                            packed: None,
                            projection: None,
                        },
                    )?;
                Ok(Constant(result))
            }
            _ => Err("source wrapping offset requires a supported thin raw pointer value".into()),
        }
    }
}
