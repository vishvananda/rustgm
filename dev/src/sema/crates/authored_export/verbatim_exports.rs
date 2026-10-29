//! Exact definitions consume the already checked foreign registry during function emission.
use super::{DeclarationRef, Declarations, Stage, s};
use crate::support::linkage::{Linkage, Naming};
impl Stage {
    pub(super) fn exact_callable(
        &mut self,
        facts: &Declarations,
        function: &crate::sema::model::Function,
        declaration: DeclarationRef,
    ) -> Result<(), &'static str> {
        let Some(registry) = facts.foreign.as_deref() else {
            return Ok(());
        };
        let Some(record) = registry.function(function.definition) else {
            return Ok(());
        };
        if record.naming == Naming::Rust && record.linkage == Linkage::Internal {
            return Ok(());
        }
        let Naming::Exact(id) = record.naming else {
            return Err("source foreign callable requires its wider explicit profile");
        };
        if !self.exact_linkage()
            || record.linkage != Linkage::Export
            || record.signature != function.signature
            || function.body.is_none()
        {
            return Err(
                "source exact callable requires actual version-five exported function/original",
            );
        }
        let symbol = registry
            .names()
            .get(id)
            .ok_or("exact source callable lost its actual symbol pool")?;
        self.charge(symbol.len().saturating_add(1))?;
        self.reserve(4 * size_of::<s::ExactCallableExport>() + symbol.len())?;
        let body = self
            .functions
            .bodies
            .get(&function.definition)
            .copied()
            .ok_or("exact callable has no actual original")?;
        self.rows.exact_callables.push(s::ExactCallableExport {
            declaration,
            body,
            symbol: symbol.into(),
        });
        Ok(())
    }
}
