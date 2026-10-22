//! Constant locations carry value, access and packed-field provenance in a flat arena.
mod ranges;
use super::{Declarations, Error, Kind, NodeKey, Operation, Queries, Query, Value};
use crate::{resolve::model::DefId, sema::members::Selection, types::TypeId};
pub(super) use ranges::ProjectionView;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Id(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Metadata {
    Length(u64),
    Object { concrete: TypeId, target: TypeId },
}
/// Write authority follows one pointer value, independently of its raw mutability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WritePermission {
    Writable,
    SharedDerived,
}
/// Metadata belongs to a pointer value, while its backing location keeps its
/// original sized type and allocation identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Pointer {
    pub place: Id,
    pub metadata: Option<Metadata>,
    pub write: WritePermission,
}
impl Pointer {
    pub(super) const fn new(place: Id) -> Self {
        Self {
            place,
            metadata: None,
            write: WritePermission::SharedDerived,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) enum ProjectionId {
    Field(u32),
    Sequence(u32),
}
struct Projection {
    base: Id,
    index: u64,
}
#[derive(Clone, Copy)]
pub(super) enum Access {
    Owned,
    Shared,
    Static(DefId),
}
#[derive(Clone, Copy)]
pub(super) struct Place {
    pub value: Option<Value>,
    pub ty: TypeId,
    pub access: Access,
    pub packed: Option<u32>,
    /// Physical subobject identity; dereferencing reuses the selected location.
    pub projection: Option<ProjectionId>,
}
#[derive(Default)]
pub(in crate::sema) struct References {
    places: Vec<Place>,
    projections: Vec<Projection>,
    ranges: Option<Vec<ranges::Subsequence>>,
    statics: HashMap<DefId, Id>,
    literals: HashMap<crate::sema::text::Id, Id>,
    pub(super) static_order: Vec<DefId>,
}
impl References {
    pub(super) fn literal_place(
        &mut self,
        text: crate::sema::text::Id,
        ty: TypeId,
    ) -> Result<Value, &'static str> {
        if let Some(&place) = self.literals.get(&text) {
            return Ok(Value::Reference(Pointer::new(place)));
        }
        // Charge a conservative 128 bytes per retained map entry and backing
        // record. Bytes themselves remain in the existing canonical text arena.
        if self.literals.len() >= 64 * 1024 * 1024 / 128 {
            return Err("constant literal backing storage limit exceeded");
        }
        let value = self.insert(Place {
            value: Some(Value::Bytes(text)),
            ty,
            access: Access::Shared,
            packed: None,
            projection: None,
        })?;
        self.literals.insert(text, Self::id(value)?);
        Ok(value)
    }
    pub(super) fn static_place(&mut self, id: DefId, ty: TypeId) -> Result<Value, &'static str> {
        if let Some(&place) = self.statics.get(&id) {
            return Ok(Value::Reference(Pointer::new(place)));
        }
        let value = self.insert(Place {
            value: None,
            ty,
            access: Access::Static(id),
            packed: None,
            projection: None,
        })?;
        self.statics.insert(id, Self::id(value)?);
        self.static_order.push(id);
        Ok(value)
    }
    pub(super) fn complete(&mut self, id: Id, value: Value) -> Result<(), &'static str> {
        self.places
            .get_mut(id.0 as usize)
            .ok_or("missing constant place")?
            .value = Some(value);
        Ok(())
    }
    pub(super) fn id(value: Value) -> Result<Id, &'static str> {
        if let Value::Reference(pointer) = value {
            Ok(pointer.place)
        } else {
            Err("constant value is not a reference")
        }
    }
    pub(super) fn insert(&mut self, place: Place) -> Result<Value, &'static str> {
        if place.projection.is_some() {
            self.projection(place)?;
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        self.places.push(place);
        let mut pointer = Pointer::new(id);
        if matches!(place.access, Access::Owned) {
            pointer.write = WritePermission::Writable;
        }
        Ok(Value::Reference(pointer))
    }
    pub(super) fn project(
        &mut self,
        base: Pointer,
        mut index: u64,
        mut place: Place,
    ) -> Result<Value, &'static str> {
        if base.place.0 as usize >= self.places.len() {
            return Err("constant projection lacks an earlier base location");
        }
        let mut base = base;
        if self.ranges.is_some()
            && let Some(ProjectionView::Sequence {
                base: original,
                from,
                length,
            }) = self.projection(self.get(Value::Reference(base))?)?
        {
            if index >= length {
                return Err("constant index is out of bounds");
            }
            index = from
                .checked_add(index)
                .ok_or("constant range offset overflow")?;
            base.place = original;
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        let projection = ProjectionId::Field(
            u32::try_from(self.projections.len()).map_err(|_| "too many constant projections")?,
        );
        place.projection = Some(projection);
        self.projections.push(Projection {
            base: base.place,
            index,
        });
        self.places.push(place);
        Ok(Value::Reference(Pointer {
            place: id,
            metadata: None,
            write: base.write,
        }))
    }
    pub(super) fn get(&self, value: Value) -> Result<Place, &'static str> {
        let Value::Reference(pointer) = value else {
            return Err("constant value is not a reference");
        };
        self.places
            .get(pointer.place.0 as usize)
            .copied()
            .ok_or("missing constant place")
    }
}
#[derive(Clone, Copy)]
pub(super) enum Expression {
    Borrow(NodeKey),
    Address(NodeKey),
    Dereference(NodeKey),
    Field { value: NodeKey, index: u64 },
    Index { value: NodeKey, index: NodeKey },
}
impl Expression {
    pub(super) fn dependencies(self, mut visit: impl FnMut(NodeKey)) {
        match self {
            Self::Borrow(value)
            | Self::Address(value)
            | Self::Dereference(value)
            | Self::Field { value, .. } => {
                visit(value);
            }
            Self::Index { value, index } => {
                visit(index);
                visit(value);
            }
        }
    }
}
impl Queries<'_> {
    pub(super) fn adjusted_reference(
        &mut self,
        query: Query,
        value: Value,
    ) -> Result<Value, Error> {
        let Query::Node(key) = query else {
            return Ok(value);
        };
        let Some(&conversion) = self.typing.facts.unsizing_adjustments.get(&key) else {
            return Ok(value);
        };
        let metadata = match conversion {
            crate::types::unsizing::Conversion::Slice(array) => {
                let array = self.specialized_type(key, array)?;
                let Some(crate::types::Kind::Array { length, .. }) =
                    self.typing.facts.types.kind(array)
                else {
                    return Err(self.error(query, "constant unsizing length is not concrete"));
                };
                Metadata::Length(length)
            }
            crate::types::unsizing::Conversion::Object { concrete, target } => Metadata::Object {
                concrete: self.specialized_type(key, concrete)?,
                target: self.specialized_type(key, target)?,
            },
            crate::types::unsizing::Conversion::Upcast { target, .. } => {
                let Some(Metadata::Object { concrete, .. }) = value.metadata() else {
                    return Err(self.error(query, "constant upcast requires object metadata"));
                };
                Metadata::Object {
                    concrete,
                    target: self.specialized_type(key, target)?,
                }
            }
        };
        super::address_values::unsize(value, metadata).map_err(|e| self.error(query, e.message()))
    }
    fn union_receiver(&self, key: NodeKey, base: NodeKey) -> Result<bool, Error> {
        let Some(mut ty) = self.typing.facts.node(base).and_then(|f| f.ty) else {
            return Ok(false);
        };
        let count = self.typing.facts.autoderefs.get(&key).copied().unwrap_or(0);
        if count > 1_000_000 {
            return Err(self.error(
                Query::Node(key),
                "union field autodereference budget exceeded",
            ));
        }
        for _ in 0..count {
            let Some(crate::types::Kind::Reference { referent, .. }) =
                self.typing.facts.types.kind(ty)
            else {
                return Err(self.error(
                    Query::Node(key),
                    "union field lost its selected reference receiver",
                ));
            };
            ty = referent;
        }
        Ok(
            matches!(self.typing.facts.types.kind(ty), Some(crate::types::Kind::Adt(item, _)) if self.typing.facts.aggregates.get(item).is_some_and(|d| d.kind == crate::types::aggregate::Kind::Union)),
        )
    }
    pub(super) fn reference_expression(&self, key: NodeKey) -> Result<Option<Expression>, Error> {
        let facts =
            self.typing.facts.node(key).ok_or_else(|| {
                self.error(Query::Node(key), "missing reference expression facts")
            })?;
        let child = |node| NodeKey { node, ..key };
        Ok(
            match (Declarations::kind(self.report, key), facts.operation) {
                (Kind::Unary { operand, .. }, Some(Operation::Borrow)) => {
                    Some(Expression::Borrow(child(operand)))
                }
                (Kind::Unary { operand, .. }, Some(Operation::Address)) => {
                    Some(Expression::Address(child(operand)))
                }
                (Kind::Unary { operand, .. }, Some(Operation::Dereference)) => {
                    if self
                        .typing
                        .facts
                        .node(child(operand))
                        .and_then(|f| f.ty)
                        .and_then(|ty| self.typing.facts.types.kind(ty))
                        .is_some_and(|kind| matches!(kind, crate::types::Kind::Pointer { .. }))
                    {
                        None
                    } else {
                        Some(Expression::Dereference(child(operand)))
                    }
                }
                (Kind::Field { value, .. }, Some(Operation::Field { field })) => {
                    if self.union_receiver(key, child(value))? {
                        return Ok(None);
                    }
                    let index = match field {
                        Selection::Tuple(index) => index,
                        Selection::Nominal(field) => {
                            self.typing.facts.members.ordinal(field).ok_or_else(|| {
                                self.error(Query::Node(key), "missing constant field ordinal")
                            })?
                        }
                    };
                    Some(Expression::Field {
                        value: child(value),
                        index: u64::from(index),
                    })
                }
                (
                    Kind::Index { value, index },
                    Some(
                        Operation::Index { .. }
                        | Operation::IndexParameter { .. }
                        | Operation::IndexSlice,
                    ),
                ) => Some(Expression::Index {
                    value: child(value),
                    index: child(index),
                }),
                _ => None,
            },
        )
    }
}

