//! Coverage sets over sparse typed places, with shared interval decision subgraphs.
mod decision;
mod inhabited;
#[cfg(test)]
mod tests;
mod witness;

use super::{Kind, PatternId, Projection, scalar::Domain};
use crate::{
    resolve::{expansion::Report, package::ModuleId},
    sema::{members::VariantId, model::Declarations},
    types::{Kind as Type, TypeId},
};
use decision::{Graph, Id, Operation, Variable};
pub use inhabited::Cache;
use inhabited::Inhabited;
use std::{collections::HashMap, sync::Arc};

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
    Tags(Arc<[VariantId]>),
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
    Finish {
        query: Query,
        operation: Operation,
        initial: Id,
        children: Vec<Query>,
    },
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
    covered: Id,
    universe: Id,
}
impl<'a> Coverage<'a> {
    pub fn new(
        facts: &'a Declarations,
        report: &'a Report,
        module: ModuleId,
        ty: TypeId,
        cache: &'a mut Cache,
    ) -> Result<Self, &'static str> {
        let mut graph = Graph::new(super::scalar::Coverage::LIMIT);
        let mut inhabited = Inhabited::new(facts, report, module, cache);
        let universe = if inhabited.get(ty, &mut graph)? {
            Id::ALL
        } else {
            Id::EMPTY
        };
        Ok(Self {
            facts,
            report,
            inhabited,
            graph,
            places: vec![Place {
                ty,
                valid: true,
                parent: None,
                variable: None,
            }],
            projections: HashMap::new(),
            inputs: Vec::new(),
            patterns: HashMap::new(),
            covered: Id::EMPTY,
            universe,
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
        let valid = self.places[base.0 as usize].valid && step != Step::Dereference;
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
            Some(Type::Primitive(primitive)) => {
                let domain =
                    Domain::new(primitive).ok_or("coverage requires a supported scalar domain")?;
                (domain.full().upper, Values::Scalar(domain))
            }
            Some(Type::Adt(item)) => {
                let tags = self
                    .inhabited
                    .tags(info.ty, item, info.valid, &mut self.graph)?;
                let maximum = tags
                    .len()
                    .checked_sub(1)
                    .ok_or("empty coverage constructor domain")?;
                (maximum as u128, Values::Tags(tags))
            }
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
            let Values::Tags(tags) = &self.inputs[variable.0 as usize].values else {
                return Err("missing constructor coverage domain");
            };
            let Ok(rank) = tags.binary_search_by_key(&variant.0, |v| v.0) else {
                return Ok(Work::Finish {
                    query,
                    operation: Operation::Intersection,
                    initial: Id::EMPTY,
                    children: Vec::new(),
                });
            };
            self.graph.interval(variable, rank as u128, rank as u128)?
        } else {
            Id::ALL
        };
        let mut children = Vec::new();
        for field in self.facts.patterns.fields(fields) {
            children.push(self.child(query.place, field.pattern, Step::Field(field.selection))?);
        }
        Ok(Work::Finish {
            query,
            operation: Operation::Intersection,
            initial,
            children,
        })
    }
    fn enter(&mut self, query: Query) -> Result<Work, &'static str> {
        let place = self.places[query.place.0 as usize];
        if place.valid && !self.inhabited.get(place.ty, &mut self.graph)? {
            return Ok(Work::Finish {
                query,
                operation: Operation::Intersection,
                initial: Id::EMPTY,
                children: Vec::new(),
            });
        }
        let mut initial = Id::ALL;
        let mut operation = Operation::Intersection;
        let mut children = Vec::new();
        match self.facts.patterns.nodes[query.pattern.0 as usize].kind {
            Kind::Wildcard
            | Kind::Binding {
                subpattern: None, ..
            } => {}
            Kind::Binding {
                subpattern: Some(pattern),
                ..
            } => children.push(Query { pattern, ..query }),
            Kind::Range(range) => {
                let variable = self.variable(query.place)?;
                let interval = self.facts.patterns.ranges[range.0 as usize];
                initial = self
                    .graph
                    .interval(variable, interval.lower, interval.upper)?;
            }
            Kind::Dereference { pattern, .. } => {
                children.push(self.child(query.place, pattern, Step::Dereference)?);
            }
            Kind::Product { variant, fields } => return self.product(query, variant, fields),
            Kind::Or(alternatives) => {
                initial = Id::EMPTY;
                operation = Operation::Union;
                for &pattern in self.facts.patterns.alternatives(alternatives) {
                    children.push(Query { pattern, ..query });
                }
            }
            Kind::Pending => return Err("incomplete coverage pattern"),
        }
        Ok(Work::Finish {
            query,
            operation,
            initial,
            children,
        })
    }
    fn pattern(&mut self, pattern: PatternId) -> Result<Id, &'static str> {
        let root = Query {
            pattern,
            place: PlaceId(0),
        };
        let mut work = vec![Work::Enter(root)];
        while let Some(next) = work.pop() {
            self.graph.charge()?;
            match next {
                Work::Enter(query) => {
                    if self.patterns.contains_key(&query) {
                        continue;
                    }
                    let next = self.enter(query)?;
                    let Work::Finish { children, .. } = &next else {
                        return Err("missing coverage continuation");
                    };
                    let pending: Vec<_> = children.iter().map(|q| Work::Enter(*q)).collect();
                    work.push(next);
                    work.extend(pending.into_iter().rev());
                }
                Work::Finish {
                    query,
                    operation,
                    initial,
                    children,
                } => {
                    let mut result = initial;
                    for child in children {
                        let child = *self
                            .patterns
                            .get(&child)
                            .ok_or("missing selected coverage child")?;
                        result = self.graph.apply(operation, result, child)?;
                    }
                    self.patterns.insert(query, result);
                }
            }
        }
        self.patterns
            .get(&root)
            .copied()
            .ok_or("missing selected coverage result")
    }
}
