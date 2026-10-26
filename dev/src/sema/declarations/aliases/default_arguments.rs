//! Unused alias defaults check written clauses, independently of alias expansion.
use super::{Checker, Error};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn alias_parameter_defaults(&mut self) -> Result<(), Error> {
        let count = self
            .trait_aliases
            .as_ref()
            .map_or(0, |table| table.headers.len());
        for index in 0..count {
            let pending = self
                .trait_aliases
                .as_ref()
                .ok_or_else(|| Error::unlocated("missing alias default owner"))?;
            let header = &pending.headers[index];
            let owner = header.owner;
            let parameters = self
                .facts
                .generics
                .parameters
                .get(&owner.definition)
                .ok_or_else(|| self.error(owner.source, "unfinished alias default parameters"))?;
            // Most aliases have no defaults and need no temporary clause storage.
            if !parameters
                .iter()
                .any(|parameter| parameter.default.is_some())
            {
                continue;
            }
            // Rust checks the alias's written default bounds here, not every
            // requirement inherited through an alias edge. The same ordinary
            // declaration table now supplies these clauses without copying.
            let mut proof = self.declaration_proof(owner.definition, None);
            self.parameter_defaults(owner.definition, owner.source, &mut proof)?;
            self.retain_trait_proof(owner.definition, owner.source, &mut proof);
        }
        Ok(())
    }
}
