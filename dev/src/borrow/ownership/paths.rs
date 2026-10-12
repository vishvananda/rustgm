//! Sparse move paths; ordinary indexing keeps its containing array indivisible.
mod arrays;
mod views;
use crate::{
    body::ir::{
        Action, Body, LocalId, LocalKind, Place, ProjectionKind, Rvalue, Selection, ValueId,
    },
    types::{Kind, Store, TypeId},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathId(pub u32);
#[derive(Clone, Copy, Debug)]
pub enum Root {
    Local(LocalId),
    Temporary(ValueId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Component {
    Field(Selection),
    Index(u64),
    Subarray { start: u64, end: u64 },
}
pub struct Path {
    pub root: Root,
    pub parent: Option<PathId>,
    pub component: Option<Component>,
    pub ty: TypeId,
    /// DFS interval includes this path's shallow structure and all tracked fields.
    /// The shallow bit also represents unmentioned fields: field writes require
    /// an initialized parent, so they cannot manufacture an uninitialized object.
    pub start: u32,
    pub end: u32,
    child: Option<PathId>,
    sibling: Option<PathId>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PlaceInfo {
    pub path: Option<PathId>,
    pub indexed: bool,
    /// An empty subarray is not a read or move of its containing array.
    pub empty: bool,
}
pub struct Paths {
    pub entries: Vec<Path>,
    pub bits: u32,
    pub(super) projections: Vec<PlaceInfo>,
    pub(super) temporaries: HashMap<ValueId, PathId>,
    pub(super) views: HashMap<LocalId, Vec<PlaceInfo>>,
    pub(super) guard_places: Vec<Vec<PlaceInfo>>,
}
impl Paths {
    pub(super) fn build(body: &Body, types: &Store) -> Result<Self, &'static str> {
        let mut paths = Self {
            entries: Vec::new(),
            bits: 0,
            projections: Vec::new(),
            temporaries: HashMap::new(),
            views: HashMap::new(),
            guard_places: Vec::new(),
        };
        for (index, local) in body.locals.iter().enumerate() {
            let id = LocalId(u32::try_from(index).map_err(|_| "too many move locals")?);
            paths.insert(Root::Local(id), None, None, local.ty)?;
        }
        let mut fields = HashMap::new();
        let mut arrays = arrays::Arrays::default();
        for projection in &body.projections {
            if let Place::Temporary(value) = projection.base {
                paths.temporary(body, value)?;
            }
            let mut info = paths.place(projection.base)?;
            let component = match projection.kind {
                ProjectionKind::Field(field) => Some(Component::Field(field)),
                ProjectionKind::ConstantIndex(index) => Some(Component::Index(index)),
                ProjectionKind::Subarray { start, end } => {
                    info.empty = start == end;
                    Some(Component::Subarray { start, end })
                }
                ProjectionKind::Index { .. } => {
                    info.indexed = true;
                    None
                }
            };
            if !info.indexed
                && let (Some(parent), Some(component)) = (info.path, component)
            {
                info.path = Some(paths.component(
                    types,
                    &mut fields,
                    &mut arrays,
                    parent,
                    component,
                    projection.ty,
                )?);
            }
            paths.projections.push(info);
        }
        for value in &body.values {
            if let Rvalue::Read(Place::Temporary(id))
            | Rvalue::Validate(Place::Temporary(id))
            | Rvalue::IsVariant {
                place: Place::Temporary(id),
                ..
            } = value.kind
            {
                paths.temporary(body, id)?;
            }
        }
        if !body.guards.is_empty() {
            paths.guard_views(body, types, &mut fields, &mut arrays)?;
        }
        if arrays.is_empty() {
            paths.number()?;
        } else {
            arrays.number(&mut paths)?;
        }
        Ok(paths)
    }
    fn temporary(&mut self, body: &Body, value: ValueId) -> Result<(), &'static str> {
        if !self.temporaries.contains_key(&value) {
            let ty = body
                .values
                .get(value.0 as usize)
                .ok_or("invalid temporary place")?
                .ty;
            let id = self.insert(Root::Temporary(value), None, None, ty)?;
            self.temporaries.insert(value, id);
        }
        Ok(())
    }
    fn component(
        &mut self,
        types: &Store,
        fields: &mut HashMap<(PathId, Component), PathId>,
        arrays: &mut arrays::Arrays,
        parent: PathId,
        component: Component,
        ty: TypeId,
    ) -> Result<PathId, &'static str> {
        let (parent, component) = self.normalize(parent, component)?;
        let key = (parent, component);
        if let Some(id) = fields.get(&key) {
            return Ok(*id);
        }
        let root = self.entries[parent.0 as usize].root;
        let id = self.insert(root, Some(parent), Some(component), ty)?;
        if !matches!(component, Component::Field(_)) {
            let Some(Kind::Array { length, .. }) = types.kind(self.entries[parent.0 as usize].ty)
            else {
                return Err("pattern move path requires a fixed array");
            };
            arrays.add(parent, id, component, length)?;
        }
        fields.insert(key, id);
        Ok(id)
    }
    fn normalize(
        &self,
        parent: PathId,
        component: Component,
    ) -> Result<(PathId, Component), &'static str> {
        let path = &self.entries[parent.0 as usize];
        let Some(Component::Subarray { start, end }) = path.component else {
            return Ok((parent, component));
        };
        let component = match component {
            Component::Index(index) if index < end - start => Component::Index(start + index),
            Component::Subarray {
                start: from,
                end: to,
            } if from <= to && to <= end - start => Component::Subarray {
                start: start + from,
                end: start + to,
            },
            _ => return Err("invalid projection through pattern subarray"),
        };
        Ok((
            path.parent.ok_or("subarray has no containing move path")?,
            component,
        ))
    }
    fn insert(
        &mut self,
        root: Root,
        parent: Option<PathId>,
        component: Option<Component>,
        ty: TypeId,
    ) -> Result<PathId, &'static str> {
        let id = PathId(u32::try_from(self.entries.len()).map_err(|_| "too many move paths")?);
        // Subarrays alias ranges of the array's elements; they do not add an
        // independent subtree or shallow bit to the physical ownership state.
        let tree_parent = parent.filter(|_| !matches!(component, Some(Component::Subarray { .. })));
        let sibling = tree_parent.and_then(|p| self.entries[p.0 as usize].child);
        self.entries.push(Path {
            root,
            parent,
            component,
            ty,
            start: 0,
            end: 0,
            child: None,
            sibling,
        });
        if let Some(parent) = tree_parent {
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
        self.bits = next;
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
            empty: false,
        })
    }
}
