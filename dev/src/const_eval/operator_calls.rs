//! Reached owned-item calls retain their complete opened signature and operands.
use super::{Failure, Frame, Machine, code::Code};
use crate::{
    body::ir::{Action, ValueId, ValueList},
    support::call_traits::CallTrait,
    types::{Kind, TypeId, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn invoked_item(
        &mut self,
        body: &Code,
        frame: &Frame,
        id: ValueId,
        kind: CallTrait,
        signature: TypeId,
        arguments: ValueList,
    ) -> Result<Option<(Instance, ValueList)>, Failure> {
        self.charge(1)?;
        self.bodies.validate_entry(frame.body)?;
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("const Invoke has no reached statement")?;
        self.charge(1)?;
        let block = body
            .blocks
            .get(frame.block.0 as usize)
            .ok_or("const Invoke has no block")?;
        self.charge(1)?;
        if position < block.start
            || position >= block.end
            || !matches!(body.statements.get(position as usize).map(|s| s.action), Some(Action::Evaluate(value)) if value == id)
            || !body.const_context
        {
            return Err("const Invoke differs from its exact checked statement".into());
        }
        // These charged O(1) facts are also counted on a rejected reached use.
        self.charge(3)?;
        crate::sema::compiler_items::callable_role(self.evaluator.facts(), kind)?;
        self.charge(3)?;
        let facts = self.evaluator.facts();
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("const Invoke requires its exact opened safe signature".into());
        };
        if body
            .values
            .get(id.0 as usize)
            .is_none_or(|value| value.ty != result)
        {
            return Err("const Invoke result differs from its exact signature".into());
        }
        let operands = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("const Invoke has invalid operands")?;
        let [receiver, operands @ ..] = operands else {
            return Err("const Invoke requires its owned item receiver".into());
        };
        // Pay before borrowing the canonical parameter list; the bytecode range
        // is the exact maximum number of children inspected below.
        self.charge(1 + operands.len())?;
        let parameters = self
            .evaluator
            .facts()
            .types
            .list(parameters)
            .ok_or("const Invoke has invalid parameters")?;
        if parameters.len() != operands.len() {
            return Err("const Invoke changed its complete operand arity".into());
        }
        // Reached work is proportional to this call's actual operands. Neither
        // the source body nor its unrelated generic formals are walked again.
        for (&operand, &parameter) in operands.iter().zip(parameters) {
            if body
                .values
                .get(operand.0 as usize)
                .is_none_or(|value| value.ty != parameter)
            {
                return Err("const Invoke operand differs from its complete signature".into());
            }
        }
        let Some(target) = self.invoked_receiver(body, frame, *receiver, kind, signature)? else {
            return Ok(None);
        };
        Ok(Some((
            target,
            ValueList {
                start: arguments
                    .start
                    .checked_add(1)
                    .ok_or("const Invoke argument overflow")?,
                ..arguments
            },
        )))
    }

    fn invoked_receiver(
        &mut self,
        body: &Code,
        frame: &Frame,
        receiver: ValueId,
        kind: CallTrait,
        signature: TypeId,
    ) -> Result<Option<Instance>, Failure> {
        self.charge(7)?;
        let operand = body
            .values
            .get(receiver.0 as usize)
            .ok_or("const Invoke has no receiver type")?
            .ty;
        let ty = if kind == CallTrait::FnOnce {
            operand
        } else {
            let Some(Kind::Reference {
                mutable, referent, ..
            }) = self.evaluator.facts().types.kind(operand)
            else {
                return Err("const Invoke requires its actual borrowed item receiver".into());
            };
            if mutable != (kind == CallTrait::FnMut) {
                return Err("const Invoke changed its receiver borrow capability".into());
            }
            referent
        };
        let facts = self.evaluator.facts();
        if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem { signature: public, .. }) if public == signature)
            || facts.types.needs_region_context(ty) != Some(false)
        {
            return Err("const Invoke changed its full item receiver or argument".into());
        }
        let mut value = Self::value(frame, receiver)?;
        if kind != CallTrait::FnOnce {
            if kind == CallTrait::FnMut
                && value.write_permission() != Some(crate::sema::values::WritePermission::Writable)
            {
                return Err("const Invoke requires its actual unique receiver permission".into());
            }
            let Some(loaded) = self.read(super::memory::Resolved {
                address: value,
                ty,
                raw: None,
                union: None,
            })?
            else {
                return Ok(None);
            };
            value = loaded;
        }
        let target = value
            .as_function()
            .ok_or("const Invoke receiver is not a function item")?;
        if target.ty() != ty {
            return Err("const Invoke changed its exact function item value".into());
        }
        Ok(Some(target))
    }
}
