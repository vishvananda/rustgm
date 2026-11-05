//! Authored async bodies retain their own checked output, separate from future storage.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::{Declarations, Function},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId},
};

impl Declarations {
    /// Authenticate the original function flag before reading its declared body output.
    /// Ordinary return-position impl Trait continues to infer its own hidden result.
    pub(crate) fn async_body_output(
        &self,
        report: &Report,
        function: &Function,
    ) -> Result<Option<TypeId>, Error> {
        let error = |message| Self::error(report, function.node, message);
        let definition = report
            .resolution
            .definitions
            .get(function.definition.0 as usize)
            .ok_or_else(|| error("function output lost its declaration identity"))?;
        // Canonical library callables retain a diagnostic site, not an authored
        // function node. Only the actual declaration can authorize async syntax.
        let Some(source) = definition.node else {
            return Ok(None);
        };
        if source != function.node {
            return Err(error("function output changed its source declaration"));
        }
        let ast = &report.package.unit(function.node.file).configured.ast;
        let Syntax::Item(index) = ast.node(function.node.node).kind else {
            return Err(error("function output lost its source declaration"));
        };
        let ItemKind::Function { flags, .. } = ast.items[index as usize].kind else {
            return Err(error("function output has a different actual source owner"));
        };
        if !flags.asynchronous {
            return Ok(None);
        }
        let Some(Kind::Function { result, .. }) = self.types.kind(function.signature) else {
            return Err(error("async function lacks its actual signature"));
        };
        let Some(Kind::Opaque(id, _)) = self.types.kind(result) else {
            return Err(error("async function lacks its genuine future identity"));
        };
        if self
            .opaques
            .declarations
            .get(&id)
            .is_none_or(|declaration| {
                declaration.owner != function.definition || declaration.source != function.node
            })
        {
            return Err(error("async future belongs to another source function"));
        }
        self.opaques
            .body_results
            .get(&function.definition)
            .copied()
            .map(Some)
            .ok_or_else(|| error("async function lacks its authored body output"))
    }
}
