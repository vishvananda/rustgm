//! Slice inherent metadata has a real impl owner and no free-function binding.
use super::{Context, DefId, DefKind, Error, Resolver};
use crate::{
    resolve::{package::ModuleId, unit::NodeKey},
    syntax::ast::NodeId,
};
#[derive(Clone, Copy)]
pub struct SliceLength {
    pub owner: DefId,
    pub definition: DefId,
    pub parameter: DefId,
    pub origin: NodeKey,
}
impl Resolver<'_> {
    pub(in crate::resolve) fn prepare_slice_length(
        &mut self,
        c: Context,
        node: NodeId,
    ) -> Result<(), Error> {
        if self.facts.library_types.slice_length.is_some() {
            return Ok(());
        }
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let context = Context { scope, ..c };
        let owner = self.builtin_definition(context, node, "Self", DefKind::SelfType, None)?;
        let definition = self.builtin_definition(context, node, "len", DefKind::Function, None)?;
        let parameter = self.builtin_definition(context, node, "T", DefKind::Generic, None)?;
        self.facts.library_types.slice_length = Some(Box::new(SliceLength {
            owner,
            definition,
            parameter,
            origin: NodeKey { file: c.file, node },
        }));
        Ok(())
    }
    pub(in crate::resolve) fn prepare_slice_method(
        &mut self,
        c: Context,
        node: NodeId,
        name: super::NameId,
    ) -> Result<(), Error> {
        if self.package.names.get(name) == "add" {
            self.prepare_pointer_methods(c, node)?;
        }
        if matches!(self.package.names.get(name), "iter" | "iter_mut") {
            self.facts
                .builtin_traits
                .note_collection(c.module, NodeKey { file: c.file, node });
        }
        if self.package.names.get(name) == "len" {
            self.prepare_slice_length(c, node)?;
        }
        Ok(())
    }
}
