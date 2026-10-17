//! Reading a constant location is separate from forming its storage address.
use super::references::{Access, Id};
use super::{Error, NodeKey, Queries, Query, State, Value, Work as Pending};
use crate::sema::model::Scalar;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) enum Work {
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
        let place = self.place(key, Value::Reference(id))?;
        if let Some(value) = place.value {
            self.states
                .insert(self.state_key(query), State::Ready(value));
        } else if let Some((base, index)) = self
            .typing
            .facts
            .constant_references
            .projection(place)
            .map_err(|e| self.error(query, e))?
        {
            self.work.push(Pending::Contents(Work::Project {
                key,
                id,
                base,
                index,
            }));
            self.work.push(Pending::Enter(Query::Contents(key, base)));
        } else if let Access::Static(definition) = place.access {
            let index = self.constants[definition.0 as usize]
                .ok_or_else(|| self.error(query, "missing constant static initializer"))?;
            self.forward(query, Query::Static(index));
        } else {
            return Err(self.error(query, "constant allocation lacks initialized contents"));
        }
        Ok(())
    }
    pub(super) fn finish_contents(&mut self, work: Work) -> Result<(), Error> {
        let Work::Project {
            key,
            id,
            base,
            index,
        } = work;
        let query = Query::Contents(key, id);
        let value = self.field_value(key, self.value(Query::Contents(key, base))?, index)?;
        self.typing
            .facts
            .constant_references
            .complete(id, value)
            .map_err(|e| self.error(query, e))?;
        self.states
            .insert(self.state_key(query), State::Ready(value));
        Ok(())
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
                .get(id, index)
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
                Value::Reference(id) => {
                    let query = Query::Contents(key, id);
                    self.work.push(Pending::Enter(query));
                    self.drain()?;
                    work.push((self.value(query)?, false));
                }
                Value::Product(id) => {
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
