//! A concrete body grants only its authenticated non-type feature membership.
use super::{Arc, Assumptions, Caller, Dispatcher, Environment};
use crate::types::Store;
impl Dispatcher {
    pub(crate) fn in_body(
        types: &Store,
        universals: u32,
        first: u32,
        environment: Arc<Environment>,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, String> {
        let mut result = Self::new(universals, first)?;
        if !environment.has_feature_assumptions() {
            return Ok(result);
        }
        let work = std::cell::Cell::new(0_usize);
        let payer = |steps: usize, bytes| {
            work.set(work.get().saturating_add(steps));
            pay(steps, bytes)
        };
        let assumptions = Assumptions::feature_environment_charged(types, &environment, &payer)?;
        payer(1, size_of::<Caller>())?;
        result
            .context
            .account_work(work.get())
            .map_err(|error| error.to_string())?;
        result.caller = Some(Box::new(Caller {
            shared_environment: Some(environment),
            environment: None,
            assumptions,
        }));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        GenericArgs, ItemId, Primitive, TypeId,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    };
    use std::cell::Cell;
    #[test]
    fn feature_snapshot_is_paid_shared_and_grants_no_symbolic_trait()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut warm = Vec::new();
        for width in [8, 32, 128] {
            let mut types = Store::default();
            let application = types.intern_trait_application(Application {
                item: ItemId(1),
                arguments: GenericArgs::EMPTY,
            })?;
            let ordinary = Predicate {
                ty: TypeId::primitive(Primitive::U8),
                bound: Bound::Trait(application),
            };
            let mut clauses = vec![Clause::Predicate(ordinary)];
            let mut ids = Vec::new();
            for ordinal in 0..width {
                let id =
                    types.intern_feature_charged(&format!("dispatch_{ordinal}"), &|_, _| Ok(()))?;
                clauses.push(Clause::UnstableFeature(id));
                ids.push(id);
            }
            let environment = Arc::new(Environment::new(&types, &[])?.with_clauses_charged(
                &types,
                &clauses,
                &|_, _| Ok(()),
            )?);
            let cold = Cell::new(0_usize);
            let first =
                Dispatcher::in_body(&types, 0, 1, Arc::clone(&environment), &|_, bytes| {
                    cold.set(cold.get().saturating_add(bytes));
                    Ok(())
                })?;
            let caller = first.caller.as_deref().ok_or("actual feature caller")?;
            assert!(caller.environment.is_none());
            assert!(
                caller
                    .assumptions
                    .contains_feature(&types, ids[0], &|_, _| Ok(()))?
            );
            assert!(
                caller
                    .assumptions
                    .prove_canonical(
                        &mut types,
                        &mut crate::types::infer::Context::default(),
                        ordinary
                    )?
                    .is_none()
            );
            let bytes = Cell::new(0_usize);
            let work = Cell::new(0_usize);
            let second =
                Dispatcher::in_body(&types, 0, 1, Arc::clone(&environment), &|steps, amount| {
                    work.set(work.get().saturating_add(steps));
                    bytes.set(bytes.get().saturating_add(amount));
                    Ok(())
                })?;
            assert!(
                second
                    .caller
                    .as_deref()
                    .ok_or("warm caller")?
                    .assumptions
                    .contains_feature(&types, ids[width - 1], &|_, _| Ok(()))?
            );
            assert!(cold.get() > bytes.get());
            warm.push((work.get(), bytes.get()));
            let attempted = Cell::new(false);
            assert!(
                Dispatcher::in_body(&types, 0, 1, Arc::clone(&environment), &|_, allocation| {
                    if allocation > 0 {
                        attempted.set(true);
                        Err("view allocation")
                    } else {
                        Ok(())
                    }
                })
                .is_err()
            );
            assert!(attempted.get());
            assert!(
                Dispatcher::in_body(&Store::default(), 0, 1, environment, &|_, _| Ok(())).is_err()
            );
        }
        assert!(warm.windows(2).all(|pair| pair[0] == pair[1]), "{warm:?}");
        let types = Store::default();
        let empty = Arc::new(Environment::new(&types, &[])?);
        let ordinary = Dispatcher::in_body(&types, 0, 1, empty, &|_, _| {
            Err("feature-free caller must allocate no scope")
        })?;
        assert!(ordinary.caller.is_none());
        Ok(())
    }
}
