//! A normal pointer ABI forwards to the tracked target with its definition site.
use super::Context;
use crate::{
    lowir::{
        data::GlobalId,
        program::{Operand, SymbolId, Terminator, Value},
    },
    resolve::model::DefId,
    sema::values::LocationId,
    types::{TypeId, instance::Instance},
};

impl Context<'_> {
    pub(in crate::lower) fn caller_location_global(
        &mut self,
        location: LocationId,
    ) -> Result<GlobalId, &'static str> {
        self.constants.caller_location_value(location)?;
        let data = self.constants.location(location)?;
        self.constant_data(
            super::super::data::Key::Location(location),
            data.value.ok_or("caller allocation is uninitialized")?,
            data.ty,
            true,
        )
    }

    pub(super) fn tracked_reification(
        &mut self,
        instance: Instance,
        target: DefId,
        signature: TypeId,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self.reified_symbols.get(&instance) {
            return Ok(symbol);
        }
        let location = self
            .constants
            .reified_location(target)
            .ok_or("tracked reification lacks the prepared declaration location")?;
        let caller_location = self.caller_location_global(location)?;
        let entry = self.signature(signature)?;
        let tracked = self.caller_signature(signature, true)?;
        let callee = self.demand(instance)?;
        let actual = self
            .output
            .symbols
            .get(callee.0 as usize)
            .ok_or("tracked reification lost its called symbol")?
            .signature;
        if !self.output.compatible(actual, tracked)? {
            return Err("tracked reification differs from its checked target ABI");
        }
        let native = self.output.scalar_signature(entry)?;
        let (result, noreturn) = (native.result, native.noreturn);
        let parameters = self.output.parameters(entry)?;
        let growth = crate::support::retained::Growth::map::<Instance, SymbolId>(
            self.reified_symbols.len(),
            self.reified_symbols.capacity(),
            1,
        );
        self.ledger.charge_caller_inputs(
            growth
                .work
                .saturating_add(parameters.len())
                .saturating_add(1),
            growth.bytes.saturating_add(
                parameters
                    .len()
                    .saturating_mul(2)
                    .saturating_add(1)
                    .saturating_mul(size_of::<Value>()),
            ),
        )?;
        self.reified_symbols
            .try_reserve(1)
            .map_err(|_| "tracked reification symbol inventory allocation failed")?;
        let mut arguments = crate::lower::virtual_dispatch::parameters(&self.output, entry)?;
        arguments
            .try_reserve_exact(1)
            .map_err(|_| "tracked reification argument allocation failed")?;
        let mut builder = self.output.body_builder();
        let block = builder.block()?;
        builder.enter(block)?;
        arguments.push(builder.address(Operand::Global(caller_location))?);
        let value = builder.call(tracked, result, Operand::Symbol(callee), &arguments)?;
        builder.terminate(if noreturn {
            Terminator::Unreachable
        } else {
            Terminator::Return(value)
        })?;
        let name = self.internal_name(format!("rg.reified.{}", self.reified_symbols.len()));
        let symbol = self.output.declare(name, entry, false)?;
        self.output.define(symbol, builder.finish()?)?;
        self.reified_symbols.insert(instance, symbol);
        Ok(symbol)
    }
}
