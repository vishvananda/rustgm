//! Demand-created language trait metadata, resolved through ordinary namespaces.
mod intrinsics;
mod library;
mod primitives;
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
        if matches!(
            kind,
            crate::support::library_types::LibraryValue::FormatLiteral { .. }
        ) {
            self.facts
                .node_scopes
                .insert(c.module, c.file, node, c.scope);
            return Ok(c.scope);
        }
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        match kind {
            crate::support::library_types::LibraryValue::FormatLiteral { .. } => {
                return Err(self.error(c, node, "formatting cannot use a bootstrap library value"));
            }
            crate::support::library_types::LibraryValue::EnvironmentOption => self
                .prepare_library_type(c, node, crate::support::library_types::LibraryType::Option),
        }?;
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
        if self.package.uses_source_bindings() && self.package.names.get(name) == "self" {
            return self
                .facts
                .module_scopes
                .first()
                .copied()
                .map(Some)
                .ok_or_else(|| self.error(c, node, "missing self-core root"));
        }
        if self.facts.crates.is_some()
            && let Some(binding) = self
                .facts
                .extern_scope
                .and_then(|scope| self.facts.bindings.get(scope, name, Namespace::Type))
        {
            return Ok(self.facts.definitions[binding.target.0 as usize].members);
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
        target: DefId,
    ) -> Result<(), &'static str> {
        if c.scope != ScopeId(0) {
            return Ok(());
        }
        let Some(external) = self.facts.extern_scope else {
            return Ok(());
        };
        let name = self.facts.definitions[target.0 as usize].name;
        self.facts.bindings.insert(
            external,
            name,
            Namespace::Type,
            Binding {
                target,
                access: Access::Public,
                origin: super::model::Origin::Direct,
                ambiguous: false,
            },
        )?;
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
        self.facts
            .extern_scope
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
}

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
