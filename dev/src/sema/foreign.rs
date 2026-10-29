//! Canonical foreign records keep storage, bodies and const permissions separate.
mod handler_linkage;
mod prepared_functions;
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    support::linkage::{Linkage, Naming, SymbolNameId, SymbolNames},
    syntax::item::Safety,
    types::TypeId,
};
pub(crate) use prepared_functions::PreparedFunctions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Function {
    pub signature: TypeId,
    pub safety: Safety,
    pub naming: Naming,
    pub linkage: Linkage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Global {
    pub ty: TypeId,
    pub mutable: bool,
    pub safety: Safety,
    pub naming: Naming,
    pub linkage: Linkage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Function(Function),
    Global(Global),
}
impl Item {
    fn descriptor(self) -> (Naming, Linkage) {
        match self {
            Self::Function(f) => (f.naming, f.linkage),
            Self::Global(g) => (g.naming, g.linkage),
        }
    }
}
#[derive(Default)]
pub struct Registry {
    names: SymbolNames,
    items: HashMap<DefId, Item>,
    export_names: HashMap<SymbolNameId, DefId>,
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
        match self.items.get(&definition)? {
            Item::Function(f) => Some(*f),
            Item::Global(_) => None,
        }
    }
    #[must_use]
    pub fn global(&self, definition: DefId) -> Option<Global> {
        match self.items.get(&definition)? {
            Item::Global(g) => Some(*g),
            Item::Function(_) => None,
        }
    }
    #[must_use]
    pub fn imports_global(&self, definition: DefId) -> bool {
        self.global(definition)
            .is_some_and(|global| global.linkage == Linkage::Import)
    }
    pub(crate) fn all_exports(&self) -> (Vec<DefId>, Vec<DefId>) {
        let (mut functions, mut globals) = (Vec::new(), Vec::new());
        for (&id, &item) in &self.items {
            if item.descriptor().1 != Linkage::Export {
                continue;
            }
            match item {
                Item::Function(_) => functions.push(id),
                Item::Global(_) => globals.push(id),
            }
        }
        functions.sort_unstable_by_key(|id| id.0);
        globals.sort_unstable_by_key(|id| id.0);
        (functions, globals)
    }
    /// Canonical callable export roots in deterministic definition order.
    #[must_use]
    pub fn exports(&self) -> Vec<DefId> {
        self.export_roots(false)
    }
    /// Global export roots remain distinct from callable roots.
    #[must_use]
    pub fn global_exports(&self) -> Vec<DefId> {
        self.export_roots(true)
    }
    fn export_roots(&self, globals: bool) -> Vec<DefId> {
        let mut exports: Vec<_> = self
            .items
            .iter()
            .filter_map(|(&id, &item)| {
                let (_, linkage) = item.descriptor();
                (linkage == Linkage::Export && matches!(item, Item::Global(_)) == globals)
                    .then_some(id)
            })
            .collect();
        exports.sort_unstable_by_key(|id| id.0);
        exports
    }
    /// Register an import with an exact name in this registry's own pool.
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
        self.insert_function(
            definition,
            Function {
                signature,
                safety,
                naming,
                linkage: Linkage::Import,
            },
        )
    }
    /// Insert a canonical function. Metadata reinterns exact bytes in this pool.
    /// # Errors
    /// Rejects absent exact names and conflicting declarations or definitions.
    pub fn insert_function(
        &mut self,
        definition: DefId,
        function: Function,
    ) -> Result<(), &'static str> {
        self.insert(definition, Item::Function(function))
    }
    /// Insert canonical storage without inventing an initializer for imports.
    /// # Errors
    /// Rejects absent exact names and conflicting declarations or definitions.
    pub fn insert_global(&mut self, definition: DefId, global: Global) -> Result<(), &'static str> {
        self.insert(definition, Item::Global(global))
    }
    fn insert(&mut self, definition: DefId, item: Item) -> Result<(), &'static str> {
        let (naming, linkage) = item.descriptor();
        if let Naming::Exact(id) = naming
            && self.names.get(id).is_none()
        {
            return Err("missing foreign symbol spelling in its owning pool");
        }
        if let Some(previous) = self.items.get(&definition) {
            if previous != &item {
                return Err("conflicting foreign item identity");
            }
            return Ok(());
        }
        if linkage == Linkage::Export
            && let Naming::Exact(name) = naming
        {
            if self
                .export_names
                .get(&name)
                .is_some_and(|&owner| owner != definition)
            {
                return Err("multiple definitions export the same exact symbol");
            }
            self.export_names.insert(name, definition);
        }
        self.items.insert(definition, item);
        Ok(())
    }
}
