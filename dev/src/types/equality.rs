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
    /// Generated derive assumptions belong only to this declaration's parameters.
    scope: Option<(ItemId, Properties)>,
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
    /// Check field templates using nested nominal implementation contracts.
    /// Every selected derive must be validated separately before publication.
    #[must_use]
    pub fn for_derive(owner: ItemId, assumptions: Properties) -> Self {
        Self {
            scope: Some((owner, assumptions)),
            ..Self::default()
        }
    }
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
        self.demand_with_copy(types, definitions, selected, ty, |_| {
            Err("generic packed equality requires Copy obligations")
        })
    }
    /// Packed generic derives also impose Copy predicates on fields and type
    /// arguments. The Copy query must use the same immutable implementation
    /// selections throughout this equality session, including cached results.
    /// # Errors
    /// Rejects incomplete type graphs, failed Copy queries and exhausted work.
    pub fn demand_with_copy<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        ty: TypeId,
        mut copy: impl FnMut(TypeId) -> Result<bool, &'static str>,
    ) -> Result<Properties, &'static str> {
        if let Some(value) = self.values.get(&ty) {
            return Ok(*value);
        }
        let mut query = Query::default();
        query.node(ty);
        query.collect(
            types,
            definitions,
            selected,
            &self.values,
            &mut copy,
            self.scope,
        )?;
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
        copy: &mut impl FnMut(TypeId) -> Result<bool, &'static str>,
        scope: Option<(ItemId, Properties)>,
    ) -> Result<(), &'static str> {
        let mut index = 0;
        while index < self.nodes.len() {
            self.charge()?;
            let ty = self.nodes[index].ty;
            let kind = types.kind(ty).ok_or("missing equality type")?;
            let packed = if let Kind::Adt(item, args) = kind {
                if scope.is_some() {
                    definitions
                        .get(item)
                        .ok_or("missing nominal derive contract")?
                        .repr
                        .packed
                        .is_some()
                } else if types.region_args(args).is_none() {
                    definitions
                        .shape(types, ty)?
                        .definition
                        .repr
                        .packed
                        .is_some()
                } else {
                    false
                }
            } else {
                false
            };
            let mut initial = match kind {
                Kind::Projection(_) => {
                    return Err("associated type equality checking requires normalization");
                }
                Kind::Parameter(id) => {
                    let (owner, abilities) = scope
                        .ok_or("generic equality checking requires a parameter environment")?;
                    if types.parameter(id).is_none_or(|p| p.owner != owner) {
                        return Err("equality parameter is outside the current environment");
                    }
                    abilities
                }
                Kind::Adt(item, _) => selected.get(&item).copied().unwrap_or_default(),
                Kind::Primitive(Primitive::F32 | Primitive::F64) => Properties {
                    partial_eq: true,
                    eq: false,
                },
                Kind::FunctionItem { .. } | Kind::Opaque(..) | Kind::Dynamic { .. } => {
                    Properties::default()
                }
                _ => Properties::BOTH,
            };
            if packed
                && initial != Properties::default()
                && !self.packed_copy(types, definitions, ty, copy, scope.is_none())?
            {
                initial = Properties::default();
            }
            self.nodes[index].value = initial;
            if initial != Properties::default() {
                children(types, definitions, ty, scope.is_none(), |child| {
                    self.dependency(index, child, known)
                })?;
            }
            index += 1;
        }
        Ok(())
    }
    fn packed_copy(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        copy: &mut impl FnMut(TypeId) -> Result<bool, &'static str>,
        fields: bool,
    ) -> Result<bool, &'static str> {
        let Some(Kind::Adt(_, args)) = types.kind(ty) else {
            return Err("packed equality requires a nominal type");
        };
        for argument in types.arguments(args).ok_or("invalid equality arguments")? {
            self.charge()?;
            if let super::arguments::Argument::Type(ty) = argument
                && !copy(ty)?
            {
                return Ok(false);
            }
        }
        if fields {
            for variant in definitions.shape(types, ty)?.variants() {
                for &field in types
                    .list(variant.fields)
                    .ok_or("invalid equality fields")?
                {
                    self.charge()?;
                    if !copy(field)? {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
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

// Declaration proofs use each nested nominal's selected conditional contract.
// Concrete queries still verify instantiated fields and reference cycles.
fn children(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
    fields: bool,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match types.kind(ty).ok_or("missing equality type")? {
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } | Kind::Slice(element) => {
            visit(element)?;
        }
        Kind::Reference { referent, .. } => visit(referent)?,
        Kind::Tuple(fields) => {
            for &ty in types.list(fields).ok_or("missing equality tuple")? {
                visit(ty)?;
            }
        }
        Kind::Adt(_, arguments) => {
            if fields {
                for variant in definitions.shape(types, ty)?.variants() {
                    for &ty in types
                        .list(variant.fields)
                        .ok_or("missing equality fields")?
                    {
                        visit(ty)?;
                    }
                }
            }
            // Derives constrain type parameters even behind raw pointers.
            for argument in types
                .arguments(arguments)
                .ok_or("invalid equality arguments")?
            {
                if let super::arguments::Argument::Type(ty) = argument {
                    visit(ty)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
