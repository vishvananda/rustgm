//! Place queries inspect storage without implicitly moving its containing value.
use super::references::{Access, Expression as Reference, Place};
use super::{Error, Expression, InitializerKind, NodeKey, Queries, Query, State, Type, Value};
use crate::types::TypeId;

pub(super) enum Work {
    Anchor {
        key: NodeKey,
        source: Query,
        access: Access,
    },
    Borrow {
        key: NodeKey,
        operand: NodeKey,
        raw: bool,
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
    Projection {
        key: NodeKey,
        location: Value,
        index: u64,
        remaining: u32,
    },
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
        if let Reference::Borrow(operand) | Reference::Address(operand) = expression {
            self.reference_work(Work::Borrow {
                key,
                operand,
                raw: matches!(expression, Reference::Address(_)),
            });
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
                        let value = self
                            .typing
                            .facts
                            .constant_references
                            .static_place(initializer.definition, initializer.ty)
                            .map_err(|e| self.error(Query::Place(key), e))?;
                        self.states
                            .insert(self.state_key(Query::Place(key)), State::Ready(value));
                        return Ok(());
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
                        value: Some(value),
                        ty,
                        access,
                        packed: None,
                        projection: None,
                    },
                )?;
                (Query::Place(key), value)
            }
            Work::Borrow { key, operand, raw } => {
                (Query::Node(key), self.borrow_place(key, operand, raw)?)
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
                let location = self.value(Query::Place(value))?;
                let remaining = self.typing.facts.autoderefs.get(&key).copied().unwrap_or(0);
                return self.project_place(key, location, index, remaining);
            }
            Work::Projection {
                key,
                location,
                index,
                remaining,
            } => {
                return self.project_place(key, location, index, remaining);
            }
            Work::Read(key) => {
                let location = self.value(Query::Place(key))?;
                let place = self.place(key, location)?;
                if !matches!(place.access, Access::Owned) {
                    let ty = self.place_type(key)?;
                    self.check_copy_read(Query::Node(key), ty)?;
                }
                if let Some(value) = place.value {
                    (Query::Node(key), value)
                } else {
                    let id = super::references::References::id(location)
                        .map_err(|e| self.error(Query::Place(key), e))?;
                    self.forward(Query::Node(key), Query::Contents(key, id));
                    return Ok(());
                }
            }
        };
        self.complete_query(query, value)
    }
    fn borrow_place(&mut self, key: NodeKey, operand: NodeKey, raw: bool) -> Result<Value, Error> {
        let ty = self.place_type(key)?;
        if matches!(
            self.typing.facts.types.kind(ty),
            Some(Type::Reference { mutable: true, .. })
        ) {
            return Err(self.error(
                Query::Node(key),
                "mutable constant references require constant memory semantics",
            ));
        }
        let value = self.value(Query::Place(operand))?;
        let mut place = self.place(key, value)?;
        if raw {
            if matches!(place.access, Access::Owned) {
                return Err(self.error(Query::Node(key), "cannot take raw address of a temporary"));
            }
            if matches!(
                self.typing.facts.types.kind(ty),
                Some(Type::Pointer { mutable: true, .. })
            ) {
                return Err(self.error(
                    Query::Node(key),
                    "mutable constant address requires mutable storage",
                ));
            }
        } else {
            self.check_reference_alignment(key, place)?;
        }
        if matches!(place.access, Access::Owned) {
            place.access = Access::Shared;
            self.insert_place(key, place)
        } else {
            Ok(value)
        }
    }
    fn place_type(&mut self, key: NodeKey) -> Result<TypeId, Error> {
        let ty = self
            .typing
            .facts
            .node(key)
            .and_then(|f| f.ty)
            .ok_or_else(|| self.error(Query::Place(key), "missing constant place type"))?;
        self.specialized_type(key, ty)
    }
    pub(super) fn place(&self, key: NodeKey, value: Value) -> Result<Place, Error> {
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
                value: Some(value),
                ty: referent,
                access: Access::Shared,
                packed: None,
                projection: None,
            },
        )
    }
    fn project_place(
        &mut self,
        key: NodeKey,
        mut location: Value,
        index: u64,
        mut remaining: u32,
    ) -> Result<(), Error> {
        let mut place = self.place(key, location)?;
        while remaining != 0 {
            self.budget()?;
            let Some(value) = place.value else {
                let id = super::references::References::id(location)
                    .map_err(|e| self.error(Query::Place(key), e))?;
                self.reference_work(Work::Projection {
                    key,
                    location,
                    index,
                    remaining,
                });
                self.work.push(super::Work::Enter(Query::Contents(key, id)));
                return Ok(());
            };
            location = self.referent(key, value, place.ty)?;
            place = self.place(key, location)?;
            remaining -= 1;
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
        let value = if let Some(value) = place.value {
            Some(self.field_value(key, value, index)?)
        } else {
            self.check_static_projection(key, place.ty, index)?;
            None
        };
        // Project physical backing, retaining the actual sized field type even
        // when the expression sees an unsized view of that field.
        let (ty, tail) = self.backing_field(key, place, index)?;
        let metadata = if tail && let Value::Reference(pointer) = location {
            pointer.metadata
        } else {
            None
        };
        let base = super::references::References::id(location)
            .map_err(|e| self.error(Query::Place(key), e))?;
        let mut value = self
            .typing
            .facts
            .constant_references
            .project(base, index, Place { value, ty, ..place })
            .map_err(|e| self.error(Query::Place(key), e))?;
        if let Value::Reference(pointer) = &mut value {
            pointer.metadata = metadata;
        }
        self.states
            .insert(self.state_key(Query::Place(key)), State::Ready(value));
        Ok(())
    }
    fn backing_field(
        &self,
        key: NodeKey,
        place: Place,
        index: u64,
    ) -> Result<(TypeId, bool), Error> {
        let facts = &*self.typing.facts;
        let error = |message| self.error(Query::Place(key), message);
        let fields = match facts.types.kind(place.ty) {
            Some(Type::Array { element, length }) if index < length => return Ok((element, false)),
            Some(Type::Slice(element)) => return Ok((element, false)),
            Some(Type::Tuple(fields)) => fields,
            Some(Type::Adt(..)) => {
                let variant = match place.value {
                    Some(Value::Product(id)) => {
                        facts.constant_products.variant(id).map_err(error)?
                    }
                    Some(Value::Variant(id)) => Some(id),
                    _ => None,
                };
                let ordinal = variant.map_or(0, |id| facts.members.variants[id.0 as usize].ordinal);
                facts
                    .aggregates
                    .shape(&facts.types, place.ty)
                    .map_err(error)?
                    .variant(ordinal as usize)
                    .ok_or_else(|| error("missing constant backing variant"))?
                    .fields
            }
            _ => return Err(error("constant backing is not a product")),
        };
        let fields = facts
            .types
            .list(fields)
            .ok_or_else(|| error("missing backing field types"))?;
        let index = usize::try_from(index).map_err(|_| error("constant backing index overflow"))?;
        let ty = fields
            .get(index)
            .copied()
            .ok_or_else(|| error("constant backing field is out of bounds"))?;
        Ok((ty, index + 1 == fields.len()))
    }
    fn check_static_projection(&self, key: NodeKey, ty: TypeId, index: u64) -> Result<(), Error> {
        let facts = &*self.typing.facts;
        if let Some(Type::Adt(item, _)) = facts.types.kind(ty)
            && facts
                .aggregates
                .get(item)
                .is_some_and(|d| d.kind == crate::types::aggregate::Kind::Union)
        {
            return Err(self.error(
                Query::Place(key),
                "union field reinterpretation in constant execution is not implemented",
            ));
        }
        let fields = match facts.types.kind(ty) {
            Some(Type::Array { length, .. }) => {
                return if index < length {
                    Ok(())
                } else {
                    Err(self.error(Query::Place(key), "constant index is out of bounds"))
                };
            }
            Some(Type::Tuple(fields)) => Some(fields),
            Some(Type::Adt(item, _)) => facts
                .aggregates
                .get(item)
                .filter(|d| d.kind == crate::types::aggregate::Kind::Struct)
                .and_then(|d| facts.aggregates.variants(d).first())
                .map(|v| v.fields),
            _ => None,
        };
        if fields
            .and_then(|f| facts.types.list(f))
            .is_some_and(|f| index < f.len() as u64)
        {
            Ok(())
        } else {
            Err(self.error(Query::Place(key), "constant projection requires a product"))
        }
    }
    fn check_reference_alignment(&mut self, key: NodeKey, place: Place) -> Result<(), Error> {
        if let Access::Static(id) = place.access {
            let index = self.typing.facts.initializer_index(id).ok_or_else(|| {
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
        if matches!(initializer.kind, InitializerKind::Static { mutable: true }) {
            return Err(self.error(
                Query::Constant(index),
                "constant evaluation cannot read mutable static storage",
            ));
        }
        if matches!(initializer.kind, InitializerKind::Static { .. }) {
            self.check_copy_read(Query::Constant(index), initializer.ty)?;
        }
        Ok(())
    }
}
