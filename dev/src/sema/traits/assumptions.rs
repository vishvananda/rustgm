//! Caller assumptions are indexed once; alternatives use isolated complete-goal equations.
mod alternatives;
mod discriminants;
#[cfg(test)]
use crate::types::parameters::predicates::Bound;
mod universal;
use super::heads::{Head, Subject};
use crate::types::{
    ItemId, Store,
    infer::{self, Context, predicates::Predicate as Goal},
    parameters::{
        ParameterId,
        clauses::Clause,
        environment::Environment,
        predicates::Predicate,
        premise_index::{ClauseIndex, QuantifiedIndex},
    },
};
pub(in crate::sema) use discriminants::Discriminants;
use std::sync::Arc;
pub(in crate::sema) use universal::{Openings, ScopedEvidence, ScopedSelection};
#[cfg(test)]
#[path = "../../../tests/invariants/declaration_assumptions.rs"]
mod declaration_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/premise_extensions.rs"]
pub(in crate::sema) mod receiver_prefix_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Default)]
pub(in crate::sema) struct Assumptions {
    index: Option<Arc<ClauseIndex>>,
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
    pub(in crate::sema) fn contains_feature(
        &self,
        types: &Store,
        id: crate::types::parameters::features::FeatureId,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        pay(1, 0)?;
        types
            .feature_name(id)
            .ok_or("foreign caller feature identity")?;
        self.index
            .as_ref()
            .map_or(Ok(false), |index| index.contains_feature(types, id, pay))
    }
    /// Preserve exact canonical membership for consumers which already have a
    /// fully formed requirement. This does not resolve an inference alternative.
    pub(in crate::sema) fn mapped_exact_member(
        &self,
        store: &mut Store,
        context: &mut Context,
        predicate: Predicate,
    ) -> Result<bool, infer::Error> {
        context.account_work(1)?;
        let Some(index) = &self.index else {
            return Ok(false);
        };
        if !index.layers().any(|(_, opening)| opening.is_some()) {
            return Err(infer::Error::InvalidIdentity);
        }
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let item = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?
            .item;
        let head = Head::canonical(
            store
                .kind(predicate.ty)
                .ok_or(infer::Error::InvalidIdentity)?,
        );
        for (candidate, opening) in self.candidates(item, head) {
            let actual = opening.map_or(Ok(candidate), |opening| {
                opening.predicate(store, context, candidate)
            })?;
            if actual == predicate {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn same_snapshot(&self, other: &Self) -> bool {
        match (&self.index, &other.index) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
    pub(in crate::sema) fn retain_allocation(
        &self,
        observer: &mut crate::types::parameters::premise_index::Retained,
    ) -> crate::support::retained::Allocation {
        self.index
            .as_ref()
            .map_or_else(crate::support::retained::Allocation::default, |index| {
                index.retain_allocation(observer)
            })
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
        if self
            .index
            .as_ref()
            .is_some_and(|index| index.layers().any(|(_, opening)| opening.is_some()))
        {
            return Ok(None);
        }
        let Some((id, _)) = predicate.bound.trait_parts() else {
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
            .quantified()
            .is_some_and(|index| index.candidates.contains_key(&(application.item, head)))
        {
            return Ok(None);
        }
        let mut candidates = self.candidates(application.item, head);
        Ok(
            (matches!(candidates.next(), Some((value, None)) if value == predicate)
                && candidates.next().is_none())
            .then_some(Evidence(predicate)),
        )
    }
    pub(super) fn may_have_subject(&self, item: ItemId, subject: Subject) -> bool {
        subject.heads().any(|head| {
            self.candidates(item, head).next().is_some()
                || self
                    .quantified()
                    .is_some_and(|index| index.candidates.contains_key(&(item, head)))
        })
    }
    /// No caller trait clause is assumed.
    pub fn is_empty(&self) -> bool {
        self.index.is_none()
    }
    pub fn parameter_traits(&self, parameter: ParameterId) -> &[ItemId] {
        self.index
            .as_ref()
            .map_or(&[], |index| index.parameter_traits(parameter))
    }

    pub fn from_environment(
        store: &Store,
        environment: Option<&Environment>,
    ) -> Result<(Self, usize), &'static str> {
        let Some(environment) = environment else {
            return Ok((Self::default(), 0));
        };
        let (index, work) = environment.trait_index(store)?;
        Ok((Self { index }, work))
    }

    pub(crate) fn from_environment_counted(
        store: &Store,
        environment: Option<&Environment>,
        work: &mut usize,
    ) -> Result<Self, &'static str> {
        let Some(environment) = environment else {
            return Ok(Self::default());
        };
        let index = environment.trait_index_counted(store, work)?;
        Ok(Self { index })
    }

    pub(crate) fn from_environment_charged(
        store: &Store,
        environment: Option<&Environment>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        charge(1, 0)?;
        let index = environment
            .map(|environment| environment.trait_index_charged(store, charge))
            .transpose()?
            .flatten();
        Ok(Self { index })
    }
    pub(crate) fn feature_environment_charged(
        store: &Store,
        environment: &Environment,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        let index = environment
            .trait_index_charged(store, pay)?
            .map(|index| index.feature_only_charged(store, pay))
            .transpose()?
            .flatten();
        Ok(Self { index })
    }

    fn candidates(
        &self,
        item: ItemId,
        head: Head,
    ) -> impl Iterator<
        Item = (
            Predicate,
            Option<&Arc<crate::types::parameters::premise_index::Opening>>,
        ),
    > {
        self.certified_candidates(item, head)
            .map(|(predicate, opening, _)| (predicate, opening))
    }
    /// Each candidate with whether its own layer's environment certified it
    /// normal. An opened view reads its base's certificates: the opening
    /// substitutes regions only.
    fn certified_candidates(
        &self,
        item: ItemId,
        head: Head,
    ) -> impl Iterator<
        Item = (
            Predicate,
            Option<&Arc<crate::types::parameters::premise_index::Opening>>,
            bool,
        ),
    > {
        self.index
            .iter()
            .flat_map(|index| index.layers())
            .flat_map(move |(index, opening)| {
                index
                    .candidates
                    .get(&(item, head))
                    .into_iter()
                    .flatten()
                    .copied()
                    .map(move |predicate| {
                        (
                            predicate,
                            opening,
                            index.normal_premise(Clause::Predicate(predicate)),
                        )
                    })
            })
    }
    /// Whether the layer holding quantified premises certified `id` normal.
    fn normal_quantified(&self, id: crate::types::parameters::clauses::QuantifiedId) -> bool {
        self.index
            .as_ref()
            .and_then(|index| index.layers().find(|(layer, _)| layer.quantified.is_some()))
            .is_some_and(|(layer, _)| layer.normal_premise(Clause::ForAll(id)))
    }
    fn quantified(&self) -> Option<&QuantifiedIndex> {
        self.index.as_ref().and_then(|index| {
            index
                .layers()
                .find_map(|(layer, _)| layer.quantified.as_deref())
        })
    }
    fn contains(&self, predicate: Predicate) -> bool {
        self.index.as_ref().is_some_and(|index| {
            index
                .layers()
                .any(|(layer, opening)| opening.is_none() && layer.seen.contains(&predicate))
        })
    }

    /// A private proof extension shares all existing canonical premise storage.
    /// Its lifetime is that proof's actual requesting environment, never a cache.
    pub(in crate::sema) fn extended(
        &self,
        store: &Store,
        clauses: &[Predicate],
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            index: Some(ClauseIndex::extended(
                self.index.clone(),
                store,
                clauses,
                charge,
            )?),
        })
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

    #[cfg(test)]
    pub fn insert(&mut self, store: &Store, predicate: Predicate) -> Result<(), infer::Error> {
        self.insert_clause(store, Clause::Predicate(predicate))
    }

    pub fn insert_clause(&mut self, store: &Store, clause: Clause) -> Result<(), infer::Error> {
        Arc::make_mut(self.index.get_or_insert_with(Default::default)).insert_clause(store, clause)
    }

    /// Premises copied from `environment`: each `(source, copy)` pair holds
    /// a source premise and that premise under a lifetime-only substitution.
    /// A copy inherits its own source's certificate and no other.
    /// # Errors
    /// Rejects invalid clauses.
    pub(in crate::sema) fn copied(
        store: &Store,
        environment: Option<&crate::types::parameters::environment::Environment>,
        premises: &[(Clause, Clause)],
    ) -> Result<Self, String> {
        let mut assumptions = Self::default();
        let mut certified = Vec::new();
        for &(source, copy) in premises {
            assumptions
                .insert_clause(store, copy)
                .map_err(|e| e.to_string())?;
            if environment.is_some_and(|environment| environment.certified_normal_premise(source)) {
                certified.push(copy);
            }
        }
        assumptions.certify_normal(&certified)?;
        Ok(assumptions)
    }

    /// Certify premises these assumptions already hold.
    /// # Errors
    /// Rejects a premise they do not hold.
    pub(in crate::sema) fn certify_normal(
        &mut self,
        clauses: &[Clause],
    ) -> Result<(), &'static str> {
        if clauses.is_empty() {
            return Ok(());
        }
        let index = self
            .index
            .as_mut()
            .ok_or("certified premise is not held by these assumptions")?;
        Arc::make_mut(index).certify_held(clauses)
    }

    /// A declaration's own exact bound needs equations only when alternatives exist.
    /// This checks ambiguity, not whether an implementation satisfies a new goal.
    /// `discriminants` belongs to these assumptions; a bound with a closed first
    /// value is equated only with assumptions that have that value or an open one.
    pub fn unambiguous_declaration(
        &self,
        store: &mut Store,
        context: &mut Context,
        predicate: Predicate,
        discriminants: &mut Discriminants,
    ) -> Result<bool, infer::Error> {
        context.account_work(1)?;
        if self
            .index
            .as_ref()
            .is_some_and(|index| index.layers().any(|(_, opening)| opening.is_some()))
        {
            return Err(infer::Error::InvalidIdentity);
        }
        if self.quantified().is_some() || !self.contains(predicate) {
            return Err(infer::Error::InvalidIdentity);
        }
        let Some((id, _)) = predicate.bound.trait_parts() else {
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
        let Some(value) = discriminants::value(store, predicate)? else {
            if self.candidates(application.item, head).count() == 1 {
                return Ok(true);
            }
            let mut substitution = infer::Instantiation::new(context, store, &[])?;
            let goal = context.instantiate_predicate(store, predicate, &mut substitution)?;
            return Ok(self.prove(store, context, &goal)?.is_some());
        };
        discriminants.build(self, store, context)?;
        if discriminants
            .candidates(application.item, head, value)
            .nth(1)
            .is_none()
        {
            return Ok(true);
        }
        let mut substitution = infer::Instantiation::new(context, store, &[])?;
        let goal = context.instantiate_predicate(store, predicate, &mut substitution)?;
        let Some(subject) = alternatives::subject(store, context, &goal)? else {
            return Ok(false);
        };
        let candidates = subject.heads().flat_map(|head| {
            discriminants
                .candidates(application.item, head, value)
                .map(|predicate| (predicate, None, false))
        });
        Ok(matches!(
            Self::choose_flat(store, context, &goal, candidates)?,
            Selection::Proven(_)
        ))
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
        if self.quantified().is_some() {
            return Err(infer::Error::InvalidIdentity);
        }
        context.account_work(1)?;
        let Some((item, _)) = goal.trait_arguments() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let Some(subject) = alternatives::subject(store, context, goal)? else {
            return Ok(Selection::Deferred);
        };
        Self::choose_flat(
            store,
            context,
            goal,
            subject
                .heads()
                .flat_map(|head| self.certified_candidates(item, head)),
        )
    }

    /// One flat selection among `candidates`, each in an isolated trial.
    fn choose_flat<'a>(
        store: &mut Store,
        context: &mut Context,
        goal: &Goal,
        candidates: impl Iterator<
            Item = (
                Predicate,
                Option<&'a Arc<crate::types::parameters::premise_index::Opening>>,
                bool,
            ),
        >,
    ) -> Result<Selection, infer::Error> {
        alternatives::choose(
            store,
            context,
            goal,
            candidates,
            |store, context, (predicate, opening, normal)| {
                let predicate = opening.map_or(Ok(predicate), |opening| {
                    opening.predicate(store, context, predicate)
                })?;
                Ok((predicate, Evidence(predicate), normal))
            },
            |_, _, left, right| Ok(alternatives::ordinary_consequence(goal, left.0, right.0)),
        )
    }
}
