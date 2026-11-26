//! After the direct path resolves, retire legacy const operands from the
//! runtime call and replay them through ordinary generic argument elaboration.
use super::{Context, Error, Kind, List, Namespace, NodeId, NodeKey, Resolver, Target};
use crate::expand::crate_syntax::ConfiguredSyntax;

impl Resolver<'_> {
    pub(super) fn legacy_call_candidate(&self, context: Context, callee: NodeId) -> bool {
        if self
            .facts
            .crates
            .as_deref()
            .is_none_or(|imports| !imports.has_legacy_const_calls())
        {
            return false;
        }
        let ast = &self.package.unit(context.file).configured.ast;
        ast.path_segments(callee).is_some_and(|(_, segments)| {
            segments.iter().all(|segment| {
                matches!(ast.node(*segment).kind, Kind::Name(..) | Kind::CrateRoot(_))
            })
        })
    }

    pub(super) fn resolve_legacy_call(
        &mut self,
        context: Context,
        node: NodeId,
        callee: NodeId,
        written: List,
    ) -> Result<(), Error> {
        let first = self.facts.references.len();
        self.visit(context, callee, Namespace::Value)?;
        let key = NodeKey {
            node: callee,
            file: context.file,
        };
        let header =
            self.facts
                .references
                .get(first)
                .and_then(|reference| match reference.target {
                    Target::Definition(definition) if reference.node == key => self
                        .facts
                        .crates
                        .as_deref()?
                        .legacy_const_header(definition),
                    _ => None,
                });
        if let Some(header) = header {
            let syntax = &mut self.package.units[context.file.0 as usize].configured;
            let (arguments, constants) =
                partition(syntax, written, &header.positions, header.runtime, callee)
                    .map_err(|error| self.error(context, node, error))?;
            let imports = self
                .facts
                .crates
                .as_deref_mut()
                .ok_or_else(|| Error::unlocated("missing imported legacy const state"))?;
            imports.record_legacy_const_arguments(key, constants, written.end - written.start)?;
            self.package.units[context.file.0 as usize]
                .configured
                .ast
                .nodes[node.get() as usize]
                .kind = Kind::Call { callee, arguments };
        }
        let ast = &self.package.unit(context.file).configured.ast;
        self.visits.extend(
            ast.list(written)
                .iter()
                .rev()
                .map(|&argument| (context, argument, Namespace::Value)),
        );
        Ok(())
    }
}

fn partition<L>(
    syntax: &mut ConfiguredSyntax<L>,
    written: List,
    positions: &[u32],
    runtime: u32,
    callee: NodeId,
) -> Result<(List, List), &'static str> {
    let count = syntax
        .ast
        .list(written)
        .iter()
        .filter(|node| syntax.active[node.get() as usize])
        .count();
    if count != runtime as usize + positions.len() {
        return Err("legacy const call argument count mismatch");
    }
    let edges = &mut syntax.ast.edges;
    let start = u32::try_from(edges.len()).map_err(|_| "syntax argument index overflow")?;
    let split = start
        .checked_add(runtime)
        .ok_or("syntax argument index overflow")?;
    let end = split
        .checked_add(u32::try_from(positions.len()).map_err(|_| "syntax argument index overflow")?)
        .ok_or("syntax argument index overflow")?;
    edges
        .try_reserve(count)
        .map_err(|_| "cannot reserve legacy const arguments")?;
    edges.resize(end as usize, callee);
    let active = &syntax.active;
    let mut runtime = start;
    let mut constant = split;
    let mut ordinal = 0;
    for index in written.start..written.end {
        let argument = edges[index as usize];
        if !active[argument.get() as usize] {
            continue;
        }
        if positions.get((constant - split) as usize) == Some(&ordinal) {
            edges[constant as usize] = argument;
            constant += 1;
        } else {
            edges[runtime as usize] = argument;
            runtime += 1;
        }
        ordinal += 1;
    }
    Ok((List { start, end: split }, List { start: split, end }))
}
