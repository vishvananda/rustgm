//! Linux C entry forwards the host arguments through the proved Source startup.
use super::{Builder, Context, Operand, SymbolId, Terminator, Type};
use crate::lowir::{program::Value, vocabulary::Convert};

#[derive(Clone, Copy)]
pub(super) struct Symbols {
    target: SymbolId,
    callback: SymbolId,
}

impl Context<'_> {
    pub(super) fn demand_startup(
        &mut self,
        startup: crate::sema::entry::Startup,
    ) -> Result<Symbols, &'static str> {
        Ok(Symbols {
            target: self.demand(startup.target)?,
            callback: self.demand_reified(startup.callback)?,
        })
    }

    pub(super) fn startup_entry(&mut self, symbols: Symbols) -> Result<(), &'static str> {
        let signature = self
            .output
            .signature(&[Type::I32, Type::Ptr], Type::I32, false)?;
        let entry = self.output.declare("main".into(), signature, true)?;
        let target = self.output.symbols[symbols.target.0 as usize].signature;
        let mut builder = Builder::default();
        let block = builder.block()?;
        builder.enter(block)?;
        self.initialize_addresses(&mut builder)?;
        let callback = builder.address(Operand::Symbol(symbols.callback))?;
        let count = builder.convert(
            Convert::Sext,
            Type::I64,
            Value {
                ty: Type::I32,
                operand: Operand::Parameter(0),
            },
        )?;
        let argv = Value {
            ty: Type::Ptr,
            operand: Operand::Parameter(1),
        };
        let sigpipe = builder.integer(Type::U8, 0)?;
        let result = builder
            .call(
                target,
                Type::I64,
                Operand::Symbol(symbols.target),
                &[callback, count, argv, sigpipe],
            )?
            .ok_or("source start has no native exit status")?;
        let result = builder.convert(Convert::Trunc, Type::I32, result)?;
        builder.terminate(Terminator::Return(Some(result)))?;
        self.output.define(entry, builder.finish()?)
    }
}
