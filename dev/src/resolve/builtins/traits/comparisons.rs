//! Equality metadata keeps a methodless Eq distinct from `PartialEq`'s two members.
mod demand;
#[cfg(test)]
#[path = "../../../../tests/invariants/latent_contracts.rs"]
mod tests;
use super::{Context, DefId, DefKind, Error, Resolver, Traits};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    support::comparisons::{Comparison, Method},
    syntax::ast::NodeId,
};
#[derive(Default)]
pub(super) struct Comparisons {
    pub entries: [Option<Equality>; 4],
    pending: u8,
    latent: u8,
    results: u8,
    origin: Option<(ModuleId, NodeKey)>,
}
#[derive(Clone, Copy)]
pub struct Equality {
    pub origin: NodeKey,
    pub kind: Comparison,
    pub definition: DefId,
    pub self_parameter: DefId,
    pub argument: Option<DefId>,
    pub methods: Option<Members>,
}
/// Different canonical families retain their exact member schema.
#[derive(Clone, Copy)]
pub enum Members {
    Equality([DefId; 2]),
    PartialOrder([DefId; 5]),
    TotalOrder([DefId; 4]),
}
impl Members {
    #[must_use]
    pub fn get(self, method: Method) -> Option<DefId> {
        match (self, method) {
            (Self::Equality([eq, _]), Method::Eq) => Some(eq),
            (Self::Equality([_, ne]), Method::Ne) => Some(ne),
            (Self::PartialOrder([partial, _, _, _, _]), Method::PartialCmp) => Some(partial),
            (Self::PartialOrder([_, lt, _, _, _]), Method::Lt) => Some(lt),
            (Self::PartialOrder([_, _, le, _, _]), Method::Le) => Some(le),
            (Self::PartialOrder([_, _, _, gt, _]), Method::Gt) => Some(gt),
            (Self::PartialOrder([_, _, _, _, ge]), Method::Ge) => Some(ge),
            (Self::TotalOrder([cmp, _, _, _]), Method::Cmp) => Some(cmp),
            (Self::TotalOrder([_, min, _, _]), Method::Min) => Some(min),
            (Self::TotalOrder([_, _, max, _]), Method::Max) => Some(max),
            (Self::TotalOrder([_, _, _, clamp]), Method::Clamp) => Some(clamp),
            _ => None,
        }
    }
    pub fn iter(self) -> impl Iterator<Item = (Method, DefId)> {
        Method::ALL
            .into_iter()
            .filter_map(move |method| self.get(method).map(|definition| (method, definition)))
    }
    fn new(kind: Comparison, members: &[DefId]) -> Option<Self> {
        match (kind, members) {
            (Comparison::PartialEq, [eq, ne]) => Some(Self::Equality([*eq, *ne])),
            (Comparison::PartialOrd, [partial, lt, le, gt, ge]) => {
                Some(Self::PartialOrder([*partial, *lt, *le, *gt, *ge]))
            }
            (Comparison::Ord, [cmp, min, max, clamp]) => {
                Some(Self::TotalOrder([*cmp, *min, *max, *clamp]))
            }
            _ => None,
        }
    }
}
impl Traits {
    #[must_use]
    pub fn comparison(&self, kind: Comparison) -> Option<Equality> {
        self.comparisons.as_deref()?.entries[kind as usize]
    }
}
impl Resolver<'_> {
    pub(in crate::resolve) fn prepare_comparison_method(
        &mut self,
        c: Context,
        node: NodeId,
        method: Method,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.library.is_none() {
            self.prepare_external_traits(c, node)?;
        }
        self.prepare_comparison(c, node, method.owner())
    }
    pub(in crate::resolve) fn prepare_comparison(
        &mut self,
        c: Context,
        node: NodeId,
        kind: Comparison,
    ) -> Result<(), Error> {
        if self.facts.builtin_traits.comparison(kind).is_some() {
            return Ok(());
        }
        for &parent in kind.parents() {
            self.prepare_comparison(c, node, parent)?;
        }
        if matches!(kind, Comparison::PartialOrd | Comparison::Ord) {
            self.prepare_library_type(
                c,
                node,
                crate::support::library_types::LibraryType::ComparisonOrdering,
            )?;
            self.prepare_library_type(c, node, crate::support::library_types::LibraryType::Option)?;
        }
        // Preparing the result nominal may complete this prerequisite through Ord.
        if self.facts.builtin_traits.comparison(kind).is_some() {
            return Ok(());
        }
        let library = self
            .facts
            .builtin_traits
            .library
            .ok_or_else(|| self.error(c, node, "missing comparison library"))?;
        let cmp = self.library_module(
            c,
            node,
            library,
            crate::support::library_types::LibraryModule::Comparison,
        )?;
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
        let argument = if kind.has_rhs() {
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
        let mut method_ids = Vec::new();
        for method in kind.methods() {
            method_ids.push(self.builtin_definition(
                Context {
                    scope: members,
                    ..c
                },
                node,
                method.name(),
                DefKind::Function,
                None,
            )?);
        }
        let methods = Members::new(kind, &method_ids);
        self.facts
            .builtin_traits
            .comparisons
            .get_or_insert_default()
            .entries[kind as usize] = Some(Equality {
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
