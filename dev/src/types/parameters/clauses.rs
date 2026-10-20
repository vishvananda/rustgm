//! Quantification binds a whole predicate, including its subject and result.
mod composition;
mod equivalence;
use super::predicates::Predicate;
use crate::types::{Store, bound_uses::captures::Slots};
pub use composition::Quantifier;
pub(crate) use equivalence::Comparison;
use std::collections::HashMap;
#[cfg(test)]
mod slot_validation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QuantifiedId(pub u32);

/// Ordinary predicates retain their compact representation. Reading a quantified
/// body requires an explicit binder operation; its identity is not proof evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Clause {
    Predicate(Predicate),
    ForAll(QuantifiedId),
}

impl From<Predicate> for Clause {
    fn from(predicate: Predicate) -> Self {
        Self::Predicate(predicate)
    }
}

impl Clause {
    /// Identify a written bound eligible for checking a closed parameter default.
    /// Any lifetime occurrence defers this check, including static lifetimes and
    /// nested callable locals. An unused clause quantifier does not. Inspect the
    /// original bound before substitution; lifetimes introduced by a default do
    /// not change this classification. This is not proof of the requirement.
    /// # Errors
    /// Rejects unknown clauses, types and trait applications.
    pub fn checks_parameter_default(self, store: &Store) -> Result<bool, &'static str> {
        use super::predicates::Bound;
        let predicate = match self {
            Self::Predicate(predicate) => predicate,
            Self::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid default bound clause")?
                    .predicate
            }
        };
        let subject = store
            .contains_regions(predicate.ty)
            .ok_or("invalid default bound subject")?;
        let bound = match predicate.bound {
            Bound::Trait(id) => store
                .trait_properties(id)
                .ok_or("invalid default bound application")?
                .has(crate::types::Property::Regions),
            Bound::ProjectionEqual(result) => store
                .contains_regions(result)
                .ok_or("invalid default bound result")?,
            Bound::Outlives(_) => true,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) => false,
        };
        Ok(!subject && !bound)
    }

    /// Preserve the source-level consequences of an explicit lifetime premise.
    /// Apply this before substituting a parameter with a concrete composite: an
    /// all-regions parameter bound already grants its caller a static bound.
    /// Projected subjects stay opaque until their own normalization is proved.
    /// # Errors
    /// Rejects unknown quantified clauses and invalid subjects.
    pub fn caller_outlives(self, store: &Store) -> Result<Option<Self>, &'static str> {
        use super::predicates::Bound;
        use crate::types::{Kind, RegionId};
        let value = match self {
            Self::Predicate(predicate) => {
                return Ok(matches!(predicate.bound, Bound::Outlives(_)).then_some(self));
            }
            Self::ForAll(id) => store
                .quantified(id)
                .ok_or("invalid outlives assumption clause")?,
        };
        let Bound::Outlives(shorter) = value.predicate.bound else {
            return Ok(None);
        };
        let subject = store
            .kind(value.predicate.ty)
            .ok_or("invalid outlives assumption subject")?;
        if !matches!(shorter.bound_parts(), Some((0, _)))
            || matches!(subject, Kind::Projection(_) | Kind::Opaque(..))
        {
            return Ok(Some(self));
        }
        Ok(
            matches!(subject, Kind::Parameter(_)).then_some(Self::Predicate(Predicate {
                ty: value.predicate.ty,
                bound: Bound::Outlives(RegionId::STATIC),
            })),
        )
    }

    /// Classify a lifetime obligation without exposing an unbound predicate body.
    /// The caller must still introduce or instantiate its complete binder.
    /// # Errors
    /// Rejects an unknown quantified identity.
    pub fn is_outlives(self, store: &Store) -> Result<bool, &'static str> {
        let bound = match self {
            Self::Predicate(predicate) => predicate.bound,
            Self::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid lifetime clause")?
                    .predicate
                    .bound
            }
        };
        Ok(matches!(bound, super::predicates::Bound::Outlives(_)))
    }

    /// Nonempty quantifiers defer satisfaction until use even when their slots
    /// are unused. Empty quantifiers still retain their structural binder depth.
    /// # Errors
    /// Rejects malformed clauses and invalid predicate inputs.
    pub fn requires_definition_proof(self, store: &Store) -> Result<bool, &'static str> {
        match self {
            Self::Predicate(predicate) => predicate.requires_definition_proof(store),
            Self::ForAll(id) => {
                let value = store
                    .quantified(id)
                    .ok_or("invalid definition bound clause")?;
                Ok(value.bound_regions == 0 && value.predicate.requires_definition_proof(store)?)
            }
        }
    }

    /// Flat consumers must reject quantified input until they own its complete scope.
    /// # Errors
    /// Rejects a quantified clause rather than exposing its unbound body.
    pub fn ordinary(self) -> Result<Predicate, &'static str> {
        match self {
            Self::Predicate(predicate) => Ok(predicate),
            Self::ForAll(_) => Err("quantified clause requires a scoped consumer"),
        }
    }

    /// Lifetime quantification does not bind type or const parameters.
    /// # Errors
    /// Rejects malformed clauses, types or trait applications.
    pub fn contains_parameters(self, store: &Store) -> Result<bool, &'static str> {
        match self {
            Self::Predicate(predicate) => predicate.contains_parameters(store),
            Self::ForAll(id) => store
                .quantified(id)
                .ok_or("invalid quantified clause")?
                .predicate
                .contains_parameters(store),
        }
    }

    /// Local quantified slots do not require substitution of the caller's scope.
    /// # Errors
    /// Rejects unknown clause, type or application identities.
    pub fn has_free_regions(self, store: &Store) -> Result<bool, &'static str> {
        match self {
            Self::Predicate(predicate) => predicate.has_free_regions(store),
            Self::ForAll(id) => store
                .quantified(id)
                .ok_or("invalid quantified clause")?
                .predicate
                .has_regions_outside(store, 1),
        }
    }

    /// Count captures outside this clause, excluding its local quantified binder.
    /// # Errors
    /// Rejects unknown clause, type or application identities.
    pub fn escaping_binders(self, store: &Store) -> Result<u16, &'static str> {
        match self {
            Self::Predicate(predicate) => predicate.escaping_binders(store),
            Self::ForAll(id) => Ok(store
                .quantified(id)
                .ok_or("invalid quantified clause")?
                .predicate
                .escaping_binders(store)?
                .saturating_sub(1)),
        }
    }
}

