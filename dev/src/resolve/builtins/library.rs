//! Nominal metadata registers distinct library types, parameters and constructor identities.
use super::{Context, DefId, DefKind, Error, NameId, Namespace, Resolver, ScopeId};
use crate::support::hash::HashMap;
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::library_types::{LibraryField, LibraryModule, LibraryParameter, LibraryType},
    syntax::ast::NodeId,
};
mod slices;
pub use slices::SliceLength;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variant {
    pub type_definition: Option<DefId>,
    pub constructor: Option<DefId>,
    pub field: Option<LibraryField>,
}
#[derive(Clone)]
pub struct Nominal {
    pub origin: NodeKey,
    pub kind: LibraryType,
    pub definition: DefId,
    pub parameters: Vec<DefId>,
    pub variants: Vec<Variant>,
}
#[derive(Default)]
pub struct LibraryTypes {
    slice_length: Option<Box<SliceLength>>,
    entries: HashMap<LibraryType, Nominal>,
    prelude: HashMap<(NameId, Namespace), DefId>,
    modules: [Option<ScopeId>; LibraryModule::COUNT],
}
impl LibraryTypes {
    #[must_use]
    pub fn slice_length(&self) -> Option<&SliceLength> {
        self.slice_length.as_deref()
    }
    #[must_use]
    pub fn get(&self, kind: LibraryType) -> Option<&Nominal> {
        self.entries.get(&kind)
    }
    #[must_use]
    pub fn identify(&self, definition: DefId) -> Option<LibraryType> {
        self.entries
            .values()
            .find(|entry| entry.definition == definition)
            .map(|entry| entry.kind)
    }
    pub(super) fn prelude(&self, name: NameId, namespace: Namespace) -> Option<DefId> {
        self.prelude.get(&(name, namespace)).copied()
    }
}
impl Resolver<'_> {
    pub(super) fn prepare_library_metadata(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: crate::resolve::paths::Identified<'_>,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.library.is_none() {
            return Ok(());
        }
        let Some(last) = path.last().map(|name| self.package.names.get(*name)) else {
            return Ok(());
        };
        if last == "slice"
            && self.nominal_library_path(c, node, absolute, path, LibraryType::SliceIter)
        {
            let parent = self
                .facts
                .builtin_traits
                .library
                .ok_or_else(|| self.error(c, node, "missing canonical slice library module"))?;
            self.library_module(c, node, parent, LibraryModule::Slice)?;
            return Ok(());
        }
        let comparison = matches!(last, "Ordering" | "cmp" | "Less" | "Equal" | "Greater")
            && self.nominal_library_path(c, node, absolute, path, LibraryType::ComparisonOrdering);
        let kind = LibraryType::ALL.into_iter().find(|kind| {
            if matches!(
                *kind,
                LibraryType::SliceIter | LibraryType::SliceIterMut | LibraryType::Range
            ) && (!path
                .iter()
                .any(|name| self.package.names.get(*name) == kind.name())
                || !self.nominal_library_path(c, node, absolute, path, *kind))
            {
                return false;
            }
            if matches!(
                kind,
                LibraryType::Ordering | LibraryType::ComparisonOrdering
            ) && (*kind == LibraryType::ComparisonOrdering) != comparison
            {
                return false;
            }

            kind.name() == last
                || !matches!(
                    *kind,
                    LibraryType::SliceIter | LibraryType::SliceIterMut | LibraryType::Range
                ) && kind.module() == last
                || kind.in_prelude() && kind.variants().iter().any(|(name, _)| *name == last)
                || path
                    .iter()
                    .any(|name| self.package.names.get(*name) == kind.name())
        });
        if let Some(kind) = kind {
            if kind == LibraryType::Range {
                self.facts
                    .builtin_traits
                    .note_range(c.module, NodeKey { file: c.file, node });
            } else if matches!(kind, LibraryType::SliceIter | LibraryType::SliceIterMut) {
                self.facts.builtin_traits.name_slice_iterator(
                    c.module,
                    NodeKey { file: c.file, node },
                    kind,
                );
            }
            self.prepare_library_type(c, node, kind)?;
        }
        Ok(())
    }
    fn nominal_library_path(
        &self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: crate::resolve::paths::Identified<'_>,
        kind: LibraryType,
    ) -> bool {
        use crate::resolve::lookup::{Lookup, LookupMode};
        let index = path
            .iter()
            .position(|name| {
                let name = self.package.names.get(*name);
                name == kind.module() || name == kind.name()
            })
            .unwrap_or_else(|| path.len().saturating_sub(1));
        if index == 0 {
            return matches!(self.lookup_path(c, node, absolute, path, LookupMode::Import(Namespace::Type), None),
                Ok(Lookup::Found(binding)) if self.facts.library_types.identify(binding.target) == Some(kind));
        }
        let Ok(Lookup::Found(binding)) = self.lookup_path(
            c,
            node,
            absolute,
            path.prefix(index),
            LookupMode::Import(Namespace::Type),
            None,
        ) else {
            return false;
        };
        let scope = self.facts.definitions[binding.target.0 as usize].members;
        scope.is_some()
            && (kind
                .modules()
                .last()
                .is_some_and(|part| scope == self.facts.library_types.modules[part.index()])
                || kind == LibraryType::Range && scope == self.facts.builtin_traits.ops
                || self.package.names.get(path[index]) == kind.module()
                    && scope == self.facts.builtin_traits.library)
    }
    pub(super) fn prepare_library_type(
        &mut self,
        c: Context,
        node: NodeId,
        kind: LibraryType,
    ) -> Result<(), Error> {
        if self.facts.library_types.get(kind).is_some() {
            return Ok(());
        }
        let library = self
            .facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing library module"))?;
        let mut module = if kind == LibraryType::Range {
            self.facts
                .builtin_traits
                .ops
                .ok_or_else(|| self.error(c, node, "missing canonical range module"))?
        } else {
            library
        };
        for &part in kind.modules() {
            module = self.library_module(c, node, module, part)?;
        }
        let members = self.new_scope(None, ModuleId(0), true)?;
        let definition = self.builtin_definition(
            Context { scope: module, ..c },
            node,
            kind.name(),
            if kind.is_struct() {
                DefKind::Struct
            } else {
                DefKind::Enum
            },
            Some(members),
        )?;
        let parameter_scope = self.new_scope(None, ModuleId(0), true)?;
        let mut parameters = Vec::new();
        for parameter in kind.parameters() {
            parameters.push(self.builtin_definition(
                Context {
                    scope: parameter_scope,
                    ..c
                },
                node,
                parameter.name(),
                if matches!(parameter, LibraryParameter::Lifetime(_)) {
                    DefKind::Lifetime
                } else {
                    DefKind::Generic
                },
                None,
            )?);
        }
        let variants = self.library_variants(c, node, kind, members)?;
        if kind.in_prelude() {
            let name = self.facts.definitions[definition.0 as usize].name;
            self.facts
                .library_types
                .prelude
                .insert((name, Namespace::Type), definition);
        }
        self.facts.library_types.entries.insert(
            kind,
            Nominal {
                origin: NodeKey { file: c.file, node },
                kind,
                definition,
                parameters,
                variants,
            },
        );
        if kind == LibraryType::ComparisonOrdering {
            self.prepare_comparison(c, node, crate::support::comparisons::Comparison::Ord)?;
        }
        if kind == LibraryType::AtomicU32 {
            self.prepare_library_type(c, node, LibraryType::Ordering)?;
            self.prepare_atomic_intrinsics(c, node, definition, members)?;
        }
        Ok(())
    }
    pub(super) fn library_module(
        &mut self,
        c: Context,
        node: NodeId,
        parent: ScopeId,
        part: LibraryModule,
    ) -> Result<ScopeId, Error> {
        if let Some(scope) = self.facts.library_types.modules[part.index()] {
            return Ok(scope);
        }
        let scope = self.new_scope(None, ModuleId(0), true)?;
        self.builtin_definition(
            Context { scope: parent, ..c },
            node,
            part.name(),
            DefKind::Module,
            Some(scope),
        )?;
        self.facts.library_types.modules[part.index()] = Some(scope);
        Ok(scope)
    }
    fn library_variants(
        &mut self,
        c: Context,
        node: NodeId,
        kind: LibraryType,
        members: ScopeId,
    ) -> Result<Vec<Variant>, Error> {
        let mut variants = Vec::new();
        for &(name, field) in kind.variants() {
            let constructor = if matches!(
                kind,
                LibraryType::SliceIter | LibraryType::SliceIterMut | LibraryType::Range
            ) {
                None
            } else {
                Some(self.library_value(
                    Context {
                        scope: members,
                        ..c
                    },
                    node,
                    name,
                    if kind.is_struct() {
                        DefKind::Constructor
                    } else {
                        DefKind::Variant
                    },
                )?)
            };
            let type_definition = if kind.is_struct() {
                None
            } else {
                Some(self.builtin_definition(
                    Context {
                        scope: members,
                        ..c
                    },
                    node,
                    name,
                    DefKind::Variant,
                    None,
                )?)
            };
            if kind.in_prelude() {
                let constructor = constructor
                    .ok_or_else(|| self.error(c, node, "prelude variant requires a constructor"))?;
                let name = self.facts.definitions[constructor.0 as usize].name;
                self.facts
                    .library_types
                    .prelude
                    .insert((name, Namespace::Value), constructor);
                if let Some(id) = type_definition {
                    self.facts
                        .library_types
                        .prelude
                        .insert((name, Namespace::Type), id);
                }
            }
            variants.push(Variant {
                type_definition,
                constructor,
                field,
            });
        }
        Ok(variants)
    }
    fn library_value(
        &mut self,
        c: Context,
        node: NodeId,
        spelling: &str,
        kind: DefKind,
    ) -> Result<DefId, Error> {
        let name = self
            .package
            .names
            .intern(spelling)
            .map_err(|e| self.error(c, node, e))?;
        self.define(
            crate::resolve::model::Definition {
                name,
                namespace: Namespace::Value,
                kind,
                scope: c.scope,
                node: None,
                access: crate::resolve::model::Access::Public,
                members: None,
            },
            false,
        )
    }
}
