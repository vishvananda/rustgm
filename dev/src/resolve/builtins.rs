//! Demand-created language trait metadata, resolved through ordinary namespaces.
mod intrinsics;
mod library;
mod primitives;
mod traits;
use super::{
    index::{Context, Resolver},
    model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, Resolution, ScopeId},
    names::NameId,
    package::{Error, ModuleId, Package},
    unit::NodeKey,
};
use crate::syntax::{
    ast::NodeId,
    item::{Item, ItemKind},
};
pub use intrinsics::{ComparisonResult, Definition as BuiltinIntrinsic, Intrinsics};
pub use library::{LibraryTypes, Nominal as LibraryNominal, SliceLength};
pub use traits::{
    BuiltinTrait, Callable, CloneImplementation, Cloning, CollectionDeclaration, Collections,
    ComparisonMembers, Equality, Iteration, IterationBlanket, IterationDeclaration, Language,
    LanguageOutput, MutableCollections, RangeImplementation, Ranges, Traits,
};

impl Resolver<'_> {
    pub(super) fn prepare_library_value(
        &mut self,
        c: Context,
        node: NodeId,
        kind: crate::support::library_types::LibraryValue,
    ) -> Result<ScopeId, Error> {
        if !matches!(
            kind,
            crate::support::library_types::LibraryValue::EnvironmentOption
        ) {
            self.facts
                .node_scopes
                .insert(c.module, c.file, node, c.scope);
            return Ok(c.scope);
        }
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        self.prepare_library_type(c, node, crate::support::library_types::LibraryType::Option)?;
        self.facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing library value scope"))
    }
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
        if self.package.names.get(name) == "self" {
            return self
                .facts
                .module_scopes
                .get(self.package.crate_root(c.module).0 as usize)
                .copied()
                .map(Some)
                .ok_or_else(|| self.error(c, node, "missing self-crate root"));
        }
        if let Some(root) = self.guest(c.module) {
            // A guest names the host and the guests before it, nothing else.
            return self
                .facts
                .bindings
                .get(root.externs, name, Namespace::Type)
                .and_then(|binding| self.facts.definitions[binding.target.0 as usize].members)
                .map(Some)
                .ok_or_else(|| {
                    self.error(
                        c,
                        node,
                        format!(
                            "can't find crate for `{}`: a guest crate names only the host and earlier guests",
                            self.package.names.get(name)
                        ),
                    )
                });
        }
        if let Some(imported) = self.facts.crates.as_deref() {
            if let Some(target) = imported
                .explicit_only_root(self.package.names.get(name))
                .map_err(|error| self.error(c, node, error))?
            {
                return Ok(self.facts.definitions[target.0 as usize].members);
            }
            if let Some(binding) = self
                .facts
                .extern_scope
                .and_then(|scope| self.facts.bindings.get(scope, name, Namespace::Type))
            {
                return Ok(self.facts.definitions[binding.target.0 as usize].members);
            }
        }
        if !matches!(self.package.names.get(name), "core" | "std") {
            return Ok(None);
        }
        let context = self.package.item_context(c.file, span.start)?;
        self.prepare_builtin_path(
            c,
            node,
            false,
            Namespace::Type,
            super::paths::Identified::new(&[name], &[context]),
        )?;
        let external = self
            .facts
            .extern_scope
            .ok_or_else(|| self.error(c, node, "missing external crate scope"))?;
        let core = self
            .package
            .names
            .intern("core")
            .map_err(|e| self.error(c, node, e))?;
        let target = self
            .facts
            .bindings
            .get(external, core, Namespace::Type)
            .ok_or_else(|| self.error(c, node, "missing external core crate"))?
            .target;
        Ok(self.facts.definitions[target.0 as usize].members)
    }

    pub(super) fn expose_external(
        &mut self,
        c: Context,
        node: NodeId,
        target: DefId,
    ) -> Result<(), Error> {
        if let Some(root) = self.guest(c.module) {
            // A root `extern crate` item adds its name, or its alias, to the
            // guest's own extern prelude; the crates' own names stay bound.
            let (scope, externs) = (root.scope, root.externs);
            let (_, name, namespace, context) =
                self.binding_key(&self.facts.definitions[target.0 as usize])?;
            if c.scope != scope || self.facts.bindings.contains(externs, name, namespace) {
                return Ok(());
            }
            return self
                .facts
                .bindings
                .insert_key(
                    (externs, name, namespace, context),
                    Binding {
                        target,
                        access: Access::Public,
                        origin: super::model::Origin::Direct,
                        ambiguous: false,
                    },
                )
                .map(drop)
                .map_err(|error| self.error(c, node, error));
        }
        if c.scope != ScopeId(0) {
            return Ok(());
        }
        let external = if let Some(scope) = self.facts.extern_scope {
            scope
        } else {
            let scope = self.new_scope(None, ModuleId(0), true)?;
            self.facts.extern_scope = Some(scope);
            scope
        };
        let (_, name, namespace, context) =
            self.binding_key(&self.facts.definitions[target.0 as usize])?;
        self.facts
            .bindings
            .insert_key(
                (external, name, namespace, context),
                Binding {
                    target,
                    access: Access::Public,
                    origin: super::model::Origin::Direct,
                    ambiguous: false,
                },
            )
            .map_err(|error| self.error(c, node, error))?;
        Ok(())
    }

    /// Whether this name's spelling is compared by builtin path preparation.
    pub(super) fn builtin_name(&mut self, name: NameId) -> bool {
        let index = name.0 as usize;
        match self.builtin_names.get(index) {
            Some(1) => return false,
            Some(2) => return true,
            _ => {}
        }
        let relevant = builtin_spelling(self.package.names.get(name));
        if index >= self.builtin_names.len() {
            let len = self.package.names.len().max(index + 1);
            self.builtin_names.resize(len, 0);
        }
        self.builtin_names[index] = if relevant { 2 } else { 1 };
        relevant
    }
    pub(super) fn prepare_builtin_path(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        namespace: Namespace,
        path: crate::resolve::paths::Identified<'_>,
    ) -> Result<(), Error> {
        if self.package.uses_source_bindings() {
            return Ok(());
        }
        let (absolute, path) = self.injected_path(absolute, path);
        // Every step below keys on a spelling recognized by `builtin_spelling`.
        let mut relevant = false;
        for &name in path.names {
            relevant |= self.builtin_name(name);
        }
        if !relevant {
            return Ok(());
        }
        if path
            .last()
            .is_some_and(|name| matches!(self.package.names.get(*name), "next" | "into_iter"))
        {
            self.prepare_iteration(c, node)?;
        }
        if path
            .last()
            .is_some_and(|name| self.package.names.get(*name) == "add")
        {
            self.prepare_pointer_methods(c, node)?;
        }
        if path
            .last()
            .is_some_and(|name| matches!(self.package.names.get(*name), "len" | "iter"))
            && let Some(name) = path.last()
        {
            self.prepare_slice_method(c, node, *name)?;
        }
        if self.facts.builtin_traits.library.is_none()
            && path.len() > 1
            && let Some(method) = path.last().and_then(|name| {
                crate::support::comparisons::Method::from_name(self.package.names.get(*name))
            })
        {
            self.prepare_comparison_method(c, node, method)?;
        }
        let library_shadowed = if !absolute && let Some(&name) = path.first() {
            let namespace = if path.len() > 1 {
                Namespace::Type
            } else {
                namespace
            };
            let context = path.context(0).map_err(|e| self.error(c, node, e))?;
            self.item_lexical(c, name, namespace, context, None)
                .map_err(|e| self.error(c, node, e))?
                .is_some_and(|selected| {
                    let definition = &self.facts.definitions[selected.binding.target.0 as usize];
                    definition.node.is_some()
                        && !definition
                            .members
                            .is_some_and(|scope| self.facts.builtin_traits.library == Some(scope))
                })
        } else {
            false
        };
        if self.facts.drop_trait.is_none()
            && path.first().is_some_and(|name| {
                matches!(
                    self.package.names.get(*name),
                    "Drop"
                        | "PartialEq"
                        | "Eq"
                        | "PartialOrd"
                        | "Ord"
                        | "Iterator"
                        | "IntoIterator"
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
            self.prepare_primitive_module(c, node, absolute, path)?;
            self.prepare_library_metadata(c, node, absolute, path)?;
            self.prepare_intrinsic_metadata(c, node, path.names)?;
        }
        Ok(())
    }
    fn prepare_external_traits(&mut self, c: Context, node: NodeId) -> Result<(), Error> {
        if self.package.uses_source_bindings() {
            return Err(self.error(
                c,
                node,
                "source-core mode has no bootstrap library fallback",
            ));
        }
        let external = if let Some(external) = self.facts.extern_scope {
            external
        } else {
            self.new_scope(None, ModuleId(0), true)?
        };
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
            if name == "std" && self.package.no_std() {
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
        if self.package.uses_source_bindings() {
            return Err(self.error(
                c,
                node,
                "source-core mode cannot create a bootstrap definition",
            ));
        }
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
        self.guest(c.module)
            .map_or(self.facts.extern_scope, |root| Some(root.externs))
            .filter(|scope| self.facts.bindings.contains(*scope, name, Namespace::Type))
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
        if namespace == Namespace::Type {
            let binding = self.facts.bindings.get(external, name, namespace);
            if let Some(binding) = binding
                && self.self_alias(name, binding)
            {
                return Some((external, *binding));
            }
        }
        if self.package.uses_source_bindings() {
            if namespace != Namespace::Type {
                return None;
            }
            let binding = self.facts.bindings.get(external, name, namespace)?;
            let definition = self.facts.definitions.get(binding.target.0 as usize)?;
            if definition.kind != DefKind::Module
                || definition.node.is_some()
                || definition.members.is_none()
                || binding.access != Access::Public
                || binding.origin != Origin::Direct
                || binding.ambiguous
                || !(self
                    .facts
                    .crates
                    .as_deref()?
                    .alias_root(self.package.names.get(name), binding.target)
                    || self.package.names.get(name) == "core"
                        && self.facts.introductions.injected_core() == Some(binding.target))
            {
                return None;
            }
            return Some((external, *binding));
        }
        if self.facts.introductions.has_source_prelude() {
            return None;
        }
        if let Some(target) = self.facts.library_types.prelude(name, namespace) {
            let scope = self.facts.definitions[target.0 as usize].scope;
            return self
                .facts
                .bindings
                .get(scope, name, namespace)
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
        self.facts
            .bindings
            .get(scope, name, Namespace::Type)
            .map(|binding| (scope, *binding))
    }

    /// The extern prelude may retain an actual root self-alias without any
    /// bootstrap library or imported crate. Its current declaration still owns
    /// the root members; neither an arbitrary module nor missing metadata does.
    fn self_alias(&self, name: NameId, binding: &Binding) -> bool {
        binding.access == Access::Public
            && binding.origin == Origin::Direct
            && !binding.ambiguous
            && root_self_alias(self.package, self.facts, name, binding.target).is_some()
    }
}

/// Locate the active declaration and direct binding of a root self alias.
pub(crate) fn root_self_alias(
    package: &Package,
    facts: &Resolution,
    name: NameId,
    target: DefId,
) -> Option<NodeKey> {
    let definition = facts.definitions.get(target.0 as usize)?;
    let key = definition.node?;
    let root = package.modules.first()?;
    let root_scope = *facts.module_scopes.first()?;
    if definition.name != name
        || definition.kind != DefKind::ExternCrate
        || definition.namespace != Namespace::Type
        || definition.scope != root_scope
        || definition.members != Some(root_scope)
        || key.file != root.file
    {
        return None;
    }
    let unit = package.unit(key.file);
    let index = key.node.get() as usize;
    let crate::syntax::ast::Kind::Item(item) = unit.configured.ast.nodes.get(index)?.kind else {
        return None;
    };
    let item = unit.configured.ast.items.get(item as usize)?;
    let span = item.name?;
    if !matches!(item.kind, ItemKind::ExternCrate { rename: Some(_) })
        || unit.spelling(span).ok()? != "self"
        || unit.configured.active.get(index) != Some(&true)
    {
        return None;
    }
    let context = super::hygiene::definition_context(package, definition).ok()?;
    let binding = facts
        .bindings
        .get_key((root_scope, name, Namespace::Type, context))?;
    (binding.target == target && binding.origin == Origin::Direct && !binding.ambiguous)
        .then_some(key)
}

#[cfg(test)]
#[path = "../../tests/invariants/extern_aliases.rs"]
mod alias_tests;

/// Spellings that can select demand-created language metadata in
/// `prepare_builtin_path` and its trait, library and intrinsic helpers. A path
/// naming none of them leaves every preparation step a no-op. Keep this a
/// superset of those comparisons when adding language items. `builtin_name`
/// caches the answer, so this runs once per name.
#[cold]
#[inline(never)]
fn builtin_spelling(spelling: &str) -> bool {
    use crate::support::{
        call_traits::CallTrait, comparisons::Comparison, comparisons::Method,
        intrinsics::Intrinsic, iteration::Trait as IterationTrait, language_hooks::Hook,
        library_types::LibraryType, marker_traits::AutoTrait,
    };
    matches!(
        spelling,
        "len"
            | "Drop"
            | "core"
            | "std"
            | "Copy"
            | "Clone"
            | "Sized"
            | "Send"
            | "Sync"
            | "Unpin"
            | "UnwindSafe"
            | "RefUnwindSafe"
            | "marker"
            | "clone"
            | "panic"
            | "Some"
            | "None"
            | "Ok"
            | "Err"
            | "Less"
            | "Equal"
            | "Greater"
            | "Ordering"
            | "cmp"
            | "sync"
            | "ptr"
            | "read_volatile"
            | "write_volatile"
            | "write_bytes"
            | "add"
            | "primitive"
            | "iter"
    ) || Method::from_name(spelling).is_some()
        || Comparison::ALL.iter().any(|kind| kind.name() == spelling)
        || Hook::ALL.iter().any(|kind| kind.name() == spelling)
        || CallTrait::ALL.iter().any(|kind| kind.name() == spelling)
        || IterationTrait::ALL
            .iter()
            .any(|kind| kind.name() == spelling || kind.method().name() == spelling)
        || AutoTrait::ALL.iter().any(|kind| kind.name() == spelling)
        || Intrinsic::BOOTSTRAP
            .iter()
            .any(|kind| kind.name() == spelling)
        || LibraryType::ALL.iter().any(|kind| {
            kind.name() == spelling
                || kind.module() == spelling
                || kind.variants().iter().any(|(name, _)| *name == spelling)
        })
}
