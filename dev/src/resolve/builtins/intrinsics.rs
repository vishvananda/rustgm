//! Canonical intrinsic definitions are ordinary resolved library values.
use super::{Context, DefId, DefKind, Error, NameId, Resolver, ScopeId};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::{intrinsics::Intrinsic, library_types::LibraryType},
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ComparisonResult {
    pub definition: DefId,
    pub ok: DefId,
    pub err: DefId,
}
#[derive(Clone, Copy, Default)]
pub struct Intrinsics {
    entries: [Option<Definition>; Intrinsic::BOOTSTRAP.len()],
    pointer_module: Option<ScopeId>,
    comparison_result: Option<ComparisonResult>,
}
impl Intrinsics {
    #[must_use]
    pub fn get(&self, kind: Intrinsic) -> Option<&Definition> {
        self.entries.get(kind.index()).and_then(Option::as_ref)
    }
    #[must_use]
    pub const fn comparison_result(&self) -> Option<ComparisonResult> {
        self.comparison_result
    }
    #[must_use]
    pub fn identify(&self, definition: DefId) -> Option<Intrinsic> {
        self.entries
            .iter()
            .flatten()
            .find(|entry| entry.definition == definition)
            .map(|entry| entry.kind)
    }
    pub(crate) fn pointer_method(&self, definition: DefId) -> Option<&Definition> {
        self.get(Intrinsic::ConstPointerAdd)
            .filter(|d| d.definition == definition)
            .or_else(|| {
                self.get(Intrinsic::MutPointerAdd)
                    .filter(|d| d.definition == definition)
            })
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
                || Intrinsic::BOOTSTRAP
                    .into_iter()
                    .any(|kind| kind.is_pointer_function() && kind.name() == name)
        });
        if requested {
            for kind in Intrinsic::BOOTSTRAP {
                if kind.is_pointer_function() {
                    self.prepare_intrinsic(c, node, kind)?;
                }
            }
        }
        Ok(())
    }
    pub(in crate::resolve::builtins) fn prepare_intrinsic(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Intrinsic,
    ) -> Result<(), Error> {
        if !kind.is_pointer_function() {
            return Err(self.error(c, node, "intrinsic has no ordinary pointer factory"));
        }
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
        self.prepare_library_type(c, node, LibraryType::Result)?;
        let result = self
            .facts
            .library_types
            .get(LibraryType::Result)
            .ok_or_else(|| self.error(c, node, "missing canonical comparison Result"))?;
        let [ok, err] = result.variants.as_slice() else {
            return Err(self.error(c, node, "comparison Result requires canonical Ok and Err"));
        };
        let (Some(ok), Some(err)) = (ok.constructor, err.constructor) else {
            return Err(self.error(
                c,
                node,
                "comparison Result requires both variant constructors",
            ));
        };
        let identity = ComparisonResult {
            definition: result.definition,
            ok,
            err,
        };
        if self
            .facts
            .intrinsics
            .comparison_result
            .is_some_and(|prior| prior != identity)
        {
            return Err(self.error(c, node, "canonical comparison Result changed identity"));
        }
        self.facts.intrinsics.comparison_result = Some(identity);
        for kind in Intrinsic::BOOTSTRAP
            .into_iter()
            .filter(|kind| kind.is_atomic())
        {
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

impl Resolver<'_> {
    pub(in crate::resolve) fn prepare_pointer_methods(
        &mut self,
        c: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        for kind in [Intrinsic::ConstPointerAdd, Intrinsic::MutPointerAdd] {
            if self.facts.intrinsics.get(kind).is_some() {
                continue;
            }
            let scope = self.new_scope(None, ModuleId(0), true)?;
            let context = Context { scope, ..c };
            let owner = self.builtin_definition(context, node, "Self", DefKind::SelfType, None)?;
            let definition =
                self.builtin_definition(context, node, kind.name(), DefKind::Function, None)?;
            let parameter = self.builtin_definition(context, node, "T", DefKind::Generic, None)?;
            self.facts.intrinsics.entries[kind.index()] = Some(Definition {
                kind,
                origin: NodeKey { file: c.file, node },
                definition,
                parameter: Some(parameter),
                owner: Some(owner),
            });
        }
        Ok(())
    }
}
