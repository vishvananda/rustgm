//! One consumed source-handler batch joins the existing exact-symbol registry.
use super::{Function, Item, Registry};
use crate::{
    sema::compiler_items::HandlerBindings,
    support::{
        linkage::{Linkage, Naming},
        retained::Growth,
    },
};
const SYMBOL: &str = "rust_begin_unwind";
impl Registry {
    pub(in crate::sema) fn bind_handler(
        &mut self,
        batch: HandlerBindings,
        mut pay: impl FnMut(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let (handler, imports) = batch.into_parts();
        pay(Growth {
            bytes: 0,
            work: imports.len().saturating_add(1),
        })?;
        let name = self.names.find_exact_paid(SYMBOL, &mut pay)?;
        if let Some(name) = name
            && self
                .export_names
                .get(&name)
                .is_some_and(|&actual| handler.is_none_or(|(definition, _)| actual != definition))
        {
            return Err("multiple definitions export the actual panic-handler symbol");
        }
        if let Some((definition, handler)) = handler
            && let Some(previous) = self.items.get(&definition)
        {
            let Item::Function(previous) = *previous else {
                return Err("panic handler conflicts with existing storage");
            };
            same_callable(previous, handler)?;
            self.handler_naming(previous, Linkage::Export)?;
        }
        for &(definition, expected) in &imports {
            let previous = self
                .function(definition)
                .ok_or("weak panic import lost its canonical function record")?;
            if previous != expected || previous.linkage != Linkage::Import {
                return Err("weak panic import changed its actual canonical record");
            }
            // Private Bindings authenticates the actual default source name
            // before this transaction consumes its unchanged original record.
        }
        if let Some((definition, _)) = handler {
            pay(self.handler_growth(name, definition))?;
            // All callable/collision/payment checks precede the first published row.
            self.items
                .try_reserve(usize::from(!self.items.contains_key(&definition)))
                .map_err(|_| "panic callable registry allocation failed")?;
            self.export_names
                .try_reserve(usize::from(
                    name.is_none_or(|id| !self.export_names.contains_key(&id)),
                ))
                .map_err(|_| "panic export registry allocation failed")?;
        }
        let name = self.names.intern_paid(SYMBOL, &mut pay)?;
        if let Some((definition, handler)) = handler {
            self.export_names.insert(name, definition);
            self.items.insert(
                definition,
                Item::Function(Function {
                    naming: Naming::Exact(name),
                    linkage: Linkage::Export,
                    ..handler
                }),
            );
        }
        for &(definition, original) in &imports {
            self.items.insert(
                definition,
                Item::Function(Function {
                    naming: Naming::Exact(name),
                    ..original
                }),
            );
        }
        Ok(())
    }
    fn handler_growth(
        &self,
        name: Option<crate::support::linkage::SymbolNameId>,
        definition: super::DefId,
    ) -> Growth {
        let mut growth = Growth::map::<super::DefId, Item>(
            self.items.len(),
            self.items.capacity(),
            usize::from(!self.items.contains_key(&definition)),
        );
        growth.include(Growth::map::<
            crate::support::linkage::SymbolNameId,
            super::DefId,
        >(
            self.export_names.len(),
            self.export_names.capacity(),
            usize::from(name.is_none_or(|id| !self.export_names.contains_key(&id))),
        ));
        growth
    }
    fn handler_naming(&self, function: Function, desired: Linkage) -> Result<(), &'static str> {
        if !matches!(function.linkage, Linkage::Internal) && function.linkage != desired {
            return Err("panic linkage changed its import/export ownership");
        }
        if let Naming::Exact(name) = function.naming
            && self.names.get(name) != Some(SYMBOL)
        {
            return Err("custom panic-handler linkage adaptation is not implemented");
        }
        Ok(())
    }
}
fn same_callable(actual: Function, expected: Function) -> Result<(), &'static str> {
    if actual.signature != expected.signature || actual.safety != expected.safety {
        return Err("panic linkage changed its actual callable signature or safety");
    }
    Ok(())
}
