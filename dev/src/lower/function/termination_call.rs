//! The checked terminal source operation consumes its existing unreachable edge.
use super::{Lower, Selected, ir};
use crate::{
    body::storage::Location,
    lowir::program::Terminator,
    types::{Kind, TypeId, instance::Instance},
};

/// No permission can outlive this actual body/call/statement borrow.
struct Call<'body> {
    body: &'body ir::Body,
    value: ir::ValueId,
    instance: Instance,
    signature: TypeId,
    location: Location,
}
impl Lower<'_, '_, '_> {
    pub(super) fn source_abort(
        &mut self,
        id: ir::ValueId,
        location: Location,
    ) -> Result<bool, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source abort lost its actual value")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(false);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source abort lost its actual callee")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(false);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selection) = self
            .context
            .constants
            .source_abort(instance, self.body.source)?
        else {
            return Ok(false);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        if self.body.const_context
            || value.ty != TypeId::NEVER
            || signature != selection.signature
            || !self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .is_some_and(<[ir::ValueId]>::is_empty)
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err("source abort differs from its evaluated exact runtime Call");
        }
        self.consume_abort(Call {
            body: self.body,
            value: id,
            instance,
            signature,
            location,
        })?;
        Ok(true)
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "consume the non-clonable exact body/call witness once"
    )]
    fn consume_abort(&mut self, call: Call<'_>) -> Result<(), &'static str> {
        terminal(call.body, call.value, call.location)?;
        let actual = self
            .body
            .values
            .get(call.value.0 as usize)
            .ok_or("source abort lost its selected Call")?;
        let ir::Rvalue::Call {
            signature, callee, ..
        } = actual.kind
        else {
            return Err("source abort changed its operation");
        };
        if !std::ptr::eq(call.body, self.body)
            || signature != call.signature
            || self
                .body
                .values
                .get(callee.0 as usize)
                .is_none_or(|v| v.ty != call.instance.ty())
        {
            return Err("source abort belongs to another body/call/instance");
        }
        // This terminates immediately, before post-call cleanup bookkeeping or
        // destructor emission. The ordinary false branch retains its timeline.
        self.builder.terminate(Terminator::Unreachable)
    }
}

fn terminal(body: &ir::Body, value: ir::ValueId, location: Location) -> Result<(), &'static str> {
    let Location::At { block, before } = location else {
        return Err("source abort requires its actual in-block statement");
    };
    let block = body
        .blocks
        .get(block.0 as usize)
        .ok_or("source abort lost its original block")?;
    if before < block.start
        || before.checked_add(1) != Some(block.end)
        || !matches!(block.terminator, ir::Terminator::Unreachable)
        || !body
            .statements
            .get(before as usize)
            .is_some_and(|s| matches!(s.action, ir::Action::Evaluate(actual) if actual == value))
    {
        return Err("source abort requires its original terminal Call and unreachable edge");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/terminal_operations.rs"]
mod tests;
