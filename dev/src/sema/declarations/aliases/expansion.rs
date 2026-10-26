//! Pure expansion shares exact applications under a single canonical surrounding binder.
use super::{Checker, Edge, Error, Pending, Predicate};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::generics::Generics,
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
    types::{
        GenericArgs, Store, TypeId,
        arguments::Argument,
        parameters::clauses::{Clause, Quantifier},
        substitute::Bound,
    },
};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Application {
    definition: DefId,
    subject: TypeId,
    arguments: GenericArgs,
    surrounding: Option<u32>,
}
#[derive(Clone, Copy)]
struct Frame {
    edge: Edge,
    surrounding: Option<u32>,
}
impl Frame {
    fn application(self) -> Application {
        Application {
            definition: self.edge.target,
            subject: self.edge.subject,
            arguments: self.edge.arguments,
            surrounding: self.surrounding,
        }
    }
}
struct Expansion<'a> {
    owner: &'a Pending,
    source: NodeKey,
    stack: Vec<Frame>,
    visited: HashSet<Application>,
    emitted: HashSet<Clause>,
    output: Vec<Predicate>,
    joiners: HashMap<u32, Quantifier>,
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn expand_alias_edge(
        &mut self,
        edge: Edge,
    ) -> Result<Vec<Predicate>, Error> {
        let pending = self
            .trait_aliases
            .as_ref()
            .ok_or_else(|| Error::unlocated("missing alias expansion owner"))?;
        pending
            .expand(&mut self.facts.types, &self.facts.generics, edge)
            .map_err(|e| crate::sema::model::Declarations::error(self.report, edge.source, e))
    }
}
impl Pending {
    fn expand(
        &self,
        store: &mut Store,
        generics: &Generics,
        root: Edge,
    ) -> Result<Vec<Predicate>, &'static str> {
        if !self.closed {
            return Err("trait alias clauses require a closed canonical graph");
        }
        let mut query = Expansion {
            owner: self,
            source: root.source,
            stack: Vec::new(),
            visited: HashSet::default(),
            emitted: HashSet::default(),
            output: Vec::new(),
            joiners: HashMap::default(),
        };
        query.push(Frame {
            edge: root,
            surrounding: root.quantified,
        })?;
        query.drain(store, generics)?;
        Ok(query.output)
    }
}
impl Expansion<'_> {
    fn drain(&mut self, store: &mut Store, generics: &Generics) -> Result<(), &'static str> {
        let owner = self.owner;
        while let Some(frame) = self.stack.pop() {
            owner.budget.charge(1, 0)?;
            owner.budget.grow(Growth::map::<Application, ()>(
                self.visited.len(),
                self.visited.capacity(),
                1,
            ))?;
            if !self.visited.insert(frame.application()) {
                continue;
            }
            let header = owner.header(frame.edge.target)?;
            let mut fold = owner.mapping(store, generics, frame.edge)?;
            let charge = |steps, bytes| owner.budget.charge(steps, bytes);
            for clause in header
                .clauses
                .as_deref()
                .ok_or("unfinished alias clauses")?
            {
                let clause = fold.clause_charged(store, clause.value, &charge)?;
                self.emit(store, frame.surrounding, clause)?;
            }
            for &dependency in &header.edges {
                self.dependency(store, &mut fold, frame.surrounding, dependency)?;
            }
        }
        Ok(())
    }
    fn dependency(
        &mut self,
        store: &mut Store,
        fold: &mut Bound,
        parent: Option<u32>,
        edge: Edge,
    ) -> Result<(), &'static str> {
        let owner = self.owner;
        let charge = |steps, bytes| owner.budget.charge(steps, bytes);
        let depth = u16::from(edge.quantified.is_some());
        let subject =
            fold.argument_inside_charged(store, Argument::Type(edge.subject), depth, &charge)?;
        let Argument::Type(mut subject) = subject else {
            return Err("alias subject substitution changed argument kind");
        };
        let mut arguments = fold.arguments_inside_charged(store, edge.arguments, depth, &charge)?;
        let surrounding = if let Some(inner) = edge.quantified {
            if let Some(parent) = parent {
                let joined = self
                    .joiner(parent)?
                    .inputs_charged(store, subject, arguments, inner, &charge)?;
                subject = joined.0;
                arguments = joined.1;
                Some(joined.2)
            } else {
                Some(inner)
            }
        } else {
            parent
        };
        self.push(Frame {
            edge: Edge {
                subject,
                arguments,
                ..edge
            },
            surrounding,
        })
    }
    fn emit(
        &mut self,
        store: &mut Store,
        surrounding: Option<u32>,
        mut value: Clause,
    ) -> Result<(), &'static str> {
        let owner = self.owner;
        let charge = |steps, bytes| owner.budget.charge(steps, bytes);
        if let Some(count) = surrounding {
            value = self.joiner(count)?.clause_charged(store, value, &charge)?;
        }
        owner.budget.grow(Growth::map::<Clause, ()>(
            self.emitted.len(),
            self.emitted.capacity(),
            1,
        ))?;
        if self.emitted.insert(value) {
            owner.budget.grow(Growth::vector::<Predicate>(
                self.output.len(),
                self.output.capacity(),
                1,
            ))?;
            self.output.push(Predicate {
                source: self.source,
                value,
            });
        }
        Ok(())
    }
    fn joiner(&mut self, count: u32) -> Result<&mut Quantifier, &'static str> {
        self.owner.budget.charge(1, 0)?;
        if !self.joiners.contains_key(&count) {
            self.owner.budget.grow(Growth::map::<u32, Quantifier>(
                self.joiners.len(),
                self.joiners.capacity(),
                1,
            ))?;
            self.joiners.insert(count, Quantifier::new(count)?);
        }
        self.joiners
            .get_mut(&count)
            .ok_or("missing canonical alias binder composer")
    }
    fn push(&mut self, frame: Frame) -> Result<(), &'static str> {
        if frame
            .surrounding
            .is_some_and(|count| count > u32::from(u16::MAX) + 1)
        {
            return Err("too many surrounding alias lifetime slots");
        }
        self.owner.budget.grow(Growth::vector::<Frame>(
            self.stack.len(),
            self.stack.capacity(),
            1,
        ))?;
        self.stack.push(frame);
        Ok(())
    }
}
