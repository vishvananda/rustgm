//! Mutable execution cells preserve symbolic addresses and freeze only value reads.
mod bytes;
mod active_tags;
mod image_paths;
mod loads;
mod ranges;
mod raw_slices;
mod reinterpretation;
mod sequences;
mod tails;
use super::{Constant, Failure, Frame, Machine};
use crate::{
    body::ir::{LocalId, Place, ProjectionKind, ValueId},
    sema::values::{LocationId, Origin, Reader, View},
    types::TypeId,
};
pub(super) use image_paths::ImagePath;
pub(in crate::const_eval) use ranges::Terminal;
pub(super) use ranges::{Range, StoragePath};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Liveness {
    Required,
    AddressOnly,
}
use crate::support::hash::{HashMap, HashSet};

pub(super) enum Read<T> {
    Pending,
    Value(T),
}
enum Cell {
    Bytes(bytes::Id),
    Frozen(Constant),
    Fields {
        values: Vec<usize>,
        variant: Option<crate::sema::members::VariantId>,
    },
    Empty,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RawAlignment {
    Required,
    Unaligned,
}
#[derive(Clone, Copy)]
pub(super) struct RawAccess {
    pub alignment: RawAlignment,
    pub location: Option<LocationId>,
    pub ty: TypeId,
}
#[derive(Clone, Copy)]
pub(super) struct UnionField {
    pub owner: TypeId,
    pub field: crate::sema::members::Selection,
}
#[derive(Clone, Copy)]
pub(super) struct Resolved {
    pub address: Constant,
    pub ty: TypeId,
    pub raw: Option<RawAccess>,
    pub union: Option<UnionField>,
}
#[derive(Default)]
pub(super) struct Memory {
    bytes: bytes::Storage,
    cells: Vec<Cell>,
    roots: HashMap<LocationId, usize>,
    expired: HashSet<LocationId>,
    // Completed ordinary root-type facts only; access permissions are never cached.
    pub(super) ordinary_static_types: HashSet<TypeId>,
}
impl Memory {
    pub(super) fn abandon_byte_reads(&mut self) {
        self.bytes.abandon_reads();
    }
    pub(super) fn retained_cells(&self) -> usize {
        self.cells
            .len()
            .saturating_add(self.bytes.retained_slots())
            .saturating_add(self.ordinary_static_types.len().saturating_mul(4))
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
            frame.raw_views.remove(&id);
            frame.union_views.remove(&id);
            frame.locals[id.0 as usize] = None;
            return Ok(());
        }
        if let Some(address) = frame.locals[id.0 as usize].take() {
            let location = address.location().ok_or("local storage has no address")?;
            self.memory.expired.insert(location);
        }
        Ok(())
    }
    pub(super) fn path(&mut self, address: Constant) -> Result<StoragePath, Failure> {
        self.path_checked::<false, false>(address, 0)
    }
    pub(super) fn path_for(
        &mut self,
        address: Constant,
        liveness: Liveness,
        retained: usize,
    ) -> Result<StoragePath, Failure> {
        match liveness {
            Liveness::Required => self.path_checked::<false, true>(address, retained),
            Liveness::AddressOnly => self.path_checked::<true, true>(address, retained),
        }
    }
    fn path_checked<const ADDRESS_ONLY: bool, const RETAINED: bool>(
        &mut self,
        address: Constant,
        retained: usize,
    ) -> Result<StoragePath, Failure> {
        let mut location = address.location().ok_or(Failure::Fault(
            "constant pointer has no allocation provenance",
        ))?;
        let mut path = Vec::new();
        let mut terminal = None;
        loop {
            self.charge(1)?;
            if RETAINED {
                self.reserve_cells(retained.saturating_add(path.len()).saturating_add(1))?;
            }
            if !ADDRESS_ONLY && self.memory.expired.contains(&location) {
                return Err(Failure::Fault("constant pointer refers to expired storage"));
            }
            let place = Reader::new(self.evaluator.facts()).location(location)?;
            match place.origin {
                Origin::Projection { base, index } => {
                    path.push(index);
                    location = base;
                }
                Origin::Subsequence { base, from, length } => {
                    if terminal.is_some() || !path.is_empty() {
                        return Err("constant range is not a terminal normalized projection".into());
                    }
                    terminal = Some(Terminal::Range(Range { from, length }));
                    location = base;
                }
                Origin::ByteOffset { base, offset } => {
                    if terminal.is_some() || !path.is_empty() {
                        return Err(
                            "constant byte address is not a terminal normalized projection".into(),
                        );
                    }
                    terminal = Some(Terminal::Bytes {
                        offset,
                        ty: place.ty,
                    });
                    location = base;
                }
                Origin::Static(_) | Origin::Allocation => break,
            }
        }
        path.reverse();
        Ok(StoragePath {
            root: location,
            fields: path,
            terminal,
        })
    }
    fn thaw(&mut self, id: usize) -> Result<(), Failure> {
        let Cell::Frozen(value) = self.memory.cells[id] else {
            return Ok(());
        };
        let (length, variant) = match Reader::new(self.evaluator.facts()).view(value)? {
            View::Product { length, variant } => (length, variant),
            View::Repeat { length, .. } | View::ByteRange { length, .. } => (length, None),
            View::Bytes(text) => (
                Reader::new(self.evaluator.facts()).bytes(text)?.len() as u64,
                None,
            ),
            _ => return Err("constant field projection requires a product".into()),
        };
        let length = usize::try_from(length).map_err(|_| "constant aggregate exceeds host size")?;
        self.charge(length)?;
        self.reserve_cells(length)?;
        let mut fields = Vec::with_capacity(length);
        for index in 0..length {
            let field = self.stored_field(value, index as u64)?;
            fields.push(self.memory.cells.len());
            self.memory.cells.push(Cell::Frozen(field));
        }
        self.memory.cells[id] = Cell::Fields {
            values: fields,
            variant,
        };
        Ok(())
    }
    pub(super) fn reserve_cells(&mut self, amount: usize) -> Result<(), Failure> {
        let retained = self
            .memory
            .retained_cells()
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
    fn freeze(
        &mut self,
        root: usize,
        purpose: super::static_reads::ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        let mut stack = vec![(root, false)];
        let mut completed = HashMap::default();
        while let Some((id, finish)) = stack.pop() {
            self.charge(1)?;
            match &self.memory.cells[id] {
                Cell::Frozen(value) => {
                    completed.insert(id, *value);
                }
                Cell::Empty => return Err(Failure::Fault("constant reads uninitialized storage")),
                Cell::Bytes(storage) => {
                    let storage = *storage;
                    let Read::Value(value) = self.freeze_byte_root(storage, purpose)? else {
                        return Ok(Read::Pending);
                    };
                    completed.insert(id, value);
                }
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
            .map(Read::Value)
            .ok_or("missing constant snapshot".into())
    }
    pub(super) fn read(&mut self, place: Resolved) -> Result<Option<Constant>, Failure> {
        self.read_for(place, super::static_reads::ReadPurpose::Value)
    }
    pub(super) fn read_for(
        &mut self,
        place: Resolved,
        purpose: super::static_reads::ReadPurpose,
    ) -> Result<Option<Constant>, Failure> {
        if let Some(selected) = self.byte_place(place)? {
            self.check_byte_access(place, selected, false)?;
            return self.read_byte_access(selected.root, selected.offset, selected.ty, purpose);
        }
        let selected = self.path(place.address)?;
        let range = selected.range()?;
        let StoragePath {
            root, fields: path, ..
        } = selected;
        if let Some(access) = place.raw {
            if range.is_some() {
                return Err("constant raw whole-range access is not implemented".into());
            }
            self.check_raw_read(
                Resolved {
                    ty: place.union.map_or(place.ty, |u| u.owner),
                    union: None,
                    ..place
                },
                access,
                root,
                &path,
            )?;
        }
        let target = self.storage_target(place)?;
        let path = &path[..target.path_len(path.len())?];
        if let Some(union) = target.field()
            && let Some(value) = self.read_byte_union(root, path, union, place.ty, purpose)?
        {
            return Ok(match value {
                Read::Value(value) => Some(value),
                Read::Pending => None,
            });
        }
        let Some(value) = self.read_storage(root, path, range, place.ty, purpose)? else {
            return Ok(None);
        };
        if let Some(union) = target.field() {
            if !Reader::new(self.evaluator.facts()).scalar_union(union.owner)? {
                return Ok(
                    match self.read_image_field(value, union, place.ty, purpose)? {
                        Read::Value(value) => Some(value),
                        Read::Pending => None,
                    },
                );
            }
            let (value, work) = self.evaluator.union_read(
                value,
                union.owner,
                union.field,
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            return Ok(Some(value));
        }
        Ok(Some(value))
    }
    pub(super) fn write_place(&mut self, place: Resolved, value: Constant) -> Result<(), Failure> {
        if let Some(selected) = self.byte_place(place)? {
            self.check_raw_write_authority(place.address, selected.root)?;
            self.check_byte_access(place, selected, false)?;
            return self.write_byte_access(selected.root, selected.offset, selected.ty, value);
        }
        if self.write_byte_place(place, value)? {
            return Ok(());
        }
        let target = self.storage_target(place)?;
        if let Some(union) = target.field() {
            let (root, path) = self.path(place.address)?.fields_only()?;
            let cell = self.cell(root, &path[..target.path_len(path.len())?])?;
            let Read::Value(image) = self.freeze(cell, super::static_reads::ReadPurpose::Value)?
            else {
                return Err(
                    "untyped union snapshot unexpectedly required typed preparation".into(),
                );
            };
            let (image, work) = self.evaluator.union_write(
                image,
                union.owner,
                union.field,
                value,
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            self.memory.cells[cell] = Cell::Frozen(image);
            Ok(())
        } else {
            self.write(place.address, value)
        }
    }
    pub(super) fn write(&mut self, address: Constant, value: Constant) -> Result<(), Failure> {
        let storage = self.path(address)?;
        if self.write_byte_path(&storage, value)? {
            return Ok(());
        }
        let range = storage.range()?;
        let StoragePath {
            root, fields: path, ..
        } = storage;
        if let Some(range) = range {
            return self.write_range(root, &path, range, value);
        }
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
    pub(super) fn place(
        &mut self,
        frame: &mut Frame,
        root: Place,
    ) -> Result<Option<Resolved>, Failure> {
        self.place_for::<false>(frame, root)
    }
    pub(super) fn address_place(
        &mut self,
        frame: &mut Frame,
        root: Place,
    ) -> Result<Option<Resolved>, Failure> {
        self.place_for::<true>(frame, root)
    }
    /// Ordinary ancestry is shared; enum paths retain their consumer identity.
    fn place_for<const ADDRESS: bool>(
        &mut self,
        frame: &mut Frame,
        root: Place,
    ) -> Result<Option<Resolved>, Failure> {
        let body = self.bodies[frame.body].clone();
        let mut current = root;
        let mut pending = Vec::new();
        let mode = super::place_cache::Mode::selected(ADDRESS);
        let mut common = true;
        let mut address = loop {
            self.charge(1)?;
            match current {
                Place::Local(id) => {
                    break Resolved {
                        address: frame
                            .locals
                            .get(id.0 as usize)
                            .copied()
                            .flatten()
                            .ok_or("constant local is outside its storage lifetime")?,
                        ty: body.locals[id.0 as usize].ty,
                        raw: frame.raw_views.get(&id).copied(),
                        union: frame.union_views.get(&id).copied(),
                    };
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
                    break Resolved {
                        address: self.evaluator.static_address(id, ty)?,
                        ty,
                        raw: None,
                        union: None,
                    };
                }
                Place::Temporary(id) => {
                    break Resolved {
                        address: self.temporary(frame, id)?,
                        ty: body.values[id.0 as usize].ty,
                        raw: None,
                        union: None,
                    };
                }
                Place::Projection(id) => {
                    if let Some((address, shared)) = frame.projected(id, mode) {
                        common = shared;
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
            let parent = address;
            let Some(result) = self.project::<ADDRESS>(frame, address, projection)? else {
                return Ok(None);
            };
            address = result;
            if common {
                common = self.common_projection(parent, projection.kind);
            }
            frame.remember_projection(id, mode, common, address);
        }
        Ok(Some(address))
    }
    fn project<const ADDRESS: bool>(
        &mut self,
        frame: &Frame,
        mut address: Resolved,
        projection: crate::body::ir::Projection,
    ) -> Result<Option<Resolved>, Failure> {
        if let Some(union) = address.union {
            let (value, work) = self.evaluator.union_address(
                address.address,
                union.owner,
                union.field,
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            address.address = value;
            address.union = None;
        }
        let result = match projection.kind {
            ProjectionKind::CaptureBoundary { .. } => {
                return Err("captured closure execution is not implemented in constants".into());
            }
            ProjectionKind::Dereference { .. } | ProjectionKind::RawDereference { .. } => {
                let Some(value) = self.read(address)? else {
                    return Ok(None);
                };
                let value = self.evaluator.literal_address(value)?;
                return Ok(Some(Resolved {
                    address: value,
                    ty: projection.ty,
                    union: None,
                    raw: matches!(projection.kind, ProjectionKind::RawDereference { .. })
                        .then_some(RawAccess {
                            alignment: RawAlignment::Required,
                            location: value.location(),
                            ty: projection.ty,
                        }),
                }));
            }
            ProjectionKind::Field(field) => {
                if ADDRESS {
                    self.field_projection_for::<true>(address, field, projection.ty)?
                } else {
                    self.field_projection(address, field, projection.ty)?
                }
            }
            ProjectionKind::UnionField(field) => {
                self.project_raw_type(address)?;
                return Ok(Some(Resolved {
                    address: address.address,
                    ty: projection.ty,
                    raw: address.raw,
                    union: Some(UnionField {
                        owner: address.ty,
                        field,
                    }),
                }));
            }
            ProjectionKind::ConstantIndex(index) => {
                return self
                    .indexed_element(address, index, projection.ty)
                    .map(Some);
            }
            ProjectionKind::Index { index, length } => {
                let index = super::values::integer(Self::value(frame, index)?)?;
                let index = u64::try_from(index)
                    .ok()
                    .filter(|i| *i < length)
                    .ok_or(Failure::Fault("constant index is out of bounds"))?;
                return self
                    .indexed_element(address, index, projection.ty)
                    .map(Some);
            }
            ProjectionKind::IndexParameter { .. } => {
                return Err("constant body retains an unresolved array length".into());
            }
            ProjectionKind::SliceIndex { .. } | ProjectionKind::FromEnd { .. } => {
                let index = self.sequence_index(frame, address, projection.kind)?;
                return self
                    .indexed_element(address, index, projection.ty)
                    .map(Some);
            }
            ProjectionKind::Subslice { .. } | ProjectionKind::Subarray { .. } => {
                self.project_raw_type(address)?;
                self.range_projection(address, projection)?
            }
        };
        Ok(Some(Resolved {
            address: result,
            ty: projection.ty,
            raw: address.raw,
            union: None,
        }))
    }
    pub(super) fn stored_field(
        &mut self,
        value: Constant,
        index: u64,
    ) -> Result<Constant, Failure> {
        let extra = Reader::new(self.evaluator.facts()).image_field_work(value)?;
        if extra != 0 {
            self.charge(extra)?;
        }
        self.evaluator.field(value, index)
    }
    pub(super) fn validate_result(&mut self, root: Constant, ty: TypeId) -> Result<(), Failure> {
        let mut pending = vec![root];
        let mut seen = HashSet::default();
        while let Some(value) = pending.pop() {
            if !seen.insert(value) {
                continue;
            }
            self.charge(1)?;
            match Reader::new(self.evaluator.facts()).view(value)? {
                View::Reference(_) => {
                    self.path(value)?;
                }
                View::Image { .. } => {
                    if let Some(image) = Reader::new(self.evaluator.facts()).array_image(value)? {
                        self.charge(image.bytes.len())?;
                        Reader::new(self.evaluator.facts()).require_publishable_array(value)?;
                    }

                    let image = Reader::new(self.evaluator.facts()).image(value)?;
                    let length = image.fragments.len();
                    self.charge(length)?;
                    self.reserve_cells(pending.len().saturating_add(length).saturating_mul(4))?;
                    let image = Reader::new(self.evaluator.facts()).image(value)?;
                    for fragment in image.complete_words()? {
                        if let crate::sema::values::WordTarget::Value(value) = fragment.target {
                            pending.push(value);
                        }
                    }
                }
                View::Product { length, .. } => {
                    self.charge(
                        usize::try_from(length).map_err(|_| "constant result exceeds host size")?,
                    )?;
                    for index in 0..length {
                        pending.push(self.stored_field(value, index)?);
                    }
                }
                View::Repeat { value, length } if length != 0 => pending.push(value),
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
