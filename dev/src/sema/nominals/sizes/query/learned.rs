//! Structural results carry one immutable caller, separately from actual trait evidence.
use super::{Environment, Query, TypeId};
use crate::{
    sema::traits::implementations::Availability,
    support::hash::HashMap,
    types::{
        Store,
        parameters::shape::{Placeholders, Properties, Shapes},
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::sema) struct Owner {
    pub environment: Option<Arc<Environment>>,
    pub placeholders: Option<Placeholders>,
    pub availability: Availability,
}
impl Owner {
    pub fn matches(
        &self,
        environment: Option<&Arc<Environment>>,
        placeholders: Option<&Placeholders>,
        availability: Availability,
    ) -> bool {
        if availability != self.availability {
            return false;
        }
        match (self.environment.as_ref(), environment) {
            (Some(left), Some(right)) => {
                self.placeholders.is_none() && placeholders.is_none() && Arc::ptr_eq(left, right)
            }
            (None, None) => match (self.placeholders.as_ref(), placeholders) {
                (Some(left), Some(right)) => left.same_owner(right),
                (None, None) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

/// The caller already completed these physical properties. A source builtin
/// still proves its own nominal formation before reading them.
pub(in crate::sema) struct Physical {
    owner: Owner,
    answers: HashMap<TypeId, Properties>,
}
impl Physical {
    pub fn capture(
        owner: &Owner,
        types: &Store,
        shapes: &mut Shapes,
        root: TypeId,
        normalized: bool,
    ) -> Result<(Option<Box<Self>>, usize), &'static str> {
        let completed = shapes.snapshot_properties(types, root, normalized)?;
        let work = completed.steps;
        if completed.properties.is_empty() {
            return Ok((None, work));
        }
        Ok((
            Some(Box::new(Self {
                owner: owner.clone(),
                answers: completed.properties.into_iter().collect(),
            })),
            work,
        ))
    }
    pub fn get(
        &self,
        ty: TypeId,
        environment: Option<&Arc<Environment>>,
        placeholders: Option<&Placeholders>,
        availability: Availability,
    ) -> Option<Properties> {
        self.owner
            .matches(environment, placeholders, availability)
            .then(|| self.answers.get(&ty).copied())
            .flatten()
    }
}

/// Completed physical sizes are not trait permissions. Actual selected proofs
/// travel separately and remain mandatory before callers can use these sizes.
pub(in crate::sema) struct Learned {
    owner: Owner,
    answers: HashMap<TypeId, bool>,
}
impl Learned {
    pub fn new(owner: Owner) -> Self {
        Self {
            owner,
            answers: HashMap::default(),
        }
    }
    pub fn belongs_to(&self, owner: &Owner) -> bool {
        self.owner.matches(
            owner.environment.as_ref(),
            owner.placeholders.as_ref(),
            owner.availability,
        )
    }
    pub fn insert(&mut self, ty: TypeId, sized: bool) -> Result<(), &'static str> {
        if let Some(&previous) = self.answers.get(&ty) {
            return if previous == sized {
                Ok(())
            } else {
                Err("completed size answer changed under an immutable caller")
            };
        }
        if self.answers.len() >= 1_000_000 {
            return Err("completed size answer storage budget exceeded");
        }
        self.answers.insert(ty, sized);
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.answers.len()
    }
    /// The receiver has checked the immutable caller before merging.
    pub fn extend(&mut self, other: Self) -> Result<(), &'static str> {
        if !other.belongs_to(&self.owner) {
            return Err("completed size answers changed their caller");
        }
        for (ty, sized) in other.answers {
            self.insert(ty, sized)?;
        }
        Ok(())
    }
}
impl Query {
    /// Wrong environments, premise handles and implementation views get no
    /// answers. A complete result can replace an earlier provisional answer.
    pub(in crate::sema) fn seed(
        &mut self,
        learned: Learned,
        availability: Availability,
    ) -> Result<(), &'static str> {
        if !learned.owner.matches(
            self.environment.as_ref(),
            self.placeholders.as_ref(),
            availability,
        ) {
            return Ok(());
        }
        for (ty, sized) in learned.answers {
            self.steps = self.steps.saturating_add(1);
            if self.steps > 1_000_000 {
                return Err("nominal size query work budget exceeded");
            }
            if let Some(previous) = self.answers.get(&ty)
                && !previous.provisional
                && previous.sized != sized
            {
                return Err("completed size answer changed under an immutable caller");
            }
            self.known(ty, sized);
        }
        Ok(())
    }
}
