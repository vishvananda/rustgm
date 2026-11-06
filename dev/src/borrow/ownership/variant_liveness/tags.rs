//! Sparse tag occurrences preserve actual declaration paths, not source permissions.
use super::parts::{self, Form, Need, Parts, Range};
use crate::{
    borrow::ownership::{PathId, Paths},
    support::{hash::HashMap, retained::Growth},
    types::{ItemId, Store, aggregate::Definitions},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Coordinate(pub u32);
#[derive(Clone, Copy)]
pub(super) enum Step {
    Root,
    Tuple(u32),
    Nominal { variant: u32, field: u32 },
    Element,
}
#[derive(Clone, Copy)]
pub(super) enum Anchor {
    Path(PathId),
    Shallow(u32),
    Inherited,
}
pub(super) struct Node {
    pub part: parts::Id,
    pub root: u32,
    pub parent: Option<Coordinate>,
    pub step: Step,
    pub children: Range,
    pub tags: Range,
    pub own_tags: Range,
    pub own_need: Range,
    pub anchor: Anchor,
}
pub(super) struct Root {
    pub path: PathId,
    pub coordinate: Coordinate,
    pub tags: Range,
    pub exposed: u32,
    pub bad: u32,
    pub borrowed: u32,
}
pub(super) struct Child {
    pub ordinal: u32,
    pub coordinate: Coordinate,
}
pub(super) struct Tags {
    pub parts: Parts,
    pub nodes: Vec<Node>,
    pub children: Vec<Child>,
    pub roots: Vec<Root>,
    pub locations: HashMap<PathId, Located>,
    pub origins: HashMap<PathId, u32>,
    pub bits: u32,
}
#[derive(Clone, Copy)]
pub(super) struct Located {
    pub coordinate: Coordinate,
    pub merged: bool,
}
enum Work {
    Enter(Coordinate),
    Finish(Coordinate),
}
impl Tags {
    pub(super) fn new(
        types: &Store,
        definitions: &Definitions,
        paths: &Paths,
        limit: usize,
        custom: &dyn Fn(ItemId) -> bool,
    ) -> Result<Self, &'static str> {
        let mut this = Self {
            parts: Parts::new(limit),
            nodes: Vec::new(),
            children: Vec::new(),
            roots: Vec::new(),
            locations: HashMap::default(),
            origins: HashMap::default(),
            bits: paths.bits,
        };
        this.parts.pay(Growth {
            bytes: size_of::<Self>()
                + size_of::<super::index::Partition>()
                + size_of::<super::Refinements>()
                + 4 * size_of::<usize>(),
            work: 1,
        })?;
        let mut work = Vec::new();
        for (index, path) in paths.entries.iter().enumerate() {
            if path.parent.is_some() {
                continue;
            }
            this.parts.charge(1)?;
            let part = this.parts.demand(types, definitions, path.ty, custom)?;
            if !this.parts.entries[part.0 as usize].relevant {
                continue;
            }
            let path_id = PathId(parts::compact(index)?);
            this.parts.pay(Growth::vector::<Root>(
                this.roots.len(),
                this.roots.capacity(),
                1,
            ))?;
            this.parts.pay(Growth::map::<PathId, Located>(
                this.locations.len(),
                this.locations.capacity(),
                1,
            ))?;
            this.roots
                .try_reserve(1)
                .map_err(|_| "cleanup tag root allocation failed")?;
            this.locations
                .try_reserve(1)
                .map_err(|_| "cleanup tag root index allocation failed")?;
            let coordinate = this.node(
                part,
                None,
                Step::Root,
                Anchor::Path(path_id),
                parts::compact(this.roots.len())?,
            )?;
            let start = this.bits;
            this.push(&mut work, Work::Enter(coordinate))?;
            this.build(&mut work)?;
            let end = this.bits;
            let exposed = this.bit(1)?.start;
            let bad = this.bit(1)?.start;
            let borrowed = this.bit(1)?.start;
            this.parts.pay(Growth::map::<PathId, u32>(
                this.origins.len(),
                this.origins.capacity(),
                1,
            ))?;
            this.origins
                .try_reserve(1)
                .map_err(|_| "cleanup qualifier root origin allocation failed")?;
            this.origins
                .insert(path_id, parts::compact(this.roots.len())?);
            this.locations.insert(
                path_id,
                Located {
                    coordinate,
                    merged: false,
                },
            );
            this.roots.push(Root {
                path: path_id,
                coordinate,
                tags: Range { start, end },
                exposed,
                bad,
                borrowed,
            });
        }
        Ok(this)
    }
    fn node(
        &mut self,
        part: parts::Id,
        parent: Option<Coordinate>,
        step: Step,
        anchor: Anchor,
        root: u32,
    ) -> Result<Coordinate, &'static str> {
        self.parts.pay(Growth::vector::<Node>(
            self.nodes.len(),
            self.nodes.capacity(),
            1,
        ))?;
        self.nodes
            .try_reserve(1)
            .map_err(|_| "cleanup tag coordinate allocation failed")?;
        let id = Coordinate(parts::compact(self.nodes.len())?);
        self.nodes.push(Node {
            part,
            root,
            parent,
            step,
            children: Range::default(),
            tags: Range::default(),
            own_tags: Range::default(),
            own_need: Range::default(),
            anchor,
        });
        Ok(id)
    }
    fn push(&mut self, work: &mut Vec<Work>, item: Work) -> Result<(), &'static str> {
        self.parts
            .pay(Growth::vector::<Work>(work.len(), work.capacity(), 1))?;
        work.try_reserve(1)
            .map_err(|_| "cleanup tag occurrence work allocation failed")?;
        work.push(item);
        Ok(())
    }
    fn bit(&mut self, count: u32) -> Result<Range, &'static str> {
        let start = self.bits;
        self.bits = start
            .checked_add(count)
            .ok_or("cleanup tag bit identity limit exceeded")?;
        Ok(Range {
            start,
            end: self.bits,
        })
    }
    fn build(&mut self, work: &mut Vec<Work>) -> Result<(), &'static str> {
        while let Some(item) = work.pop() {
            self.parts.charge(1)?;
            match item {
                Work::Finish(id) => self.nodes[id.0 as usize].tags.end = self.bits,
                Work::Enter(id) => {
                    let part = &self.parts.entries[self.nodes[id.0 as usize].part.0 as usize];
                    let (form, variants, need) = (part.form, part.variants, part.need);
                    let own = self.bit(if matches!(form, Form::Nominal { enumeration: true }) {
                        variants.end - variants.start
                    } else {
                        0
                    })?;
                    let required = self.bit(u32::from(need != Need::None))?;
                    self.nodes[id.0 as usize].own_tags = own;
                    self.nodes[id.0 as usize].own_need = required;
                    self.nodes[id.0 as usize].tags.start = own.start;
                    self.push(work, Work::Finish(id))?;
                    self.descend(id, form, variants, work)?;
                }
            }
        }
        Ok(())
    }
    fn descend(
        &mut self,
        id: Coordinate,
        form: Form,
        variants: Range,
        work: &mut Vec<Work>,
    ) -> Result<(), &'static str> {
        let start = parts::compact(self.children.len())?;
        let mut ordinal = 0_u32;
        for variant in variants.start..variants.end {
            let fields = self.parts.variants[variant as usize].fields;
            self.parts.charge((fields.end - fields.start) as usize)?;
            for field in fields.start..fields.end {
                let part = self.parts.fields[field as usize]
                    .part
                    .ok_or("cleanup tag occurrence has an incomplete descriptor")?;
                let actual = ordinal;
                ordinal = ordinal
                    .checked_add(1)
                    .ok_or("cleanup qualifier child ordinal overflow")?;
                if !self.parts.entries[part.0 as usize].relevant {
                    continue;
                }
                let step = match form {
                    Form::Tuple => Step::Tuple(field - fields.start),
                    Form::Nominal { .. } => Step::Nominal {
                        variant: variant - variants.start,
                        field: field - fields.start,
                    },
                    Form::Array => Step::Element,
                    Form::Leaf => return Err("cleanup tag leaf has fields"),
                };
                let child = self.node(
                    part,
                    Some(id),
                    step,
                    Anchor::Inherited,
                    self.nodes[id.0 as usize].root,
                )?;
                self.parts.pay(Growth::vector::<Child>(
                    self.children.len(),
                    self.children.capacity(),
                    1,
                ))?;
                self.children
                    .try_reserve(1)
                    .map_err(|_| "cleanup tag child allocation failed")?;
                self.children.push(Child {
                    ordinal: actual,
                    coordinate: child,
                });
            }
        }
        let end = parts::compact(self.children.len())?;
        self.nodes[id.0 as usize].children = Range { start, end };
        for child in start..end {
            self.push(work, Work::Enter(self.children[child as usize].coordinate))?;
        }
        Ok(())
    }
}
