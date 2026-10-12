//! Sparse field paths; array indexing keeps its containing array indivisible.
use crate::{
    body::ir::{Body, LocalId, Place, ProjectionKind, Selection, ValueId},
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathId(pub u32);
#[derive(Clone, Copy, Debug)]
pub enum Root {
    Local(LocalId),
    Temporary(ValueId),
}
pub struct Path {
    pub root: Root,
    pub parent: Option<PathId>,
    pub field: Option<Selection>,
    pub ty: TypeId,
    /// DFS interval includes this path's shallow structure and all tracked fields.
    /// The shallow bit also represents unmentioned fields: field writes require
    /// an initialized parent, so they cannot manufacture an uninitialized object.
    pub start: u32,
    pub end: u32,
    child: Option<PathId>,
    sibling: Option<PathId>,
}
#[derive(Clone, Copy)]
pub(super) struct PlaceInfo {
    pub path: Option<PathId>,
    pub indexed: bool,
}
pub struct Paths {
    pub entries: Vec<Path>,
    pub(super) projections: Vec<PlaceInfo>,
    pub(super) temporaries: HashMap<ValueId, PathId>,
}
impl Paths {
    pub(super) fn build(body: &Body) -> Result<Self, &'static str> {
        let mut paths = Self {
            entries: Vec::new(),
            projections: Vec::new(),
            temporaries: HashMap::new(),
        };
        for (index, local) in body.locals.iter().enumerate() {
            let id = LocalId(u32::try_from(index).map_err(|_| "too many move locals")?);
            paths.insert(Root::Local(id), None, None, local.ty)?;
        }
        let mut fields = HashMap::new();
        for projection in &body.projections {
            if let Place::Temporary(value) = projection.base
                && !paths.temporaries.contains_key(&value)
            {
                let ty = body
                    .values
                    .get(value.0 as usize)
                    .ok_or("invalid temporary place")?
                    .ty;
                let id = paths.insert(Root::Temporary(value), None, None, ty)?;
                paths.temporaries.insert(value, id);
            }
            let mut info = paths.place(projection.base)?;
            match projection.kind {
                ProjectionKind::Field(field) if !info.indexed => {
                    if let Some(parent) = info.path {
                        let id = if let Some(id) = fields.get(&(parent, field)) {
                            *id
                        } else {
                            let root = paths.entries[parent.0 as usize].root;
                            let id =
                                paths.insert(root, Some(parent), Some(field), projection.ty)?;
                            fields.insert((parent, field), id);
                            id
                        };
                        info.path = Some(id);
                    }
                }
                ProjectionKind::Index { .. } => info.indexed = true,
                ProjectionKind::Field(_) => {}
            }
            paths.projections.push(info);
        }
        paths.number()?;
        Ok(paths)
    }
    fn insert(
        &mut self,
        root: Root,
        parent: Option<PathId>,
        field: Option<Selection>,
        ty: TypeId,
    ) -> Result<PathId, &'static str> {
        let id = PathId(u32::try_from(self.entries.len()).map_err(|_| "too many move paths")?);
        let sibling = parent.and_then(|p| self.entries[p.0 as usize].child);
        self.entries.push(Path {
            root,
            parent,
            field,
            ty,
            start: 0,
            end: 0,
            child: None,
            sibling,
        });
        if let Some(parent) = parent {
            self.entries[parent.0 as usize].child = Some(id);
        }
        Ok(id)
    }
    fn number(&mut self) -> Result<(), &'static str> {
        let mut work = Vec::new();
        let mut next = 0_u32;
        for index in 0..self.entries.len() {
            if self.entries[index].parent.is_some() {
                continue;
            }
            let path = &mut self.entries[index];
            if path.child.is_none() {
                path.start = next;
                path.end = next.checked_add(1).ok_or("too many move state bits")?;
                next = path.end;
                continue;
            }
            work.push((index, false));
            while let Some((index, finish)) = work.pop() {
                if finish {
                    self.entries[index].end = next;
                    continue;
                }
                self.entries[index].start = next;
                next = next.checked_add(1).ok_or("too many move state bits")?;
                work.push((index, true));
                let mut child = self.entries[index].child;
                while let Some(id) = child {
                    work.push((id.0 as usize, false));
                    child = self.entries[id.0 as usize].sibling;
                }
            }
        }
        Ok(())
    }
    pub(super) fn place(&self, place: Place) -> Result<PlaceInfo, &'static str> {
        let path = match place {
            Place::Local(id) => Some(PathId(id.0)),
            Place::Temporary(id) => Some(
                *self
                    .temporaries
                    .get(&id)
                    .ok_or("missing temporary move path")?,
            ),
            Place::Projection(id) => {
                return self
                    .projections
                    .get(id.0 as usize)
                    .copied()
                    .ok_or("projection precedes its base place");
            }
            Place::Static(_) => None,
        };
        Ok(PlaceInfo {
            path,
            indexed: false,
        })
    }
}
