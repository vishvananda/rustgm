//! Canonical intrinsic definitions are ordinary resolved library values.
use super::{Context, DefId, DefKind, Error, NameId, Resolver, ScopeId};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::intrinsics::Intrinsic,
    syntax::ast::NodeId,
};

#[derive(Clone, Copy)]
pub struct Definition {
    pub kind: Intrinsic,
    pub origin: NodeKey,
    pub definition: DefId,
    pub parameter: Option<DefId>,
    pub owner: Option<DefId>,
}
#[derive(Clone, Copy, Default)]
pub struct Intrinsics {
    entries: [Option<Definition>; Intrinsic::ALL.len()],
    pointer_module: Option<ScopeId>,
}
impl Intrinsics {
    #[must_use]
    pub fn get(&self, kind: Intrinsic) -> Option<&Definition> {
        self.entries[kind.index()].as_ref()
    }
    #[must_use]
    pub fn identify(&self, definition: DefId) -> Option<Intrinsic> {
        self.entries
            .iter()
            .flatten()
            .find(|entry| entry.definition == definition)
            .map(|entry| entry.kind)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Definition> {
        self.entries.iter().flatten()
    }
}
impl Resolver<'_> {
    pub(super) fn prepare_intrinsic_metadata(
        &mut self,
        c: Context,
        node: NodeId,
        path: &[NameId],
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.library.is_none() {
            return Ok(());
        }
        let requested = path.iter().any(|name| {
            let name = self.package.names.get(*name);
            name == "ptr"
                || Intrinsic::ALL
                    .into_iter()
                    .any(|kind| kind.is_volatile() && kind.name() == name)
        });
        if requested {
            for kind in Intrinsic::ALL {
                if kind.is_volatile() {
                    self.prepare_intrinsic(c, node, kind)?;
                }
            }
        }
        Ok(())
    }
    fn prepare_intrinsic(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Intrinsic,
    ) -> Result<(), Error> {
        if self.facts.intrinsics.get(kind).is_some() {
            return Ok(());
        }
        let module = if let Some(module) = self.facts.intrinsics.pointer_module {
            module
        } else {
            let library = self
                .facts
                .builtin_traits
                .library
                .ok_or_else(|| self.error(c, node, "missing intrinsic library module"))?;
            let module = self.new_scope(None, ModuleId(0), true)?;
            self.builtin_definition(
                Context {
                    scope: library,
                    ..c
                },
                node,
                "ptr",
                DefKind::Module,
                Some(module),
            )?;
            self.facts.intrinsics.pointer_module = Some(module);
            module
        };
        let definition = self.builtin_definition(
            Context { scope: module, ..c },
            node,
            kind.name(),
            DefKind::Function,
            None,
        )?;
        let parameter_scope = self.new_scope(None, ModuleId(0), true)?;
        let parameter = self.builtin_definition(
            Context {
                scope: parameter_scope,
                ..c
            },
            node,
            "T",
            DefKind::Generic,
            None,
        )?;
        self.facts.intrinsics.entries[kind.index()] = Some(Definition {
            kind,
            origin: NodeKey { file: c.file, node },
            definition,
            parameter: Some(parameter),
            owner: None,
        });
        Ok(())
    }
}

impl Resolver<'_> {
    pub(super) fn prepare_atomic_intrinsics(
        &mut self,
        c: Context,
        node: NodeId,
        owner: DefId,
        scope: ScopeId,
    ) -> Result<(), Error> {
        for kind in [Intrinsic::AtomicLoad, Intrinsic::AtomicStore] {
            let definition = self.builtin_definition(
                Context { scope, ..c },
                node,
                kind.name(),
                DefKind::Function,
                None,
            )?;
            self.facts.intrinsics.entries[kind.index()] = Some(Definition {
                kind,
                origin: NodeKey { file: c.file, node },
                definition,
                parameter: None,
                owner: Some(owner),
            });
        }
        Ok(())
    }
}
