//! Equality metadata keeps a methodless Eq distinct from `PartialEq`'s two members.
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::comparisons::{Comparison, Method},
    syntax::ast::NodeId,
};
#[derive(Clone, Copy)]
pub struct Equality {
    pub origin: NodeKey,
    pub kind: Comparison,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub argument: Option<DefId>,
    pub methods: Option<[DefId; 2]>,
}
impl Traits {
    #[must_use]
    pub fn comparison(&self, kind: Comparison) -> Option<Equality> {
        self.comparisons.as_deref()?[kind as usize]
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn prepare_comparisons(
        &mut self,
        c: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        self.prepare_comparison(c, node, Comparison::PartialEq)
    }
    pub(super) fn prepare_comparison(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Comparison,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.comparison(kind).is_some() {
            return Ok(());
        }
        if kind == Comparison::Eq {
            self.prepare_comparison(c, node, Comparison::PartialEq)?;
        }
        let cmp = if let Some(cmp) = self.facts.builtin_traits.cmp {
            cmp
        } else {
            let cmp = self.new_scope(None, ModuleId(0), true)?;
            let library = self
                .facts
                .builtin_traits
                .library
                .ok_or_else(|| self.error(c, node, "missing comparison library"))?;
            self.builtin_definition(
                Context {
                    scope: library,
                    ..c
                },
                node,
                "cmp",
                DefKind::Module,
                Some(cmp),
            )?;
            self.facts.builtin_traits.cmp = Some(cmp);
            cmp
        };
        let members = self.new_scope(None, ModuleId(0), true)?;
        self.facts.scopes[members.0 as usize].associated = true;
        let definition = self.builtin_definition(
            Context { scope: cmp, ..c },
            node,
            kind.name(),
            DefKind::BuiltinTrait,
            Some(members),
        )?;
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let self_parameter =
            self.builtin_definition(Context { scope, ..c }, node, "Self", DefKind::Generic, None)?;
        let (argument, methods) = if kind == Comparison::PartialEq {
            let argument = self.builtin_definition(
                Context { scope, ..c },
                node,
                "Rhs",
                DefKind::Generic,
                None,
            )?;
            let mut methods = [DefId(0); 2];
            for method in Method::ALL {
                methods[method as usize] = self.builtin_definition(
                    Context {
                        scope: members,
                        ..c
                    },
                    node,
                    method.name(),
                    DefKind::Function,
                    None,
                )?;
            }
            (Some(argument), Some(methods))
        } else {
            (None, None)
        };
        self.facts
            .builtin_traits
            .comparisons
            .get_or_insert_default()[kind as usize] = Some(Equality {
            origin: NodeKey { file: c.file, node },
            kind,
            definition,
            self_parameter,
            argument,
            methods,
        });
        Ok(())
    }
}
