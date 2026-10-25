//! Immutable sessions validate each retained retirement path and shared parent once.
use super::{Error, Removal, Removals, Report, saved_uses};
use crate::{
    resolve::unit::{FileId, Unit},
    support::{
        hash::{HashMap, HashSet},
        source::OriginalImage,
    },
    syntax::ast::{Node, NodeId},
};
use std::{cell::Cell, sync::Arc};
#[cfg(test)]
#[path = "../../../../tests/invariants/removed_test_demand.rs"]
mod tests;

#[derive(Clone, Copy, Default)]
pub(super) struct Work {
    pub steps: usize,
    pub bytes: usize,
}
pub(super) fn charge(
    work: &Cell<Work>,
    file: FileId,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    let mut current = work.get();
    current.steps = current.steps.saturating_add(steps);
    current.bytes = current.bytes.saturating_add(bytes);
    work.set(current);
    if current.steps > 10_000_000 || current.bytes > 64 * 1024 * 1024 {
        return Err(Error::at(
            file,
            0,
            "Test demand receipt work or storage budget exceeded",
        ));
    }
    Ok(())
}
pub(super) struct Path {
    pub node: NodeId,
    pub snapshot: Node,
    pub parent: Option<usize>,
    pub children: Vec<(usize, usize)>,
    pub auxiliary: Option<Box<super::parent_edges::Auxiliary>>,
}
pub(super) struct Body {
    pub removal: usize,
    pub image: Arc<OriginalImage>,
    pub declaration: Node,
    /// Actual Function.body edge target, whose own attributes remain demanded.
    pub root: NodeId,
    pub snapshot: Node,
    pub edge_extent: usize,
    pub identity: super::identities::Identity,
}
#[derive(Default)]
pub(super) struct Index {
    pub paths: Vec<Path>,
    pub bodies: Vec<Body>,
    pub effects: Vec<saved_uses::Witness>,
    pub lookup: HashMap<(FileId, usize), usize>,
}
#[cfg(test)]
impl Index {
    fn retained_payload(&self) -> usize {
        let mut allocation = crate::support::retained::Allocation::default();
        allocation.boxed(self);
        allocation.vector(&self.paths);
        allocation.vector(&self.bodies);
        allocation.vector(&self.effects);
        allocation.map(&self.lookup);
        for path in &self.paths {
            allocation.vector(&path.children);
            if let Some(auxiliary) = &path.auxiliary {
                allocation.boxed(auxiliary.as_ref());
            }
        }
        allocation.bytes
            + self
                .effects
                .iter()
                .map(saved_uses::Witness::retained_payload)
                .sum::<usize>()
            + self
                .bodies
                .iter()
                .map(|body| body.identity.retained_payload())
                .sum::<usize>()
    }
}
#[derive(Default)]
struct Memo {
    bodies: HashSet<usize>,
    paths: HashSet<usize>,
    nodes: HashSet<usize>,
    effects: HashSet<usize>,
}
pub(crate) struct ConfiguredDemandSession<'report> {
    report: &'report Report,
    memo: Option<Box<Memo>>,
    ordinary_work: Cell<Work>,
}
pub(crate) enum DemandDisposition<'session> {
    Demanded,
    RemovedTestBody(RemovedTestBody<'session>),
}
/// Borrowed evidence of retirement only; this cannot authorize an attribute or feature.
pub(crate) struct RemovedTestBody<'session> {
    _effect: &'session saved_uses::Witness,
}
impl Report {
    pub(crate) fn configured_demand_session(&self) -> ConfiguredDemandSession<'_> {
        ConfiguredDemandSession {
            report: self,
            memo: None,
            ordinary_work: Cell::default(),
        }
    }
}
impl<'report> ConfiguredDemandSession<'report> {
    fn work(&self) -> &Cell<Work> {
        self.report
            .removed_functions()
            .map_or(&self.ordinary_work, |r| &r.work)
    }
    pub(crate) fn checked(
        &self,
        file: FileId,
        ordinal: usize,
    ) -> Result<saved_uses::CheckedConfiguredUse<'report>, Error> {
        saved_uses::checked(self.report, file, ordinal, self.work())
    }
    pub(crate) fn classify(
        &mut self,
        used: &saved_uses::CheckedConfiguredUse<'report>,
    ) -> Result<DemandDisposition<'_>, Error> {
        let file = used.file();
        charge(self.work(), file, 1, 0)?;
        if !used.belongs_to(self.report) {
            return Err(Error::at(
                file,
                0,
                "configured demand use belongs to another report",
            ));
        }
        let Some(removals) = self.report.removed_functions() else {
            return Ok(DemandDisposition::Demanded);
        };
        let Some(index) = removals.demand.as_deref() else {
            return Ok(DemandDisposition::Demanded);
        };
        let Some(&effect) = index.lookup.get(&(file, used.effect())) else {
            return Ok(DemandDisposition::Demanded);
        };
        let witness = &index.effects[effect];
        witness.validate_use(used)?;
        if self.memo.is_none() {
            charge(&removals.work, file, 1, size_of::<Memo>())?;
        }
        let memo = self.memo.get_or_insert_default();
        if !memo.effects.contains(&effect) {
            witness.validate(used, &removals.work)?;
            charge(&removals.work, file, 1, 4 * size_of::<usize>())?;
            memo.effects.insert(effect);
        }
        validate_body(self.report, removals, index, witness.body, memo)?;
        validate_path(self.report, removals, index, witness.path, file, memo)?;
        Ok(DemandDisposition::RemovedTestBody(RemovedTestBody {
            _effect: witness,
        }))
    }
}
fn validate_body(
    report: &Report,
    removals: &Removals,
    index: &Index,
    body: usize,
    memo: &mut Memo,
) -> Result<(), Error> {
    let retained = &index.bodies[body];
    let removal = &removals.entries[retained.removal];
    let file = removal.owner.file;
    charge(&removals.work, file, 1, 0)?;
    if memo.bodies.contains(&body) {
        return Ok(());
    }
    if removals.owners.get(&removal.owner) != Some(&retained.removal) {
        return Err(Error::at(
            file,
            0,
            "removed Test owner publication index changed",
        ));
    }
    charge(&removals.work, file, 1 + removal.duplicates.len(), 0)?;
    removal.validate_owner(report, &removals.work)?;
    removal.validate_role(report)?;
    retained
        .identity
        .validate(report, removal, &removals.work)?;
    let unit = report.package.unit(file);
    if !Arc::ptr_eq(&retained.image, unit.source.original_image())
        || unit
            .configured
            .ast
            .nodes
            .get(removal.declaration.node.get() as usize)
            != Some(&retained.declaration)
        || retirement_body(unit, removal)? != Some(retained.root)
        || unit.configured.ast.nodes.get(retained.root.get() as usize) != Some(&retained.snapshot)
        || unit.configured.ast.edges.len() < retained.edge_extent
    {
        return Err(Error::at(file, 0, "removed Test body identity changed"));
    }
    charge(&removals.work, file, 1, 4 * size_of::<usize>())?;
    memo.bodies.insert(body);
    Ok(())
}
pub(super) fn retirement_body(unit: &Unit, removal: &Removal) -> Result<Option<NodeId>, Error> {
    let crate::syntax::ast::Kind::Item(item) =
        unit.configured.ast.node(removal.declaration.node).kind
    else {
        return Err(Error::at(
            removal.owner.file,
            0,
            "Test body lost its function declaration",
        ));
    };
    match unit
        .configured
        .ast
        .items
        .get(item as usize)
        .map(|item| item.kind)
    {
        Some(crate::syntax::item::ItemKind::Function { body, .. }) => Ok(body),
        _ => Err(Error::at(
            removal.owner.file,
            0,
            "Test body declaration changed kind",
        )),
    }
}
fn validate_path(
    report: &Report,
    removals: &Removals,
    index: &Index,
    mut path: usize,
    file: FileId,
    memo: &mut Memo,
) -> Result<(), Error> {
    // Every selected shared parent is checked once; no wide parent is rescanned per effect.
    let mut pending = Vec::new();
    loop {
        charge(&removals.work, file, 1, 0)?;
        if memo.paths.contains(&path) {
            break;
        }
        let saved = &index.paths[path];
        charge(&removals.work, file, 1, 4 * size_of::<usize>())?;
        pending.push(path);
        if memo.nodes.contains(&path) {
            let Some(parent) = saved.parent else {
                break;
            };
            path = parent;
            continue;
        }
        let unit = report.package.unit(file);
        if unit.configured.ast.nodes.get(saved.node.get() as usize) != Some(&saved.snapshot)
            || unit.configured.active.get(saved.node.get() as usize) != Some(&false)
            || !super::parent_edges::matches(unit, saved.node, saved.auxiliary.as_deref())
        {
            return Err(Error::at(file, 0, "removed Test path node changed"));
        }
        let mut ordinal = 0;
        let mut selected = saved.children.iter().peekable();
        unit.configured
            .ast
            .try_for_each_child(saved.node, &mut |child| {
                charge(&removals.work, file, 1, 0)?;
                if selected.peek().is_some_and(|(at, _)| *at == ordinal) {
                    let (_, next) = selected
                        .next()
                        .ok_or_else(|| Error::at(file, 0, "missing Test edge"))?;
                    if index.paths[*next].node != child {
                        return Err(Error::at(file, 0, "removed Test typed child edge changed"));
                    }
                }
                ordinal += 1;
                Ok(())
            })?;
        if selected.next().is_some() {
            return Err(Error::at(file, 0, "removed Test child ordinal disappeared"));
        }
        charge(&removals.work, file, 1, 4 * size_of::<usize>())?;
        memo.nodes.insert(path);
        let Some(parent) = saved.parent else {
            break;
        };
        path = parent;
    }
    charge(
        &removals.work,
        file,
        pending.len(),
        4 * pending.len() * size_of::<usize>(),
    )?;
    memo.paths.extend(pending);
    Ok(())
}
