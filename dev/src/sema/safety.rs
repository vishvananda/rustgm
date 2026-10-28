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
pub(super) struct Policies {
    nodes: Option<Vec<Vec<u32>>>,
    rows: Vec<Row>,
    pub(super) progress: Option<Box<crate::resolve::lint_policy::Progress>>,
    unreachable_denials: bool,
    work: Work,
}
impl Default for Policies {
    fn default() -> Self {
        Self {
            nodes: None,
            rows: Vec::new(),
            progress: None,
            unreachable_denials: true,
            work: Work::default(),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) enum Completion {
    Whole,
    Function(crate::resolve::model::DefId),
    Initializer(crate::resolve::model::DefId),
    Declaration,
}
struct Row {
    policy: Policy,
    completion: Completion,
}
impl Policies {
    #[cfg(test)]
    fn retained_storage(&self) -> (usize, usize, usize) {
        self.nodes.as_ref().map_or((0, 0, 0), |files| {
            let (len, capacity) = files.iter().fold((0, 0), |(len, capacity), nodes| {
                (len + nodes.len(), capacity + nodes.capacity())
            });
            (len, capacity, files.capacity() * size_of::<Vec<u32>>())
        })
    }
    pub(super) fn analyze(
        report: &Report,
        progress: Option<Box<crate::resolve::lint_policy::Progress>>,
    ) -> Result<Self, Error> {
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
        let key = NodeKey {
            file: root_file(report),
            node: report.package.modules[0].root,
        };
        policies.progress = if let Some(producer) = producer {
            if let Some(progress) = progress {
                producer.validate_progress(&progress, key)?;
                Some(progress)
            } else {
                producer.new_progress(key)?
            }
        } else if progress.is_some() {
            return Err(Error::at(
                key.file,
                0,
                "lint request lost its source producer",
            ));
        } else {
            None
        };
        policies.work.attributes = attributes;
        policies.work.entries = entries;
        if !has_changes {
            return Ok(policies);
        }
        policies.work.retained_nodes = report
            .package
            .units
            .iter()
            .filter(|unit| unit.include_context.is_none())
            .map(|unit| unit.configured.ast.nodes.len())
            .sum();
        if let Some(producer) = producer {
            let growth = crate::support::retained::Growth::vector::<Row>(0, 0, 1);
            producer.diagnostic_payload(
                key,
                policies.work.retained_nodes.saturating_add(growth.work),
                policies
                    .work
                    .retained_nodes
                    .saturating_mul(size_of::<u32>())
                    .saturating_add(
                        report
                            .package
                            .units
                            .len()
                            .saturating_mul(size_of::<Vec<u32>>()),
                    )
                    .saturating_add(growth.bytes),
            )?;
        }
        let mut nodes: Vec<_> = report
            .package
            .units
            .iter()
            .map(|unit| {
                if unit.include_context.is_some() {
                    Vec::new()
                } else {
                    vec![0; unit.configured.ast.nodes.len()]
                }
            })
            .collect();
        policies.rows.push(Row {
            policy: Policy::default(),
            completion: Completion::Whole,
        });
        policies.walk(report, producer, &mut nodes)?;
        policies.nodes = Some(nodes);
        Ok(policies)
    }
    fn walk(
        &mut self,
        report: &Report,
        producer: Option<&Producer>,
        nodes: &mut [Vec<u32>],
    ) -> Result<(), Error> {
        let package = &report.package;
        let root = &package.modules[0];
        let mut work = vec![(
            ModuleId(0),
            NodeKey {
                file: root.file,
                node: root.root,
            },
            0_u32,
            None::<Tail>,
            Completion::Whole,
        )];
        let mut children = Vec::new();
        let budget = self
            .work
            .retained_nodes
            .checked_mul(2)
            .and_then(|n| n.checked_add(package.modules.len()))
            .ok_or_else(|| Error::at(root.file, 0, "safety policy work budget overflow"))?;
        while let Some((module, key, mut row, mut tail, mut completion)) = work.pop() {
            self.work.nodes += 1;
            if self.work.nodes > budget {
                return Err(Error::at(key.file, 0, "safety policy work budget exceeded"));
            }
            let unit = package.unit(key.file);
            let removed = report.removed_function_owner(key);
            if !unit.configured.active[key.node.get() as usize] && !removed {
                continue;
            }
            if !removed {
                completion = completion_for(report, module, key, completion, producer)?;
            }
            if !removed && let Some(producer) = producer {
                let changes = producer.changes(package, key, tail)?;
                let deprecated_origin = changes.deprecated;
                if !changes.entries.is_empty() || deprecated_origin.is_some() {
                    let growth = crate::support::retained::Growth::vector::<Row>(
                        self.rows.len(),
                        self.rows.capacity(),
                        1,
                    );
                    producer.diagnostic_payload(key, 1 + growth.work, growth.bytes)?;
                    let mut policy = self.rows[row as usize].policy;
                    for change in changes.entries {
                        policy.apply(key, change)?;
                    }
                    row = u32::try_from(self.rows.len()).map_err(|_| {
                        Error::at(key.file, 0, "lint policy row identity exceeds 32 bits")
                    })?;
                    if let Some(origin) = deprecated_origin {
                        policy.deprecated_origin = Some(origin);
                    }
                    self.rows.push(Row { policy, completion });
                }
                tail = changes.tail;
            }
            self.unreachable_denials |=
                self.rows[row as usize].policy.unreachable_patterns != Level::Allow;
            nodes[key.file.0 as usize][key.node.get() as usize] = row;
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
                    row,
                    tail,
                    completion,
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
                    .map(|&node| (module, NodeKey { node, ..key }, row, tail, completion)),
            );
        }
        Ok(())
    }
    fn at(&self, key: NodeKey) -> &Policy {
        self.nodes.as_ref().map_or(&Policy::DEFAULT, |files| {
            &self.rows[files[key.file.0 as usize][key.node.get() as usize] as usize].policy
        })
    }
    pub(super) fn explicit_deprecation(&self, key: NodeKey) -> bool {
        self.at(key).deprecation_explicit
    }
    pub(super) fn deprecated_origin(&self, key: NodeKey) -> Option<Tail> {
        self.at(key).deprecated_origin
    }
    pub(super) fn completion(&self, key: NodeKey) -> Completion {
        self.nodes.as_ref().map_or(Completion::Whole, |files| {
            self.rows[files[key.file.0 as usize][key.node.get() as usize] as usize].completion
        })
    }
    pub(super) fn expectation(
        &self,
        key: NodeKey,
        lint: crate::resolve::lint_policy::Lint,
    ) -> Option<crate::resolve::lint_policy::ExpectationId> {
        self.at(key).expectation(lint)
    }
    pub(super) fn deprecated(&self, key: NodeKey) -> Level {
        self.at(key).deprecated
    }
    pub(super) fn unfulfilled(&self, key: NodeKey) -> Level {
        self.at(key).unfulfilled
    }
    pub(super) fn atomic_ordering(&self, key: NodeKey) -> Level {
        self.at(key).atomic_ordering
    }
    pub(super) fn unreachable_patterns(&self, key: NodeKey) -> Level {
        self.at(key).unreachable_patterns
    }
    pub(super) fn operation(&self, key: NodeKey) -> Level {
        self.at(key).operation
    }
    pub(super) fn reference(&self, key: NodeKey) -> Level {
        self.at(key).reference
    }
    pub(super) fn has_unreachable_denials(&self) -> bool {
        self.unreachable_denials
    }
    pub(super) fn must_use(&self, key: NodeKey) -> Level {
        self.at(key).must_use
    }
    pub(super) fn unused_attributes(&self, key: NodeKey) -> Level {
        self.at(key).unused_attributes
    }
    pub(super) fn unused_visibilities(&self, key: NodeKey) -> Level {
        self.at(key).unused_visibilities
    }
    pub(super) fn duplicate_macro_attributes(&self, key: NodeKey) -> Level {
        self.at(key).duplicate_macro_attributes
    }
    pub(super) fn ambiguous_derive_helpers(&self, key: NodeKey) -> Level {
        self.at(key).ambiguous_derive_helpers
    }
    pub(super) fn diagnostic_attributes(&self, key: NodeKey, index: usize) -> Level {
        self.at(key).diagnostic_attributes[index]
    }
    pub(super) fn work(&self) -> Work {
        self.work
    }
}

#[cfg(test)]
impl crate::sema::model::Declarations {
    pub(crate) fn retained_policy_storage(&self) -> (usize, usize, usize) {
        self.safety_policies.retained_storage()
    }
    pub(crate) fn retained_policy_rows(&self) -> (usize, usize, usize) {
        let rows = &self.safety_policies.rows;
        (rows.len(), rows.capacity(), size_of::<Row>())
    }
}

fn root_file(report: &Report) -> crate::resolve::unit::FileId {
    report.package.modules[0].file
}

fn package_key_definition(
    report: &Report,
    module: ModuleId,
    owner: NodeKey,
    node: crate::syntax::ast::NodeId,
) -> Option<crate::resolve::model::DefId> {
    report
        .resolution
        .item_definitions
        .get(&(module, node, crate::resolve::model::Namespace::Value))
        .copied()
        .or_else(|| {
            report
                .resolution
                .item_definitions
                .get(&(module, owner.node, crate::resolve::model::Namespace::Value))
                .copied()
        })
}

fn completion_for(
    report: &Report,
    module: ModuleId,
    key: NodeKey,
    inherited: Completion,
    producer: Option<&Producer>,
) -> Result<Completion, Error> {
    use crate::syntax::{ast::Kind, item::ItemKind};
    let unit = report.package.unit(key.file);
    let mut node = key.node;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        if let Some(producer) = producer {
            producer.attribute_read(key, 1)?;
        }
        node = value;
    }
    let Kind::Item(index) = unit.configured.ast.node(node).kind else {
        return Ok(inherited);
    };
    let identity = || {
        package_key_definition(report, module, key, node).ok_or_else(|| {
            Error::at(
                key.file,
                0,
                "lint completion lost actual declaration identity",
            )
        })
    };
    match unit.configured.ast.items[index as usize].kind {
        ItemKind::Function { body: Some(_), .. } => Ok(Completion::Function(identity()?)),
        ItemKind::Const { value: Some(_), .. } | ItemKind::Static { value: Some(_), .. } => {
            Ok(Completion::Initializer(identity()?))
        }
        ItemKind::Function { body: None, .. }
        | ItemKind::Const { value: None, .. }
        | ItemKind::Static { value: None, .. }
        | ItemKind::Use(_) => Ok(Completion::Declaration),
        _ => Ok(inherited),
    }
}
