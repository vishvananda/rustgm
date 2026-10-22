//! Reading a constant location is separate from forming its storage address.
use super::references::{Access, Id, ProjectionView};
use super::{Error, NodeKey, Queries, Query, State, Value, Work as Pending};
use crate::sema::model::Scalar;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) enum Work {
    Range {
        key: NodeKey,
        id: Id,
        base: Id,
        from: u64,
        length: u64,
    },
    Project {
        key: NodeKey,
        id: Id,
        base: Id,
        index: u64,
    },
}
impl Queries<'_> {
    pub(super) fn enter_contents(&mut self, key: NodeKey, id: Id) -> Result<(), Error> {
        let query = Query::Contents(key, id);
        let place = self.place(key, Value::Reference(super::references::Pointer::new(id)))?;
        if let Some(value) = place.value {
            self.states
                .insert(self.state_key(query), State::Ready(value));
        } else if let Some(projection) = self
            .typing
            .facts
            .constant_references
            .projection(place)
            .map_err(|e| self.error(query, e))?
        {
            let (work, base) = match projection {
                ProjectionView::Field { base, index } => (
                    Work::Project {
                        key,
                        id,
                        base,
                        index,
                    },
                    base,
                ),
                ProjectionView::Sequence { base, from, length } => (
                    Work::Range {
                        key,
                        id,
                        base,
                        from,
                        length,
                    },
                    base,
                ),
            };
            self.work.push(Pending::Contents(work));
            self.work.push(Pending::Enter(Query::Contents(key, base)));
        } else if let Access::Static(definition) = place.access {
            let index = self
                .typing
                .facts
                .initializer_index(definition)
                .ok_or_else(|| self.error(query, "missing constant static initializer"))?;
            self.forward(query, Query::Static(index));
        } else {
            return Err(self.error(query, "constant allocation lacks initialized contents"));
        }
        Ok(())
    }
    pub(super) fn finish_contents(&mut self, work: Work) -> Result<(), Error> {
        let (key, id, value) = match work {
            Work::Project {
                key,
                id,
                base,
                index,
            } => {
                let value = self.value(Query::Contents(key, base))?;
                (
                    key,
                    id,
                    self.projected_contents(key, id, base, index, value)?,
                )
            }
            Work::Range {
                key,
                id,
                base,
                from,
                length,
            } => {
                self.budget()?;
                let value = self.value(Query::Contents(key, base))?;
                let facts = &mut *self.typing.facts;
                let value = facts
                    .constant_products
                    .range(value, from, length, &facts.texts)
                    .map_err(|e| self.error(Query::Contents(key, id), e))?;
                (key, id, value)
            }
        };
        let query = Query::Contents(key, id);
        self.typing
            .facts
            .constant_references
            .complete(id, value)
            .map_err(|e| self.error(query, e))?;
        self.states
            .insert(self.state_key(query), State::Ready(value));
        Ok(())
    }
    fn projected_contents(
        &mut self,
        key: NodeKey,
        id: Id,
        base: Id,
        index: u64,
        value: Value,
    ) -> Result<Value, Error> {
        if !matches!(value, Value::Image(_)) {
            return self.field_value(key, value, index);
        }
        self.budget()?;
        let query = Query::Contents(key, id);
        let (owner, primitive, size) = self.image_projection(key, id, base, index)?;
        let width = super::images::scalars::width(primitive).map_err(|e| self.error(query, e))?;
        for _ in 0..width * 2 {
            self.budget()?;
        }
        let image = super::values::Reader::new(self.typing.facts)
            .image(super::values::Constant(value))
            .map_err(|e| self.error(query, e))?;
        if image.ty != owner || image.bytes.len() != size {
            return Err(self.error(
                query,
                "constant image contents differ from their canonical owner",
            ));
        }
        super::images::scalars::decode(primitive, image.bytes, image.initialized)
            .map(Value::Scalar)
            .map_err(|e| self.error(query, e))
    }
    fn image_projection(
        &self,
        key: NodeKey,
        id: Id,
        base: Id,
        index: u64,
    ) -> Result<(crate::types::TypeId, crate::types::Primitive, usize), Error> {
        let query = Query::Contents(key, id);
        let base = self.place(key, Value::Reference(super::references::Pointer::new(base)))?;
        let child = self.place(key, Value::Reference(super::references::Pointer::new(id)))?;
        let projection = super::values::Reader::new(self.typing.facts)
            .projection_layout(base.ty, index)
            .map_err(|e| self.error(query, e))?;
        let super::values::ProjectionStorage::UnionScalar {
            field: crate::sema::members::Selection::Nominal(field),
        } = projection.storage
        else {
            return Err(self.error(
                query,
                "constant image contents lack a union field capability",
            ));
        };
        let (primitive, size) = {
            let family = self
                .typing
                .facts
                .constant_images
                .as_ref()
                .and_then(|images| images.families.get(&base.ty))
                .ok_or_else(|| self.error(query, "missing constant image family"))?;
            let ordinal = field
                .0
                .checked_sub(family.start)
                .filter(|&ordinal| u64::from(ordinal) == index)
                .ok_or_else(|| {
                    self.error(
                        query,
                        "constant image field has a different owner or ordinal",
                    )
                })?;
            let primitive = family
                .fields
                .get(ordinal as usize)
                .copied()
                .ok_or_else(|| self.error(query, "constant image field is out of bounds"))?;
            if projection.ty != child.ty
                || projection.ty != crate::types::TypeId::primitive(primitive)
                || projection.offset != 0
            {
                return Err(self.error(
                    query,
                    "constant image projection differs from its declared type",
                ));
            }
            (primitive, family.size)
        };
        Ok((base.ty, primitive, size))
    }
    pub(super) fn field_value(
        &self,
        key: NodeKey,
        value: Value,
        index: u64,
    ) -> Result<Value, Error> {
        let error = |e| self.error(Query::Place(key), e);
        match value {
            Value::Product(id) => self
                .typing
                .facts
                .constant_products
                .get(id, index, &self.typing.facts.texts)
                .map_err(error),
            Value::Bytes(id) => {
                let byte = self
                    .typing
                    .facts
                    .texts
                    .get(id)
                    .map_err(error)?
                    .get(usize::try_from(index).map_err(|_| error("constant index overflow"))?)
                    .copied()
                    .ok_or_else(|| error("constant index is out of bounds"))?;
                Ok(Value::Scalar(Scalar::Integer(u128::from(byte))))
            }
            _ => Err(error("constant projection requires a product")),
        }
    }
    /// Patterns inspect referents, whereas ordinary pointer values only retain addresses.
    pub(super) fn complete_pattern(&mut self, key: NodeKey, root: Value) -> Result<(), Error> {
        // Pure allocation graphs are already complete and acyclic by construction.
        if self
            .typing
            .facts
            .constant_references
            .static_order
            .is_empty()
            || !matches!(root, Value::Reference(_) | Value::Product(_))
            || self.pattern_values.contains(&root)
        {
            return Ok(());
        }
        let mut work = vec![(root, false)];
        let mut active = HashSet::new();
        while let Some((value, finish)) = work.pop() {
            self.budget()?;
            if !matches!(value, Value::Reference(_) | Value::Product(_))
                || self.pattern_values.contains(&value)
            {
                continue;
            }
            if finish {
                active.remove(&value);
                self.pattern_values.insert(value);
                continue;
            }
            if !active.insert(value) {
                return Err(self.error(
                    Query::Node(key),
                    "cyclic constant patterns require finite structural expansion",
                ));
            }
            work.push((value, true));
            match value {
                Value::Reference(pointer) => {
                    let query = Query::Contents(key, pointer.place);
                    self.work.push(Pending::Enter(query));
                    self.drain()?;
                    work.push((self.value(query)?, false));
                }
                Value::Product(id) => {
                    if self
                        .typing
                        .facts
                        .constant_products
                        .byte_range(id)
                        .map_err(|e| self.error(Query::Node(key), e))?
                        .is_some()
                    {
                        continue;
                    }
                    if let Some((value, length)) = self
                        .typing
                        .facts
                        .constant_products
                        .repeated(id)
                        .map_err(|e| self.error(Query::Node(key), e))?
                    {
                        if length != 0 {
                            work.push((value, false));
                        }
                    } else {
                        let length = self
                            .typing
                            .facts
                            .constant_products
                            .length(id)
                            .map_err(|e| self.error(Query::Node(key), e))?;
                        for index in (0..length).rev() {
                            self.budget()?;
                            work.push((self.field_value(key, value, index)?, false));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/resolved_contents.rs"]
mod tests;
