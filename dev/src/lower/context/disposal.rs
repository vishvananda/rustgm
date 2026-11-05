//! Only the existing generated-entry index resolves a proved future Drop body.
use super::{Context, Generated, Owner, SymbolId};
use crate::types::future::{EntryKey, Operation};
impl Context<'_> {
    pub(in crate::lower) fn future_drop(
        &mut self,
        key: EntryKey,
    ) -> Result<SymbolId, &'static str> {
        if key.operation != Operation::Drop {
            return Err("future cleanup requested another generated operation");
        }
        let &index = self
            .adapter_indices
            .get(&Generated::Future(key))
            .ok_or("missing checked future Drop entry")?;
        let body = self.body(index);
        let Owner::FutureEntry {
            key: selected,
            signature,
            ..
        } = body.owner
        else {
            return Err("future Drop index has another actual owner");
        };
        if selected != key || body.instance.is_some() {
            return Err("future Drop index changed its full application or operation");
        }
        if let Some(symbol) = self.symbols[index] {
            return Ok(symbol);
        }
        let signature = self.signature(signature)?;
        let name = self.internal_name(format!("rg.future.drop.i{index}"));
        let symbol = self.output.declare(name, signature, false)?;
        self.symbols[index] = Some(symbol);
        self.queue.push(index);
        Ok(symbol)
    }
}
