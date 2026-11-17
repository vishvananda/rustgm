//! A rust-call object member unpacks the original Args tuple for the slot ABI.
use super::{Context, Representation, dispatch, offset, parameters};
use crate::{
    body::ir::Selection,
    lowir::{
        builder::Builder,
        program::{SymbolId, Terminator, Value},
        vocabulary::Type,
    },
    sema::vtable_plans::Member,
    types::{Kind, TypeId},
};

impl Context<'_> {
    pub(in crate::lower) fn object_member(
        &mut self,
        member: Member,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self
            .object_callables
            .as_ref()
            .and_then(|s| s.members.get(&member))
        {
            return Ok(symbol);
        }
        let Some(Kind::Function { result, .. }) = self.types.kind(member.signature) else {
            return Err("explicit object member lost its checked signature");
        };
        let has_output = matches!(self.representation(result)?, Representation::Object(_));
        let signature = self.signature(member.signature)?;
        let execution = self.signature(member.execution)?;
        let index = self
            .object_callables
            .as_ref()
            .map_or(0, |s| s.members.len());
        let symbol = self.output.declare(
            self.internal_name(format!("rg.callable.member.{index}")),
            signature,
            false,
        )?;
        let mut builder = Builder::default();
        let block = builder.block()?;
        builder.enter(block)?;
        let incoming = parameters(&self.output, signature)?;
        let (output, receiver_and_tuple) = incoming
            .split_at_checked(usize::from(has_output))
            .ok_or("explicit object member lacks its result storage")?;
        let (&receiver, tuple) = receiver_and_tuple
            .split_first()
            .ok_or("explicit object member lacks its fat receiver")?;
        let (data, callee) = dispatch(&mut builder, receiver, member.call)?;
        let mut arguments = output.to_vec();
        arguments.push(data);
        self.unpack_member_tuple(&mut builder, member.tuple, tuple, &mut arguments)?;
        let native = self.output.scalar_signature(execution)?;
        let value = builder.call(execution, native.result, callee.operand, &arguments)?;
        builder.terminate(if native.noreturn {
            Terminator::Unreachable
        } else {
            Terminator::Return(value)
        })?;
        self.output.define(symbol, builder.finish()?)?;
        self.object_callables
            .get_or_insert_with(Default::default)
            .members
            .insert(member, symbol);
        Ok(symbol)
    }

    fn unpack_member_tuple(
        &mut self,
        builder: &mut Builder,
        tuple: TypeId,
        incoming: &[Value],
        arguments: &mut Vec<Value>,
    ) -> Result<(), &'static str> {
        match (self.representation(tuple)?, incoming) {
            (Representation::Scalar(Type::Void), []) => {}
            (Representation::Object(_), [base]) => {
                let layout = self.layout(tuple)?;
                let Some(Kind::Tuple(fields)) = self.types.kind(tuple) else {
                    return Err("object member argument is not its checked tuple");
                };
                let types = self.types;
                let fields = types
                    .list(fields)
                    .ok_or("missing object member tuple fields")?;
                for (index, &field) in fields.iter().enumerate() {
                    let index = u32::try_from(index)
                        .map_err(|_| "object member tuple exceeds host indexing")?;
                    let (selected, displacement) =
                        self.selected_field(&layout, Selection::Tuple(index))?;
                    if selected != field {
                        return Err("object member tuple layout differs from its checked field");
                    }
                    let representation = self.representation(field)?;
                    if representation.parameter() == Type::Void {
                        continue;
                    }
                    let address = offset(builder, *base, displacement)?;
                    let value = match representation {
                        Representation::Object(_) => address,
                        _ => builder.load(representation.parameter(), address.operand)?,
                    };
                    arguments.push(value);
                }
            }
            _ => return Err("object member tuple differs from its physical ABI"),
        }
        Ok(())
    }
}
