//! Place queries inspect storage without implicitly moving its containing value.
use super::references::{Access, Expression as Reference, Place};
use super::{Error, Expression, InitializerKind, NodeKey, Queries, Query, State, Type, Value};
use crate::{sema::model::Scalar, types::TypeId};

pub(super) enum Work {
    Anchor {
        key: NodeKey,
        source: Query,
        access: Access,
    },
    Borrow {
        key: NodeKey,
        operand: NodeKey,
    },
    Dereference {
        key: NodeKey,
        operand: NodeKey,
    },
    Project {
        key: NodeKey,
        value: NodeKey,
        index: Index,
    },
    Read(NodeKey),
}
pub(super) enum Index {
    Fixed(u64),
    Computed(NodeKey),
}
impl Queries<'_> {
    fn reference_work(&mut self, work: Work) {
        self.work.push(super::Work::Reference(work));
    }
    pub(super) fn enter_reference(&mut self, key: NodeKey, expression: Reference) {
        if let Reference::Borrow(operand) = expression {
            self.reference_work(Work::Borrow { key, operand });
            self.work.push(super::Work::Enter(Query::Place(operand)));
        } else {
            self.reference_work(Work::Read(key));
            self.work.push(super::Work::Enter(Query::Place(key)));
        }
    }
    pub(super) fn enter_place(&mut self, key: NodeKey) -> Result<(), Error> {
        match self.expression(key)? {
            Expression::Forward(child) => self.forward(Query::Place(key), Query::Place(child)),
            Expression::Reference(Reference::Dereference(operand)) => {
                self.reference_work(Work::Dereference { key, operand });
                self.work.push(super::Work::Enter(Query::Node(operand)));
            }
            Expression::Reference(Reference::Field { value, index }) => {
                self.reference_work(Work::Project {
                    key,
                    value,
                    index: Index::Fixed(index),
                });
                self.work.push(super::Work::Enter(Query::Place(value)));
            }
            Expression::Reference(Reference::Index { value, index }) => {
                self.reference_work(Work::Project {
                    key,
                    value,
                    index: Index::Computed(index),
                });
                self.work.push(super::Work::Enter(Query::Node(index)));
                self.work.push(super::Work::Enter(Query::Place(value)));
            }
            Expression::Constant(index) => {
                let initializer = self.typing.facts.initializers[index];
                let (source, access) = match initializer.kind {
                    InitializerKind::Const => (Query::Constant(index), Access::Owned),
                    InitializerKind::Static { mutable: false } => {
                        (Query::Static(index), Access::Static(initializer.definition))
                    }
                    InitializerKind::Static { mutable: true } => {
                        return Err(self.error(
                            Query::Place(key),
                            "constant references cannot access mutable static storage",
                        ));
                    }
                };
                self.reference_work(Work::Anchor {
                    key,
                    source,
                    access,
                });
                self.work.push(super::Work::Enter(source));
            }
            _ => {
                let source = Query::Node(key);
                self.reference_work(Work::Anchor {
                    key,
                    source,
                    access: Access::Owned,
                });
                self.work.push(super::Work::Enter(source));
            }
        }
        Ok(())
    }
    pub(super) fn finish_reference(&mut self, work: Work) -> Result<(), Error> {
        let (query, value) = match work {
            Work::Anchor {
                key,
                source,
                access,
            } => {
                let value = self.value(source)?;
                let ty = self.place_type(key)?;
                let value = self.insert_place(
                    key,
                    Place {
                        value,
                        ty,
                        access,
                        packed: None,
                        projection: None,
                    },
                )?;
                (Query::Place(key), value)
            }
            Work::Borrow { key, operand } => {
                if matches!(
                    self.typing.facts.types.kind(self.place_type(key)?),
                    Some(Type::Reference { mutable: true, .. })
                ) {
                    return Err(self.error(
                        Query::Node(key),
                        "mutable constant references require constant memory semantics",
                    ));
                }
                let value = self.value(Query::Place(operand))?;
                let mut place = self.place(key, value)?;
                self.check_reference_alignment(key, place)?;
                if matches!(place.access, Access::Owned) {
                    place.access = Access::Shared;
                    (Query::Node(key), self.insert_place(key, place)?)
                } else {
                    (Query::Node(key), value)
                }
            }
            Work::Dereference { key, operand } => {
                let value = self.value(Query::Node(operand))?;
                let ty = self.place_type(operand)?;
                let value = self.referent(key, value, ty)?;
                (Query::Place(key), value)
            }
            Work::Project { key, value, index } => {
                let index = match index {
                    Index::Fixed(index) => index,
                    Index::Computed(index) => u64::try_from(
                        self.value(Query::Node(index))?
                            .integer()
                            .map_err(|e| self.error(Query::Place(key), e))?,
                    )
                    .map_err(|_| self.error(Query::Place(key), "constant index overflow"))?,
                };
                let value = self.project_place(key, value, index)?;
                (Query::Place(key), value)
            }
            Work::Read(key) => {
                let place = self.place(key, self.value(Query::Place(key))?)?;
                if !matches!(place.access, Access::Owned) {
                    self.check_copy_read(Query::Node(key), self.place_type(key)?)?;
                }
                (Query::Node(key), place.value)
            }
        };
        self.states.insert(query, State::Ready(value));
        Ok(())
    }
    fn place_type(&self, key: NodeKey) -> Result<TypeId, Error> {
        self.typing
            .facts
            .node(key)
            .and_then(|f| f.ty)
            .ok_or_else(|| self.error(Query::Place(key), "missing constant place type"))
    }
    fn place(&self, key: NodeKey, value: Value) -> Result<Place, Error> {
        self.typing
            .facts
            .constant_references
            .get(value)
            .map_err(|e| self.error(Query::Place(key), e))
    }
    fn insert_place(&mut self, key: NodeKey, place: Place) -> Result<Value, Error> {
        self.typing
            .facts
            .constant_references
            .insert(place)
            .map_err(|e| self.error(Query::Place(key), e))
    }
    fn referent(&mut self, key: NodeKey, value: Value, ty: TypeId) -> Result<Value, Error> {
        if matches!(value, Value::Reference(_)) {
            return Ok(value);
        }
        let Some(Type::Reference {
            mutable: false,
            referent,
            ..
        }) = self.typing.facts.types.kind(ty)
        else {
            return Err(self.error(
                Query::Place(key),
                "constant dereference requires an immutable reference",
            ));
        };
        if !matches!(value, Value::Text(_) | Value::Bytes(_)) {
            return Err(self.error(Query::Place(key), "missing constant reference value"));
        }
        self.insert_place(
            key,
            Place {
                value,
                ty: referent,
                access: Access::Shared,
                packed: None,
                projection: None,
            },
        )
    }
    fn project_place(&mut self, key: NodeKey, base: NodeKey, index: u64) -> Result<Value, Error> {
        let mut location = self.value(Query::Place(base))?;
        let mut place = self.place(key, location)?;
        let count = self.typing.facts.autoderefs.get(&key).copied().unwrap_or(0);
        for _ in 0..count {
            self.budget()?;
            let value = self.referent(key, place.value, place.ty)?;
            location = value;
            place = self.place(key, value)?;
        }
        if let Some(Type::Adt(item, _)) = self.typing.facts.types.kind(place.ty)
            && let Some(pack) = self
                .typing
                .facts
                .aggregates
                .get(item)
                .and_then(|d| d.repr.packed)
        {
            place.packed = Some(place.packed.map_or(pack, |p| p.min(pack)));
        }
        let value = match place.value {
            Value::Product(id) => self
                .typing
                .facts
                .constant_products
                .get(id, index)
                .map_err(|e| self.error(Query::Place(key), e))?,
            Value::Bytes(id) => {
                let byte =
                    self.typing
                        .facts
                        .texts
                        .get(id)
                        .map_err(|e| self.error(Query::Place(key), e))?
                        .get(usize::try_from(index).map_err(|_| {
                            self.error(Query::Place(key), "constant index overflow")
                        })?)
                        .copied()
                        .ok_or_else(|| {
                            self.error(Query::Place(key), "constant index is out of bounds")
                        })?;
                Value::Scalar(Scalar::Integer(u128::from(byte)))
            }
            _ => {
                return Err(self.error(Query::Place(key), "constant projection requires a product"));
            }
        };
        let ty = self.place_type(key)?;
        let base = super::references::References::id(location)
            .map_err(|e| self.error(Query::Place(key), e))?;
        self.typing
            .facts
            .constant_references
            .project(base, index, Place { value, ty, ..place })
            .map_err(|e| self.error(Query::Place(key), e))
    }
    fn check_reference_alignment(&mut self, key: NodeKey, place: Place) -> Result<(), Error> {
        if let Access::Static(id) = place.access {
            let index = self.constants[id.0 as usize].ok_or_else(|| {
                self.error(
                    Query::Place(key),
                    "constant reference has unsupported static provenance",
                )
            })?;
            if !matches!(
                self.typing.facts.initializers[index].kind,
                InitializerKind::Static { mutable: false }
            ) {
                return Err(self.error(
                    Query::Place(key),
                    "constant reference requires immutable static storage",
                ));
            }
        }
        if let Some(packed) = place.packed {
            let facts = &mut *self.typing.facts;
            let layout = facts
                .layouts
                .demand(&facts.types, &facts.aggregates, place.ty)
                .map_err(|e| self.error(Query::Place(key), e.message))?;
            let align = self
                .typing
                .facts
                .layouts
                .at(layout)
                .ok_or_else(|| self.error(Query::Place(key), "missing constant place layout"))?
                .align;
            if align > packed {
                return Err(self.error(
                    Query::Place(key),
                    "constant reference to packed storage is unaligned",
                ));
            }
        }
        Ok(())
    }
    fn check_copy_read(&mut self, query: Query, ty: TypeId) -> Result<(), Error> {
        let facts = &mut *self.typing.facts;
        let properties = facts
            .copies
            .demand(&facts.types, &facts.aggregates, &facts.derives, ty)
            .map_err(|e| self.error(query, e))?;
        if !properties.copy {
            return Err(self.error(
                query,
                if matches!(query, Query::Constant(_)) {
                    "cannot move a non-Copy constant value out of static storage"
                } else {
                    "cannot move a non-Copy constant value out of borrowed or static storage"
                },
            ));
        }
        Ok(())
    }
    pub(super) fn check_static_read(&mut self, index: usize) -> Result<(), Error> {
        let initializer = self.typing.facts.initializers[index];
        if matches!(initializer.kind, InitializerKind::Static { .. }) {
            self.check_copy_read(Query::Constant(index), initializer.ty)?;
        }
        Ok(())
    }
}
