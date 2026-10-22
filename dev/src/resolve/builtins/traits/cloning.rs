//! Clone's callable identity is demanded separately from its marker-bound spelling.
use super::{BuiltinTrait, Context, DefId, DefKind, Error, NameId, Resolver, Traits};
use crate::resolve::{
    lookup::{Lookup, LookupMode},
    model::Namespace,
    package::ModuleId,
    unit::NodeKey,
};
use crate::syntax::ast::NodeId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Implementation {
    pub definition: DefId,
    pub lifetime: DefId,
    pub parameter: DefId,
    pub method: DefId,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Declaration {
    pub origin: NodeKey,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub method: DefId,
    pub implementation: Option<Implementation>,
}
#[derive(Default)]
pub(super) struct Registry {
    origin: Option<(ModuleId, NodeKey)>,
    complete: Option<Declaration>,
}
impl Traits {
    #[must_use]
    pub fn cloning(&self) -> Option<Declaration> {
        self.cloning.as_deref()?.complete
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn note_clone_name(&mut self, c: Context, node: NodeId, name: NameId) {
        if self.package.names.get(name) == "clone" {
            self.facts
                .builtin_traits
                .cloning
                .get_or_insert_default()
                .origin
                .get_or_insert((c.module, NodeKey { file: c.file, node }));
        }
    }
    pub(in crate::resolve) fn prepare_clone_path(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: &[NameId],
    ) -> Result<(), Error> {
        let Some((&name, prefix)) = path.split_last() else {
            return Ok(());
        };
        self.note_clone_name(c, node, name);
        if self.package.names.get(name) != "clone" || prefix.is_empty() {
            return Ok(());
        }
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        let library = self
            .facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing Clone library"))?;
        self.trait_module(c, node, library, true)?;
        let canonical = self
            .facts
            .builtin_traits
            .get(BuiltinTrait::Clone)
            .ok_or_else(|| self.error(c, node, "missing canonical Clone"))?;
        if matches!(self.lookup_path(c, node, absolute, prefix, LookupMode::Import(Namespace::Type), None),
            Ok(Lookup::Found(binding)) if binding.target == canonical)
        {
            self.prepare_clone_member(c, node)?;
        }
        Ok(())
    }
    pub(in crate::resolve) fn reserve_cloning(&mut self) -> Result<(), Error> {
        let Some((module, source)) = self
            .facts
            .builtin_traits
            .cloning
            .as_deref()
            .and_then(|r| r.origin)
        else {
            return Ok(());
        };
        let c = Context {
            module,
            file: source.file,
            scope: self.facts.module_scopes[module.0 as usize],
            foreign: None,
        };
        self.prepare_clone_member(c, source.node)?;
        if self.facts.builtin_traits.collections().is_none() {
            return Ok(());
        }
        let mut declaration = self
            .facts
            .builtin_traits
            .cloning()
            .ok_or_else(|| self.error(c, source.node, "missing Clone member"))?;
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let c = Context { scope, ..c };
        declaration.implementation = Some(Implementation {
            definition: self.builtin_definition(c, source.node, "Self", DefKind::SelfType, None)?,
            lifetime: self.builtin_definition(c, source.node, "'a", DefKind::Lifetime, None)?,
            parameter: self.builtin_definition(c, source.node, "T", DefKind::Generic, None)?,
            method: self.builtin_definition(c, source.node, "clone", DefKind::Function, None)?,
        });
        self.facts
            .builtin_traits
            .cloning
            .get_or_insert_default()
            .complete = Some(declaration);
        Ok(())
    }
    fn prepare_clone_member(&mut self, c: Context, node: NodeId) -> Result<(), Error> {
        if self.facts.builtin_traits.cloning().is_some() {
            return Ok(());
        }
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        let library = self
            .facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing Clone library"))?;
        self.trait_module(c, node, library, true)?;
        let definition = self
            .facts
            .builtin_traits
            .get(BuiltinTrait::Clone)
            .ok_or_else(|| self.error(c, node, "missing canonical Clone"))?;
        if self.facts.definitions[definition.0 as usize]
            .members
            .is_some()
        {
            return Err(self.error(c, node, "canonical Clone member scope already installed"));
        }
        let members = self.new_scope(None, ModuleId(0), true)?;
        self.facts.scopes[members.0 as usize].associated = true;
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let self_parameter =
            self.builtin_definition(Context { scope, ..c }, node, "Self", DefKind::Generic, None)?;
        let method = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            "clone",
            DefKind::Function,
            None,
        )?;
        self.facts.definitions[definition.0 as usize].members = Some(members);
        self.facts
            .builtin_traits
            .cloning
            .get_or_insert_default()
            .complete = Some(Declaration {
            origin: NodeKey { file: c.file, node },
            definition,
            self_parameter,
            method,
            implementation: None,
        });
        Ok(())
    }
}
