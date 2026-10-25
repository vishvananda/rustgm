//! The real retirement walk promotes each demanded ancestor prefix at most once.
use super::{
    Error, NodeId, Removal, Removals,
    demand::{self, Body, Index, Path},
    saved_uses,
};
use crate::resolve::unit::{FileId, Unit, retirement::Observer};
use std::sync::Arc;

struct Frame {
    node: NodeId,
    edge: Option<(NodeId, usize)>,
    promoted: Option<usize>,
    within_body: bool,
}
pub(super) struct Prepared(Box<Index>);
pub(super) struct Builder<'a> {
    removals: &'a mut Removals,
    removal: &'a Removal,
    before: &'a [Unit],
    after: &'a [Unit],
    body: Option<NodeId>,
    definition: &'a super::super::Definition,
    scope: &'a crate::resolve::model::Scope,
    frames: Vec<Frame>,
    local: Index,
}
impl<'a> Builder<'a> {
    pub(super) fn new(
        removals: &'a mut Removals,
        removal: &'a Removal,
        unit: &Unit,
        before: &'a [Unit],
        after: &'a [Unit],
        definition: &'a super::super::Definition,
        scope: &'a crate::resolve::model::Scope,
    ) -> Result<Self, Error> {
        Ok(Self {
            removals,
            removal,
            before,
            after,
            definition,
            scope,
            body: demand::retirement_body(unit, removal)?,
            frames: Vec::new(),
            local: Index::default(),
        })
    }
    fn promote(&mut self, unit: &Unit) -> Result<usize, Error> {
        let mut first = self.frames.len();
        while first > 0 && self.frames[first - 1].promoted.is_none() {
            self.charge(1, 0)?;
            first -= 1;
        }
        for frame in first..self.frames.len() {
            self.charge(1, 4 * (size_of::<Path>() + size_of::<(usize, usize)>()))?;
            let index = self.local.paths.len();
            let active = &self.frames[frame];
            let parent = frame.checked_sub(1).and_then(|p| self.frames[p].promoted);
            let auxiliary = super::parent_edges::capture(
                unit,
                active.node,
                &self.removals.work,
                self.removal.owner.file,
            )?;
            self.local.paths.push(Path {
                node: active.node,
                snapshot: unit.configured.ast.node(active.node),
                parent,
                children: Vec::new(),
                auxiliary,
            });
            if let Some(parent) = parent {
                let ordinal = active
                    .edge
                    .ok_or_else(|| {
                        Error::at(
                            self.removal.owner.file,
                            0,
                            "retirement path has no typed incoming edge",
                        )
                    })?
                    .1;
                self.local.paths[parent].children.push((ordinal, index));
            }
            self.frames[frame].promoted = Some(index);
        }
        self.frames
            .last()
            .and_then(|frame| frame.promoted)
            .ok_or_else(|| {
                Error::at(
                    self.removal.owner.file,
                    0,
                    "retirement lost its active prefix",
                )
            })
    }
    fn record(&mut self, unit: &Unit, effect: usize) -> Result<(), Error> {
        let file = self.removal.owner.file;
        if self.local.lookup.contains_key(&(file, effect)) {
            return Err(Error::at(
                file,
                0,
                "Test retirement reached one configured effect through repeated paths",
            ));
        }
        let path = self.promote(unit)?;
        self.charge(
            1,
            4 * size_of::<saved_uses::Witness>()
                + crate::support::retained::new_map::<(FileId, usize), usize>(1),
        )?;
        let witness =
            saved_uses::Witness::capture(unit, file, effect, path, &self.removals.work, |id| {
                if id.0 < file.0 {
                    self.before.get(id.0 as usize)
                } else if id.0 > file.0 {
                    self.after.get((id.0 - file.0 - 1) as usize)
                } else {
                    None
                }
            })?;
        self.local
            .lookup
            .insert((file, effect), self.local.effects.len());
        self.local.effects.push(witness);
        if self.local.bodies.is_empty() {
            self.charge(1, size_of::<Body>())?;
            self.local
                .bodies
                .try_reserve_exact(1)
                .map_err(|_| Error::at(file, 0, "Test body receipt allocation failed"))?;
            let body = self
                .body
                .ok_or_else(|| Error::at(file, 0, "retirement lost actual Function.body"))?;
            let identity = super::identities::Identity::capture(
                unit,
                self.removal,
                self.definition,
                self.scope,
                &self.removals.work,
                |id| {
                    if id.0 < file.0 {
                        self.before.get(id.0 as usize)
                    } else if id.0 > file.0 {
                        self.after.get((id.0 - file.0 - 1) as usize)
                    } else {
                        None
                    }
                },
            )?;
            self.local.bodies.push(Body {
                removal: self.removals.entries.len(),
                image: Arc::clone(unit.source.original_image()),
                declaration: unit.configured.ast.node(self.removal.declaration.node),
                body,
                snapshot: unit.configured.ast.node(body),
                edge_extent: unit.configured.ast.edges.len(),
                identity,
            });
        }
        Ok(())
    }
}
impl Observer for Builder<'_> {
    type Output = Option<Prepared>;
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        demand::charge(&self.removals.work, self.removal.owner.file, steps, bytes)
    }
    fn enter(
        &mut self,
        unit: &Unit,
        node: NodeId,
        edge: Option<(NodeId, usize)>,
    ) -> Result<(), Error> {
        self.charge(1, 4 * size_of::<Frame>())?;
        let parent = self.frames.last();
        if edge.is_some_and(|(expected, _)| parent.is_none_or(|parent| parent.node != expected)) {
            return Err(Error::at(
                self.removal.owner.file,
                0,
                "Test retirement active parent changed",
            ));
        }
        let strict = parent.is_some_and(|parent| parent.within_body);
        let within_body = strict
            || edge.is_some_and(|(parent, _)| {
                parent == self.removal.declaration.node && Some(node) == self.body
            });
        self.frames.push(Frame {
            node,
            edge,
            promoted: None,
            within_body,
        });
        if !strict {
            return Ok(());
        }
        let Some(owner) = unit.owner_indices[node.get() as usize]
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
        else {
            return Ok(());
        };
        for effect in owner.attributes.clone() {
            let search = unit.configured.cfg_uses.as_ref().map_or(0, |uses| {
                2 * uses.len().checked_ilog2().unwrap_or(0) as usize
            });
            self.charge(1 + search, 0)?;
            if unit
                .configured
                .cfg_uses
                .as_ref()
                .is_some_and(|uses| !uses.effect_range(effect).is_empty())
            {
                self.record(unit, effect)?;
            }
        }
        Ok(())
    }
    fn leave(&mut self) {
        self.frames.pop();
    }
    fn finish(mut self) -> Result<Self::Output, Error> {
        if self.local.effects.is_empty() {
            return Ok(None);
        }
        self.charge(1, size_of::<Index>())?;
        if let Some(existing) = self.removals.demand.as_mut() {
            reserve(
                existing,
                &mut self.local,
                &self.removals.work,
                self.removal.owner.file,
            )?;
        }
        Ok(Some(Prepared(Box::new(self.local))))
    }
}
fn reserve(
    existing: &mut Index,
    local: &mut Index,
    work: &std::cell::Cell<demand::Work>,
    file: FileId,
) -> Result<(), Error> {
    let fail = || Error::at(file, 0, "Test receipt publication allocation failed");
    demand::charge(
        work,
        file,
        local.paths.len() + local.effects.len(),
        4 * (local.paths.len() * size_of::<Path>()
            + local.effects.len() * size_of::<saved_uses::Witness>()
            + size_of::<Body>()),
    )?;
    existing
        .paths
        .try_reserve(local.paths.len())
        .map_err(|_| fail())?;
    existing
        .effects
        .try_reserve(local.effects.len())
        .map_err(|_| fail())?;
    existing.bodies.try_reserve(1).map_err(|_| fail())?;
    existing
        .lookup
        .try_reserve(local.lookup.len())
        .map_err(|_| fail())?;
    for path in &mut local.paths {
        path.parent = path.parent.map(|index| index + existing.paths.len());
        for (_, child) in &mut path.children {
            *child += existing.paths.len();
        }
    }
    for effect in &mut local.effects {
        effect.path += existing.paths.len();
        effect.body = existing.bodies.len();
    }
    for (key, effect) in &mut local.lookup {
        if existing.lookup.contains_key(key) {
            return Err(Error::at(
                file,
                0,
                "Test receipt duplicated an actual cfg effect",
            ));
        }
        *effect += existing.effects.len();
    }
    Ok(())
}
impl Removals {
    pub(super) fn install_demand(&mut self, prepared: Option<Prepared>) {
        if let Some(Prepared(mut incoming)) = prepared {
            if let Some(existing) = self.demand.as_mut() {
                existing.paths.append(&mut incoming.paths);
                existing.bodies.append(&mut incoming.bodies);
                existing.effects.append(&mut incoming.effects);
                existing.lookup.extend(incoming.lookup);
            } else {
                self.demand = Some(incoming);
            }
        }
    }
}
