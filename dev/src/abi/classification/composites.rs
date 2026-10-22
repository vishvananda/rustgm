//! Composite validation and placed classes belong to one immutable prepared layout graph.
mod classes;
mod shapes;
use super::{Class, Classes, Definitions, Layouts, Store, TypeId};
use crate::support::hash::HashMap;
#[derive(Clone, Copy)]
struct Field {
    ty: TypeId,
    offset: u64,
}
enum Kind {
    Scalar(Class),
    Array {
        element: TypeId,
        stride: u64,
        length: u64,
    },
    Struct(Box<[Field]>),
}
struct Node {
    bytes: u64,
    kind: Kind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ty: TypeId,
    phase: u8,
}
#[derive(Default)]
pub(super) struct Cache {
    nodes: HashMap<TypeId, Node>,
    classes: HashMap<Key, [Class; 2]>,
}
struct Query<'a> {
    cache: &'a mut Cache,
    types: &'a Store,
    definitions: &'a Definitions,
    layouts: &'a Layouts,
    steps: &'a mut usize,
    retained: &'a mut usize,
}
impl Cache {
    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize) {
        (self.nodes.len(), self.classes.len())
    }

    pub(super) fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &Layouts,
        ty: TypeId,
        steps: &mut usize,
        retained: &mut usize,
    ) -> Result<Classes, &'static str> {
        let mut query = Query {
            cache: self,
            types,
            definitions,
            layouts,
            steps,
            retained,
        };
        query.validate(ty)?;
        let bytes = query
            .cache
            .nodes
            .get(&ty)
            .ok_or("missing validated C object")?
            .bytes;
        if bytes > 16 {
            return Ok(Classes::Memory);
        }
        let classes = query.placed(Key { ty, phase: 0 })?;
        Ok(Classes::Eightbytes {
            first: classes[0],
            second: (bytes > 8).then_some(classes[1]),
        })
    }
}
impl Query<'_> {
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        super::account_work(self.steps, work)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        super::storage(self.retained, bytes)
    }
}
