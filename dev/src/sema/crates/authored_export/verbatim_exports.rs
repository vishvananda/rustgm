//! Exact native linkage consumes the already checked foreign registry.
use super::{DeclarationRef, Declarations, Report, Stage, s};
use crate::{
    resolve::model::DefId,
    support::linkage::{Linkage, Naming, PANIC_HANDLER_SYMBOL},
};

impl Stage {
    pub(super) fn exact_callable(
        &mut self,
        report: &Report,
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
        if !self.exact_linkage() || record.signature != function.signature {
            return Err("source exact callable changed its actual checked signature or profile");
        }
        let symbol = registry
            .names()
            .get(id)
            .ok_or("exact source callable lost its actual symbol pool")?;
        let weak_panic = self.declaration_policies()
            && record.linkage == Linkage::Import
            && function.body.is_none()
            && symbol == PANIC_HANDLER_SYMBOL
            && facts
                .weak_panic_import(report, function.definition)
                .map_err(|_| "weak panic import lost its checked source claim")?;
        let body = match record.linkage {
            Linkage::Export if function.body.is_some() => Some(
                self.functions
                    .bodies
                    .get(&function.definition)
                    .copied()
                    .ok_or("exact callable has no actual original")?,
            ),
            Linkage::Import
                if self.declaration_policies()
                    && function.body.is_none()
                    && (weak_panic || runtime_import(facts, function.definition)?) =>
            {
                None
            }
            _ => return Err("source exact callable requires an actual original or runtime import"),
        };
        self.charge(symbol.len().saturating_add(1))?;
        self.reserve(4 * size_of::<s::ExactCallable>() + symbol.len())?;
        self.rows.exact_callables.push(s::ExactCallable {
            declaration,
            body,
            weak_panic,
            symbol: symbol.into(),
        });
        Ok(())
    }
}

fn runtime_import(facts: &Declarations, definition: DefId) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    for role in crate::sema::compiler_items::WEAK_IMPORTS {
        if catalog
            .validated_role(role)?
            .is_some_and(|actual| actual.definition == definition)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
