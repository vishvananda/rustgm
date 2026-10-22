//! Collection owners are reserved once, after ordinary source identities.
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::{iteration::Collection, library_types::LibraryType},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionDeclaration {
    pub kind: Collection,
    pub definition: DefId,
    pub parameter: DefId,
    pub lifetime: Option<DefId>,
    pub extent: Option<DefId>,
    pub item: Option<DefId>,
    pub into_iter: Option<DefId>,
    pub method: DefId,
}
pub struct Collections {
    pub origin: NodeKey,
    pub iterator: DefId,
    pub declarations: [CollectionDeclaration; 4],
}
impl Resolver<'_> {
    pub(super) fn reserve_collections(&mut self) -> Result<(), Error> {
        let Some((module, origin)) = self.facts.builtin_traits.collection_origin() else {
            return Ok(());
        };
        if self.facts.builtin_traits.collections().is_some() {
            return Ok(());
        }
        let c = Context {
            module,
            file: origin.file,
            scope: self.facts.module_scopes[module.0 as usize],
            foreign: None,
        };
        self.prepare_library_type(c, origin.node, LibraryType::SliceIter)?;
        let iterator = self
            .facts
            .library_types
            .get(LibraryType::SliceIter)
            .ok_or_else(|| self.error(c, origin.node, "missing slice iterator identity"))?
            .definition;
        let mut entries = Vec::new();
        for kind in Collection::ALL {
            entries.push(self.collection_declaration(c, origin, kind)?);
        }
        let declarations = entries
            .try_into()
            .map_err(|_| self.error(c, origin.node, "invalid collection schema count"))?;
        self.facts.builtin_traits.install_collections(Collections {
            origin,
            iterator,
            declarations,
        })?;
        Ok(())
    }
    fn collection_declaration(
        &mut self,
        c: Context,
        origin: NodeKey,
        kind: Collection,
    ) -> Result<CollectionDeclaration, Error> {
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let c = Context { scope, ..c };
        let definition =
            self.builtin_definition(c, origin.node, "Self", DefKind::SelfType, None)?;
        let parameter = self.builtin_definition(c, origin.node, "T", DefKind::Generic, None)?;
        let lifetime = if kind == Collection::SliceMethod {
            None
        } else {
            Some(self.builtin_definition(c, origin.node, "'a", DefKind::Lifetime, None)?)
        };
        let extent = if kind == Collection::SharedArray {
            Some(self.builtin_definition(c, origin.node, "N", DefKind::Generic, None)?)
        } else {
            None
        };
        let item = if kind.trait_().is_some() {
            Some(self.builtin_definition(c, origin.node, "Item", DefKind::TypeAlias, None)?)
        } else {
            None
        };
        let into_iter = if matches!(kind, Collection::SharedArray | Collection::SharedSlice) {
            Some(self.builtin_definition(c, origin.node, "IntoIter", DefKind::TypeAlias, None)?)
        } else {
            None
        };
        let name = kind.trait_().map_or("iter", |t| t.method().name());
        let method = self.builtin_definition(c, origin.node, name, DefKind::Function, None)?;
        Ok(CollectionDeclaration {
            kind,
            definition,
            parameter,
            lifetime,
            extent,
            item,
            into_iter,
            method,
        })
    }
}
impl Traits {
    fn install_collections(&mut self, collections: Collections) -> Result<(), Error> {
        let registry = self
            .iteration
            .as_deref_mut()
            .ok_or_else(|| Error::at(collections.origin.file, 0, "missing iteration registry"))?;
        registry.collections = Some(Box::new(collections));
        Ok(())
    }
}
