//! Caller assumptions are indexed once; alternatives use isolated complete-goal equations.
use super::heads::{Head, Subject};
use crate::types::{
    ItemId, Store,
    infer::{self, Context, predicates::Predicate as Goal},
    parameters::{
        ParameterId,
        predicates::{Bound, Predicate},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
#[cfg(test)]
mod tests;

#[derive(Default)]
pub(in crate::sema) struct Assumptions {
    candidates: HashMap<(ItemId, Head), Vec<Predicate>>,
    seen: HashSet<Predicate>,
    parameter_traits: OnceLock<HashMap<ParameterId, Vec<ItemId>>>,
}

/// Equations against this actual caller assumption have been committed to inference.
/// Their region equalities still require the ordinary body lifetime proof.
pub(in crate::sema) struct Evidence(Predicate);
/// Applicability may defer unresolved evidence; only Proven authorizes a body goal.
pub(in crate::sema) enum Selection {
    Absent,
    Deferred,
    Proven(Evidence),
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
        Ok(self
            .candidates
            .get(&(application.item, head))
            .filter(|candidates| candidates.as_slice() == [predicate])
            .map(|_| Evidence(predicate)))
    }
    pub(super) fn may_have_subject(&self, item: ItemId, subject: Subject) -> bool {
        subject
            .heads()
            .any(|head| self.candidates.contains_key(&(item, head)))
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
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        if self.seen.len() >= 1_000_000 {
            return Err(infer::Error::Budget);
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
        if !self.seen.contains(&predicate) {
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

    pub fn select(
        &self,
        store: &mut Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<Selection, infer::Error> {
        context.account_work(1)?;
        let Some((item, _)) = goal.trait_arguments() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let subject = match Subject::query(store, context, goal.subject()) {
            Ok(subject) => subject,
            Err(infer::Error::Normalization) => Some(Subject::Rigid(Head::Projection(
                context
                    .projection_item(store, goal.subject())?
                    .ok_or(infer::Error::InvalidIdentity)?,
            ))),
            Err(error) => return Err(error),
        };
        let Some(subject) = subject else {
            return Ok(Selection::Deferred);
        };
        let mut selected = None;
        let mut needs_normalization = false;
        for &candidate in subject
            .heads()
            .flat_map(|head| self.candidates.get(&(item, head)).into_iter().flatten())
        {
            let mut probe = Context::default();
            let imported = probe.import_predicate(store, context, goal)?;
            let result = probe.equate_trait_predicate(store, &imported, candidate);
            context.account_work(probe.steps)?;
            match result {
                Ok(()) => {
                    if selected.replace(candidate).is_some() {
                        return Ok(Selection::Deferred);
                    }
                }
                Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
                // An opaque equation is not a second matching assumption. Keep
                // looking, including for other fully equatable alternatives.
                Err(infer::Error::Normalization) => needs_normalization = true,
                // Applicability may wait for call arguments or a result type.
                // Another alternative cannot choose this unresolved input.
                Err(infer::Error::Unresolved) => return Ok(Selection::Deferred),
                Err(error) => return Err(error),
            }
        }
        if let Some(assumption) = selected {
            context.equate_trait_predicate(store, goal, assumption)?;
            Ok(Selection::Proven(Evidence(assumption)))
        } else if needs_normalization {
            Err(infer::Error::Normalization)
        } else {
            Ok(Selection::Absent)
        }
    }
}
