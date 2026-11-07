//! Const qualification preserves the parsed wrapper and its actual ordinary child.
use crate::{
    resolve::{expansion::Report, unit::NodeKey},
    syntax::{ast::Kind, gate_sites::ExpressionKind},
};

pub(crate) fn child(report: &Report, wrapper: NodeKey) -> Result<NodeKey, &'static str> {
    let ast = &report.package.unit(wrapper.file).configured.ast;
    let Some(node) = ast.nodes.get(wrapper.node.get() as usize) else {
        return Err("const closure wrapper is outside its source unit");
    };
    let Kind::ConstClosure { closure, gate } = node.kind else {
        return Err("const closure lost its actual source wrapper");
    };
    let site = ast
        .expression_gate(gate)
        .ok_or("const closure gate is missing")?;
    if site.node != wrapper.node
        || site
            .validate(ast, gate)
            .map_err(|_| "const closure gate source changed")?
            != closure
        || site
            .kind(ast)
            .map_err(|_| "const closure gate kind changed")?
            != ExpressionKind::ConstClosure
    {
        return Err("const closure wrapper differs from its actual gate");
    }
    let node = ast
        .nodes
        .get(closure.get() as usize)
        .ok_or("const closure child is missing")?;
    let closure = if let Kind::Attributed { value, .. } = node.kind {
        value
    } else {
        closure
    };
    if !matches!(
        ast.nodes.get(closure.get() as usize).map(|node| node.kind),
        Some(Kind::Closure {
            asynchronous: false,
            ..
        })
    ) {
        return Err("const closure requires its actual ordinary closure child");
    }
    Ok(NodeKey {
        node: closure,
        ..wrapper
    })
}

impl super::Body {
    pub(crate) fn check_const_source(&self, report: &Report) -> Result<bool, &'static str> {
        let Some(wrapper) = self.constant else {
            return Ok(false);
        };
        if child(
            report,
            NodeKey {
                node: wrapper,
                ..self.source
            },
        )? != self.source
        {
            return Err("const closure source differs from its authenticated child");
        }
        Ok(true)
    }
}
