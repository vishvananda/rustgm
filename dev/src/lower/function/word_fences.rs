//! Source fences use the backend's readwrite assembly memory boundary on x86-64.
use super::{Lower, Selected, ir};
use crate::{
    lowir::asm,
    sema::compiler_items::checked_atomic_access::{Fence, fence_order},
    support::atomic_ordering::Ordering,
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_fence(
        &mut self,
        scope: Fence,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(order) = fence_order(self.context.ledger, callee, signature)? else {
            return Ok(None);
        };
        if arguments.start != arguments.end || result != TypeId::UNIT {
            return Err("source atomic fence differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        // x86 preserves the load-load, load-store and store-store order
        // required by the weaker fences. A sequentially consistent thread
        // fence must also order a preceding store against a later load.
        let hardware = matches!(scope, Fence::Thread) && order == Ordering::Sequential;
        let index = usize::from(hardware);
        let template = if let Some(known) = self.context.source_fences[index] {
            known
        } else {
            let mut draft = self.context.output.assembly()?;
            if hardware {
                draft.text("mfence")?;
            }
            let template = draft.finish(asm::SIDE_EFFECT, 0, 0)?;
            self.context.source_fences[index] = Some(template);
            template
        };
        self.builder.asm(template, &[])?;
        Ok(Some(Selected::Unit))
    }
}