/// The body is written inside one binder, including when no local slot occurs.
/// Enclosing captures therefore retain their depth through an empty binder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quantified {
    pub bound_regions: u32,
    pub predicate: Predicate,
}

#[derive(Default)]
pub(in crate::types) struct Storage {
    values: Vec<Quantified>,
    index: HashMap<Quantified, QuantifiedId>,
    slots: Slots,
    comparisons: Option<Box<equivalence::Cache>>,
}

impl Store {
    /// Canonical predicate binders in identity order, without reopening any scope.
    #[must_use]
    pub fn quantified_clauses(&self) -> &[Quantified] {
        self.arguments
            .as_ref()
            .and_then(|storage| storage.clauses.as_ref())
            .map_or(&[], |storage| storage.values.as_slice())
    }

    #[must_use]
    pub fn quantified(&self, id: QuantifiedId) -> Option<Quantified> {
        self.arguments
            .as_ref()?
            .clauses
            .as_ref()?
            .values
            .get(id.0 as usize)
            .copied()
    }

    /// Intern a canonical binder after validating its local slot references.
    /// This validates representation only, not formation, satisfaction or scope
    /// permissions. Declaration checking supplies the lifetime slot ordering.
    /// # Errors
    /// Rejects unknown children, unbound locals, exhausted occurrence work and
    /// counts or identities outside the compact representation.
    pub fn intern_quantified(&mut self, value: Quantified) -> Result<QuantifiedId, &'static str> {
        if let Some(&id) = self
            .arguments
            .as_ref()
            .and_then(|storage| storage.clauses.as_ref())
            .and_then(|storage| storage.index.get(&value))
        {
            return Ok(id);
        }
        if value.bound_regions > u32::from(u16::MAX) + 1 {
            return Err("too many quantified lifetime slots");
        }
        let (required, _) = self.clause_local_slots(value.predicate)?;
        if required > value.bound_regions {
            return Err("quantified predicate references an unbound lifetime");
        }
        let storage = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.clauses.as_mut())
            .ok_or("missing quantified predicate storage")?;
        let id = QuantifiedId(
            u32::try_from(storage.values.len()).map_err(|_| "too many quantified predicates")?,
        );
        storage.values.push(value);
        storage.index.insert(value, id);
        Ok(id)
    }

    /// Occurrence summaries belong to the immutable type graph, independently
    /// of a clause's binder arity or its permitted fixed comparison prefix.
    pub(crate) fn clause_local_slots(
        &mut self,
        predicate: Predicate,
    ) -> Result<(u32, usize), &'static str> {
        let mut slots = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.clauses.as_mut())
            .map_or_else(Slots::default, |storage| std::mem::take(&mut storage.slots));
        let before = slots.steps;
        let required = slots.predicate(self, predicate);
        let work = slots.steps.saturating_sub(before).saturating_add(1);
        let storage = self
            .arguments
            .get_or_insert_with(Default::default)
            .clauses
            .get_or_insert_with(Default::default);
        storage.slots = slots;
        required.map(|required| (required, work))
    }
}
