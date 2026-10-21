//! Function and pointer object entries are mechanical forwarding after source proof.
use super::{Context, Representation, parameters};
use crate::{
    lowir::{
        builder::Builder,
        program::{Operand, SymbolId, Terminator},
        vocabulary::Type,
    },
    sema::vtable_plans::{Callable, Operator, ShapeId},
    types::{Kind, TypeId, instance::Instance},
};
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::lower) struct Callables {
    entries: HashMap<Callable, SymbolId>,
    calls: HashMap<(ShapeId, u32, u32), SymbolId>,
}
impl Context<'_> {
    pub(in crate::lower) fn object_operator(
        &mut self,
        operator: Operator,
    ) -> Result<SymbolId, &'static str> {
        let abi = self.signature(operator.execution)?;
        let key = (
            operator.call.shape,
            operator.call.slot,
            operator.dereferences,
        );
        if let Some(symbol) = self
            .object_callables
            .as_ref()
            .and_then(|s| s.calls.get(&key))
        {
            let selected = self
                .output
                .symbols
                .get(symbol.0 as usize)
                .ok_or("missing shared object callable entry")?
                .signature;
            if !self.output.compatible(abi, selected)? {
                return Err("object callable slot has incompatible physical signatures");
            }
            return Ok(*symbol);
        }
        let index = self.object_callables.as_ref().map_or(0, |s| s.calls.len());
        let symbol = self.emit_virtual(
            operator.execution,
            operator.call,
            operator.dereferences,
            format!("rg.callable.virtual.{index}"),
        )?;
        self.object_callables
            .get_or_insert_with(Default::default)
            .calls
            .insert(key, symbol);
        Ok(symbol)
    }

    pub(in crate::lower) fn callable_target(
        &mut self,
        target: Callable,
    ) -> Result<SymbolId, &'static str> {
        if let Some(symbol) = self
            .object_callables
            .as_ref()
            .and_then(|s| s.entries.get(&target))
        {
            return Ok(*symbol);
        }
        let Some(Kind::Function { result, .. }) = self.types.kind(target.signature) else {
            return Err("object callable target lacks its signature");
        };
        let receiver_index = usize::from(matches!(
            self.representation(result)?,
            Representation::Object(_)
        ));
        let entry = self.signature(target.execution)?;
        let signature = self.signature(target.signature)?;
        let index = self
            .object_callables
            .as_ref()
            .map_or(0, |s| s.entries.len());
        let symbol = self.output.declare(
            self.internal_name(format!("rg.callable.target.{index}")),
            entry,
            false,
        )?;
        let mut builder = Builder::default();
        let block = builder.block()?;
        builder.enter(block)?;
        let mut arguments = parameters(&self.output, entry)?;
        if receiver_index >= arguments.len() {
            return Err("object callable target lacks its ABI receiver");
        }
        let receiver = arguments.remove(receiver_index);
        let callee = match self.types.kind(target.concrete) {
            Some(Kind::FunctionItem { .. }) => {
                Operand::Symbol(self.demand(Instance::new(self.types, target.concrete)?)?)
            }
            Some(Kind::Closure {
                environment: TypeId::UNIT,
                ..
            }) => Operand::Symbol(self.demand_closure(target.concrete)?),
            Some(Kind::Function { .. }) => builder.load(Type::Ptr, receiver.operand)?.operand,
            _ => return Err("object callable target requires a checked captured adapter"),
        };
        let native = self.output.scalar_signature(signature)?;
        let result = builder.call(signature, native.result, callee, &arguments)?;
        builder.terminate(if native.noreturn {
            Terminator::Unreachable
        } else {
            Terminator::Return(result)
        })?;
        self.output.define(symbol, builder.finish()?)?;
        self.object_callables
            .get_or_insert_with(Default::default)
            .entries
            .insert(target, symbol);
        Ok(symbol)
    }
}
