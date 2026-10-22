//! Small demand-loaded trait modules retain identity through aliases and globs.
mod callables;
mod cloning;
mod collections;
mod comparisons;
mod iteration;
mod language;
use super::{Context, DefId, DefKind, Error, NameId, Resolver, ScopeId};
use crate::support::language_hooks::Hook;
use crate::{
    resolve::lookup::{Lookup, LookupMode},
    resolve::model::Namespace,
    support::call_traits::CallTrait,
    support::marker_traits::AutoTrait,
    syntax::ast::NodeId,
};
pub use callables::Callable;
pub use cloning::{Declaration as Cloning, Implementation as CloneImplementation};
pub use collections::{CollectionDeclaration, Collections};
pub use comparisons::{Equality, Members as ComparisonMembers};
pub use iteration::{Blanket as IterationBlanket, Declaration as IterationDeclaration, Iteration};
pub use language::{Language, Output as LanguageOutput};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinTrait {
    Copy,
    Clone,
    Sized,
    Auto(AutoTrait),
    Callable(CallTrait),
    Language(Hook),
    Comparison(crate::support::comparisons::Comparison),
    Iteration(crate::support::iteration::Trait),
}
impl BuiltinTrait {
    fn index(self) -> Option<usize> {
        Some(match self {
            Self::Copy => 0,
            Self::Clone => 1,
            Self::Sized => 2,
            Self::Auto(marker) => 3 + marker as usize,
            Self::Callable(kind) => 8 + kind as usize,
            Self::Language(_) | Self::Comparison(_) | Self::Iteration(_) => return None,
        })
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Clone => "Clone",
            Self::Sized => "Sized",
            Self::Auto(marker) => marker.name(),
            Self::Callable(kind) => kind.name(),
            Self::Language(kind) => kind.name(),
            Self::Comparison(kind) => kind.name(),
            Self::Iteration(kind) => kind.name(),
        }
    }
}
#[derive(Default)]
pub struct Traits {
    pub(super) library: Option<ScopeId>,
    marker: Option<ScopeId>,
    clone_: Option<ScopeId>,
    cloning: Option<Box<cloning::Registry>>,
    panic: Option<ScopeId>,
    pub(super) ops: Option<ScopeId>,
    entries: [Option<DefId>; 11],
    callables: Option<Box<[Callable; 3]>>,
    language: crate::support::hash::HashMap<Hook, Language>,
    comparisons: Option<Box<comparisons::Comparisons>>,
    iteration: Option<Box<iteration::Registry>>,
}
impl Traits {
    fn insert_fixed(
        &mut self,
        trait_: BuiltinTrait,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let index = trait_
            .index()
            .ok_or("language trait has no fixed metadata slot")?;
        self.entries[index] = Some(definition);
        Ok(())
    }
    #[must_use]
    pub fn get(&self, trait_: BuiltinTrait) -> Option<DefId> {
        if let BuiltinTrait::Language(kind) = trait_ {
            return self.language(kind).map(|entry| entry.definition);
        }
        if let BuiltinTrait::Comparison(kind) = trait_ {
            return self.comparison(kind).map(|entry| entry.definition);
        }
        if let BuiltinTrait::Iteration(kind) = trait_ {
            return self.iteration().map(|entry| entry.get(kind).definition);
        }
        self.entries[trait_.index()?]
    }
    #[must_use]
    pub fn identify(&self, id: DefId) -> Option<BuiltinTrait> {
        [BuiltinTrait::Copy, BuiltinTrait::Clone, BuiltinTrait::Sized]
            .into_iter()
            .chain(AutoTrait::ALL.into_iter().map(BuiltinTrait::Auto))
            .chain(CallTrait::ALL.into_iter().map(BuiltinTrait::Callable))
            .chain(
                crate::support::comparisons::Comparison::ALL
                    .into_iter()
                    .map(BuiltinTrait::Comparison),
            )
            .chain(
                crate::support::iteration::Trait::ALL
                    .into_iter()
                    .map(BuiltinTrait::Iteration),
            )
            .find(|t| self.get(*t) == Some(id))
            .or_else(|| {
                self.language
                    .values()
                    .find(|entry| entry.definition == id)
                    .map(|entry| BuiltinTrait::Language(entry.kind))
            })
    }
    pub(super) fn identities(&self) -> impl Iterator<Item = DefId> + '_ {
        self.entries[..6]
            .iter()
            .chain(&self.entries[8..])
            .copied()
            .flatten()
            .chain(
                self.comparisons
                    .iter()
                    .flat_map(|entries| entries.entries.iter().flatten())
                    .map(|entry| entry.definition),
            )
            .chain(
                self.iteration()
                    .into_iter()
                    .flat_map(|entry| entry.declarations.map(|d| d.definition)),
            )
    }
}
impl Resolver<'_> {
    pub(super) fn prepare_trait_metadata(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: &[NameId],
    ) -> Result<(), Error> {
        let Some(library) = self.facts.builtin_traits.library else {
            return Ok(());
        };
        let Some(last) = path.last().map(|n| self.package.names.get(*n)) else {
            return Ok(());
        };
        if matches!(
            last,
            "Iterator" | "IntoIterator" | "next" | "into_iter" | "iter"
        ) {
            return self.prepare_iteration(c, node);
        }
        // Loading language metadata does not select it. Ordinary lookup still
        // decides whether the written name denotes a user declaration or alias.
        // Defer this family until a call-trait name is actually demanded so an
        // existing ops glob does not eagerly create unrelated declarations.
        if let Some(kind) = Hook::ALL.into_iter().find(|kind| kind.name() == last) {
            return self.prepare_language_trait(c, node, kind);
        }
        if let Some(kind) = crate::support::comparisons::Comparison::ALL
            .into_iter()
            .find(|kind| kind.name() == last)
        {
            return self.prepare_comparison(c, node, kind);
        }
        if let Some(method) = crate::support::comparisons::Method::from_name(last) {
            return self.prepare_comparison_method(c, node, method);
        }
        if CallTrait::ALL.into_iter().any(|kind| kind.name() == last) {
            return self.prepare_call_traits(c, node);
        }
        if path.len() == 1 {
            if let Some(marker) = AutoTrait::ALL
                .into_iter()
                .take(3)
                .find(|m| m.name() == last)
            {
                return self.auto_trait(c, node, library, marker);
            }
            return match last {
                "Copy" | "Sized" => self.trait_module(c, node, library, false),
                "Clone" => self.trait_module(c, node, library, true),
                _ => Ok(()),
            };
        }
        if let Some(marker) = AutoTrait::ALL.into_iter().find(|m| m.name() == last)
            && self.auto_module_alias(c, node, absolute, &path[..path.len() - 1], marker)
        {
            return self.auto_trait(c, node, library, marker);
        }
        if !matches!(
            last,
            "Copy"
                | "Sized"
                | "Clone"
                | "Send"
                | "Sync"
                | "Unpin"
                | "UnwindSafe"
                | "RefUnwindSafe"
                | "marker"
                | "clone"
                | "panic"
                | "self"
        ) {
            return Ok(());
        }
        let Some(index) = path
            .iter()
            .rposition(|n| matches!(self.package.names.get(*n), "marker" | "clone" | "panic"))
        else {
            return Ok(());
        };
        if index == 0 {
            return Ok(());
        }
        let clone_ = self.package.names.get(path[index]) == "clone";
        let panic = self.package.names.get(path[index]) == "panic";
        let marker = AutoTrait::ALL.into_iter().find(|m| m.name() == last);
        // Prefix lookup selects the actual library, including imports that only
        // became available during this worklist iteration. Spelling alone never
        // turns a user module into language metadata.
        if let Ok(Lookup::Found(binding)) = self.lookup_path(
            c,
            node,
            absolute,
            &path[..index],
            LookupMode::Import(Namespace::Type),
            None,
        ) && self.facts.definitions[binding.target.0 as usize].members == Some(library)
        {
            if let Some(marker) = marker {
                if !clone_
                    && panic == matches!(marker, AutoTrait::UnwindSafe | AutoTrait::RefUnwindSafe)
                {
                    self.auto_trait(c, node, library, marker)?;
                }
            } else if panic {
                self.auto_trait(c, node, library, AutoTrait::UnwindSafe)?;
                self.auto_trait(c, node, library, AutoTrait::RefUnwindSafe)?;
            } else {
                self.trait_module(c, node, library, clone_)?;
            }
        }
        Ok(())
    }
    fn auto_module_alias(
        &self,
        c: Context,
        node: NodeId,
        absolute: bool,
        prefix: &[NameId],
        marker: AutoTrait,
    ) -> bool {
        let expected = if matches!(marker, AutoTrait::UnwindSafe | AutoTrait::RefUnwindSafe) {
            self.facts.builtin_traits.panic
        } else {
            self.facts.builtin_traits.marker
        };
        expected.is_some()
            && matches!(
                self.lookup_path(c, node, absolute, prefix, LookupMode::Import(Namespace::Type), None),
                Ok(Lookup::Found(binding)) if self.facts.definitions[binding.target.0 as usize].members == expected
            )
    }
    fn auto_trait(
        &mut self,
        c: Context,
        node: NodeId,
        library: ScopeId,
        marker: AutoTrait,
    ) -> Result<(), Error> {
        let trait_ = BuiltinTrait::Auto(marker);
        if self.facts.builtin_traits.get(trait_).is_some() {
            return Ok(());
        }
        let scope = if matches!(marker, AutoTrait::UnwindSafe | AutoTrait::RefUnwindSafe) {
            if let Some(scope) = self.facts.builtin_traits.panic {
                scope
            } else {
                let scope = self.new_scope(None, crate::resolve::package::ModuleId(0), true)?;
                self.builtin_definition(
                    Context {
                        scope: library,
                        ..c
                    },
                    node,
                    "panic",
                    DefKind::Module,
                    Some(scope),
                )?;
                self.facts.builtin_traits.panic = Some(scope);
                scope
            }
        } else {
            self.trait_module(c, node, library, false)?;
            self.facts
                .builtin_traits
                .marker
                .ok_or_else(|| self.error(c, node, "missing marker module"))?
        };
        let id = self.builtin_definition(
            Context { scope, ..c },
            node,
            marker.name(),
            DefKind::BuiltinTrait,
            None,
        )?;
        self.facts
            .builtin_traits
            .insert_fixed(trait_, id)
            .map_err(|e| self.error(c, node, e))?;
        Ok(())
    }
    fn trait_module(
        &mut self,
        c: Context,
        node: NodeId,
        library: ScopeId,
        clone_: bool,
    ) -> Result<(), Error> {
        let module = if clone_ {
            self.facts.builtin_traits.clone_
        } else {
            self.facts.builtin_traits.marker
        };
        if module.is_some() {
            return Ok(());
        }
        let scope = self.new_scope(None, crate::resolve::package::ModuleId(0), true)?;
        let (name, traits): (&str, &[BuiltinTrait]) = if clone_ {
            ("clone", &[BuiltinTrait::Clone])
        } else {
            ("marker", &[BuiltinTrait::Copy, BuiltinTrait::Sized])
        };
        for trait_ in traits {
            let id = self.builtin_definition(
                Context { scope, ..c },
                node,
                trait_.name(),
                DefKind::BuiltinTrait,
                None,
            )?;
            self.facts
                .builtin_traits
                .insert_fixed(*trait_, id)
                .map_err(|e| self.error(c, node, e))?;
        }
        self.builtin_definition(
            Context {
                scope: library,
                ..c
            },
            node,
            name,
            DefKind::Module,
            Some(scope),
        )?;
        if clone_ {
            self.facts.builtin_traits.clone_ = Some(scope);
        } else {
            self.facts.builtin_traits.marker = Some(scope);
        }
        Ok(())
    }
}
