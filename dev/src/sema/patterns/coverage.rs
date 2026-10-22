//! Coverage sets over sparse typed places, with shared interval decision subgraphs.
mod decision;
mod inhabited;
mod slices;
#[cfg(test)]
mod tests;
mod text;
mod witness;

use super::{Arm, Kind, PatternId, Projection, scalar::Domain};
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, package::ModuleId},
    sema::{members::VariantId, model::Declarations},
    types::{Kind as Type, TypeId},
};
use decision::{Graph, Id, Operation, Variable};
pub use inhabited::Cache;
use inhabited::Inhabited;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PlaceId(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Step {
    Field(Projection),
    Dereference,
}
#[derive(Clone, Copy)]
struct Place {
    ty: TypeId,
    valid: bool,
    parent: Option<(PlaceId, Step)>,
    variable: Option<Variable>,
}
enum Values {
    Scalar(Domain),
    Tags {
        known: Arc<[VariantId]>,
        external_open: bool,
    },
    Length,
    Text,
}
/// Published coverage results of one match.
#[derive(Clone)]
pub struct Outcome {
    pub useful: Vec<bool>,
    pub missing: Option<String>,
    pub work: u64,
    pub uninhabited: bool,
}
/// Everything coverage reads from a match: input, module, validity, and
/// each arm's guard and pattern structure, including pattern identity offsets
/// (sharing) and node types. Bindings contribute only their subpatterns.
/// Each record starts with a tag word, so the encoding is a prefix code.
/// Returns false for patterns that coverage would reject.
pub fn match_key(
    facts: &Declarations,
    arms: &[Arm],
    (ty, valid, module): (TypeId, bool, ModuleId),
    key: &mut Vec<u64>,
    work: &mut Vec<PatternId>,
) -> bool {
    key.clear();
    work.clear();
    key.extend([
        u64::from(ty.0) << 1 | u64::from(valid),
        u64::from(module.0) << 32 | arms.len() as u64,
    ]);
    let Some(base) = arms.iter().filter_map(|arm| arm.pattern).map(|p| p.0).min() else {
        return false;
    };
    for arm in arms {
        let Some(root) = arm.pattern else {
            return false;
        };
        key.push(u64::from(arm.guard.is_some()));
        work.push(root);
        while let Some(id) = work.pop() {
            let pattern = &facts.patterns.nodes[id.0 as usize];
            let Some(node_ty) = pattern.ty else {
                return false;
            };
            key.push(u64::from(id.0.wrapping_sub(base)) << 32 | u64::from(node_ty.0));
            match pattern.kind {
                Kind::Pending => return false,
                Kind::Wildcard => key.push(0),
                Kind::Range(range) => {
                    let interval = facts.patterns.ranges[range.0 as usize];
                    #[expect(clippy::cast_possible_truncation, reason = "split rank halves")]
                    key.extend([
                        1,
                        interval.lower as u64,
                        (interval.lower >> 64) as u64,
                        interval.upper as u64,
                        (interval.upper >> 64) as u64,
                    ]);
                }
                Kind::Text(text) => key.push(2 | u64::from(text.0) << 8),
                Kind::Binding { subpattern, .. } => {
                    key.push(3 | u64::from(subpattern.is_some()) << 8);
                    work.extend(subpattern);
                }
                Kind::Dereference { pattern, .. } => {
                    key.push(4);
                    work.push(pattern);
                }
                Kind::Product { variant, fields } => {
                    key.push(5 | u64::from(variant.map_or(u32::MAX, |v| v.0)) << 8);
                    fields_key(facts, fields, key, work);
                }
                Kind::Slice {
                    minimum,
                    exact,
                    fields,
                } => {
                    key.extend([6 | u64::from(exact) << 8, minimum]);
                    fields_key(facts, fields, key, work);
                }
                Kind::Or(alternatives) => {
                    let alternatives = facts.patterns.alternatives(alternatives);
                    key.push(7 | (alternatives.len() as u64) << 8);
                    work.extend(alternatives.iter().rev());
                }
            }
        }
    }
    true
}
fn fields_key(
    facts: &Declarations,
    fields: super::Fields,
    key: &mut Vec<u64>,
    work: &mut Vec<PatternId>,
) {
    let fields = facts.patterns.fields(fields);
    key.push(fields.len() as u64);
    for field in fields {
        match field.selection {
            Projection::Field(crate::sema::members::Selection::Tuple(index)) => {
                key.push(u64::from(index) << 8);
            }
            Projection::Field(crate::sema::members::Selection::Nominal(id)) => {
                key.push(1 | u64::from(id.0) << 8);
            }
            Projection::Index(index) => key.extend([2, index]),
            Projection::FromEnd { offset, minimum } => key.extend([3, offset, minimum]),
            Projection::Subarray { start, end } => key.extend([4, start, end]),
            Projection::Subslice { prefix, suffix } => key.extend([5, prefix, suffix]),
        }
    }
    work.extend(fields.iter().rev().map(|field| field.pattern));
}
pub enum Roots<'a> {
    Pattern(PatternId),
    Arms(&'a [Arm]),
}
struct Input {
    place: PlaceId,
    values: Values,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Query {
    pattern: PatternId,
    place: PlaceId,
}
enum Work {
    Enter(Query),
    /// Children occupy `children[start..]` of the shared child stack; every
    /// descendant's entries have been released when this finish runs.
    Finish {
        query: Query,
        operation: Operation,
        initial: Id,
        start: usize,
    },
    Slices {
        query: Query,
        branches: Vec<slices::Branch>,
    },
}
/// Storage retained between coverage queries of one checker. Every query
/// starts from cleared tables; only capacity crosses query boundaries.
#[derive(Default)]
struct Scratch {
    graph: Graph,
    places: Vec<Place>,
    projections: HashMap<(PlaceId, Step), PlaceId>,
    inputs: Vec<Input>,
    patterns: HashMap<Query, Id>,
    work: Vec<Work>,
    children: Vec<Query>,
}
pub struct Coverage<'a> {
    facts: &'a Declarations,
    report: &'a Report,
    inhabited: Inhabited<'a>,
    graph: Graph,
    places: Vec<Place>,
    projections: HashMap<(PlaceId, Step), PlaceId>,
    inputs: Vec<Input>,
    patterns: HashMap<Query, Id>,
    work: Vec<Work>,
    children: Vec<Query>,
    covered: Id,
    universe: Id,
    roots: Roots<'a>,
    slice_aliases: Option<HashMap<TypeId, Vec<u64>>>,
    texts: Option<text::Domain>,
}
impl Drop for Coverage<'_> {
    fn drop(&mut self) {
        self.inhabited.cache.scratch = Scratch {
            graph: std::mem::take(&mut self.graph),
            places: std::mem::take(&mut self.places),
            projections: std::mem::take(&mut self.projections),
            inputs: std::mem::take(&mut self.inputs),
            patterns: std::mem::take(&mut self.patterns),
            work: std::mem::take(&mut self.work),
            children: std::mem::take(&mut self.children),
        };
    }
}
impl<'a> Coverage<'a> {
    pub fn new(
        facts: &'a Declarations,
        report: &'a Report,
        module: ModuleId,
        ty: TypeId,
        root_valid: bool,
        roots: Roots<'a>,
        cache: &'a mut Cache,
    ) -> Result<Self, &'static str> {
        let Scratch {
            mut graph,
            mut places,
            mut projections,
            mut inputs,
            mut patterns,
            mut work,
            mut children,
        } = std::mem::take(&mut cache.scratch);
        graph.reset(super::scalar::Coverage::LIMIT);
        places.clear();
        projections.clear();
        inputs.clear();
        patterns.clear();
        work.clear();
        children.clear();
        let mut inhabited = Inhabited::new(facts, report, module, cache);
        // Dereferenced places need not contain valid field values. Only an
        // explicitly empty root domain permits an empty match in that case.
        let possible = if root_valid {
            inhabited.get(ty, &mut graph)?
        } else {
            match facts.types.kind(ty) {
                Some(Type::Never) => false,
                Some(Type::Adt(item, _)) => {
                    facts.members.external_open(item) || !facts.members.variants(item).is_empty()
                }
                _ => true,
            }
        };
        let universe = if possible { Id::ALL } else { Id::EMPTY };
        places.push(Place {
            ty,
            valid: root_valid,
            parent: None,
            variable: None,
        });
        Ok(Self {
            facts,
            report,
            inhabited,
            graph,
            places,
            projections,
            inputs,
            patterns,
            work,
            children,
            covered: Id::EMPTY,
            universe,
            roots,
            slice_aliases: None,
            texts: None,
        })
    }
    pub fn work(&self) -> u64 {
        self.graph.work()
    }
    pub fn uninhabited(&self) -> bool {
        self.universe == Id::EMPTY
    }
    pub fn arm(&mut self, pattern: PatternId, guarded: bool) -> Result<bool, &'static str> {
        let set = self.pattern(pattern)?;
        let remainder = self.graph.apply(Operation::Difference, set, self.covered)?;
        if !guarded {
            self.covered = self.graph.apply(Operation::Union, self.covered, set)?;
        }
        Ok(remainder != Id::EMPTY)
    }
    pub fn missing(&mut self) -> Result<Option<String>, &'static str> {
        let set = self
            .graph
            .apply(Operation::Difference, self.universe, self.covered)?;
        if set == Id::EMPTY {
            return Ok(None);
        }
        self.witness(set).map(Some)
    }
    fn project(&mut self, base: PlaceId, step: Step, ty: TypeId) -> Result<PlaceId, &'static str> {
        self.graph.charge()?;
        if let Some(&place) = self.projections.get(&(base, step)) {
            if self.places[place.0 as usize].ty != ty {
                return Err("inconsistent coverage projection type");
            }
            return Ok(place);
        }
        let union = if let Some(Type::Adt(item, _)) =
            self.facts.types.kind(self.places[base.0 as usize].ty)
        {
            self.facts
                .aggregates
                .get(item)
                .is_some_and(|d| d.kind == crate::types::aggregate::Kind::Union)
        } else {
            false
        };
        let valid = self.places[base.0 as usize].valid && step != Step::Dereference && !union;
        let id = PlaceId(u32::try_from(self.places.len()).map_err(|_| "too many coverage places")?);
        self.places.push(Place {
            ty,
            valid,
            parent: Some((base, step)),
            variable: None,
        });
        self.projections.insert((base, step), id);
        Ok(id)
    }
    fn variable(&mut self, place: PlaceId) -> Result<Variable, &'static str> {
        let info = self.places[place.0 as usize];
        if let Some(variable) = info.variable {
            return Ok(variable);
        }
        let (maximum, values) = match self.facts.types.kind(info.ty) {
            Some(Type::Primitive(crate::types::Primitive::Str)) => {
                self.prepare_texts()?;
                let length = self
                    .texts
                    .as_ref()
                    .ok_or("missing text coverage domain")?
                    .values
                    .len();
                (length as u128, Values::Text)
            }
            Some(Type::Primitive(primitive)) => {
                let domain =
                    Domain::new(primitive).ok_or("coverage requires a supported scalar domain")?;
                (domain.full().upper, Values::Scalar(domain))
            }
            Some(Type::Adt(item, _)) => {
                let tags = self
                    .inhabited
                    .tags(info.ty, item, info.valid, &mut self.graph)?;
                let external_open = self.facts.members.external_open(item);
                let maximum = tags
                    .len()
                    .checked_add(usize::from(external_open))
                    .and_then(|count| count.checked_sub(1))
                    .ok_or("empty coverage constructor domain")?;
                (
                    maximum as u128,
                    Values::Tags {
                        known: tags,
                        external_open,
                    },
                )
            }
            Some(Type::Slice(_)) => (u128::from(u64::MAX), Values::Length),
            _ => return Err("coverage place has no decision domain"),
        };
        let id = self.graph.variable(maximum)?;
        self.inputs.push(Input { place, values });
        self.places[place.0 as usize].variable = Some(id);
        Ok(id)
    }
    fn child(
        &mut self,
        base: PlaceId,
        pattern: PatternId,
        step: Step,
    ) -> Result<Query, &'static str> {
        let ty = self.facts.patterns.nodes[pattern.0 as usize]
            .ty
            .ok_or("unsolved coverage child")?;
        Ok(Query {
            pattern,
            place: self.project(base, step, ty)?,
        })
    }
    fn product(
        &mut self,
        query: Query,
        variant: Option<VariantId>,
        fields: super::Fields,
    ) -> Result<Work, &'static str> {
        let initial = if let Some(variant) = variant {
            let variable = self.variable(query.place)?;
            let Values::Tags { known: tags, .. } = &self.inputs[variable.0 as usize].values else {
                return Err("missing constructor coverage domain");
            };
            let Ok(rank) = tags.binary_search_by_key(&variant.0, |v| v.0) else {
                return Ok(Work::Finish {
                    query,
                    operation: Operation::Intersection,
                    initial: Id::EMPTY,
                    start: self.children.len(),
                });
            };
            self.graph.interval(variable, rank as u128, rank as u128)?
        } else {
            Id::ALL
        };
        let start = self.children.len();
        for field in self.facts.patterns.fields(fields) {
            let child = self.child(query.place, field.pattern, Step::Field(field.selection))?;
            self.children.push(child);
        }
        Ok(Work::Finish {
            query,
            operation: Operation::Intersection,
            initial,
            start,
        })
    }
    fn enter(&mut self, query: Query) -> Result<Work, &'static str> {
        let place = self.places[query.place.0 as usize];
        let start = self.children.len();
        if place.valid && !self.inhabited.get(place.ty, &mut self.graph)? {
            return Ok(Work::Finish {
                query,
                operation: Operation::Intersection,
                initial: Id::EMPTY,
                start,
            });
        }
        let mut initial = Id::ALL;
        let mut operation = Operation::Intersection;
        match self.facts.patterns.nodes[query.pattern.0 as usize].kind {
            Kind::Wildcard
            | Kind::Binding {
                subpattern: None, ..
            } => {}
            Kind::Binding {
                subpattern: Some(pattern),
                ..
            } => self.children.push(Query { pattern, ..query }),
            Kind::Text(text) => {
                let variable = self.variable(query.place)?;
                let rank = self
                    .texts
                    .as_ref()
                    .and_then(|d| d.ranks.get(&text))
                    .copied()
                    .ok_or("missing text pattern rank")?;
                initial = self.graph.interval(variable, rank, rank)?;
            }
            Kind::Range(range) => {
                let variable = self.variable(query.place)?;
                let interval = self.facts.patterns.ranges[range.0 as usize];
                initial = self
                    .graph
                    .interval(variable, interval.lower, interval.upper)?;
            }
            Kind::Dereference { pattern, .. } => {
                let child = self.child(query.place, pattern, Step::Dereference)?;
                self.children.push(child);
            }
            Kind::Product { variant, fields } => return self.product(query, variant, fields),
            Kind::Slice {
                minimum,
                exact,
                fields,
            } => return self.slice(query, minimum, exact, fields),
            Kind::Or(alternatives) => {
                initial = Id::EMPTY;
                operation = Operation::Union;
                for &pattern in self.facts.patterns.alternatives(alternatives) {
                    self.children.push(Query { pattern, ..query });
                }
            }
            Kind::Pending => return Err("incomplete coverage pattern"),
        }
        Ok(Work::Finish {
            query,
            operation,
            initial,
            start,
        })
    }
    fn pattern(&mut self, pattern: PatternId) -> Result<Id, &'static str> {
        let root = Query {
            pattern,
            place: PlaceId(0),
        };
        let mut work = std::mem::take(&mut self.work);
        work.clear();
        self.children.clear();
        work.push(Work::Enter(root));
        let result = self.pattern_work(&mut work);
        work.clear();
        self.work = work;
        result?;
        self.patterns
            .get(&root)
            .copied()
            .ok_or("missing selected coverage result")
    }
    fn pattern_work(&mut self, work: &mut Vec<Work>) -> Result<(), &'static str> {
        while let Some(next) = work.pop() {
            self.graph.charge()?;
            match next {
                Work::Enter(query) => {
                    if self.patterns.contains_key(&query) {
                        continue;
                    }
                    let next = self.enter(query)?;
                    match &next {
                        &Work::Finish { start, .. } => {
                            work.push(next);
                            work.extend(
                                self.children[start..].iter().rev().map(|q| Work::Enter(*q)),
                            );
                        }
                        Work::Slices { branches, .. } => {
                            let mark = work.len();
                            work.extend(
                                branches
                                    .iter()
                                    .flat_map(|b| b.children.iter())
                                    .map(|q| Work::Enter(*q)),
                            );
                            work[mark..].reverse();
                            work.insert(mark, next);
                        }
                        Work::Enter(_) => return Err("missing coverage continuation"),
                    }
                }
                Work::Finish {
                    query,
                    operation,
                    initial,
                    start,
                } => {
                    let mut result = initial;
                    for index in start..self.children.len() {
                        let child = *self
                            .patterns
                            .get(&self.children[index])
                            .ok_or("missing selected coverage child")?;
                        result = self.graph.apply(operation, result, child)?;
                    }
                    self.children.truncate(start);
                    self.patterns.insert(query, result);
                }
                Work::Slices { query, branches } => {
                    let mut result = Id::EMPTY;
                    for branch in branches {
                        let mut value = branch.length;
                        for child in branch.children {
                            let child = *self
                                .patterns
                                .get(&child)
                                .ok_or("missing slice coverage child")?;
                            value = self.graph.apply(Operation::Intersection, value, child)?;
                        }
                        result = self.graph.apply(Operation::Union, result, value)?;
                    }
                    self.patterns.insert(query, result);
                }
            }
        }
        Ok(())
    }
}
