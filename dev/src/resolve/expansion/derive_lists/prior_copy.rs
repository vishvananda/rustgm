//! A prior Copy attribute must have completed on this exact live source owner.
use super::{Body, Builtin, Engine, Error, Sources, receipt};
impl<S: Sources> Engine<'_, S> {
    pub(super) fn completed_copy(&self, index: usize) -> Result<bool, Error> {
        let current = self.derive_entry(index)?;
        let unit = self.loader.package.unit(current.owner.file);
        let owner = unit.owner_indices[current.owner.node.get() as usize]
            .and_then(|owner| unit.configured.owners.get(owner.get() as usize - 1))
            .ok_or_else(|| {
                self.calls[index].diagnostic("prior Copy lost its actual configured owner")
            })?;
        let transactions = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(|| {
                self.calls[index].diagnostic("prior Copy lost its actual transaction owner")
            })?;
        let source = self.loader.package.source_core().ok_or_else(|| {
            self.calls[index].diagnostic("prior Copy requires its actual source provider")
        })?;
        for effect in owner.attributes.start..current.effect {
            source
                .macros
                .charge(1, 0)
                .map_err(|message| self.calls[index].diagnostic(message))?;
            let Some(&position) = transactions.index.get(&(current.owner.file, effect)) else {
                continue;
            };
            let prior = &transactions.entries[position];
            if prior.owner != current.owner || prior.output.is_none() {
                continue;
            }
            receipt::current(&self.loader.package, &self.paths.state.facts, prior, true)?;
            for member in prior.contexts.clone() {
                source
                    .macros
                    .charge(1, 0)
                    .map_err(|message| self.calls[index].diagnostic(message))?;
                let invocation = &self.invocations[member];
                if invocation.node != current.owner
                    || invocation.generated != self.invocations[prior.invocation].generated
                {
                    return Err(self.calls[index]
                        .diagnostic("prior Copy receipt has a different actual owner or image"));
                }
                let definition = &self.definitions[invocation.definition];
                if matches!(definition.body, Body::Builtin(Builtin::Copy)) {
                    self.checked_derive_definition(invocation.definition, Builtin::Copy)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
