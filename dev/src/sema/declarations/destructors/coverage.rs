//! Drop covers the complete nominal schema, independent of impl-side assumptions.
use super::super::{Checker, DefId};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store,
    arguments::{Argument, ConstKind},
};
use std::collections::HashSet;

pub(in super::super) struct Coverage {
    pub(super) arguments: GenericArgs,
    pub(super) regions: Vec<RegionId>,
}

impl Checker<'_> {
    pub(super) fn drop_target_coverage(
        &mut self,
        owner: DefId,
        item: ItemId,
        arguments: GenericArgs,
        lifetimes: u32,
    ) -> Result<Coverage, &'static str> {
        let Some(Kind::Adt(_, nominal)) =
            self.facts.definitions[item.0 as usize].and_then(|ty| self.facts.types.kind(ty))
        else {
            return Err("missing nominal Drop schema");
        };
        let declared = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        let mut inverse = vec![None; declared.len()];
        let mut regions = vec![None; lifetimes as usize];
        let types = &self.facts.types;
        let actual = types
            .arguments(arguments)
            .ok_or("missing Drop target arguments")?;
        let nominal = types
            .arguments(nominal)
            .ok_or("missing nominal Drop arguments")?;
        if actual.len() != nominal.len() {
            return Err("Drop target argument arity mismatch");
        }
        let mut seen = HashSet::new();
        let mut nominal_lifetimes = 0;
        for (actual, nominal) in actual.zip(nominal) {
            if let (Argument::Lifetime(actual), Argument::Lifetime(nominal)) = (actual, nominal) {
                let Some((0, index)) = actual.bound_parts() else {
                    return Err(
                        "Drop target requires distinct lifetime parameters without specialization",
                    );
                };
                let slot = regions.get_mut(usize::from(index)).ok_or(
                    "Drop target requires distinct lifetime parameters without specialization",
                )?;
                if slot.replace(nominal).is_some() {
                    return Err(
                        "Drop target requires distinct lifetime parameters without specialization",
                    );
                }
                nominal_lifetimes += 1;
            } else {
                let index = parameter_index(types, owner, actual)?;
                if !seen.insert(index) {
                    return Err(
                        "Drop target requires distinct type and const parameters without specialization",
                    );
                }
                *inverse
                    .get_mut(index)
                    .ok_or("invalid Drop parameter slot")? = Some(nominal);
            }
        }
        let mut next = nominal_lifetimes;
        let regions = regions
            .into_iter()
            .map(|region| {
                if let Some(region) = region {
                    return Ok(region);
                }
                let slot = u16::try_from(next).map_err(|_| "too many Drop lifetime parameters")?;
                next += 1;
                RegionId::bound(0, slot)
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (parameter, slot) in declared.iter().zip(&mut inverse) {
            if let Argument::Lifetime(region) = parameter.argument {
                let Some((0, index)) = region.bound_parts() else {
                    return Err("invalid Drop lifetime parameter");
                };
                *slot = Some(Argument::Lifetime(
                    *regions
                        .get(usize::from(index))
                        .ok_or("missing Drop lifetime slot")?,
                ));
            }
        }
        let inverse = inverse
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or("Drop impl has unconstrained type or const parameters")?;
        let arguments = self.facts.types.intern_arguments(&inverse)?;
        Ok(Coverage { arguments, regions })
    }
}

fn parameter_index(types: &Store, owner: DefId, argument: Argument) -> Result<usize, &'static str> {
    let id = match argument {
        Argument::Type(ty) => match types.kind(ty) {
            Some(Kind::Parameter(id)) => Some(id),
            _ => None,
        },
        Argument::Const(id) => match types.const_kind(id) {
            Some(ConstKind::Parameter(id)) => Some(id),
            _ => None,
        },
        Argument::Lifetime(_) => None,
    }
    .ok_or("Drop target requires distinct type and const parameters without specialization")?;
    let parameter = types
        .parameter(id)
        .ok_or("missing Drop parameter identity")?;
    if parameter.owner != ItemId(owner.0) {
        return Err("Drop target parameter belongs to another declaration");
    }
    Ok(parameter.index as usize)
}

impl Checker<'_> {
    pub(super) fn validate_drop_traits(&mut self, index: usize) -> Result<(), super::Error> {
        use crate::types::parameters::{
            ParameterKind,
            predicates::{Bound, Predicate},
        };
        let implementation = &self.facts.destructors.implementations()[index];
        let (owner, method, item, key) = (
            implementation.self_definition,
            implementation.method,
            implementation.item,
            implementation.source,
        );
        if self
            .facts
            .derives
            .get(&item)
            .is_some_and(|properties| properties.copy)
        {
            return Err(self.error(key, "a type cannot implement both Copy and Drop"));
        }
        let coverage = self
            .drop_coverage
            .remove(&owner)
            .ok_or_else(|| self.error(key, "missing Drop coverage mapping"))?;
        let mut substitution = crate::types::substitute::Bound::joint_captured(
            &self.facts.types,
            ItemId(owner.0),
            coverage.arguments,
            coverage.regions,
        )
        .map_err(|e| self.error(key, e))?;
        let mut requirements = self
            .facts
            .generics
            .predicates_in_scope(method)
            .filter(|p| !matches!(p.value.bound, Bound::Outlives(_)))
            .map(|p| p.value)
            .collect::<Vec<_>>();
        for parameter in self.facts.generics.parameters_in_scope(owner) {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Kind::Parameter(id)) = self.facts.types.kind(ty)
                && matches!(
                    self.facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                )
            {
                requirements.push(Predicate {
                    ty,
                    bound: Bound::Sized,
                });
            }
        }
        let environment = self
            .facts
            .generics
            .environments
            .get(&DefId(item.0))
            .cloned();
        let mut proof = crate::sema::nominals::Proof::new(environment);
        for mut requirement in requirements {
            requirement.ty = substitution
                .apply(&mut self.facts.types, requirement.ty)
                .map_err(|e| self.error(key, e))?;
            if !proof
                .prove(&mut self.facts, requirement)
                .map_err(|e| self.error(key, e))?
            {
                return Err(self.error(key, "Drop impl adds type requirements to its nominal type"));
            }
        }
        Ok(())
    }
}
