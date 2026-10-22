//! Iteration metadata has real library owners and no generated source syntax.
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{model::ScopeId, package::ModuleId, unit::NodeKey},
    support::{
        iteration::{Blanket as Kind, Trait},
        library_types::LibraryType,
    },
    syntax::ast::NodeId,
};
#[derive(Clone, Copy)]
pub struct Declaration {
    pub kind: Trait,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub item: DefId,
    pub into_iter: Option<DefId>,
    pub method: DefId,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Blanket {
    pub kind: Kind,
    pub definition: DefId,
    pub parameter: DefId,
    pub lifetime: Option<DefId>,
    pub item: DefId,
    pub into_iter: Option<DefId>,
    pub method: DefId,
}
#[derive(Clone, Copy)]
pub struct Iteration {
    pub origin: NodeKey,
    pub module: ScopeId,
    pub option: DefId,
    pub declarations: [Declaration; 2],
    pub blankets: [Blanket; 2],
}
impl Iteration {
    #[must_use]
    pub const fn get(self, kind: Trait) -> Declaration {
        self.declarations[kind as usize]
    }
}
#[derive(Default)]
pub(super) struct Registry {
    origin: Option<(ModuleId, NodeKey)>,
    complete: Option<Iteration>,
    pub(super) collections: Option<Box<super::collections::Collections>>,
    collection_origin: Option<(ModuleId, NodeKey)>,
    named_iterator: bool,
}
impl Traits {
    pub(in crate::resolve) fn note_iteration(&mut self, module: ModuleId, source: NodeKey) {
        self.iteration
            .get_or_insert_default()
            .origin
            .get_or_insert((module, source));
    }
    pub(in crate::resolve) fn note_collection(&mut self, module: ModuleId, source: NodeKey) {
        self.note_iteration(module, source);
        self.iteration
            .get_or_insert_default()
            .collection_origin
            .get_or_insert((module, source));
    }
    pub(in crate::resolve) fn name_slice_iterator(&mut self, module: ModuleId, source: NodeKey) {
        self.note_collection(module, source);
        self.iteration.get_or_insert_default().named_iterator = true;
    }
    #[must_use]
    pub fn named_slice_iterator(&self) -> bool {
        self.iteration.as_deref().is_some_and(|r| r.named_iterator)
    }
    pub(super) fn collection_origin(&self) -> Option<(ModuleId, NodeKey)> {
        self.iteration.as_deref()?.collection_origin
    }
    #[must_use]
    pub fn collections(&self) -> Option<&super::collections::Collections> {
        self.iteration.as_deref()?.collections.as_deref()
    }
    #[must_use]
    pub fn iteration(&self) -> Option<Iteration> {
        self.iteration.as_deref()?.complete
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn reserve_iteration(&mut self) -> Result<(), Error> {
        let Some((module, source)) = self
            .facts
            .builtin_traits
            .iteration
            .as_deref()
            .and_then(|r| r.origin)
        else {
            return Ok(());
        };
        self.prepare_iteration(
            Context {
                module,
                file: source.file,
                scope: self.facts.module_scopes[module.0 as usize],
                foreign: None,
            },
            source.node,
        )?;
        self.reserve_collections()
    }
    pub(in crate::resolve) fn prepare_iteration(
        &mut self,
        c: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.iteration().is_some() {
            return Ok(());
        }
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        self.prepare_library_type(c, node, LibraryType::Option)?;
        let option = self
            .facts
            .library_types
            .get(LibraryType::Option)
            .ok_or_else(|| self.error(c, node, "missing iteration Option identity"))?
            .definition;
        let library = self
            .facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing iteration library"))?;
        let module = self.new_scope(None, ModuleId(0), true)?;
        self.builtin_definition(
            Context {
                scope: library,
                ..c
            },
            node,
            "iter",
            DefKind::Module,
            Some(module),
        )?;
        let iterator = self.iteration_declaration(c, node, module, Trait::Iterator)?;
        let into = self.iteration_declaration(c, node, module, Trait::IntoIterator)?;
        let identity = self.iteration_blanket(c, node, Kind::Identity)?;
        let reference = self.iteration_blanket(c, node, Kind::MutableReference)?;
        self.facts
            .builtin_traits
            .iteration
            .get_or_insert_default()
            .complete = Some(Iteration {
            origin: NodeKey { file: c.file, node },
            module,
            option,
            declarations: [iterator, into],
            blankets: [identity, reference],
        });
        Ok(())
    }
    fn iteration_declaration(
        &mut self,
        c: Context,
        node: NodeId,
        module: ScopeId,
        kind: Trait,
    ) -> Result<Declaration, Error> {
        let members = self.new_scope(None, ModuleId(0), true)?;
        self.facts.scopes[members.0 as usize].associated = true;
        let definition = self.builtin_definition(
            Context { scope: module, ..c },
            node,
            kind.name(),
            DefKind::BuiltinTrait,
            Some(members),
        )?;
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let self_parameter =
            self.builtin_definition(Context { scope, ..c }, node, "Self", DefKind::Generic, None)?;
        let item = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            "Item",
            DefKind::TypeAlias,
            None,
        )?;
        let into_iter = if kind == Trait::IntoIterator {
            Some(self.builtin_definition(
                Context {
                    scope: members,
                    ..c
                },
                node,
                "IntoIter",
                DefKind::TypeAlias,
                None,
            )?)
        } else {
            None
        };
        let method = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            kind.method().name(),
            DefKind::Function,
            None,
        )?;
        Ok(Declaration {
            kind,
            definition,
            self_parameter,
            item,
            into_iter,
            method,
        })
    }
    fn iteration_blanket(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Kind,
    ) -> Result<Blanket, Error> {
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let definition = self.builtin_definition(
            Context { scope, ..c },
            node,
            "Self",
            DefKind::SelfType,
            None,
        )?;
        let parameter =
            self.builtin_definition(Context { scope, ..c }, node, "I", DefKind::Generic, None)?;
        let lifetime = if kind == Kind::MutableReference {
            Some(self.builtin_definition(
                Context { scope, ..c },
                node,
                "'a",
                DefKind::Lifetime,
                None,
            )?)
        } else {
            None
        };
        let members = self.new_scope(None, ModuleId(0), true)?;
        let item = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            "Item",
            DefKind::TypeAlias,
            None,
        )?;
        let into_iter = if kind == Kind::Identity {
            Some(self.builtin_definition(
                Context {
                    scope: members,
                    ..c
                },
                node,
                "IntoIter",
                DefKind::TypeAlias,
                None,
            )?)
        } else {
            None
        };
        let method = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            if kind == Kind::Identity {
                "into_iter"
            } else {
                "next"
            },
            DefKind::Function,
            None,
        )?;
        Ok(Blanket {
            kind,
            definition,
            parameter,
            lifetime,
            item,
            into_iter,
            method,
        })
    }
}
