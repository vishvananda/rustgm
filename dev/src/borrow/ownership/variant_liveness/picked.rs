//! Owned semantic coordinates accompany one initialized cleanup replay until loans finish.
mod ctor;
mod rooted;
use super::{
    parts::{self, Form, Need, Parts, Range},
    tags::{Coordinate, Step, Tags},
};
use crate::{
    body::{
        defining_cleanups::Subject,
        ir::{Body, BodyKey, BodyOrigin, Place},
    },
    borrow::{
        destruction::schedule::{Cleanup, Schedule},
        ownership::{
            Paths, Root as OwnedRoot,
            initialization::{Candidate, Point},
        },
    },
    support::retained::Growth,
    types::{TypeId, destruction::Eligibility},
};
struct Root {
    place: Place,
    ty: TypeId,
}
struct Shape {
    ty: TypeId,
    form: Form,
    counts: Range,
}
struct Node {
    root: u32,
    parent: Option<Coordinate>,
    step: Step,
    shape: parts::Id,
}
#[derive(Clone, Copy)]
enum Origin {
    Coordinate(Coordinate),
    Direct(TypeId),
}
struct Leaf {
    origin: Origin,
    access: bool,
    purpose: Eligibility,
    history: bool,
}
struct Row {
    cleanup: Cleanup,
    leaves: Range,
    subjects: Range,
}
pub(super) struct Snapshot {
    candidate: Candidate,
    ty: TypeId,
    leaves: Range,
    tracked: bool,
}
pub(crate) struct Selected {
    pub(super) body: BodyKey,
    pub(super) source: BodyOrigin,
    roots: Vec<Root>,
    shapes: Vec<Shape>,
    counts: Vec<u32>,
    nodes: Vec<Node>,
    rows: Vec<Row>,
    leaves: Vec<Leaf>,
    subjects: Vec<Subject>,
    work: Vec<Coordinate>,
    resolved: bool,
}
impl Selected {
    pub(super) fn new(tags: &Tags, paths: &Paths, body: &Body) -> Result<Self, &'static str> {
        let mut this = Self {
            body: body.key(),
            source: body.source,
            roots: Vec::new(),
            shapes: Vec::new(),
            counts: Vec::new(),
            nodes: Vec::new(),
            rows: Vec::new(),
            leaves: Vec::new(),
            subjects: Vec::new(),
            work: Vec::new(),
            resolved: false,
        };
        tags.parts.pay(Growth {
            bytes: size_of::<Self>() + 2 * size_of::<usize>(),
            work: 1,
        })?;
        tags.parts.charge(
            tags.roots
                .len()
                .saturating_add(tags.parts.entries.len())
                .saturating_add(tags.parts.variants.len())
                .saturating_add(tags.nodes.len()),
        )?;
        reserve(&tags.parts, &mut this.roots, tags.roots.len())?;
        reserve(&tags.parts, &mut this.shapes, tags.parts.entries.len())?;
        reserve(&tags.parts, &mut this.counts, tags.parts.variants.len())?;
        reserve(&tags.parts, &mut this.nodes, tags.nodes.len())?;
        reserve(&tags.parts, &mut this.work, tags.nodes.len())?;
        for root in &tags.roots {
            let path = paths
                .entries
                .get(root.path.0 as usize)
                .ok_or("selected cleanup root path is missing")?;
            let place = match path.root {
                OwnedRoot::Local(id) => Place::Local(id),
                OwnedRoot::Temporary(id) => Place::Temporary(id),
            };
            let ty = body
                .place_type(place)
                .ok_or("selected cleanup root type is missing")?;
            if path.parent.is_some() || ty != path.ty {
                return Err("selected cleanup root changed its full actual type");
            }
            this.roots.push(Root { place, ty });
        }
        for shape in &tags.parts.entries {
            let start = parts::compact(this.counts.len())?;
            for variant in
                &tags.parts.variants[shape.variants.start as usize..shape.variants.end as usize]
            {
                this.counts.push(variant.fields.end - variant.fields.start);
            }
            this.shapes.push(Shape {
                ty: shape.ty,
                form: shape.form,
                counts: Range {
                    start,
                    end: parts::compact(this.counts.len())?,
                },
            });
        }
        this.nodes.extend(tags.nodes.iter().map(|n| Node {
            root: n.root,
            parent: n.parent,
            step: n.step,
            shape: n.part,
        }));
        Ok(this)
    }
    /// This is called by the internal replay before handing Point to its visitor.
    pub(super) fn snapshot(
        &mut self,
        tags: &Tags,
        paths: &Paths,
        body: &Body,
        candidate: Candidate,
        point: &mut Point<'_>,
        queries: Option<&super::query::Queries>,
    ) -> Result<Snapshot, &'static str> {
        if self.resolved {
            return Err("resolved cleanup selection cannot capture another point");
        }
        let ty = body
            .place_type(candidate.place)
            .ok_or("selected cleanup place type is missing")?;
        let start = parts::compact(self.leaves.len())?;
        if let Some(queries) = queries {
            let partial = candidate
                .path
                .map(|id| {
                    paths
                        .entries
                        .get(id.0 as usize)
                        .ok_or("qualification cleanup path is missing")
                        .and_then(|path| point.eligibility_partial(path))
                })
                .transpose()?
                .unwrap_or(false);
            let bad = if let Some(located) = tags.place(paths, candidate.place)? {
                let node = tags
                    .nodes
                    .get(located.coordinate.0 as usize)
                    .ok_or("qualification cleanup coordinate is missing")?;
                let root = tags
                    .roots
                    .get(node.root as usize)
                    .ok_or("qualification cleanup root is missing")?;
                point.possible(root.bad)?
            } else {
                true
            };
            if bad && queries.may(ty)? {
                self.leaf(
                    &tags.parts,
                    Origin::Direct(ty),
                    Eligibility::Body,
                    !partial,
                    true,
                )?;
            }
        }

        let located = tags.place(paths, candidate.place)?.filter(|p| !p.merged);
        if let Some(located) = located {
            self.work.clear();
            self.work.push(located.coordinate);
            self.capture(tags, point, located.coordinate, queries)?;
        }
        Ok(Snapshot {
            candidate,
            ty,
            tracked: located.is_some(),
            leaves: Range {
                start,
                end: parts::compact(self.leaves.len())?,
            },
        })
    }
    fn capture(
        &mut self,
        tags: &Tags,
        point: &mut Point<'_>,
        root: Coordinate,
        queries: Option<&super::query::Queries>,
    ) -> Result<(), &'static str> {
        while let Some(coordinate) = self.work.pop() {
            tags.parts.charge(1)?;
            let node = &tags.nodes[coordinate.0 as usize];
            let shape = &tags.parts.entries[node.part.0 as usize];
            let purpose = match shape.need {
                Need::None => None,
                Need::Whole => Some(if coordinate == root {
                    Eligibility::Body
                } else {
                    Eligibility::Whole
                }),
                Need::OwnDrop => Some(Eligibility::OwnDrop),
            };
            if let Some(purpose) = purpose
                && point.possible(node.own_need.start)?
            {
                let filtered = purpose != Eligibility::OwnDrop
                    && queries.map(|q| q.mask(coordinate)).transpose()? == Some(false);
                if !filtered {
                    self.leaf(
                        &tags.parts,
                        Origin::Coordinate(coordinate),
                        purpose,
                        true,
                        false,
                    )?;
                }
            }
            for child in &tags.children[node.children.start as usize..node.children.end as usize] {
                tags.parts.charge(1)?;
                let selected = &tags.nodes[child.coordinate.0 as usize];
                if let Step::Nominal { variant, .. } = selected.step
                    && matches!(shape.form, Form::Nominal { enumeration: true })
                    && !point.possible(node.own_tags.start + variant)?
                {
                    continue;
                }
                self.work.push(child.coordinate);
            }
        }
        Ok(())
    }
    fn leaf(
        &mut self,
        parts: &Parts,
        origin: Origin,
        purpose: Eligibility,
        access: bool,
        history: bool,
    ) -> Result<(), &'static str> {
        reserve(parts, &mut self.leaves, 1)?;
        self.leaves.push(Leaf {
            origin,
            access,
            purpose,
            history,
        });
        Ok(())
    }
    pub(super) fn complete(
        &mut self,
        parts: &Parts,
        snapshot: &Snapshot,
        cleanup: Cleanup,
    ) -> Result<(), &'static str> {
        // Prepay this append and the one same-plan validation at sealed finish.
        parts.charge(2)?;
        if self.leaves.len() != snapshot.leaves.end as usize
            || !same_candidate(snapshot.candidate, cleanup.candidate)
            || cleanup.live_type.is_some_and(|ty| ty != snapshot.ty)
        {
            return Err("selected cleanup row changed its actual replay candidate or full type");
        }
        if cleanup.live_type.is_some() && !snapshot.tracked {
            self.leaf(
                parts,
                Origin::Direct(snapshot.ty),
                Eligibility::Body,
                true,
                false,
            )?;
        }
        let leaves = Range {
            start: snapshot.leaves.start,
            end: parts::compact(self.leaves.len())?,
        };
        reserve(parts, &mut self.rows, 1)?;
        self.rows.push(Row {
            cleanup,
            leaves,
            subjects: Range::default(),
        });
        Ok(())
    }
    pub(crate) fn subjects(&self) -> Result<&[Subject], &'static str> {
        if !self.resolved {
            return Err("selected cleanup requires completed semantic coordinate resolution");
        }
        Ok(&self.subjects)
    }
    pub(crate) fn for_cleanup(&self, index: usize) -> Result<&[Subject], &'static str> {
        if !self.resolved {
            return Err("selected cleanup is not semantically resolved");
        }
        let row = self
            .rows
            .get(index)
            .ok_or("selected cleanup row is missing")?;
        self.subjects
            .get(row.subjects.start as usize..row.subjects.end as usize)
            .ok_or("selected cleanup subjects are missing")
    }
    pub(crate) fn validate(&self, body: &Body, schedule: Schedule<'_>) -> Result<(), &'static str> {
        if self.body != body.key() || self.source != body.source || !self.resolved {
            return Err("selected cleanup belongs to another body or unresolved stage");
        }
        let mut rows = self.rows.iter();
        for actual in schedule.cleanups() {
            let expected = rows
                .next()
                .ok_or("selected cleanup has incomplete plan coverage")?
                .cleanup;
            if !same_candidate(expected.candidate, actual.candidate)
                || expected.live_type != actual.live_type
                || expected.access != actual.access
            {
                return Err("selected cleanup belongs to another actual plan");
            }
        }
        if rows.next().is_some() {
            return Err("selected cleanup has extra plan coverage");
        }
        Ok(())
    }
}
fn same_candidate(a: Candidate, b: Candidate) -> bool {
    a.place == b.place && a.path == b.path && a.location == b.location && a.cause == b.cause
}
fn reserve<T>(parts: &Parts, values: &mut Vec<T>, additional: usize) -> Result<(), &'static str> {
    parts.pay(Growth::vector::<T>(
        values.len(),
        values.capacity(),
        additional,
    ))?;
    values
        .try_reserve(additional)
        .map_err(|_| "selected cleanup allocation failed")
}
