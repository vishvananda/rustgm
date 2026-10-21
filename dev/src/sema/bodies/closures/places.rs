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
    precision: Option<PathId>,
    pub raw: bool,
}
impl Place {
    pub(super) fn capture_path(self, moving: bool) -> PathId {
        // All candidates belong to this one ordered ancestor chain. Choosing
        // the smallest interned index is constant work, without ancestry scans.
        [
            self.precision,
            if moving {
                self.before_dereference
            } else {
                self.shared
            },
        ]
        .into_iter()
        .flatten()
        .min_by_key(|path| path.index())
        .unwrap_or(self.path)
    }
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
    pub(super) fn import_root(
        &mut self,
        checker: &Checker<'_>,
        node: NodeId,
        definition: DefId,
    ) -> Result<Option<Place>, Error> {
        self.work += 1;
        if self.work > 1_000_000 {
            return Err(checker.error(node, "nested capture place work limit exceeded"));
        }
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
            precision: None,
            raw: false,
        }))
    }
    pub(super) fn import_step(
        &mut self,
        checker: &mut Checker<'_>,
        node: NodeId,
        mut place: Place,
        step: Step,
    ) -> Result<Place, Error> {
        self.work += 1;
        if self.work > 1_000_000 {
            return Err(checker.error(node, "nested capture place work limit exceeded"));
        }
        if matches!(step, Step::Dereference { .. } | Step::RawDereference { .. }) {
            let term = self.types[place.path.index()];
            let valid = match (step, checker.head(node, term)?) {
                (
                    Step::Dereference { mutable },
                    Some(Constructor::Reference {
                        mutable: actual, ..
                    }),
                )
                | (
                    Step::RawDereference { mutable },
                    Some(Constructor::Pointer { mutable: actual }),
                ) => mutable == actual,
                _ => false,
            };
            if !valid {
                return Err(checker.error(node, "nested capture dereference capability changed"));
            }
            return self.dereference(checker, node, place);
        }
        let (Step::Field(selection) | Step::UnionField(selection)) = step else {
            return Err(checker.error(node, "missing nested capture field selection"));
        };
        let term = self.types[place.path.index()];
        let ty = match (selection, checker.head(node, term)?) {
            (crate::sema::members::Selection::Tuple(index), Some(Constructor::Tuple)) => {
                checker.child(node, term, index as usize)?
            }
            (
                crate::sema::members::Selection::Nominal(field),
                Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)),
            ) => {
                let variant = checker
                    .facts
                    .members
                    .variant(field)
                    .and_then(|variant| checker.facts.members.variants.get(variant.0 as usize));
                if !variant.is_some_and(|variant| variant.owner == owner) {
                    return Err(checker.error(node, "nested capture field owner changed"));
                }
                checker.instance_field(node, term, field)?
            }
            _ => {
                return Err(checker.error(
                    node,
                    "nested capture projection has a different parent type",
                ));
            }
        };
        let expected = self.field_step(checker, node, &mut place, selection)?;
        if expected != step {
            return Err(checker.error(node, "nested capture field capability changed"));
        }
        let path = self
            .graph
            .project(place.path, step)
            .map_err(|e| checker.error(node, e))?;
        self.retain(path, ty);
        place.path = path;
        Ok(place)
    }
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
                    precision: None,
                    raw: false,
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
                let Some(Operation::Field { field }) = checker
                    .facts
                    .node(checker.key(node))
                    .and_then(|fact| fact.operation)
                else {
                    return Err(checker.error(node, "missing typed closure field selection"));
                };
                let step = self.field_step(checker, node, &mut place, field)?;
                let path = self
                    .graph
                    .project(place.path, step)
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
        let (step, mutable, raw) = match checker.head(node, term)? {
            Some(Constructor::Reference { mutable, .. }) => {
                (Step::Dereference { mutable }, mutable, false)
            }
            Some(Constructor::Pointer { mutable }) => {
                (Step::RawDereference { mutable }, mutable, true)
            }
            _ => {
                return Err(checker.error(
                    node,
                    "capture dereference requires a reference or raw pointer",
                ));
            }
        };
        if raw {
            place.precision = place.precision.or(Some(place.path));
            place.raw = true;
        }
        let ty = checker.child(node, term, 0)?;
        let path = self
            .graph
            .project(place.path, step)
            .map_err(|e| checker.error(node, e))?;
        self.retain(path, ty);
        place.before_dereference = place.before_dereference.or(Some(place.path));
        place.path = path;
        if !raw && !mutable && place.precision.is_none() {
            place.shared = Some(path);
        }
        Ok(place)
    }
    fn field_step(
        &self,
        checker: &mut Checker<'_>,
        node: NodeId,
        place: &mut Place,
        field: crate::sema::members::Selection,
    ) -> Result<Step, Error> {
        let term = self.types[place.path.index()];
        let Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)) =
            checker.head(node, term)?
        else {
            return Ok(Step::Field(field));
        };
        let definition = checker
            .facts
            .aggregates
            .get(owner)
            .ok_or_else(|| checker.error(node, "missing capture aggregate shape"))?;
        let union = definition.kind == aggregate::Kind::Union;
        if union || definition.repr.packed.is_some() {
            place.precision = place.precision.or(Some(place.path));
        }
        Ok(if union {
            Step::UnionField(field)
        } else {
            Step::Field(field)
        })
    }
    pub const fn work(&self) -> usize {
        self.work
    }
}
