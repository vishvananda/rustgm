//! Constant locations carry value, access and packed-field provenance in a flat arena.
use super::{Declarations, Error, Kind, NodeKey, Operation, Queries, Query, Value};
use crate::{resolve::model::DefId, sema::members::Selection, types::TypeId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Id(u32);
#[derive(Clone, Copy)]
pub(super) struct ProjectionId(u32);
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
    statics: HashMap<DefId, Id>,
    pub(super) static_order: Vec<DefId>,
}
impl References {
    pub(super) fn static_place(&mut self, id: DefId, ty: TypeId) -> Result<Value, &'static str> {
        if let Some(&place) = self.statics.get(&id) {
            return Ok(Value::Reference(place));
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
        if let Value::Reference(id) = value {
            Ok(id)
        } else {
            Err("constant value is not a reference")
        }
    }
    pub(super) fn insert(&mut self, place: Place) -> Result<Value, &'static str> {
        if place
            .projection
            .is_some_and(|id| id.0 as usize >= self.projections.len())
        {
            return Err("invalid constant projection identity");
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        self.places.push(place);
        Ok(Value::Reference(id))
    }
    pub(super) fn project(
        &mut self,
        base: Id,
        index: u64,
        mut place: Place,
    ) -> Result<Value, &'static str> {
        if base.0 as usize >= self.places.len() {
            return Err("constant projection lacks an earlier base location");
        }
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        let projection = ProjectionId(
            u32::try_from(self.projections.len()).map_err(|_| "too many constant projections")?,
        );
        place.projection = Some(projection);
        self.projections.push(Projection { base, index });
        self.places.push(place);
        Ok(Value::Reference(id))
    }
    pub(super) fn projection(&self, place: Place) -> Result<Option<(Id, u64)>, &'static str> {
        place
            .projection
            .map(|id| {
                self.projections
                    .get(id.0 as usize)
                    .map(|p| (p.base, p.index))
                    .ok_or("missing constant projection")
            })
            .transpose()
    }
    pub(super) fn get(&self, value: Value) -> Result<Place, &'static str> {
        let Value::Reference(id) = value else {
            return Err("constant value is not a reference");
        };
        self.places
            .get(id.0 as usize)
            .copied()
            .ok_or("missing constant place")
    }
}
#[derive(Clone, Copy)]
pub(super) enum Expression {
    Borrow(NodeKey),
    Dereference(NodeKey),
    Field { value: NodeKey, index: u64 },
    Index { value: NodeKey, index: NodeKey },
}
impl Expression {
    pub(super) fn dependencies(self, mut visit: impl FnMut(NodeKey)) {
        match self {
            Self::Borrow(value) | Self::Dereference(value) | Self::Field { value, .. } => {
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
                (Kind::Unary { operand, .. }, Some(Operation::Dereference)) => {
                    Some(Expression::Dereference(child(operand)))
                }
                (Kind::Field { value, .. }, Some(Operation::Field { field })) => {
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
