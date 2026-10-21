//! Foreign records use canonical definitions; bodies and const permissions stay separate.
use crate::{
    resolve::model::DefId,
    support::linkage::{Linkage, Naming, SymbolNames},
    syntax::item::Safety,
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Function {
    pub signature: TypeId,
    pub safety: Safety,
    pub naming: Naming,
    pub linkage: Linkage,
}
#[derive(Default)]
pub struct Registry {
    names: SymbolNames,
    functions: HashMap<DefId, Function>,
}
impl Registry {
    #[must_use]
    pub fn names(&self) -> &SymbolNames {
        &self.names
    }
    pub fn names_mut(&mut self) -> &mut SymbolNames {
        &mut self.names
    }
    #[must_use]
    pub fn function(&self, definition: DefId) -> Option<Function> {
        self.functions.get(&definition).copied()
    }
    /// Register an import with an exact name in this registry's own pool.
    ///
    /// # Errors
    /// Rejects invalid symbol bytes, exhausted identities or conflicting declarations.
    pub fn import_function(
        &mut self,
        definition: DefId,
        signature: TypeId,
        safety: Safety,
        spelling: &str,
    ) -> Result<(), &'static str> {
        let naming = Naming::Exact(self.names.intern(spelling)?);
        let function = Function {
            signature,
            safety,
            naming,
            linkage: Linkage::Import,
        };
        self.insert_function(definition, function)
    }
    /// Insert a checked canonical import or export, including source-free metadata.
    /// Exact names must belong to this registry's pool; metadata reinterns the bytes.
    ///
    /// # Errors
    /// Rejects absent exact names and conflicting declarations.
    pub fn insert_function(
        &mut self,
        definition: DefId,
        function: Function,
    ) -> Result<(), &'static str> {
        if let Naming::Exact(id) = function.naming
            && self.names.get(id).is_none()
        {
            return Err("missing foreign symbol spelling in its owning pool");
        }
        if let Some(previous) = self.functions.get(&definition) {
            if previous != &function {
                return Err("conflicting foreign function identity");
            }
            return Ok(());
        }
        self.functions.insert(definition, function);
        Ok(())
    }
}
