//! Small demand-loaded trait modules retain identity through aliases and globs.
use super::{Context, DefId, DefKind, Error, NameId, Resolver, ScopeId};
use crate::{
    resolve::lookup::{Lookup, LookupMode},
    resolve::model::Namespace,
    syntax::ast::NodeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinTrait {
    Copy,
    Clone,
    Sized,
}
impl BuiltinTrait {
    fn index(self) -> usize {
        match self {
            Self::Copy => 0,
            Self::Clone => 1,
            Self::Sized => 2,
        }
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Clone => "Clone",
            Self::Sized => "Sized",
        }
    }
}
#[derive(Default)]
pub struct Traits {
    pub(super) library: Option<ScopeId>,
    marker: Option<ScopeId>,
    clone_: Option<ScopeId>,
    entries: [Option<DefId>; 3],
}
impl Traits {
    #[must_use]
    pub fn get(&self, trait_: BuiltinTrait) -> Option<DefId> {
        self.entries[trait_.index()]
    }
    #[must_use]
    pub fn identify(&self, id: DefId) -> Option<BuiltinTrait> {
        [BuiltinTrait::Copy, BuiltinTrait::Clone, BuiltinTrait::Sized]
            .into_iter()
            .find(|t| self.get(*t) == Some(id))
    }
    pub(super) fn identities(&self) -> impl Iterator<Item = DefId> + '_ {
        self.entries.iter().copied().flatten()
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
        if path.len() == 1 {
            return match last {
                "Copy" | "Sized" => self.trait_module(c, node, library, false),
                "Clone" => self.trait_module(c, node, library, true),
                _ => Ok(()),
            };
        }
        if !matches!(
            last,
            "Copy" | "Sized" | "Clone" | "marker" | "clone" | "self"
        ) {
            return Ok(());
        }
        let Some(index) = path
            .iter()
            .rposition(|n| matches!(self.package.names.get(*n), "marker" | "clone"))
        else {
            return Ok(());
        };
        if index == 0 {
            return Ok(());
        }
        let clone_ = self.package.names.get(path[index]) == "clone";
        if (if clone_ {
            self.facts.builtin_traits.clone_
        } else {
            self.facts.builtin_traits.marker
        })
        .is_some()
        {
            return Ok(());
        }
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
            self.trait_module(c, node, library, clone_)?;
        }
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
            self.facts.builtin_traits.entries[trait_.index()] = Some(id);
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
