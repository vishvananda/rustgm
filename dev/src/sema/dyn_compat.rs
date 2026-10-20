//! Dyn compatibility is a demand-driven query over completed declaration contracts.
mod contracts;
use super::model::Declarations;
use crate::{
    resolve::{expansion::Report, model::DefId},
    types::{
        Store,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Sized,
    Constant,
    GenericAssociated,
    GenericMethod,
    Receiver,
    SelfType,
    OpaqueReturn,
    SelfWhereClause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Incompatible { definition: DefId, rule: Rule },
}
impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self::Invalid(message)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => out.write_str(message),
            Self::Incompatible { definition, rule } => write!(
                out,
                "trait is not dyn compatible: definition {}: {}",
                definition.0,
                rule.message(),
            ),
        }
    }
}
impl std::error::Error for Error {}
impl Rule {
    const fn message(self) -> &'static str {
        match self {
            Self::Sized => "requires Self: Sized",
            Self::Constant => "has an associated constant",
            Self::GenericAssociated => "has a dispatchable generic associated type",
            Self::GenericMethod => "has a dispatchable generic method",
            Self::Receiver => "method requires a dispatchable receiver",
            Self::SelfType => "uses Self outside an associated projection or receiver",
            Self::OpaqueReturn => "method returns an opaque type",
            Self::SelfWhereClause => "method has an unsupported Self requirement",
        }
    }
}

/// Only direct edges and members are retained. Flattening every ancestor's
/// method prefix would make a linear trait chain consume quadratic storage.
pub struct Contract {
    pub supertraits: Vec<Clause>,
    pub methods: Vec<DefId>,
    pub associated: Vec<DefId>,
}
enum State {
    Active(Contract),
    Complete(Result<Contract, Error>),
}

/// Use one cache with one immutable, completed declaration epoch. A missing
/// declaration is an error, never a cached compatibility decision.
#[derive(Default)]
pub struct Queries {
    states: HashMap<DefId, State>,
    pub steps: usize,
}
impl Queries {
    /// Validate an object's base trait and its shared supertrait graph once.
    /// # Errors
    /// Reports incompatible contracts, unfinished declarations and bounded work.
    pub fn demand(
        &mut self,
        report: &Report,
        facts: &Declarations,
        root: DefId,
    ) -> Result<&Contract, Error> {
        if let Err(error) = self.walk(report, facts, root) {
            self.states
                .retain(|_, state| !matches!(state, State::Active(_)));
            return Err(error);
        }
        match self.states.get(&root) {
            Some(State::Complete(Ok(contract))) => Ok(contract),
            Some(State::Complete(Err(error))) => Err(*error),
            _ => Err(Error::Invalid("missing completed dyn compatibility query")),
        }
    }
    fn walk(&mut self, report: &Report, facts: &Declarations, root: DefId) -> Result<(), Error> {
        let mut work = vec![(root, false)];
        while let Some((item, finish)) = work.pop() {
            self.charge(1)?;
            if matches!(self.states.get(&item), Some(State::Complete(_))) {
                continue;
            }
            if finish {
                self.finish(&facts.types, item)?;
                continue;
            }
            if self.states.contains_key(&item) {
                return Err(Error::Invalid("cycle in dyn compatibility graph"));
            }
            let contract = match contracts::collect(report, facts, item, &mut self.steps) {
                Ok(contract) => contract,
                Err(error @ Error::Incompatible { .. }) => {
                    self.states.insert(item, State::Complete(Err(error)));
                    continue;
                }
                Err(error) => return Err(error),
            };
            self.charge(contract.supertraits.len())?;
            work.push((item, true));
            for &clause in contract.supertraits.iter().rev() {
                work.push((parent(&facts.types, clause)?, false));
            }
            self.states.insert(item, State::Active(contract));
        }
        Ok(())
    }
    fn finish(&mut self, types: &Store, item: DefId) -> Result<(), Error> {
        let Some(State::Active(contract)) = self.states.get(&item) else {
            return Err(Error::Invalid("missing pending dyn contract"));
        };
        let mut failure = None;
        for &clause in &contract.supertraits {
            match self.states.get(&parent(types, clause)?) {
                Some(State::Complete(Ok(_))) => {}
                Some(State::Complete(Err(error))) => {
                    failure = Some(*error);
                    break;
                }
                _ => return Err(Error::Invalid("unfinished dyn supertrait contract")),
            }
        }
        let Some(State::Active(contract)) = self.states.remove(&item) else {
            return Err(Error::Invalid("lost pending dyn contract"));
        };
        self.states
            .insert(item, State::Complete(failure.map_or(Ok(contract), Err)));
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        charge(&mut self.steps, count)
    }
}
fn charge(steps: &mut usize, count: usize) -> Result<(), Error> {
    *steps = steps.saturating_add(count);
    if *steps > 1_000_000 {
        return Err(Error::Invalid("dyn compatibility work budget exceeded"));
    }
    Ok(())
}
fn parent(types: &Store, clause: Clause) -> Result<DefId, Error> {
    let Bound::Trait(id) = predicate(types, clause)?.bound else {
        return Err(Error::Invalid("dyn supertrait edge requires a trait bound"));
    };
    types
        .trait_application(id)
        .map(|app| DefId(app.item.0))
        .ok_or(Error::Invalid("invalid dyn supertrait application"))
}
fn predicate(
    types: &Store,
    clause: Clause,
) -> Result<crate::types::parameters::predicates::Predicate, Error> {
    match clause {
        Clause::Predicate(predicate) => Ok(predicate),
        Clause::ForAll(id) => types
            .quantified(id)
            .map(|q| q.predicate)
            .ok_or(Error::Invalid("invalid dyn compatibility clause")),
    }
}
