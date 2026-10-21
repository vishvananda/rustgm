//! Callable metadata has real namespace identities without generated source syntax.
use super::{BuiltinTrait, Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{resolve::package::ModuleId, support::call_traits::CallTrait, syntax::ast::NodeId};

#[derive(Clone, Copy)]
pub struct Callable {
    /// First demand supplies diagnostic provenance, not declaration syntax.
    pub origin: crate::resolve::unit::NodeKey,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub arguments_parameter: DefId,
    /// Only `FnOnce` declares `Output`. The other call traits inherit that member.
    pub output: Option<DefId>,
}

impl Traits {
    #[must_use]
    pub fn callable(&self, kind: CallTrait) -> Option<Callable> {
        self.callables
            .as_ref()
            .map(|entries| entries[kind as usize])
    }
}

impl Resolver<'_> {
    pub(super) fn prepare_call_traits(&mut self, c: Context, node: NodeId) -> Result<(), Error> {
        if self.facts.builtin_traits.callables.is_some() {
            return Ok(());
        }
        let ops = self
            .facts
            .builtin_traits
            .ops
            .ok_or_else(|| self.error(c, node, "missing language ops module"))?;
        let mut entries = Vec::with_capacity(CallTrait::ALL.len());
        for kind in CallTrait::ALL {
            let members = self.new_scope(None, ModuleId(0), true)?;
            self.facts.scopes[members.0 as usize].associated = true;
            let definition = self.builtin_definition(
                Context { scope: ops, ..c },
                node,
                kind.name(),
                DefKind::BuiltinTrait,
                Some(members),
            )?;
            // Parameters belong to a private declaration scope. They are not
            // associated items reachable through FnOnce::Args or FnOnce::Self.
            let scope = self.new_scope(None, ModuleId(0), true)?;
            let self_parameter = self.builtin_definition(
                Context { scope, ..c },
                node,
                "Self",
                DefKind::Generic,
                None,
            )?;
            let arguments_parameter = self.builtin_definition(
                Context { scope, ..c },
                node,
                "Args",
                DefKind::Generic,
                None,
            )?;
            let output = if kind == CallTrait::FnOnce {
                Some(self.builtin_definition(
                    Context {
                        scope: members,
                        ..c
                    },
                    node,
                    "Output",
                    DefKind::TypeAlias,
                    None,
                )?)
            } else {
                None
            };
            self.facts
                .builtin_traits
                .insert_fixed(BuiltinTrait::Callable(kind), definition)
                .map_err(|e| self.error(c, node, e))?;
            entries.push(Callable {
                origin: crate::resolve::unit::NodeKey { file: c.file, node },
                definition,
                self_parameter,
                arguments_parameter,
                output,
            });
        }
        self.facts.builtin_traits.callables =
            Some(Box::new(entries.try_into().map_err(|_| {
                self.error(c, node, "invalid call trait family")
            })?));
        Ok(())
    }
}
