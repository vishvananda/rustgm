//! Pure clause expansion visits each exact canonical application once per use.
use super::{Checker, Edge, Error, Pending, Predicate};
use crate::{
    sema::generics::Generics,
    support::{hash::HashSet, retained::Growth},
    types::{GenericArgs, Store, TypeId},
    resolve::model::DefId,
};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Application {
    definition: DefId,
    subject: TypeId,
    arguments: GenericArgs,
}
impl From<Edge> for Application {
    fn from(edge: Edge) -> Self {
        Self { definition: edge.target, subject: edge.subject, arguments: edge.arguments }
    }
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn expand_alias_edge(&mut self, edge: Edge) -> Result<Vec<Predicate>, Error> {
        let pending = self.trait_aliases.as_ref().ok_or_else(|| Error::unlocated("missing alias expansion owner"))?;
        pending.expand(&mut self.facts.types, &self.facts.generics, edge)
            .map_err(|e| crate::sema::model::Declarations::error(self.report, edge.source, e))
    }
}
impl Pending {
    fn expand(&self, store: &mut Store, generics: &Generics, root: Edge) -> Result<Vec<Predicate>, &'static str> {
        if !self.closed { return Err("trait alias clauses require a closed canonical graph") }
        let mut stack = Vec::new();
        let mut visited = HashSet::<Application>::default();
        let mut emitted = HashSet::default();
        let mut output = Vec::new();
        self.push_use(&mut stack, root)?;
        while let Some(edge) = stack.pop() {
            self.budget.charge(1, 0)?;
            self.budget.grow(Growth::map::<Application, ()>(visited.len(), visited.capacity(), 1))?;
            if !visited.insert(edge.into()) { continue }
            let header = self.header(edge.target)?;
            let mut fold = self.mapping(store, generics, edge)?;
            let charge = |steps, bytes| self.budget.charge(steps, bytes);
            for clause in header.clauses.as_deref().ok_or("unfinished alias clauses")? {
                let value = fold.clause_charged(store, clause.value, &charge)?;
                self.budget.grow(Growth::map::<crate::types::parameters::clauses::Clause, ()>(emitted.len(), emitted.capacity(), 1))?;
                if emitted.insert(value) {
                    self.budget.grow(Growth::vector::<Predicate>(output.len(), output.capacity(), 1))?;
                    output.push(Predicate { source: root.source, value });
                }
            }
            for dependency in &header.edges {
                if dependency.quantified.is_some() {
                    return Err("nested quantified alias application requires binder composition");
                }
                let subject = fold.apply_argument_charged(store, crate::types::arguments::Argument::Type(dependency.subject), &charge)?;
                let crate::types::arguments::Argument::Type(subject) = subject else { return Err("alias subject substitution changed argument kind") };
                let arguments = fold.arguments_charged(store, dependency.arguments, &charge)?;
                self.push_use(&mut stack, Edge { subject, arguments, ..*dependency })?;
            }
        }
        Ok(output)
    }
    fn push_use(&self, stack: &mut Vec<Edge>, edge: Edge) -> Result<(), &'static str> {
        if edge.quantified.is_some() {
            return Err("quantified alias application requires binder composition");
        }
        self.budget.grow(Growth::vector::<Edge>(stack.len(), stack.capacity(), 1))?;
        stack.push(edge);
        Ok(())
    }
}
