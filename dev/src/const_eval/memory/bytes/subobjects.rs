//! Typed reconstruction selects declared subobjects without inheriting access permission.
mod index;
use super::{Constant, Failure, Machine, Reader, TypeId};
use crate::sema::values::{StoredShape, WordAddress};
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    root: TypeId,
    offset: u64,
    target: TypeId,
    erased: bool,
}
#[derive(Default)]
pub(super) struct Paths {
    fields: HashMap<TypeId, index::Index>,
    checked: HashMap<Key, (Box<[u64]>, TypeId)>,
    views: HashMap<(WordAddress, TypeId, bool), (Constant, TypeId)>,
    retained: usize,
}
impl Paths {
    pub(super) fn retained_slots(&self) -> usize {
        self.retained
            .saturating_add(4 * size_of::<Self>().div_ceil(size_of::<Constant>()))
    }
}
struct Search {
    ty: TypeId,
    offset: u64,
    parent: Option<usize>,
    field: u64,
}
impl Machine<'_, '_> {
    pub(super) fn reconstruct_word(
        &mut self,
        address: WordAddress,
        target: TypeId,
    ) -> Result<Constant, Failure> {
        self.reconstruct_subobject(address, target, false)
            .map(|(value, _)| value)
    }
    pub(super) fn reconstruct_object_word(
        &mut self,
        address: WordAddress,
        concrete: TypeId,
    ) -> Result<(Constant, TypeId), Failure> {
        self.reconstruct_subobject(address, concrete, true)
    }
    fn reconstruct_subobject(
        &mut self,
        address: WordAddress,
        target: TypeId,
        erased: bool,
    ) -> Result<(Constant, TypeId), Failure> {
        self.charge(1)?;
        let root = Reader::new(self.evaluator.facts())
            .location(address.root())?
            .ty;
        self.footprint(root)?;
        // Only the original checked concrete type supplies layout; erased keys never do.
        let zero = self.footprint(target)?.size == 0;
        let identity = if erased {
            self.erased_word_type(target)?
        } else {
            target
        };
        let view = (address, identity, erased);
        if let Some(value) = self
            .memory
            .bytes
            .paths
            .as_ref()
            .and_then(|paths| paths.views.get(&view))
        {
            return Ok(*value);
        }
        let key = Key {
            root,
            offset: address.offset(),
            target: identity,
            erased,
        };
        if !self
            .memory
            .bytes
            .paths
            .as_ref()
            .is_some_and(|paths| paths.checked.contains_key(&key))
        {
            let (path, actual) = self.find_subobject(key, zero)?;
            self.reserve_cells(path.len().saturating_add(16))?;
            let paths = self
                .memory
                .bytes
                .paths
                .get_or_insert_with(|| Box::new(Paths::default()));
            paths
                .checked
                .try_reserve(1)
                .map_err(|_| "symbolic typed path cache allocation failed")?;
            paths.retained = paths
                .retained
                .checked_add(
                    path.capacity()
                        .saturating_mul(size_of::<u64>())
                        .div_ceil(size_of::<Constant>())
                        .saturating_add(16),
                )
                .ok_or("symbolic typed path retention overflow")?;
            paths.checked.insert(key, (path.into_boxed_slice(), actual));
        }
        let length = self
            .memory
            .bytes
            .paths
            .as_ref()
            .and_then(|paths| paths.checked.get(&key))
            .ok_or("missing checked symbolic subobject path")?
            .0
            .len();
        self.charge(length.saturating_mul(2).saturating_add(1))?;
        self.reserve_cells(length.saturating_add(32))?;
        let (path, actual) = self
            .memory
            .bytes
            .paths
            .as_ref()
            .and_then(|paths| paths.checked.get(&key))
            .ok_or("missing checked symbolic subobject path")?;
        let actual = *actual;
        let value = self.evaluator.word_projection(address, path, actual)?;
        let paths = self
            .memory
            .bytes
            .paths
            .as_mut()
            .ok_or("missing symbolic pointer view cache")?;
        paths
            .views
            .try_reserve(1)
            .map_err(|_| "symbolic pointer view cache allocation failed")?;
        paths.retained = paths
            .retained
            .checked_add(length.saturating_add(32))
            .ok_or("symbolic pointer view retention overflow")?;
        paths.views.insert(view, (value, actual));
        Ok((value, actual))
    }
    fn find_subobject(&mut self, key: Key, zero: bool) -> Result<(Vec<u64>, TypeId), Failure> {
        self.reserve_cells(16)?;
        let mut nodes = vec![Search {
            ty: key.root,
            offset: key.offset,
            parent: None,
            field: 0,
        }];
        let mut pending = vec![0usize];
        let mut selected = None;
        while let Some(next) = pending.pop() {
            self.charge(1)?;
            let node = &nodes[next];
            let (ty, offset) = (node.ty, node.offset);
            let matches = offset == 0
                && if key.erased {
                    self.erased_word_type(ty)? == key.target
                } else {
                    Reader::new(self.evaluator.facts()).same_storage_type(ty, key.target)
                };
            if matches {
                if selected.replace(next).is_some() {
                    return Err("symbolic pointer selects ambiguous coincident subobjects".into());
                }
                continue;
            }
            let children = self.subobject_children(ty, offset, zero)?;
            self.reserve_cells(
                nodes
                    .len()
                    .saturating_add(pending.len())
                    .saturating_add(children.len().saturating_mul(2))
                    .saturating_mul(4),
            )?;
            nodes
                .try_reserve(children.len())
                .map_err(|_| "symbolic subobject ancestry allocation failed")?;
            pending
                .try_reserve(children.len())
                .map_err(|_| "symbolic subobject search allocation failed")?;
            for field in children {
                let child = Reader::new(self.evaluator.facts()).projection_layout(ty, field)?;
                let relative = offset
                    .checked_sub(child.offset)
                    .ok_or("symbolic subobject offset precedes its field")?;
                pending.push(nodes.len());
                nodes.push(Search {
                    ty: child.ty,
                    offset: relative,
                    parent: Some(next),
                    field,
                });
            }
        }
        let mut current =
            selected.ok_or("symbolic pointer typed reinterpretation has no declared subobject")?;
        let actual = nodes[current].ty;
        let mut path = Vec::new();
        while let Some(parent) = nodes[current].parent {
            self.charge(1)?;
            self.reserve_cells(
                nodes
                    .len()
                    .saturating_mul(4)
                    .saturating_add(path.len())
                    .saturating_add(1),
            )?;
            path.try_reserve(1)
                .map_err(|_| "symbolic typed path allocation failed")?;
            path.push(nodes[current].field);
            current = parent;
        }
        path.reverse();
        Ok((path, actual))
    }
    fn subobject_children(
        &mut self,
        ty: TypeId,
        offset: u64,
        zero: bool,
    ) -> Result<Vec<u64>, Failure> {
        self.footprint(ty)?;
        match Reader::new(self.evaluator.facts()).storage_shape(ty)? {
            StoredShape::Array { stride, length, .. } => {
                self.reserve_cells(2)?;
                if stride == 0 {
                    return Ok(if offset == 0 {
                        (0..length.min(2)).collect()
                    } else {
                        Vec::new()
                    });
                }
                let index = offset / stride;
                let mut children = Vec::new();
                if index < length {
                    children.push(index);
                }
                if zero && offset.is_multiple_of(stride) && index != 0 && index - 1 < length {
                    children.push(index - 1);
                }
                Ok(children)
            }
            StoredShape::Product { fields, .. } => {
                let count = fields.len();
                self.product_subobjects(ty, count, offset, zero)
            }
            _ => Ok(Vec::new()),
        }
    }
    fn product_subobjects(
        &mut self,
        ty: TypeId,
        fields: usize,
        offset: u64,
        zero: bool,
    ) -> Result<Vec<u64>, Failure> {
        if !self
            .memory
            .bytes
            .paths
            .as_ref()
            .is_some_and(|paths| paths.fields.contains_key(&ty))
        {
            let index = self.index_subobjects(ty, fields)?;
            self.reserve_cells(index.retained_slots().saturating_add(16))?;
            let paths = self
                .memory
                .bytes
                .paths
                .get_or_insert_with(|| Box::new(Paths::default()));
            paths
                .fields
                .try_reserve(1)
                .map_err(|_| "symbolic subobject family cache allocation failed")?;
            paths.retained = paths
                .retained
                .checked_add(index.retained_slots().saturating_add(16))
                .ok_or("symbolic subobject cache retention overflow")?;
            paths.fields.insert(ty, index);
        }
        let index = self
            .memory
            .bytes
            .paths
            .as_ref()
            .and_then(|paths| paths.fields.get(&ty))
            .ok_or("missing symbolic subobject index")?;
        let mut count = 0usize;
        let work = index.visit(
            offset,
            zero,
            self.limits.steps.saturating_sub(self.work.steps),
            |_| count += 1,
        )?;
        self.charge(work)?;
        self.reserve_cells(count.saturating_mul(2).saturating_add(4))?;
        let mut children = Vec::new();
        children
            .try_reserve_exact(count)
            .map_err(|_| "symbolic subobject candidates allocation failed")?;
        let index = self
            .memory
            .bytes
            .paths
            .as_ref()
            .and_then(|paths| paths.fields.get(&ty))
            .ok_or("missing symbolic subobject index")?;
        let work = index.visit(
            offset,
            zero,
            self.limits.steps.saturating_sub(self.work.steps),
            |field| children.push(field),
        )?;
        self.charge(work)?;
        Ok(children)
    }
}
