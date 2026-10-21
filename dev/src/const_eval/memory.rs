//! Mutable execution cells preserve symbolic addresses and freeze only value reads.
use super::{Constant, Failure, Frame, Machine};
use crate::{
    body::ir::{LocalId, Place, ProjectionKind, ValueId},
    sema::values::{LocationId, Origin, Reader, View},
    types::TypeId,
};
use std::collections::{HashMap, HashSet};

enum Cell {
    Frozen(Constant),
    Fields {
        values: Vec<usize>,
        variant: Option<crate::sema::members::VariantId>,
    },
    Empty,
}
#[derive(Default)]
pub(super) struct Memory {
    cells: Vec<Cell>,
    roots: HashMap<LocationId, usize>,
    expired: HashSet<LocationId>,
}
impl Memory {
    pub(super) fn retained_cells(&self) -> usize {
        self.cells.len()
    }
    pub(super) fn expire(&mut self, locations: &[LocationId]) {
        self.expired.extend(locations.iter().copied());
    }
}
impl Machine<'_, '_> {
    pub(super) fn allocate(
        &mut self,
        allocations: &mut Vec<LocationId>,
        value: Option<Constant>,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        self.charge(1)?;
        self.reserve_cells(1)?;
        let address = self.evaluator.allocation(value, ty)?;
        let id = address
            .location()
            .ok_or("allocation lacks symbolic identity")?;
        self.memory.roots.insert(id, self.memory.cells.len());
        self.memory
            .cells
            .push(value.map_or(Cell::Empty, Cell::Frozen));
        allocations.push(id);
        Ok(address)
    }
    pub(super) fn end_local(&mut self, frame: &mut Frame, id: LocalId) -> Result<(), Failure> {
        if matches!(
            self.bodies[frame.body].locals[id.0 as usize].kind,
            crate::body::ir::LocalKind::Guard(_)
        ) {
            frame.locals[id.0 as usize] = None;
            return Ok(());
        }
        if let Some(address) = frame.locals[id.0 as usize].take() {
            let location = address.location().ok_or("local storage has no address")?;
            self.memory.expired.insert(location);
        }
        Ok(())
    }
    fn path(&mut self, address: Constant) -> Result<(LocationId, Vec<u64>), Failure> {
        let mut location = address.location().ok_or(Failure::Fault(
            "constant pointer has no allocation provenance",
        ))?;
        let mut path = Vec::new();
        loop {
            self.charge(1)?;
            if self.memory.expired.contains(&location) {
                return Err(Failure::Fault("constant pointer refers to expired storage"));
            }
            let place = Reader::new(self.evaluator.facts()).location(location)?;
            if let Origin::Projection { base, index } = place.origin {
                path.push(index);
                location = base;
            } else {
                break;
            }
        }
        path.reverse();
        Ok((location, path))
    }
    fn thaw(&mut self, id: usize) -> Result<(), Failure> {
        let Cell::Frozen(value) = self.memory.cells[id] else {
            return Ok(());
        };
        let (length, variant) = match Reader::new(self.evaluator.facts()).view(value)? {
            View::Product { length, variant } => (length, variant),
            View::Repeat { length, .. } => (length, None),
            _ => return Err("constant field projection requires a product".into()),
        };
        let length = usize::try_from(length).map_err(|_| "constant aggregate exceeds host size")?;
        self.charge(length)?;
        self.reserve_cells(length)?;
        let mut fields = Vec::with_capacity(length);
        for index in 0..length {
            let field = self.evaluator.field(value, index as u64)?;
            fields.push(self.memory.cells.len());
            self.memory.cells.push(Cell::Frozen(field));
        }
        self.memory.cells[id] = Cell::Fields {
            values: fields,
            variant,
        };
        Ok(())
    }
    fn reserve_cells(&mut self, amount: usize) -> Result<(), Failure> {
        let retained = self
            .memory
            .cells
            .len()
            .checked_add(self.slots)
            .and_then(|n| n.checked_add(amount))
            .ok_or("constant retained storage size overflow")?;
        if retained > self.limits.slots {
            return Err(Failure::Fault("constant execution storage limit exceeded"));
        }
        self.work.peak_slots = self.work.peak_slots.max(retained);
        Ok(())
    }
    fn cell(&mut self, root: LocationId, path: &[u64]) -> Result<usize, Failure> {
        let mut cell = *self
            .memory
            .roots
            .get(&root)
            .ok_or("constant write requires local storage")?;
        for &index in path {
            self.thaw(cell)?;
            let Cell::Fields { values, .. } = &self.memory.cells[cell] else {
                return Err("constant projection reads uninitialized storage".into());
            };
            cell = usize::try_from(index)
                .ok()
                .and_then(|i| values.get(i))
                .copied()
                .ok_or(Failure::Fault("constant index is out of bounds"))?;
            self.charge(1)?;
        }
        Ok(cell)
    }
    fn freeze(&mut self, root: usize) -> Result<Constant, Failure> {
        let mut stack = vec![(root, false)];
        let mut completed = HashMap::new();
        while let Some((id, finish)) = stack.pop() {
            self.charge(1)?;
            match &self.memory.cells[id] {
                Cell::Frozen(value) => {
                    completed.insert(id, *value);
                }
                Cell::Empty => return Err(Failure::Fault("constant reads uninitialized storage")),
                Cell::Fields { values, .. } if !finish => {
                    stack.push((id, true));
                    stack.extend(values.iter().rev().map(|&child| (child, false)));
                }
                Cell::Fields { values, variant } => {
                    let fields = values
                        .iter()
                        .map(|id| {
                            completed
                                .get(id)
                                .copied()
                                .ok_or("missing constant snapshot field")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let value = self.evaluator.fields(&fields, *variant)?;
                    completed.insert(id, value);
                }
            }
        }
        completed
            .get(&root)
            .copied()
            .ok_or("missing constant snapshot".into())
    }
    pub(super) fn read(&mut self, address: Constant) -> Result<Constant, Failure> {
        let (root, path) = self.path(address)?;
        if self.memory.roots.contains_key(&root) {
            let cell = self.cell(root, &path)?;
            return self.freeze(cell);
        }
        let reader = Reader::new(self.evaluator.facts());
        let location = reader.location(root)?;
        let mut value = location
            .value
            .ok_or("constant allocation has no completed contents")?;
        for index in path {
            value = self.evaluator.field(value, index)?;
        }
        Ok(value)
    }
    pub(super) fn write(&mut self, address: Constant, value: Constant) -> Result<(), Failure> {
        let (root, path) = self.path(address)?;
        let cell = self.cell(root, &path)?;
        self.memory.cells[cell] = Cell::Frozen(value);
        Ok(())
    }
    fn temporary(&mut self, frame: &mut Frame, id: ValueId) -> Result<Constant, Failure> {
        if let Some(&address) = frame.temporaries.get(&id) {
            return Ok(address);
        }
        let body = &self.bodies[frame.body];
        let ty = body.values[id.0 as usize].ty;
        let address = if let Some(&value) = body.promotions.get(&id) {
            self.evaluator.borrow(value, ty)?
        } else {
            let value = Self::value(frame, id)?;
            self.allocate(&mut frame.allocations, Some(value), ty)?
        };
        frame.temporaries.insert(id, address);
        Ok(address)
    }
    pub(super) fn place(&mut self, frame: &mut Frame, root: Place) -> Result<Constant, Failure> {
        let body = self.bodies[frame.body].clone();
        let mut current = root;
        let mut pending = Vec::new();
        let mut address = loop {
            self.charge(1)?;
            match current {
                Place::Local(id) => {
                    break frame
                        .locals
                        .get(id.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("constant local is outside its storage lifetime")?;
                }
                Place::Static(id) => {
                    let ty = self
                        .evaluator
                        .facts()
                        .definitions
                        .get(id.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("missing static type")?;
                    break self.evaluator.static_address(id, ty)?;
                }
                Place::Temporary(id) => {
                    break self.temporary(frame, id)?;
                }
                Place::Projection(id) => {
                    if let Some(&address) = frame.projections.get(&id) {
                        break address;
                    }
                    let projection = body
                        .projections
                        .get(id.0 as usize)
                        .ok_or("missing constant projection")?;
                    pending.push((id, *projection));
                    current = projection.base;
                }
            }
        };
        while let Some((id, projection)) = pending.pop() {
            address = match projection.kind {
                ProjectionKind::Dereference { .. } => self.read(address)?,
                ProjectionKind::RawDereference { .. } => {
                    return Err(
                        "raw pointer dereference in constant execution is not implemented".into(),
                    );
                }
                ProjectionKind::Field(field) => {
                    let index = match field {
                        crate::sema::members::Selection::Tuple(index) => index,
                        crate::sema::members::Selection::Nominal(field) => self
                            .evaluator
                            .facts()
                            .members
                            .ordinal(field)
                            .ok_or("missing constant field ordinal")?,
                    };
                    self.evaluator
                        .projection(address, u64::from(index), projection.ty)?
                }
                ProjectionKind::UnionField(_) => {
                    return Err(
                        "union field reinterpretation in constant execution is not implemented"
                            .into(),
                    );
                }
                ProjectionKind::ConstantIndex(index) => {
                    self.evaluator.projection(address, index, projection.ty)?
                }
                ProjectionKind::Index { index, length } => {
                    let index = super::values::integer(Self::value(frame, index)?)?;
                    let index = u64::try_from(index)
                        .ok()
                        .filter(|i| *i < length)
                        .ok_or(Failure::Fault("constant index is out of bounds"))?;
                    self.evaluator.projection(address, index, projection.ty)?
                }
                ProjectionKind::IndexParameter { .. } => {
                    return Err("constant body retains an unresolved array length".into());
                }
                ProjectionKind::SliceIndex { .. }
                | ProjectionKind::FromEnd { .. }
                | ProjectionKind::Subslice { .. }
                | ProjectionKind::Subarray { .. } => {
                    return Err("constant slice projection is not implemented".into());
                }
            };
            frame.projections.insert(id, address);
        }
        Ok(address)
    }
    pub(super) fn validate_result(&mut self, root: Constant, ty: TypeId) -> Result<(), Failure> {
        let mut pending = vec![root];
        let mut seen = HashSet::new();
        while let Some(value) = pending.pop() {
            if !seen.insert(value) {
                continue;
            }
            self.charge(1)?;
            match Reader::new(self.evaluator.facts()).view(value)? {
                View::Reference(_) => {
                    self.path(value)?;
                }
                View::Product { length, .. } => {
                    self.charge(
                        usize::try_from(length).map_err(|_| "constant result exceeds host size")?,
                    )?;
                    for index in 0..length {
                        pending.push(self.evaluator.field(value, index)?);
                    }
                }
                View::Repeat { value, .. } => pending.push(value),
                _ => {}
            }
        }
        let (answer, work) = Reader::new(self.evaluator.facts()).shared_reference_work(ty, root)?;
        self.charge(work)?;
        match answer {
            crate::types::interior_mutability::Answer::Ordinary => {}
            crate::types::interior_mutability::Answer::Interior => {
                return Err(Failure::Fault(
                    "constant reference cannot escape with interior mutable allocation backing",
                ));
            }
            crate::types::interior_mutability::Answer::Unresolved => {
                return Err(
                    "constant reference storage requires completed type and value facts".into(),
                );
            }
        }
        Ok(())
    }
}
