//! Lexical lint policy is independent of unsafe permission and item boundaries.
mod attributes;
use crate::resolve::{
    expansion::Report,
    package::{Error, ModuleId},
    unit::NodeKey,
};
use crate::support::hash::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Allow,
    Warn,
    Deny,
    Forbid,
}
impl Level {
    fn denies(self) -> bool {
        matches!(self, Self::Deny | Self::Forbid)
    }
}
#[derive(Clone, Copy)]
struct Policy {
    operation: Level,
    reference: Level,
    atomic_ordering: Level,
    unreachable_patterns: Level,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            operation: Level::Warn,
            reference: Level::Deny,
            atomic_ordering: Level::Deny,
            unreachable_patterns: Level::Warn,
        }
    }
}
#[derive(Clone, Copy)]
enum Lint {
    Operation,
    Reference,
    AtomicOrdering,
    UnreachablePatterns,
}
struct Change {
    lint: Lint,
    level: Level,
    offset: u32,
}
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
            if !unit.configured.active[key.node.0 as usize] {
                continue;
            }
            if let Some(changes) = changes.get(&key) {
                for change in changes {
                    policy.apply(key, change)?;
                }
            }
            nodes[key.file.0 as usize][key.node.0 as usize] = policy;
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
            files[key.file.0 as usize][key.node.0 as usize]
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
    pub(super) fn work(&self) -> Work {
        self.work
    }
}
impl Policy {
    fn apply(&mut self, key: NodeKey, change: &Change) -> Result<(), Error> {
        let level = match change.lint {
            Lint::Operation => &mut self.operation,
            Lint::Reference => &mut self.reference,
            Lint::AtomicOrdering => &mut self.atomic_ordering,
            Lint::UnreachablePatterns => &mut self.unreachable_patterns,
        };
        if *level == Level::Forbid {
            if matches!(change.level, Level::Allow | Level::Warn) {
                return Err(Error::at(
                    key.file,
                    change.offset,
                    match change.lint {
                        Lint::AtomicOrdering => "forbidden atomic ordering lint cannot be weakened",
                        Lint::UnreachablePatterns => {
                            "forbidden unreachable pattern lint cannot be weakened"
                        }
                        _ => "forbidden unsafe lint cannot be weakened",
                    },
                ));
            }
            return Ok(());
        }
        *level = change.level;
        Ok(())
    }
}
