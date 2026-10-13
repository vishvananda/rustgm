//! Closed-profile equality obligations with a greatest fixed point for reference cycles.
use super::{ItemId, Kind, Primitive, Store, TypeId, aggregate::Definitions};
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Properties {
    pub partial_eq: bool,
    pub eq: bool,
}
impl Properties {
    const BOTH: Self = Self {
        partial_eq: true,
        eq: true,
    };
    fn intersect(self, other: Self) -> Self {
        Self {
            partial_eq: self.partial_eq && other.partial_eq,
            eq: self.eq && other.eq,
        }
    }
}
#[derive(Default)]
pub struct Equality {
    values: HashMap<TypeId, Properties>,
    pub work: u64,
}
struct Node {
    ty: TypeId,
    value: Properties,
    parents: Option<usize>,
}
struct Edge {
    parent: usize,
    next: Option<usize>,
}
#[derive(Default)]
struct Query {
    nodes: Vec<Node>,
    indices: HashMap<TypeId, usize>,
    edges: Vec<Edge>,
    work: u64,
}
impl Equality {
    /// Canonical types and selected implementations must remain immutable.
    ///
    /// # Errors
    /// Rejects incomplete type graphs and queries exceeding the work limit.
    pub fn demand<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        ty: TypeId,
    ) -> Result<Properties, &'static str> {
        if let Some(value) = self.values.get(&ty) {
            return Ok(*value);
        }
        let mut query = Query::default();
        query.node(ty);
        query.collect(types, definitions, selected, &self.values)?;
        query.propagate()?;
        self.work = self.work.saturating_add(query.work);
        for node in query.nodes {
            self.values.insert(node.ty, node.value);
        }
        self.values
            .get(&ty)
            .copied()
            .ok_or("missing equality obligation result")
    }
}
impl Query {
    fn charge(&mut self) -> Result<(), &'static str> {
        self.work += 1;
        if self.work > 1_000_000 {
            return Err("equality obligation work budget exceeded");
        }
        Ok(())
    }
    fn node(&mut self, ty: TypeId) -> usize {
        if let Some(&id) = self.indices.get(&ty) {
            return id;
        }
        let id = self.nodes.len();
        self.indices.insert(ty, id);
        self.nodes.push(Node {
            ty,
            value: Properties::BOTH,
            parents: None,
        });
        id
    }
    fn dependency(
        &mut self,
        parent: usize,
        child: TypeId,
        known: &HashMap<TypeId, Properties>,
    ) -> Result<(), &'static str> {
        self.charge()?;
        if let Some(&value) = known.get(&child) {
            self.nodes[parent].value = self.nodes[parent].value.intersect(value);
        } else {
            let child = self.node(child);
            let index = self.edges.len();
            self.edges.push(Edge {
                parent,
                next: self.nodes[child].parents,
            });
            self.nodes[child].parents = Some(index);
        }
        Ok(())
    }
    fn collect<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        known: &HashMap<TypeId, Properties>,
    ) -> Result<(), &'static str> {
        let mut index = 0;
        while index < self.nodes.len() {
            self.charge()?;
            let kind = types
                .kind(self.nodes[index].ty)
                .ok_or("missing equality type")?;
            let initial = match kind {
                Kind::Adt(item) => selected.get(&item).copied().unwrap_or_default(),
                Kind::Primitive(Primitive::F32 | Primitive::F64) => Properties {
                    partial_eq: true,
                    eq: false,
                },
                Kind::FunctionItem { .. } => Properties::default(),
                _ => Properties::BOTH,
            };
            self.nodes[index].value = initial;
            if initial != Properties::default() {
                let mut visit = |child| self.dependency(index, child, known);
                match kind {
                    Kind::Array { element, .. } | Kind::Slice(element) => visit(element)?,
                    Kind::Reference { referent, .. } => visit(referent)?,
                    Kind::Tuple(fields) => {
                        for &ty in types.list(fields).ok_or("missing equality tuple")? {
                            visit(ty)?;
                        }
                    }
                    Kind::Adt(item) => {
                        let definition =
                            definitions.get(item).ok_or("missing equality aggregate")?;
                        for variant in definitions.variants(definition) {
                            for &ty in types
                                .list(variant.fields)
                                .ok_or("missing equality fields")?
                            {
                                visit(ty)?;
                            }
                        }
                    }
                    // Raw/function pointer equality does not inspect the referent.
                    _ => {}
                }
            }
            index += 1;
        }
        Ok(())
    }
    fn propagate(&mut self) -> Result<(), &'static str> {
        let mut pending: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| (node.value != Properties::BOTH).then_some(id))
            .collect();
        while let Some(child) = pending.pop() {
            self.charge()?;
            let mut edge = self.nodes[child].parents;
            while let Some(index) = edge {
                self.charge()?;
                let entry = &self.edges[index];
                let parent = entry.parent;
                edge = entry.next;
                let value = self.nodes[parent].value.intersect(self.nodes[child].value);
                if value != self.nodes[parent].value {
                    self.nodes[parent].value = value;
                    pending.push(parent);
                }
            }
        }
        Ok(())
    }
}
