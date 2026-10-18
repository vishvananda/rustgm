//! Caller assumptions are indexed once; alternatives use isolated complete-goal equations.
mod alternatives;
mod universal;
use super::heads::{Head, Subject};
use crate::types::{
    ItemId, Store,
    infer::{self, Context, predicates::Predicate as Goal},
    parameters::{
        ParameterId,
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
pub(in crate::sema) use universal::{ScopedEvidence, ScopedSelection};
#[cfg(test)]
mod tests;

#[derive(Default)]
pub(in crate::sema) struct Assumptions {
    candidates: HashMap<(ItemId, Head), Vec<Predicate>>,
    seen: HashSet<Predicate>,
    parameter_traits: OnceLock<HashMap<ParameterId, Vec<ItemId>>>,
    quantified: Option<Box<universal::Index>>,
}

/// Equations against this actual caller assumption have been committed to inference.
/// Their region equalities still require the ordinary body lifetime proof.
pub(in crate::sema) struct Evidence(Predicate);
/// Applicability may defer unresolved evidence; only Proven authorizes a body goal.
pub(in crate::sema) enum Selection<T = Evidence> {
    Absent,
    Deferred,
    Proven(T),
}
impl Evidence {
    pub fn assumption(&self) -> Predicate {
        self.0
    }
}

impl Assumptions {
    /// A sole identical canonical clause needs no inference or lifetime mapping.
    /// Additional coarse candidates require ordinary complete-goal equations.
    pub(super) fn exact(
        &self,
        store: &Store,
        predicate: Predicate,
    ) -> Result<Option<Evidence>, infer::Error> {
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        let head = Head::canonical(
            store
                .kind(predicate.ty)
                .ok_or(infer::Error::InvalidIdentity)?,
        );
        if self
            .quantified
            .as_ref()
            .is_some_and(|index| index.candidates.contains_key(&(application.item, head)))
        {
            return Ok(None);
        }
        Ok(self
            .candidates
            .get(&(application.item, head))
            .filter(|candidates| candidates.as_slice() == [predicate])
            .map(|_| Evidence(predicate)))
    }
    pub(super) fn may_have_subject(&self, item: ItemId, subject: Subject) -> bool {
        subject.heads().any(|head| {
            self.candidates.contains_key(&(item, head))
                || self
                    .quantified
                    .as_ref()
                    .is_some_and(|index| index.candidates.contains_key(&(item, head)))
        })
    }
    pub fn parameter_traits(&self, parameter: ParameterId) -> &[ItemId] {
        self.parameter_traits
            .get_or_init(|| {
                let mut index: HashMap<ParameterId, Vec<ItemId>> = HashMap::new();
                for &(item, head) in self.candidates.keys() {
                    if let Head::Parameter(parameter) = head {
                        index.entry(parameter).or_default().push(item);
                    }
                }
                if let Some(quantified) = &self.quantified {
                    for &(item, head) in quantified.candidates.keys() {
                        if let Head::Parameter(parameter) = head
                            && !self.candidates.contains_key(&(item, head))
                        {
                            index.entry(parameter).or_default().push(item);
                        }
                    }
                }
                index
            })
            .get(&parameter)
            .map_or(&[], Vec::as_slice)
    }
    /// Canonical goals keep caller parameters rigid. Any resulting region equations
    /// remain obligations of the caller's declaration or body lifetime proof.
    pub fn prove_canonical(
        &self,
        store: &mut Store,
        context: &mut Context,
        predicate: Predicate,
    ) -> Result<Option<Evidence>, infer::Error> {
        let mut substitution = infer::Instantiation::new(context, store, &[])?;
        let goal = context.instantiate_predicate(store, predicate, &mut substitution)?;
        self.prove(store, context, &goal)
    }

    pub fn insert(&mut self, store: &Store, predicate: Predicate) -> Result<(), infer::Error> {
        self.insert_clause(store, Clause::Predicate(predicate))
    }

    pub fn insert_clause(&mut self, store: &Store, clause: Clause) -> Result<(), infer::Error> {
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or(infer::Error::InvalidIdentity)?
                    .predicate
            }
        };
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        if self
            .seen
            .len()
            .saturating_add(self.quantified.as_ref().map_or(0, |index| index.seen.len()))
            >= 1_000_000
        {
            return Err(infer::Error::Budget);
        }
        if let Clause::ForAll(id) = clause {
            let head = Head::canonical(
                store
                    .kind(predicate.ty)
                    .ok_or(infer::Error::InvalidIdentity)?,
            );
            let index = self.quantified.get_or_insert_with(Default::default);
            if index.seen.insert(id) {
                self.parameter_traits.take();
                index
                    .candidates
                    .entry((application.item, head))
                    .or_default()
                    .push(id);
            }
            return Ok(());
        }
        if self.seen.insert(predicate) {
            self.parameter_traits.take();
            let head = Head::canonical(
                store
                    .kind(predicate.ty)
                    .ok_or(infer::Error::InvalidIdentity)?,
            );
            self.candidates
                .entry((application.item, head))
                .or_default()
                .push(predicate);
        }
        Ok(())
    }

    /// A declaration's own exact bound needs equations only when alternatives exist.
    /// This checks ambiguity, not whether an implementation satisfies a new goal.
    pub fn unambiguous_declaration(
        &self,
        store: &mut Store,
        context: &mut Context,
        predicate: Predicate,
    ) -> Result<bool, infer::Error> {
        context.account_work(1)?;
        if self.quantified.is_some() || !self.seen.contains(&predicate) {
            return Err(infer::Error::InvalidIdentity);
        }
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        let head = Head::canonical(
            store
                .kind(predicate.ty)
                .ok_or(infer::Error::InvalidIdentity)?,
        );
        if self
            .candidates
            .get(&(application.item, head))
            .is_some_and(|candidates| candidates.len() == 1)
        {
            return Ok(true);
        }
        let mut substitution = infer::Instantiation::new(context, store, &[])?;
        let goal = context.instantiate_predicate(store, predicate, &mut substitution)?;
        Ok(self.prove(store, context, &goal)?.is_some())
    }

    /// Unknown Self is not guessed from the available assumptions. Other unresolved
    /// arguments may be inferred once exactly one candidate remains compatible.
    pub fn prove(
        &self,
        store: &mut Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<Option<Evidence>, infer::Error> {
        Ok(match self.select(store, context, goal)? {
            Selection::Proven(evidence) => Some(evidence),
            Selection::Absent | Selection::Deferred => None,
        })
    }

    /// Flat evidence has no region owner. Quantified environments must use
    /// `select_scoped` with the requesting proof's actual free-region frontier.
    pub fn select(
        &self,
        store: &mut Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<Selection, infer::Error> {
        if self.quantified.is_some() {
            return Err(infer::Error::InvalidIdentity);
        }
        context.account_work(1)?;
        let Some((item, _)) = goal.trait_arguments() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let Some(subject) = alternatives::subject(store, context, goal)? else {
            return Ok(Selection::Deferred);
        };
        alternatives::choose(
            store,
            context,
            goal,
            subject
                .heads()
                .flat_map(|head| self.candidates.get(&(item, head)).into_iter().flatten())
                .copied(),
            |_, _, predicate| Ok((predicate, Evidence(predicate))),
        )
    }
}
