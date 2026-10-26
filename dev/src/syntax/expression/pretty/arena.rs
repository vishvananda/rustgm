//! The cold writer borrows real arena rows, including already-parsed opaque syntax.
use crate::syntax::{
    ast::{Node, Syntax},
    item::{AttributeCapture, FunctionType, GenericParameter, Item, UseTree, Variant},
    token_tree::Forest,
};
pub(super) trait Arena {
    fn nodes(&self) -> &[Node];
    fn edges(&self) -> &[crate::syntax::ast::NodeId];
    fn items(&self) -> &[Item];
    fn generics(&self) -> &[GenericParameter];
    fn functions(&self) -> &[FunctionType];
    fn variants(&self) -> &[Variant];
    fn uses(&self) -> &[UseTree];
    fn attributes(&self) -> &[AttributeCapture];
    fn captures(&self) -> &[Forest];
    fn defaults(&self) -> &[crate::syntax::defaultness::Declaration];
}
impl<L> Arena for Syntax<L> {
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[crate::syntax::ast::NodeId] {
        &self.edges
    }
    fn items(&self) -> &[Item] {
        &self.items
    }
    fn generics(&self) -> &[GenericParameter] {
        &self.generics
    }
    fn functions(&self) -> &[FunctionType] {
        &self.function_types
    }
    fn variants(&self) -> &[Variant] {
        &self.variants
    }
    fn uses(&self) -> &[UseTree] {
        &self.use_trees
    }
    fn attributes(&self) -> &[AttributeCapture] {
        &self.attributes
    }
    fn captures(&self) -> &[Forest] {
        &self.captures
    }
    fn defaults(&self) -> &[crate::syntax::defaultness::Declaration] {
        self.defaultness
            .as_ref()
            .map_or(&[], |entries| entries.entries())
    }
}
