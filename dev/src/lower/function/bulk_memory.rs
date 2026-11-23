//! Checked source transfers copy or fill object representations of the exact element layout.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    sema::compiler_items::zeroing_contract::Operation,
    support::intrinsics::Intrinsic,
    types::{
        Kind, Primitive, Store, TypeId, arguments::Argument, callable::Convention,
        instance::Instance,
    },
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_counted_transfer(
        &mut self,
        transfer: Intrinsic,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let types = self.context.types;
        let instance = self.body.callable_target(Instance::new(types, callee)?);
        let (element, expected) = if transfer.is_fill() {
            self.source_fill(instance, signature)?
        } else {
            source_copy(types, callee, signature)?
        };
        let Some(&[first, second, count]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source transfer requires three evaluated inputs");
        };
        if [
            self.input_type(first)?,
            self.input_type(second)?,
            self.input_type(count)?,
        ] != expected
            || result != TypeId::UNIT
        {
            return Err("source transfer differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let symbol = self
            .context
            .demand_source_transfer(instance, signature, transfer, element)?;
        self.call_operand(signature, Operand::Symbol(symbol), arguments, result)
    }

    fn source_fill(
        &self,
        instance: Instance,
        signature: TypeId,
    ) -> Result<(TypeId, [TypeId; 3]), &'static str> {
        let selection = self
            .context
            .constants
            .source_zeroing(instance, self.body.source)?
            .ok_or("source write_bytes requires its checked declaration")?;
        let Operation::Fill(destination) = selection.operation() else {
            return Err("source write_bytes is not a checked fill");
        };
        let Some(Kind::Pointer {
            mutable: true,
            pointee,
        }) = self.context.types.kind(destination)
        else {
            return Err("source write_bytes requires its checked raw destination");
        };
        if !selection.opened_matches(self.context.ledger, signature) {
            return Err("source write_bytes differs from its opened signature");
        }
        Ok((pointee, selection.inputs()))
    }
}

fn source_copy(
    types: &Store,
    callee: TypeId,
    signature: TypeId,
) -> Result<(TypeId, [TypeId; 3]), &'static str> {
    let Some(Kind::FunctionItem { arguments, .. }) = types.kind(callee) else {
        return Err("source copy requires its complete function item");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source copy lost its type substitution")?;
    let (Some(Argument::Type(element)), None) = (arguments.next(), arguments.next()) else {
        return Err("source copy requires one element type substitution");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: true,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source copy requires its opened unsafe Rust signature");
    };
    let Some(&[source, destination, count]) = types.list(parameters) else {
        return Err("source copy requires three opened inputs");
    };
    if !matches!(types.kind(source), Some(Kind::Pointer { mutable: false, pointee }) if pointee == element)
        || !matches!(types.kind(destination), Some(Kind::Pointer { mutable: true, pointee }) if pointee == element)
        || count != TypeId::primitive(Primitive::Usize)
    {
        return Err("source copy requires raw pointers to the same element and a usize count");
    }
    Ok((element, [source, destination, count]))
}
