//! Paid spellings acquire only the authentic consuming Store's canonical identity.
use super::{Budget, Error, Store, error};
use crate::types::parameters::features::FeatureId;
use std::cell::RefCell;

pub(in crate::sema::crates::typed_headers) fn intern(
    store: &mut Store,
    name: &str,
    budget: &mut impl Budget,
) -> Result<FeatureId, Error> {
    paid(budget, |pay| {
        store.intern_feature_charged(name, &|steps, bytes| pay(steps, bytes))
    })
}

pub(in crate::sema::crates::typed_headers) fn with_clauses(
    environment: crate::types::parameters::environment::Environment,
    store: &Store,
    clauses: &[crate::types::parameters::clauses::Clause],
    budget: &mut impl Budget,
) -> Result<crate::types::parameters::environment::Environment, Error> {
    paid(budget, |pay| {
        environment.with_clauses_charged(store, clauses, &|steps, bytes| pay(steps, bytes))
    })
}
pub(in crate::sema::crates::typed_headers) fn paid<T>(
    budget: &mut impl Budget,
    action: impl FnOnce(crate::support::retained::Payment<'_>) -> Result<T, &'static str>,
) -> Result<T, Error> {
    let budget = RefCell::new(budget);
    let failure = RefCell::new(None);
    let result = action(&|steps, bytes| {
        let mut budget = budget
            .try_borrow_mut()
            .map_err(|_| "reentrant feature accounting")?;
        if let Err(error) = budget.charge(steps).and_then(|()| budget.reserve(bytes)) {
            *failure
                .try_borrow_mut()
                .map_err(|_| "reentrant feature failure accounting")? = Some(error);
            return Err("feature transport payment failed");
        }
        Ok(())
    });
    if let Some(error) = failure.into_inner() {
        return Err(error);
    }
    result.map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Account {
        steps: usize,
        bytes: usize,
        denied: bool,
    }
    impl Budget for Account {
        fn charge(&mut self, steps: usize) -> Result<(), Error> {
            self.steps = self.steps.saturating_add(steps);
            if self.denied {
                Err(error("denied feature work"))
            } else {
                Ok(())
            }
        }
        fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
            self.bytes = self.bytes.saturating_add(bytes);
            if self.denied {
                Err(error("denied feature storage"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn fresh_names_are_paid_once_foreign_ids_fail_and_warm_growth_is_bounded()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut previous = (0, 0);
        for count in [8, 32, 128] {
            let mut store = Store::default();
            let other = Store::default();
            let mut account = Account::default();
            let mut ids = Vec::new();
            for index in 0..count {
                ids.push(intern(
                    &mut store,
                    &format!("feature_{index:03}"),
                    &mut account,
                )?);
            }
            let cold = (account.steps, account.bytes);
            for (index, id) in ids.iter().enumerate() {
                assert_eq!(
                    intern(&mut store, &format!("feature_{index:03}"), &mut account)?,
                    *id
                );
                assert_eq!(
                    store.feature_name(*id),
                    Some(format!("feature_{index:03}").as_str())
                );
                assert!(other.feature_name(*id).is_none());
            }
            assert_eq!(
                account.bytes, cold.1,
                "warm canonical names allocate no new storage"
            );
            assert!(account.steps - cold.0 < count * 64);
            if previous.0 != 0 {
                assert!(cold.0 <= previous.0 * 6);
                assert!(cold.1 <= previous.1 * 6);
            }
            println!(
                "v4 names{count}: cold={cold:?} warmsteps={}",
                account.steps - cold.0
            );
            previous = cold;
        }
        let mut denied = Account {
            denied: true,
            ..Account::default()
        };
        assert!(intern(&mut Store::default(), "rejected_feature", &mut denied).is_err());
        assert!(denied.steps > 0);
        Ok(())
    }
}
