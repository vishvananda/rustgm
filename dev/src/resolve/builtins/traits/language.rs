//! Demand-loaded operator declarations retain real trait, parameter and member identities.
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::language_hooks::Hook,
    syntax::ast::NodeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Declared(DefId),
    Inherited {
        trait_definition: DefId,
        declaration: DefId,
    },
}
impl Output {
    #[must_use]
    pub const fn definition(self) -> DefId {
        match self {
            Self::Declared(id)
            | Self::Inherited {
                declaration: id, ..
            } => id,
        }
    }
    #[must_use]
    pub const fn owner(self, declared_owner: DefId) -> DefId {
        match self {
            Self::Declared(_) => declared_owner,
            Self::Inherited {
                trait_definition, ..
            } => trait_definition,
        }
    }
    #[must_use]
    pub const fn declared(self) -> Option<DefId> {
        match self {
            Self::Declared(id) => Some(id),
            Self::Inherited { .. } => None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Language {
    pub origin: NodeKey,
    pub kind: Hook,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub argument: Option<DefId>,
    pub output: Output,
    pub method: DefId,
}
impl Traits {
    #[must_use]
    pub fn language(&self, kind: Hook) -> Option<Language> {
        self.language.get(&kind).copied()
    }
}
impl Resolver<'_> {
    pub(super) fn prepare_language_trait(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Hook,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.language.contains_key(&kind) {
            return Ok(());
        }
        if kind == Hook::IndexMut {
            self.prepare_language_trait(c, node, Hook::Index)?;
        }
        let ops = self
            .facts
            .builtin_traits
            .ops
            .ok_or_else(|| self.error(c, node, "missing language ops module"))?;
        let members = self.new_scope(None, ModuleId(0), true)?;
        self.facts.scopes[members.0 as usize].associated = true;
        let definition = self.builtin_definition(
            Context { scope: ops, ..c },
            node,
            kind.name(),
            DefKind::BuiltinTrait,
            Some(members),
        )?;
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let self_parameter =
            self.builtin_definition(Context { scope, ..c }, node, "Self", DefKind::Generic, None)?;
        let argument = if kind.binary() {
            Some(self.builtin_definition(
                Context { scope, ..c },
                node,
                "Rhs",
                DefKind::Generic,
                None,
            )?)
        } else {
            None
        };
        let output = if kind == Hook::IndexMut {
            let parent = self
                .facts
                .builtin_traits
                .language(Hook::Index)
                .ok_or_else(|| self.error(c, node, "missing Index supertrait"))?;
            Output::Inherited {
                trait_definition: parent.definition,
                declaration: parent.output.definition(),
            }
        } else {
            Output::Declared(self.builtin_definition(
                Context {
                    scope: members,
                    ..c
                },
                node,
                "Output",
                DefKind::TypeAlias,
                None,
            )?)
        };
        let method = self.builtin_definition(
            Context {
                scope: members,
                ..c
            },
            node,
            kind.method(),
            DefKind::Function,
            None,
        )?;
        self.facts.builtin_traits.language.insert(
            kind,
            Language {
                origin: NodeKey { file: c.file, node },
                kind,
                definition,
                self_parameter,
                argument,
                output,
                method,
            },
        );
        Ok(())
    }
}
