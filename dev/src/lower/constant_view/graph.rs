//! Fresh display IDs preserve physical backing independently of value equality.
mod rows;
use super::{Inventory, Limits};
use crate::sema::model::Declarations;
use crate::{
    borrow::Checked,
    resolve::model::DefId,
    sema::{
        text,
        values::{Constant, LocationId, Origin, Reader},
    },
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Backing {
    Location(LocationId),
    Static(DefId),
    Literal(text::Id),
}
#[derive(Clone, Copy)]
enum Pending {
    Value(TypeId, Constant),
    Allocation(Backing, TypeId, Option<Constant>),
}
pub(super) struct Graph<'a> {
    pub(super) facts: &'a Declarations,
    pub(super) reader: Reader<'a>,
    inventory: Inventory,
    values: HashMap<(TypeId, Constant), usize>,
    allocations: HashMap<Backing, usize>,
    pending: Vec<(usize, Pending)>,
    work: usize,
    bytes: usize,
    limits: Limits,
}
pub(super) fn build(
    facts: &Declarations,
    checked: &Checked<'_>,
    limits: Limits,
) -> Result<Inventory, &'static str> {
    let mut graph = Graph {
        facts,
        reader: Reader::new(facts),
        inventory: Inventory {
            declarations: Vec::new(),
            roots: Vec::new(),
            values: Vec::new(),
            allocations: Vec::new(),
        },
        values: HashMap::new(),
        allocations: HashMap::new(),
        pending: Vec::new(),
        work: 0,
        bytes: 0,
        limits,
    };
    super::roots::collect(&mut graph, checked)?;
    let mut cursor = 0;
    while let Some(&(id, pending)) = graph.pending.get(cursor) {
        graph.charge(1)?;
        let row = match pending {
            Pending::Value(ty, value) => graph.value_row(id, ty, value)?,
            Pending::Allocation(backing, ty, value) => {
                graph.allocation_row(id, backing, ty, value)?
            }
        };
        graph.storage(row.len())?;
        match pending {
            Pending::Value(..) => graph.inventory.values[id] = row,
            Pending::Allocation(..) => graph.inventory.allocations[id] = row,
        }
        cursor += 1;
    }
    Ok(graph.inventory)
}
impl Graph<'_> {
    pub(super) fn declaration(&mut self, row: String) -> Result<(), &'static str> {
        self.charge(1)?;
        self.storage(64 + row.len())?;
        self.inventory.declarations.push(row);
        Ok(())
    }
    pub(super) fn root(&mut self, row: String) -> Result<(), &'static str> {
        self.charge(1)?;
        self.storage(64 + row.len())?;
        self.inventory.roots.push(row);
        Ok(())
    }

    pub(super) fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.work = self
            .work
            .checked_add(count)
            .filter(|n| *n <= self.limits.work)
            .ok_or("constant view work budget exceeded")?;
        Ok(())
    }
    pub(super) fn storage(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .filter(|n| *n <= self.limits.bytes)
            .ok_or("constant view byte budget exceeded")?;
        Ok(())
    }
    fn reserve(&mut self) -> Result<(), &'static str> {
        self.charge(1)?;
        if self.pending.len() >= self.limits.nodes {
            return Err("constant view node budget exceeded");
        }
        // Logical owned inventory: queued work, indices, row handles and map entries.
        self.storage(256)
    }
    pub(super) fn value(&mut self, ty: TypeId, value: Constant) -> Result<usize, &'static str> {
        self.charge(1)?;
        if let Some(&id) = self.values.get(&(ty, value)) {
            return Ok(id);
        }
        self.reserve()?;
        let id = self.inventory.values.len();
        self.values.insert((ty, value), id);
        self.inventory.values.push(String::new());
        self.pending.push((id, Pending::Value(ty, value)));
        Ok(id)
    }
    fn allocation(
        &mut self,
        backing: Backing,
        ty: TypeId,
        value: Option<Constant>,
    ) -> Result<usize, &'static str> {
        self.charge(1)?;
        if let Some(&id) = self.allocations.get(&backing) {
            return Ok(id);
        }
        self.reserve()?;
        let id = self.inventory.allocations.len();
        self.allocations.insert(backing, id);
        self.inventory.allocations.push(String::new());
        self.pending
            .push((id, Pending::Allocation(backing, ty, value)));
        Ok(id)
    }
    pub(super) fn static_(
        &mut self,
        id: DefId,
        ty: TypeId,
        value: Option<Constant>,
    ) -> Result<usize, &'static str> {
        self.allocation(Backing::Static(id), ty, value)
    }
    fn location(&mut self, id: LocationId) -> Result<usize, &'static str> {
        let location = self.reader.location(id)?;
        match location.origin {
            Origin::Static(id) => self.static_(id, location.ty, self.reader.definition(id)),
            Origin::Allocation | Origin::Projection { .. } => {
                self.allocation(Backing::Location(id), location.ty, location.value)
            }
        }
    }
}
