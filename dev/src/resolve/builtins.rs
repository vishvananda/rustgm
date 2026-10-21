//! Demand-created language trait metadata, resolved through ordinary namespaces.
mod library;
mod traits;
use super::{
    index::{Context, Resolver},
    model::{Access, Binding, DefId, DefKind, Definition, Namespace, ScopeId},
    names::NameId,
    package::{Error, ModuleId},
};
use crate::syntax::{
    ast::NodeId,
    item::{Item, ItemKind},
};
pub use library::{LibraryTypes, Nominal as LibraryNominal};
pub use traits::{BuiltinTrait, Callable, Language, Traits};

impl Resolver<'_> {
    pub(super) fn builtin_crate(
        &mut self,
        c: Context,
        node: NodeId,
        item: Item,
    ) -> Result<Option<ScopeId>, Error> {
        if !matches!(item.kind, ItemKind::ExternCrate { .. }) {
            return Ok(None);
        }
        let Some(span) = item.name else {
            return Ok(None);
        };
        let name = self.package.name(c.file, span)?;
        if !matches!(self.package.names.get(name), "core" | "std") {
            return Ok(None);
        }
        self.prepare_builtin_path(c, node, false, Namespace::Type, &[name])?;
        let external = self
            .facts
            .extern_scope
            .ok_or_else(|| self.error(c, node, "missing external crate scope"))?;
        let core = self
            .package
            .names
            .intern("core")
            .map_err(|e| self.error(c, node, e))?;
        let target =
            self.facts.scopes[external.0 as usize].bindings[&(core, Namespace::Type)].target;
        Ok(self.facts.definitions[target.0 as usize].members)
    }

    pub(super) fn expose_external(&mut self, c: Context, target: DefId) {
        if c.scope != ScopeId(0) {
            return;
        }
        let Some(external) = self.facts.extern_scope else {
            return;
        };
        let definition = &self.facts.definitions[target.0 as usize];
        let key = (definition.name, Namespace::Type);
        let scope = &mut self.facts.scopes[external.0 as usize];
        if scope
            .bindings
            .insert(
                key,
                Binding {
                    target,
                    access: Access::Public,
                    origin: super::model::Origin::Direct,
                    ambiguous: false,
                },
            )
            .is_none()
        {
            scope.order.push(key);
        }
    }

    pub(super) fn prepare_builtin_path(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        namespace: Namespace,
        path: &[NameId],
    ) -> Result<(), Error> {
        let library_shadowed = !absolute
            && path.first().is_some_and(|&name| {
                let namespace = if path.len() > 1 { Namespace::Type } else { namespace };
                matches!(self.lexical(c.scope, name, namespace), crate::resolve::lookup::Lookup::Found(binding)
                    if self.facts.definitions[binding.target.0 as usize].node.is_some())
            });
        if self.facts.drop_trait.is_none()
            && path.first().is_some_and(|name| {
                matches!(
                    self.package.names.get(*name),
                    "Drop"
                        | "core"
                        | "std"
                        | "Copy"
                        | "Clone"
                        | "Sized"
                        | "Send"
                        | "Sync"
                        | "Unpin"
                        | "Fn"
                        | "FnMut"
                        | "FnOnce"
                ) || !library_shadowed
                    && matches!(
                        self.package.names.get(*name),
                        "Option" | "Some" | "None" | "Result" | "Ok" | "Err" | "ManuallyDrop"
                    )
            })
        {
            self.prepare_external_traits(c, node)?;
        }
        self.prepare_trait_metadata(c, node, absolute, path)?;
        if !library_shadowed {
            self.prepare_library_metadata(c, node, path)?;
        }
        Ok(())
    }
    fn prepare_external_traits(&mut self, c: Context, node: NodeId) -> Result<(), Error> {
        let external = self.new_scope(None, ModuleId(0), true)?;
        let library = self.new_scope(None, ModuleId(0), true)?;
        let ops = self.new_scope(None, ModuleId(0), true)?;
        let drop = self.builtin_definition(
            Context { scope: ops, ..c },
            node,
            "Drop",
            DefKind::BuiltinTrait,
            None,
        )?;
        self.builtin_definition(
            Context {
                scope: library,
                ..c
            },
            node,
            "ops",
            DefKind::Module,
            Some(ops),
        )?;
        for name in ["core", "std"] {
            if name == "std" && self.package.no_std {
                continue;
            }
            self.builtin_definition(
                Context {
                    scope: external,
                    ..c
                },
                node,
                name,
                DefKind::Module,
                Some(library),
            )?;
        }
        self.facts.drop_trait = Some(drop);
        self.facts.extern_scope = Some(external);
        self.facts.builtin_traits.library = Some(library);
        self.facts.builtin_traits.ops = Some(ops);
        Ok(())
    }

    fn builtin_definition(
        &mut self,
        c: Context,
        node: NodeId,
        spelling: &str,
        kind: DefKind,
        members: Option<ScopeId>,
    ) -> Result<DefId, Error> {
        let name = self
            .package
            .names
            .intern(spelling)
            .map_err(|error| self.error(c, node, error))?;
        self.define(
            Definition {
                name,
                namespace: if kind == DefKind::Function {
                    Namespace::Value
                } else {
                    Namespace::Type
                },
                kind,
                scope: c.scope,
                node: None,
                access: Access::Public,
                members,
            },
            false,
        )
    }

    pub(super) fn external_root(
        &self,
        c: Context,
        node: NodeId,
        name: NameId,
    ) -> Result<ScopeId, Error> {
        self.facts
            .extern_scope
            .filter(|scope| {
                self.facts.scopes[scope.0 as usize]
                    .bindings
                    .contains_key(&(name, Namespace::Type))
            })
            .ok_or_else(|| self.error(c, node, "external-crate paths require external metadata"))
    }

    pub(super) fn builtin_binding(
        &self,
        module: ModuleId,
        name: NameId,
        namespace: Namespace,
    ) -> Option<(ScopeId, Binding)> {
        if !self.package.modules[module.0 as usize].implicit_prelude {
            return None;
        }
        let external = self.facts.extern_scope?;
        if let Some(target) = self.facts.library_types.prelude(name, namespace) {
            let scope = self.facts.definitions[target.0 as usize].scope;
            return self.facts.scopes[scope.0 as usize]
                .bindings
                .get(&(name, namespace))
                .copied()
                .map(|binding| (scope, binding));
        }
        if namespace != Namespace::Type {
            return None;
        }
        let builtin = self
            .facts
            .drop_trait
            .into_iter()
            .chain(self.facts.builtin_traits.identities())
            .find(|id| self.facts.definitions[id.0 as usize].name == name);
        let scope = builtin.map_or(external, |id| self.facts.definitions[id.0 as usize].scope);
        self.facts.scopes[scope.0 as usize]
            .bindings
            .get(&(name, Namespace::Type))
            .map(|binding| (scope, *binding))
    }
}
