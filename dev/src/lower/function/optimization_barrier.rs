//! A checked source `black_box` moves through a private slot and a memory barrier.
use super::{Lower, Selected, ir};
use crate::{
    lowir::asm::{self, AssemblyId},
    sema::compiler_items::black_box_type,
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_black_box(
        &mut self,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let ty = black_box_type(self.context.ledger, callee, signature)?;
        let Some(&[input]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source black_box requires one evaluated input");
        };
        if result != ty || self.input_type(input)? != ty {
            return Err(
                "source black_box evaluated input or result differs from its opened signature",
            );
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let selected = self.value(input)?;
        if matches!(selected, Selected::Unit | Selected::Item(_)) {
            return Ok(selected);
        }
        let slot = self
            .allocate(ty)?
            .ok_or("source black_box value lacks private storage")?;
        self.write(Some(slot), selected, ty)?;
        let address = self.pointer(slot)?;
        let barrier = self.opaque_value_barrier()?;
        self.builder.asm(barrier, &[address])?;
        if let Selected::Runtime(value) = selected {
            self.builder
                .load(value.ty, slot.operand)
                .map(Selected::Runtime)
        } else {
            Ok(Selected::Object(slot))
        }
    }

    fn opaque_value_barrier(&mut self) -> Result<AssemblyId, &'static str> {
        if let Some(barrier) = self.context.opaque_value_barrier {
            return Ok(barrier);
        }
        // The backend sees a side effect that may read and write through the
        // private slot's address. The empty machine template changes no bits.
        let mut draft = self.context.output.assembly()?;
        draft.operand(asm::Kind::Pointer)?;
        let barrier = draft.finish(asm::SIDE_EFFECT, 0, 0)?;
        self.context.opaque_value_barrier = Some(barrier);
        Ok(barrier)
    }
}
