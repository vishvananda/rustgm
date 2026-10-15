//! Prepare written method types and const dependencies before receiver selection.
use super::{Checker, Error, NodeId, NodeKey, regions};
use crate::sema::{
    arguments,
    generics::{DeferredArgument, DeferredUse},
};

impl Checker<'_> {
    pub(super) fn prepare_method_arguments(
        &mut self,
        key: NodeKey,
        method: NodeId,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        let (lifetimes, supplied) = arguments::groups(
            self.report,
            NodeKey {
                node: method,
                ..key
            },
            false,
        )?;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let mut arguments = Vec::new();
        for &node in supplied.map_or(&[][..], |list| ast.list(list)) {
            let source = NodeKey { node, ..key };
            let argument = arguments::written(self.report, &self.facts, source);
            match argument {
                DeferredArgument::Type(source) => {
                    self.annotation_holes(source, steps, limit, scan)?;
                    self.node_work(source.file, source.node, context);
                }
                DeferredArgument::Const(source) => self.constant_dependencies(source, context)?,
                DeferredArgument::Infer => {}
            }
            arguments.push(argument);
        }
        self.drain(steps, limit)?;
        self.facts.generics.method_uses.insert(
            key,
            DeferredUse {
                lifetimes,
                arguments,
            },
        );
        Ok(())
    }
}
