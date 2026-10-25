//! Lexical lint policy is independent of unsafe permission and item boundaries.
use crate::resolve::{
    expansion::Report,
    package::{Error, ModuleId},
    unit::NodeKey,
};

pub(super) use crate::resolve::lint_policy::Level;
use crate::resolve::lint_policy::{
    Policy,
    producer::{Producer, Tail},
};
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
        if let Some(removals) = report.removed_functions() {
            removals.validate(report)?;
        }
        let mut policies = Self::default();
        let producer = report.lint_producer();
        let (attributes, entries, has_changes) = if let Some(producer) = producer {
            producer.validate(&report.package)?
        } else {
            for (file, unit) in report.package.units.iter().enumerate() {
                if unit.include_context.is_some() {
                    continue;
                }
                if let Some(owner) = unit.configured.owners.iter().find(|owner| {
                    owner.enabled
                        && unit.configured.active.get(owner.node.get() as usize) == Some(&true)
                        && !owner.attributes.is_empty()
                }) {
                    return Err(Error::at(
                        crate::resolve::unit::FileId(u32::try_from(file).map_err(|_| {
                            Error::at(root_file(report), 0, "lint source file index overflow")
                        })?),
                        unit.configured.ast.node(owner.node).span.start,
                        "active lint source owner has no producer",
                    ));
                }
            }
            (0, 0, false)
        };
        policies.work.attributes = attributes;
        policies.work.entries = entries;
        if !has_changes {
            return Ok(policies);
        }
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
        policies.walk(report, producer, &mut nodes)?;
        policies.nodes = Some(nodes);
        Ok(policies)
    }
    fn walk(
        &mut self,
        report: &Report,
        producer: Option<&Producer>,
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
            None::<Tail>,
        )];
        let mut children = Vec::new();
        let budget = self
            .work
            .retained_nodes
            .checked_mul(2)
            .and_then(|n| n.checked_add(package.modules.len()))
            .ok_or_else(|| Error::at(root.file, 0, "safety policy work budget overflow"))?;
        while let Some((module, key, mut policy, mut tail)) = work.pop() {
            self.work.nodes += 1;
            if self.work.nodes > budget {
                return Err(Error::at(key.file, 0, "safety policy work budget exceeded"));
            }
            let unit = package.unit(key.file);
            let removed = report.removed_function_owner(key);
            if !unit.configured.active[key.node.get() as usize] && !removed {
                continue;
            }
            if !removed && let Some(producer) = producer {
                let (changes, next) = producer.changes(package, key, tail)?;
                for change in changes {
                    policy.apply(key, change)?;
                }
                tail = next;
            }
            self.unreachable_denials |= policy.unreachable_patterns.denies();
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
                    tail,
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
                    .map(|&node| (module, NodeKey { node, ..key }, policy, tail)),
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

fn root_file(report: &Report) -> crate::resolve::unit::FileId {
    report.package.modules[0].file
}
