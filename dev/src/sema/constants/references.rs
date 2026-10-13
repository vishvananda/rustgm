//! Constant locations carry value, access and packed-field provenance in a flat arena.
use super::{Declarations, Error, Kind, NodeKey, Operation, Queries, Query, Value};
use crate::{resolve::model::DefId, sema::members::Selection, types::TypeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Id(u32);
#[derive(Clone, Copy)]
pub(super) enum Access {
    Owned,
    Shared,
    Static(DefId),
}
#[derive(Clone, Copy)]
pub(super) struct Place {
    pub value: Value,
    pub ty: TypeId,
    pub access: Access,
    pub packed: Option<u32>,
}
#[derive(Default)]
pub(in crate::sema) struct References {
    places: Vec<Place>,
}
impl References {
    pub(super) fn insert(&mut self, place: Place) -> Result<Value, &'static str> {
        let id = Id(u32::try_from(self.places.len()).map_err(|_| "too many constant places")?);
        self.places.push(place);
        Ok(Value::Reference(id))
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
                    Some(Operation::Index { .. } | Operation::IndexSlice),
                ) => Some(Expression::Index {
                    value: child(value),
                    index: child(index),
                }),
                _ => None,
            },
        )
    }
}
