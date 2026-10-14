//! Lifetime variance is a finite monotone graph over nominal parameter identities.
use super::{ItemId, Kind, RegionId, Store, TypeId, aggregate::Definitions};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variance {
    Bivariant,
    Covariant,
    Contravariant,
    Invariant,
}
impl Variance {
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        if self == other || other == Self::Bivariant {
            self
        } else if self == Self::Bivariant {
            other
        } else {
            Self::Invariant
        }
    }
    #[must_use]
    pub fn compose(self, inner: Self) -> Self {
        match (self, inner) {
            (Self::Bivariant, _) | (_, Self::Bivariant) => Self::Bivariant,
            (Self::Covariant, value) | (value, Self::Covariant) => value,
            (Self::Contravariant, Self::Contravariant) => Self::Covariant,
            _ => Self::Invariant,
        }
    }
}
#[derive(Default)]
pub struct Variances {
    ranges: HashMap<ItemId, (usize, usize)>,
    values: Vec<Variance>,
    pub steps: usize,
}
impl Variances {
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&[Variance]> {
        let &(start, end) = self.ranges.get(&item)?;
        self.values.get(start..end)
    }

    /// Infer parameters from completed field templates. Bounds do not create
    /// occurrences: recursive-only parameters remain bivariant and are unused.
    ///
    /// # Errors
    /// Rejects inconsistent templates/parameter counts and exhausted graph work.
    pub fn infer(
        types: &Store,
        definitions: &Definitions,
        parameters: &[(ItemId, u32)],
    ) -> Result<Self, &'static str> {
        let mut graph = Graph::default();
        for &(item, count) in parameters {
            let start = graph.result.values.len();
            let end = start
                .checked_add(count as usize)
                .filter(|end| *end <= LIMIT)
                .ok_or("lifetime variance parameter budget exceeded")?;
            if graph.result.ranges.insert(item, (start, end)).is_some() {
                return Err("duplicate variance parameter declaration");
            }
            graph.result.values.resize(end, Variance::Bivariant);
        }
        graph.heads.resize(graph.result.values.len(), None);
        graph.pending.resize(graph.result.values.len(), false);
        for &(item, _) in parameters {
            let definition = definitions
                .get(item)
                .ok_or("missing variance aggregate definition")?;
            let range = graph.result.ranges[&item];
            graph.seen.clear();
            for variant in definitions.variants(definition) {
                for ty in types
                    .list(variant.fields)
                    .ok_or("missing variance field types")?
                {
                    graph.visit(types, range, *ty)?;
                }
            }
        }
        graph.propagate()?;
        Ok(graph.result)
    }
}

