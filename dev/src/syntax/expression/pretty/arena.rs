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
    fn site_count(&self) -> usize;
    fn defaultness(&self, item: u32) -> Option<&crate::syntax::gate_sites::Declaration>;
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
    fn site_count(&self) -> usize {
        self.gate_sites.as_ref().map_or(0, |entries| entries.len())
    }
    fn defaultness(&self, item: u32) -> Option<&crate::syntax::gate_sites::Declaration> {
        Syntax::defaultness(self, item)
    }
}
