//! Lexical lint policy is independent of unsafe permission and item boundaries.
mod attributes;
use crate::resolve::{
    expansion::Report,
    package::{Error, ModuleId},
    unit::NodeKey,
};
use crate::support::hash::HashMap;

pub(super) use crate::resolve::lint_policy::Level;
use crate::resolve::lint_policy::{Change, Lint, Policy};
/// Actual retained-policy work; absent policies allocate no node table.
#[derive(Clone, Copy, Default)]
pub struct Work {
    pub attributes: usize,
    pub entries: usize,
    pub nodes: usize,
    pub edges: usize,
    pub retained_nodes: usize,
}
#[derive(Default)]
pub(super) struct Policies {
    nodes: Option<Vec<Vec<Policy>>>,
    unreachable_denials: bool,
    work: Work,
}
impl Policies {
    pub(super) fn analyze(report: &Report) -> Result<Self, Error> {
        if let Some(removals) = report.removed_functions.as_deref() {
            removals.validate(report)?;
        }
        let mut policies = Self::default();
        let changes = attributes::collect(report, &mut policies.work)?;
        if changes.is_empty() {
            return Ok(policies);
        }
        policies.unreachable_denials = changes.values().flatten().any(|change| {
            matches!(change.lint, Lint::UnreachablePatterns) && change.level.denies()
        });
        let mut nodes: Vec<_> = report
            .package
            .units
            .iter()
            .map(|unit| {
                if unit.include_context.is_some() {
                    Vec::new()
                } else {
                    vec![Policy::default(); unit.configured.ast.nodes.len()]
                }
            })
            .collect();
        policies.work.retained_nodes = nodes.iter().map(Vec::len).sum();
        policies.walk(report, &changes, &mut nodes)?;
        policies.nodes = Some(nodes);
        Ok(policies)
    }
    fn walk(
        &mut self,
        report: &Report,
        changes: &HashMap<NodeKey, Vec<Change>>,
        nodes: &mut [Vec<Policy>],
    ) -> Result<(), Error> {
        let package = &report.package;
        let root = &package.modules[0];
        let mut work = vec![(
            ModuleId(0),
            NodeKey {
                file: root.file,
                node: root.root,
            },
            Policy::default(),
        )];
        let mut children = Vec::new();
        let budget = self
            .work
            .retained_nodes
            .checked_mul(2)
            .and_then(|n| n.checked_add(package.modules.len()))
            .ok_or_else(|| Error::at(root.file, 0, "safety policy work budget overflow"))?;
        while let Some((module, key, mut policy)) = work.pop() {
            self.work.nodes += 1;
            if self.work.nodes > budget {
                return Err(Error::at(key.file, 0, "safety policy work budget exceeded"));
            }
            let unit = package.unit(key.file);
            let removed = report.removed_function_owner(key);
            if !unit.configured.active[key.node.get() as usize] && !removed {
                continue;
            }
            if let Some(changes) = changes.get(&key) {
                for change in changes {
                    policy.apply(key, change)?;
                }
            }
            nodes[key.file.0 as usize][key.node.get() as usize] = policy;
            if removed {
                continue;
            }
            if let Some(child) = package.children.get(&(module, key)) {
                let child_module = &package.modules[child.0 as usize];
                work.push((
                    *child,
                    NodeKey {
                        file: child_module.file,
                        node: child_module.root,
                    },
                    policy,
                ));
                self.work.edges += 1;
                continue;
            }
            children.clear();
            unit.configured.ast.push_children(key.node, &mut children);
            self.work.edges += children.len();
            work.extend(
                children
                    .iter()
                    .rev()
                    .map(|&node| (module, NodeKey { node, ..key }, policy)),
            );
        }
        Ok(())
    }
    fn at(&self, key: NodeKey) -> Policy {
        self.nodes.as_ref().map_or_else(Policy::default, |files| {
            files[key.file.0 as usize][key.node.get() as usize]
        })
    }
    pub(super) fn denies_operation(&self, key: NodeKey) -> bool {
        self.at(key).operation.denies()
    }
    pub(super) fn denies_reference(&self, key: NodeKey) -> bool {
        self.at(key).reference.denies()
    }
    pub(super) fn denies_atomic_ordering(&self, key: NodeKey) -> bool {
        self.at(key).atomic_ordering.denies()
    }
    pub(super) fn has_unreachable_denials(&self) -> bool {
        self.unreachable_denials
    }
    pub(super) fn denies_unreachable_patterns(&self, key: NodeKey) -> bool {
        self.at(key).unreachable_patterns.denies()
    }
    pub(super) fn must_use(&self, key: NodeKey) -> Level {
        self.at(key).must_use
    }
    pub(super) fn unused_attributes(&self, key: NodeKey) -> Level {
        self.at(key).unused_attributes
    }
    pub(super) fn duplicate_macro_attributes(&self, key: NodeKey) -> Level {
        self.at(key).duplicate_macro_attributes
    }
    pub(super) fn diagnostic_attributes(&self, key: NodeKey, index: usize) -> Level {
        self.at(key).diagnostic_attributes[index]
    }
    pub(super) fn work(&self) -> Work {
        self.work
    }
}
