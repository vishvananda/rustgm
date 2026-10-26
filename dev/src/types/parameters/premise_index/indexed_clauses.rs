//! One canonical insertion pays actual candidate storage before publishing a clause.
use super::*;
use crate::support::retained::Growth;
enum ChargedError {
    Semantic(infer::Error),
    Retention(&'static str),
}
impl From<infer::Error> for ChargedError {
    fn from(error: infer::Error) -> Self {
        Self::Semantic(error)
    }
}
impl ClauseIndex {
    pub(crate) fn insert_clause_charged(
        &mut self,
        store: &Store,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.insert_observed::<true, ChargedError>(store, clause, &|steps, bytes| {
            charge(steps, bytes).map_err(ChargedError::Retention)
        })
        .map_err(|error| match error {
            ChargedError::Semantic(infer::Error::Budget) => {
                "caller trait assumption index work budget exceeded"
            }
            ChargedError::Semantic(_) => "invalid caller trait assumption index",
            ChargedError::Retention(message) => message,
        })
    }
    pub(super) fn insert_observed<const CHARGED: bool, E: From<infer::Error>>(
        &mut self,
        store: &Store,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        if CHARGED {
            charge(1, 0)?;
        }
        if self.view.is_some() {
            return Err(infer::Error::InvalidIdentity.into());
        }
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or(infer::Error::InvalidIdentity)?
                    .predicate
            }
        };
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(infer::Error::InvalidIdentity.into());
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
            return Err(infer::Error::Budget.into());
        }
        let head = Head::canonical(
            store
                .kind(predicate.ty)
                .ok_or(infer::Error::InvalidIdentity)?,
        );
        let key = (application.item, head);
        if let Clause::ForAll(id) = clause {
            return self.insert_quantified::<CHARGED, E>(id, key, charge);
        }
        if !self.seen.contains(&predicate) {
            if CHARGED {
                let mut growth =
                    Growth::map::<Predicate, ()>(self.seen.len(), self.seen.capacity(), 1);
                growth.include(candidate_growth(&self.candidates, key));
                charge(growth.work, growth.bytes)?;
            }
            self.seen.insert(predicate);
            self.parameter_traits.take();
            self.candidates.entry(key).or_default().push(predicate);
        }
        Ok(())
    }
    fn insert_quantified<const CHARGED: bool, E>(
        &mut self,
        id: QuantifiedId,
        key: (ItemId, Head),
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        if CHARGED {
            let mut growth = self.quantified.as_ref().map_or_else(
                || {
                    let mut growth = Growth::map::<QuantifiedId, ()>(0, 0, 1);
                    growth.bytes = growth.bytes.saturating_add(size_of::<QuantifiedIndex>());
                    growth.include(candidate_growth::<QuantifiedId>(&HashMap::default(), key));
                    growth
                },
                |index| {
                    let mut growth = Growth::map::<QuantifiedId, ()>(
                        index.seen.len(),
                        index.seen.capacity(),
                        usize::from(!index.seen.contains(&id)),
                    );
                    if !index.seen.contains(&id) {
                        growth.include(candidate_growth(&index.candidates, key));
                    }
                    growth
                },
            );
            // A duplicate still pays its lookup; no owner or payload is allocated.
            growth.work = growth.work.saturating_add(1);
            charge(growth.work, growth.bytes)?;
        }
        let index = self.quantified.get_or_insert_with(Default::default);
        if index.seen.insert(id) {
            self.parameter_traits.take();
            index.candidates.entry(key).or_default().push(id);
        }
        Ok(())
    }
}
fn candidate_growth<T>(values: &HashMap<(ItemId, Head), Vec<T>>, key: (ItemId, Head)) -> Growth {
    let mut growth = Growth::map::<(ItemId, Head), Vec<T>>(
        values.len(),
        values.capacity(),
        usize::from(!values.contains_key(&key)),
    );
    let (length, capacity) = values
        .get(&key)
        .map_or((0, 0), |values| (values.len(), values.capacity()));
    growth.include(Growth::vector::<T>(length, capacity, 1));
    growth
}