#[cfg(test)]
#[test]
fn pointer_value_storage_layout() {
    // Keep the x86-64 host storage contract used by execution slot accounting.
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(std::mem::size_of::<Pointer>(), 24);
        assert_eq!(std::mem::size_of::<super::values::Constant>(), 32);
    }
    eprintln!(
        "pointer={} constant={} value={} location={} reference_place={}",
        std::mem::size_of::<Pointer>(),
        std::mem::size_of::<super::values::Constant>(),
        std::mem::size_of::<Value>(),
        std::mem::size_of::<super::values::Location>(),
        std::mem::size_of::<Place>()
    );
}

#[cfg(test)]
#[test]
fn pointer_write_permission_is_per_value_and_projection() -> Result<(), &'static str> {
    use super::values::{Constant, WritePermission};
    let mut arena = References::default();
    let place = Place {
        value: None,
        ty: TypeId::primitive(crate::types::Primitive::U32),
        access: Access::Owned,
        packed: None,
        projection: None,
    };
    let writable = Constant(arena.insert(place)?);
    let shared = writable.restrict_shared();
    assert_eq!(writable.location(), shared.location());
    assert_eq!(writable.write_permission(), Some(WritePermission::Writable));
    assert_eq!(
        shared.write_permission(),
        Some(WritePermission::SharedDerived)
    );
    assert!(shared == shared.restrict_shared());
    let distinct = std::collections::HashSet::from([writable, shared]);
    assert_eq!(distinct.len(), 2);
    for value in [writable, shared] {
        let Value::Reference(pointer) = value.0 else {
            return Err("missing pointer");
        };
        let projected = Constant(arena.project(pointer, 0, place)?);
        assert_eq!(projected.write_permission(), value.write_permission());
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn repeated_literal_addresses_reuse_one_backing_per_text() -> Result<(), &'static str> {
    let mut types = crate::types::Store::default();
    let ty = types.intern(crate::types::Kind::Array {
        element: TypeId::primitive(crate::types::Primitive::U8),
        length: 3,
    })?;
    let mut references = References::default();
    for width in [8, 32, 128] {
        for index in 0..width {
            let text = crate::sema::text::Id(index);
            let first = references.literal_place(text, ty)?;
            for _ in 0..128 {
                assert!(first == references.literal_place(text, ty)?);
            }
            let place = references.get(first)?;
            assert!(matches!(place.access, Access::Shared));
            assert!(matches!(place.value, Some(Value::Bytes(id)) if id == text));
        }
        assert_eq!(references.places.len(), width as usize);
        assert_eq!(references.literals.len(), width as usize);
        assert!(references.projections.is_empty());
    }
    Ok(())
}
