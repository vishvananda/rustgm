//! Applied trait inputs share one fold and one canonical requirement cache.
use super::{Contracts, Header, Requirements, effects, parameter_id};
use crate::{
    support::retained::Growth,
    types::{
        Store,
        arguments::Argument,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
use std::sync::Arc;
impl Contracts {
    pub fn requirements(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
    ) -> Result<Arc<Requirements>, &'static str> {
        self.requirements_observed::<false>(store, predicate, &|_, _| Ok(()))
    }
    pub(in crate::sema) fn requirements_counted(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<Requirements>, &'static str> {
        self.requirements_observed::<true>(store, predicate, charge)
    }
    fn requirements_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<Requirements>, &'static str> {
        if CHARGED {
            charge(1, 0)?;
        }
        if let Some(result) = self.applications.get(&predicate) {
            return Ok(Arc::clone(result));
        }
        let Some((id, effect)) = predicate.bound.trait_parts() else {
            return Err("trait contract requires a trait application");
        };
        let application = store
            .trait_application(id)
            .ok_or("invalid trait application")?;
        let header = self
            .headers
            .get(&application.item)
            .ok_or("missing trait contract")?;
        let arguments = store
            .arguments(application.arguments)
            .ok_or("invalid trait arguments")?;
        if arguments.len() != header.parameters.len() {
            return Err("trait contract argument arity mismatch");
        }
        self.steps = self.steps.saturating_add(
            1 + arguments.len() + header.inputs.view().len() + header.results.len(),
        );
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        if CHARGED {
            let mut growth = Growth::vector::<(crate::types::parameters::ParameterId, Argument)>(
                0,
                0,
                header.parameters.len().saturating_add(1),
            );
            growth.include(Growth::vector::<crate::types::RegionId>(
                0,
                0,
                header.parameters.len(),
            ));
            growth.include(Growth::vector::<Clause>(
                0,
                0,
                header
                    .inputs
                    .view()
                    .len()
                    .saturating_add(header.parameters.len()),
            ));
            growth.include(Growth::vector::<Clause>(0, 0, header.inputs.view().len()));
            growth.include(Growth::vector::<Clause>(0, 0, header.results.len()));
            growth.include(Growth::vector::<crate::types::TypeId>(
                0,
                0,
                header.parameters.len(),
            ));
            growth.include(Growth::vector::<crate::types::outlives::Pair>(
                0,
                0,
                header.regions.len(),
            ));
            charge(
                1 + arguments.len()
                    + header.inputs.view().len()
                    + header.results.len()
                    + growth.work,
                growth.bytes,
            )?;
        }
        let self_parameter = parameter_id(store, Argument::Type(header.self_type))?;
        let mut mapping = vec![(self_parameter, Argument::Type(predicate.ty))];
        let mut regions = Vec::new();
        let mut result = Requirements {
            implied: Vec::new(),
            results: Vec::new(),
            required: Vec::new(),
            types: Vec::new(),
            regions: Vec::new(),
        };
        for (parameter, argument) in header.parameters.iter().zip(arguments) {
            if let (Argument::Lifetime(_), Argument::Lifetime(region)) =
                (parameter.argument, argument)
            {
                regions.push(region);
                continue;
            }
            let id = parameter_id(store, parameter.argument)?;
            mapping.push((id, argument));
            if let Argument::Type(ty) = argument {
                result.types.push(ty);
                if matches!(
                    store.parameter(id).map(|p| p.kind),
                    Some(crate::types::parameters::ParameterKind::Type { sized: true })
                ) {
                    result.required.push(
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        }
                        .into(),
                    );
                }
            }
        }
        let mut replace = if CHARGED {
            substitute::Bound::parameters_captured_charged(store, &mapping, regions, charge)?
        } else {
            substitute::Bound::parameters_captured(store, &mapping, regions)?
        };
        let rows = header.instantiate_observed::<CHARGED>(
            store,
            effect,
            &mut replace,
            &mut result,
            charge,
        );
        if !CHARGED {
            rows?;
        }
        if CHARGED {
            charge(replace.steps, 0)?;
        }
        self.steps = self.steps.saturating_add(replace.steps);
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        rows?;
        if CHARGED {
            let growth = Growth::map::<Predicate, Arc<Requirements>>(
                self.applications.len(),
                self.applications.capacity(),
                1,
            );
            charge(
                growth.work,
                growth
                    .bytes
                    .saturating_add(size_of::<Requirements>())
                    .saturating_add(2 * size_of::<usize>()),
            )?;
        }
        let result = Arc::new(result);
        self.applications.insert(predicate, Arc::clone(&result));
        Ok(result)
    }
}

impl Header {
    fn instantiate_observed<const CHARGED: bool>(
        &self,
        store: &mut Store,
        effect: crate::support::constness::Constness,
        replace: &mut substitute::Bound,
        result: &mut Requirements,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        for (index, predicate) in self.inputs.view().iter().enumerate() {
            let value = if CHARGED {
                replace.clause_charged(store, predicate.value, charge)?
            } else {
                replace.clause(store, predicate.value)?
            };
            let value = effects::in_context(store, value, effect)?;
            if self.implies(store, index, predicate)? {
                result.implied.push(value);
            } else {
                result.required.push(value);
            }
        }
        for (_, predicate) in &self.results {
            let value = if CHARGED {
                replace.clause_charged(store, predicate.value, charge)?
            } else {
                replace.clause(store, predicate.value)?
            };
            result
                .results
                .push(effects::in_context(store, value, effect)?);
        }
        for &(longer, shorter) in &self.regions {
            let (Argument::Lifetime(longer), Argument::Lifetime(shorter)) = (
                replace.apply_argument(store, Argument::Lifetime(longer))?,
                replace.apply_argument(store, Argument::Lifetime(shorter))?,
            ) else {
                return Err("invalid trait region substitution");
            };
            result.regions.push((longer, shorter));
        }
        Ok(())
    }
}
