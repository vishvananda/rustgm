//! Const expressions may bind their own lifetimes, but cannot capture outer generics.
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefKind, ScopeId, Target},
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::ast::Kind,
};

#[derive(Clone, Copy)]
struct Capture {
    position: usize,
    source: NodeKey,
}
#[derive(Clone, Copy)]
struct Range {
    first: Capture,
    last: Capture,
}
#[derive(Default)]
pub(in crate::sema) struct Captures {
    scopes: Option<Vec<(usize, usize)>>,
    complete: Vec<Vec<bool>>,
    nodes: usize,
    summaries: HashMap<NodeKey, Range>,
}
impl Captures {
    pub fn check(
        &mut self,
        report: &Report,
        facts: &Declarations,
        root: NodeKey,
        module: ModuleId,
    ) -> Result<(), Error> {
        self.collect(report, facts, root)
            .map_err(|e| Declarations::error(report, root, e))?;
        let Some(&range) = self.summaries.get(&root) else {
            return Ok(());
        };
        let interval = report
            .resolution
            .node_scopes
            .get(&(module, root.node))
            .and_then(|id| self.scopes.as_ref()?.get(id.0 as usize))
            .copied();
        let invalid = if interval.is_none_or(|(start, _)| range.first.position < start) {
            Some(range.first.source)
        } else if interval.is_none_or(|(_, end)| range.last.position >= end) {
            Some(range.last.source)
        } else {
            None
        };
        if let Some(source) = invalid {
            return Err(Declarations::error(
                report,
                source,
                "constant expression cannot use an enclosing generic parameter",
            ));
        }
        Ok(())
    }
    fn collect(
        &mut self,
        report: &Report,
        facts: &Declarations,
        root: NodeKey,
    ) -> Result<(), &'static str> {
        // Most constant syntax contains no generic reference. Keep completion in
        // a compact node-indexed table; only actual captures need hashed ranges.
        self.complete
            .resize_with(report.package.units.len(), Vec::new);
        let file = root.file.0 as usize;
        if self.complete[file].is_empty() {
            self.complete[file].resize(
                report.package.unit(root.file).configured.ast.nodes.len(),
                false,
            );
        }
        if self.complete[file][root.node.0 as usize] {
            return Ok(());
        }
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((key, finish)) = work.pop() {
            if self.complete[file][key.node.0 as usize] {
                continue;
            }
            let unit = report.package.unit(key.file);
            if !unit.configured.active[key.node.0 as usize]
                || matches!(unit.configured.ast.node(key.node).kind, Kind::Item(_))
            {
                // Nested items have independent capture barriers in resolution.
                self.complete[file][key.node.0 as usize] = true;
                self.nodes += 1;
                continue;
            }
            let scope = facts
                .node(key)
                .and_then(|f| f.reference)
                .and_then(|target| {
                    let Target::Definition(id) = target else {
                        return None;
                    };
                    let definition = &report.resolution.definitions[id.0 as usize];
                    (definition.kind == DefKind::Generic).then_some(definition.scope)
                });
            if let Some(scope) = scope {
                let capture = Capture {
                    position: self.scope(report, scope)?,
                    source: key,
                };
                self.summaries.insert(
                    key,
                    Range {
                        first: capture,
                        last: capture,
                    },
                );
                self.complete[file][key.node.0 as usize] = true;
                self.nodes += 1;
                continue;
            }
            children.clear();
            unit.configured.ast.push_children(key.node, &mut children);
            if self
                .nodes
                .saturating_add(work.len())
                .saturating_add(children.len())
                > 1_000_000
            {
                return Err("constant capture work budget exceeded");
            }
            if finish {
                let mut range: Option<Range> = None;
                for &node in &children {
                    if !self.complete[file][node.0 as usize] {
                        return Err("missing constant capture dependency");
                    }
                    if let Some(child) = self.summaries.get(&NodeKey { node, ..key }) {
                        let range = range.get_or_insert(*child);
                        if child.first.position < range.first.position {
                            range.first = child.first;
                        }
                        if child.last.position > range.last.position {
                            range.last = child.last;
                        }
                    }
                }
                if let Some(range) = range {
                    self.summaries.insert(key, range);
                }
                self.complete[file][key.node.0 as usize] = true;
                self.nodes += 1;
            } else {
                work.push((key, true));
                work.extend(
                    children
                        .iter()
                        .rev()
                        .map(|&node| (NodeKey { node, ..key }, false)),
                );
            }
        }
        Ok(())
    }
    fn scope(&mut self, report: &Report, scope: ScopeId) -> Result<usize, &'static str> {
        if self.scopes.is_none() {
            self.scopes = Some(intervals(report)?);
        }
        self.scopes
            .as_ref()
            .and_then(|s| s.get(scope.0 as usize))
            .map(|&(start, _)| start)
            .ok_or("missing generic declaration scope")
    }
}

/// The enclosing constant contains every referenced binder exactly when both
/// extremal scope positions lie in its subtree. Two positions summarize any DAG
/// without copying free-variable sets into every ancestor.
fn intervals(report: &Report) -> Result<Vec<(usize, usize)>, &'static str> {
    let count = report.resolution.scopes.len();
    let mut first = vec![None; count];
    let mut next = vec![None; count];
    let mut work = Vec::new();
    for (index, scope) in report.resolution.scopes.iter().enumerate().rev() {
        if let Some(parent) = scope.parent {
            let first = first
                .get_mut(parent.0 as usize)
                .ok_or("invalid scope parent")?;
            next[index] = first.replace(index);
        } else {
            work.push((index, false));
        }
    }
    let mut result = vec![(0, 0); count];
    let mut position = 0;
    while let Some((index, finish)) = work.pop() {
        if finish {
            result[index].1 = position;
            continue;
        }
        result[index].0 = position;
        position += 1;
        work.push((index, true));
        let mut child = first[index];
        while let Some(index) = child {
            work.push((index, false));
            child = next[index];
        }
    }
    if position != count {
        return Err("invalid scope ancestry graph");
    }
    Ok(result)
}
