//! Actual Ordering layout receives the result of two typed scalar comparisons.
use super::{Arithmetic, Lower, Selected, ir};
use crate::{
    abi::layout::Shape,
    lex::token::Punctuation,
    lowir::vocabulary::{Binary, Type},
    types::{Kind, Primitive, instance::Instance},
};
impl Lower<'_, '_, '_> {
    pub(super) fn source_three_way(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("three_way_compare Call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("three_way_compare callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_three_way(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [left, right] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("three_way_compare arguments disappeared")?
        else {
            return Err("three_way_compare requires two evaluated scalar arguments");
        };
        let (left, right) = (*left, *right);
        if signature != selected.signature
            || value.ty != selected.output
            || [left, right].into_iter().any(|id| {
                self.body
                    .values
                    .get(id.0 as usize)
                    .is_none_or(|v| v.ty != selected.input)
            })
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err(
                "three_way_compare differs from its actual complete Call/Ordering contract",
            );
        }
        let (Selected::Runtime(left), Selected::Runtime(right)) =
            (self.value(left)?, self.value(right)?)
        else {
            return Err("three_way_compare requires actual runtime scalar operands");
        };
        let Some(Kind::Primitive(primitive)) = self.context.types.kind(selected.input) else {
            return Err("three_way_compare lost its selected scalar type");
        };
        let less = Arithmetic {
            builder: &mut self.builder,
            panic: &mut self.panic,
        }
        .binary(Punctuation::Less, primitive, left, right)?;
        let greater = Arithmetic {
            builder: &mut self.builder,
            panic: &mut self.panic,
        }
        .binary(Punctuation::Greater, primitive, left, right)?;
        let less = super::super::casts::integer(&mut self.builder, less, Type::U8, false)?;
        let greater = super::super::casts::integer(&mut self.builder, greater, Type::U8, false)?;
        let bits = self.builder.binary(Binary::Sub, greater, less)?;
        let layout = self.context.layout(selected.output)?;
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("Ordering lacks its actual enum tag layout");
        };
        if tag.ty != Primitive::I8 || tag.offset != 0 || layout.size != 1 || layout.align != 1 {
            return Err("three_way_compare requires actual one-byte Ordering storage");
        }
        let address = self
            .allocate(selected.output)?
            .ok_or("Ordering output lacks storage")?;
        self.builder.store(bits, address.operand)?;
        Ok(Some(Selected::Object(address)))
    }
}
