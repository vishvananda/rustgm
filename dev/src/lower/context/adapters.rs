//! Checked receiver adapters retain separate demanded symbols and ordinary execution ABI.
use super::{Context, Owner, SymbolId, TypeId};
impl Context<'_> {
    pub(super) fn adapter_symbol(
        &mut self,
        entry: crate::sema::vtable_plans::Captured,
    ) -> Result<SymbolId, &'static str> {
        let adapter = entry.adapter.ok_or("missing selected receiver adapter")?;
        adapter.validate(self.types)?;
        if adapter.instance() != entry.instance || adapter.requested() != entry.requested {
            return Err("receiver adapter demand differs from selected identity");
        }
        let &index = self
            .adapter_indices
            .get(&adapter)
            .ok_or("missing checked receiver adapter body")?;
        let Owner::ClosureAdapter {
            adapter: selected,
            signature,
        } = self.bodies.bodies[index].owner
        else {
            return Err("receiver adapter body has another owner");
        };
        if selected != adapter {
            return Err("receiver adapter body has another selected instance");
        }
        self.entry_abi(entry.execution, signature)?;
        if let Some(symbol) = self.symbols[index] {
            return Ok(symbol);
        }
        let signature = self.signature(signature)?;
        let name = self.internal_name(format!(
            "rg.closure.adapter.{}.i{index}",
            adapter.requested().name()
        ));
        let symbol = self.output.declare(name, signature, false)?;
        self.symbols[index] = Some(symbol);
        self.queue.push(index);
        Ok(symbol)
    }
    pub(super) fn entry_abi(&mut self, actual: TypeId, formal: TypeId) -> Result<(), &'static str> {
        let actual = self.signature(actual)?;
        let formal = self.signature(formal)?;
        if !self.output.compatible(actual, formal)? {
            return Err("proved closure entry ABI differs from execution body");
        }
        Ok(())
    }
}
