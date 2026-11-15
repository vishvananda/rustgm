//! Bring-up only: a source intrinsic without run-time lowering traps when reached.
use super::{Builder, Context, Instance, Operand, SymbolId, Terminator, Type, TypeId};

impl Context<'_> {
    /// A shim with the call's signature that aborts. The waiver names the
    /// intrinsic; outside the bring-up profile the fence stands.
    pub(in crate::lower) fn demand_trap(
        &mut self,
        instance: Instance,
        signature: TypeId,
        name: &str,
    ) -> Result<SymbolId, &'static str> {
        let signature = self.signature(signature)?;
        if let Some(&symbol) = self.intrinsic_symbols.get(&instance) {
            if self.output.symbols[symbol.0 as usize].signature != signature {
                return Err("trapping intrinsic shim is called with two signatures");
            }
            return Ok(symbol);
        }
        if self.intrinsic_symbols.len() >= 100_000 {
            return Err("intrinsic shim count exceeds 100000");
        }
        let fence = format!("source intrinsic executable lowering is not implemented: {name}");
        if !crate::support::bringup::waive(&fence) {
            return Err("source intrinsic executable lowering is not implemented");
        }
        let abort = self.abort()?;
        let aborting = self.output.symbols[abort.0 as usize].signature;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        builder.call(aborting, Type::Void, Operand::Symbol(abort), &[])?;
        builder.terminate(Terminator::Unreachable)?;
        let symbol = self.output.declare(
            self.internal_name(format!("rg.trap.{name}.t{}", instance.ty().0)),
            signature,
            false,
        )?;
        self.output.define(symbol, builder.finish()?)?;
        self.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }
}