const LIMIT: usize = 1_000_000;
#[derive(Clone, Copy)]
struct Edge {
    target: usize,
    variance: Variance,
    next: Option<usize>,
}
#[derive(Default)]
struct Graph {
    result: Variances,
    heads: Vec<Option<usize>>,
    edges: Vec<Edge>,
    pending: Vec<bool>,
    queue: Vec<usize>,
    work: Vec<(TypeId, Variance, u16)>,
    seen: HashSet<(TypeId, Variance, u16)>,
}
impl Graph {
    fn charge(&mut self) -> Result<(), &'static str> {
        self.result.steps += 1;
        if self.result.steps.saturating_add(self.work.len()) > LIMIT {
            return Err("lifetime variance work budget exceeded");
        }
        Ok(())
    }
    fn constrain(&mut self, target: usize, variance: Variance) {
        let prior = self.result.values[target];
        let value = prior.join(variance);
        if prior != value {
            self.result.values[target] = value;
            if !std::mem::replace(&mut self.pending[target], true) {
                self.queue.push(target);
            }
        }
    }
    fn parameter(
        range: (usize, usize),
        region: RegionId,
        depth: u16,
    ) -> Result<Option<usize>, &'static str> {
        let Some((binder, index)) = region.bound_parts() else {
            return Ok(None);
        };
        if binder != depth {
            return Ok(None);
        }
        let index = range.0 + usize::from(index);
        if index >= range.1 {
            return Err("variance parameter outside its declaration");
        }
        Ok(Some(index))
    }
    fn queue_type(
        &mut self,
        ty: TypeId,
        variance: Variance,
        depth: u16,
    ) -> Result<(), &'static str> {
        self.charge()?;
        self.work.push((ty, variance, depth));
        Ok(())
    }
    fn visit(
        &mut self,
        types: &Store,
        range: (usize, usize),
        ty: TypeId,
    ) -> Result<(), &'static str> {
        self.queue_type(ty, Variance::Covariant, 0)?;
        while let Some((ty, position, depth)) = self.work.pop() {
            self.charge()?;
            if types.escaping_binders(ty).ok_or("invalid variance type")? <= depth
                || !self.seen.insert((ty, position, depth))
            {
                continue;
            }
            match types.kind(ty).ok_or("invalid variance type")? {
                Kind::Reference {
                    mutable,
                    region,
                    referent,
                } => {
                    if let Some(target) = Self::parameter(range, region, depth)? {
                        self.constrain(target, position);
                    }
                    let variance = if mutable {
                        position.compose(Variance::Invariant)
                    } else {
                        position
                    };
                    self.queue_type(referent, variance, depth)?;
                }
                Kind::Pointer { mutable, pointee } => {
                    let variance = if mutable {
                        position.compose(Variance::Invariant)
                    } else {
                        position
                    };
                    self.queue_type(pointee, variance, depth)?;
                }
                Kind::Array { element, .. } | Kind::Slice(element) => {
                    self.queue_type(element, position, depth)?;
                }
                Kind::Tuple(fields) => {
                    for field in types.list(fields).ok_or("invalid variance tuple")? {
                        self.queue_type(*field, position, depth)?;
                    }
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or("variance binder depth overflow")?;
                    for parameter in types.list(parameters).ok_or("invalid variance signature")? {
                        self.queue_type(
                            *parameter,
                            position.compose(Variance::Contravariant),
                            depth,
                        )?;
                    }
                    self.queue_type(result, position, depth)?;
                }
                Kind::FunctionItem { signature, .. } => {
                    self.queue_type(signature, position, depth)?;
                }
                Kind::Adt(item, args) => {
                    self.nominal(types, range, (item, args), position, depth)?;
                }
                Kind::Never | Kind::Primitive(_) => {}
            }
        }
        Ok(())
    }
    fn nominal(
        &mut self,
        types: &Store,
        range: (usize, usize),
        nominal: (ItemId, super::GenericArgs),
        position: Variance,
        depth: u16,
    ) -> Result<(), &'static str> {
        let arguments = types
            .region_args(nominal.1)
            .ok_or("invalid variance arguments")?;
        if arguments.is_empty() {
            return Ok(());
        }
        let &(start, end) = self
            .result
            .ranges
            .get(&nominal.0)
            .ok_or("missing nominal variance parameters")?;
        if arguments.len() != end - start {
            return Err("incorrect nominal variance arity");
        }
        for (source, region) in (start..end).zip(arguments) {
            self.charge()?;
            if let Some(target) = Self::parameter(range, *region, depth)? {
                let edge = self.edges.len();
                self.edges.push(Edge {
                    target,
                    variance: position,
                    next: self.heads[source],
                });
                self.heads[source] = Some(edge);
            }
        }
        Ok(())
    }
    fn propagate(&mut self) -> Result<(), &'static str> {
        while let Some(source) = self.queue.pop() {
            self.pending[source] = false;
            let variance = self.result.values[source];
            let mut next = self.heads[source];
            while let Some(index) = next {
                self.charge()?;
                let edge = self.edges[index];
                self.constrain(edge.target, edge.variance.compose(variance));
                next = edge.next;
            }
        }
        Ok(())
    }
}
