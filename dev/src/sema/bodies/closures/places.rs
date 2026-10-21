//! Cache typed source places once; every projection shares its interned prefix.
use super::{Checker, Constructor, Error, Kind, NodeId, TermId};
use crate::{
    resolve::{
        model::{DefId, Target},
        unit::NodeKey,
    },
    sema::{
        closures::captures::{Graph, PathId, Step},
        model::Operation,
    },
    syntax::ast::Unary,
    types::aggregate,
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(super) struct Place {
    pub path: PathId,
    pub shared: Option<PathId>,
    pub before_dereference: Option<PathId>,
}
#[derive(Default)]
pub(super) struct Places {
    pub graph: Graph,
    pub types: Vec<TermId>,
    pub accesses: HashMap<NodeKey, PathId>,
    roots: HashMap<DefId, TermId>,
    cache: HashMap<NodeId, Option<Place>>,
    pending: Vec<NodeId>,
    work: usize,
}
impl Places {
    pub fn new(bindings: &[(DefId, TermId)]) -> Self {
        Self {
            roots: bindings.iter().copied().collect(),
            ..Self::default()
        }
    }
    fn retain(&mut self, path: PathId, ty: TermId) {
        if path.index() == self.types.len() {
            self.types.push(ty);
        }
    }
    pub fn get(&mut self, checker: &mut Checker<'_>, node: NodeId) -> Result<Option<Place>, Error> {
        let mut cursor = node;
        loop {
            self.work += 1;
            if self.work > 1_000_000 {
                return Err(checker.error(node, "closure place work limit exceeded"));
            }
            if self.cache.contains_key(&cursor) {
                break;
            }
            self.pending.push(cursor);
            cursor = match checker.kind(cursor) {
                Kind::Group(value)
                | Kind::Attributed { value, .. }
                | Kind::Field { value, .. }
                | Kind::Unary {
                    op: Unary::Dereference,
                    operand: value,
                } => value,
                _ => break,
            };
        }
        while let Some(current) = self.pending.pop() {
            let place = self.build(checker, current)?;
            if let Some(place) = place {
                self.accesses.insert(checker.key(current), place.path);
            }
            self.cache.insert(current, place);
        }
        Ok(self.cache.get(&node).copied().flatten())
    }
    fn build(&mut self, checker: &mut Checker<'_>, node: NodeId) -> Result<Option<Place>, Error> {
        match checker.kind(node) {
            Kind::Name(_) | Kind::Path { .. } => {
                let Some(Target::Definition(definition)) = checker
                    .facts
                    .node(checker.key(node))
                    .and_then(|fact| fact.reference)
                else {
                    return Ok(None);
                };
                let Some(&ty) = self.roots.get(&definition) else {
                    return Ok(None);
                };
                let path = self
                    .graph
                    .root(definition)
                    .map_err(|e| checker.error(node, e))?;
                self.retain(path, ty);
                Ok(Some(Place {
                    path,
                    shared: None,
                    before_dereference: None,
                }))
            }
            Kind::Group(value) | Kind::Attributed { value, .. } => Ok(self.cache[&value]),
            Kind::Unary {
                op: Unary::Dereference,
                operand,
            } => {
                let Some(place) = self.cache[&operand] else {
                    return Ok(None);
                };
                self.dereference(checker, node, place).map(Some)
            }
            Kind::Field { value, .. } => {
                let Some(mut place) = self.cache[&value] else {
                    return Ok(None);
                };
                let count = checker
                    .facts
                    .autoderefs
                    .get(&checker.key(node))
                    .copied()
                    .unwrap_or(0);
                for _ in 0..count {
                    place = self.dereference(checker, node, place)?;
                }
                let ty = self.types[place.path.index()];
                if let Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) =
                    checker.head(node, ty)?
                    && checker
                        .facts
                        .aggregates
                        .get(item)
                        .is_some_and(|definition| {
                            definition.kind == aggregate::Kind::Union
                                || definition.repr.packed.is_some()
                        })
                {
                    return Err(checker.error(
                        node,
                        "packed and union capture precision is not implemented",
                    ));
                }
                let Some(Operation::Field { field }) = checker
                    .facts
                    .node(checker.key(node))
                    .and_then(|fact| fact.operation)
                else {
                    return Err(checker.error(node, "missing typed closure field selection"));
                };
                let path = self
                    .graph
                    .project(place.path, Step::Field(field))
                    .map_err(|e| checker.error(node, e))?;
                self.retain(path, checker.term(node)?);
                place.path = path;
                Ok(Some(place))
            }
            _ => Ok(None),
        }
    }
    pub(super) fn dereference(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        mut place: Place,
    ) -> Result<Place, Error> {
        let term = self.types[place.path.index()];
        let Some(Constructor::Reference { mutable, .. }) = checker.head(node, term)? else {
            return Err(checker.error(node, "raw pointer capture paths are not implemented"));
        };
        let ty = checker.child(node, term, 0)?;
        let path = self
            .graph
            .project(place.path, Step::Dereference { mutable })
            .map_err(|e| checker.error(node, e))?;
        self.retain(path, ty);
        place.before_dereference = place.before_dereference.or(Some(place.path));
        place.path = path;
        if !mutable {
            place.shared = Some(path);
        }
        Ok(place)
    }
    pub const fn work(&self) -> usize {
        self.work
    }
}
