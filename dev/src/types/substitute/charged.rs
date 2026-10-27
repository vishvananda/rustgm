//! Call-scoped growth hooks share the ordinary fold without an owner on Bound.
use super::{Action, Bound, bounded};
use crate::{
    support::retained::{self, Growth},
    types::{Store, TypeId, arguments::Argument, parameters::ParameterId},
};
impl Bound {
    pub(crate) fn arguments_charged(
        &mut self,
        store: &mut Store,
        arguments: crate::types::GenericArgs,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<crate::types::GenericArgs, &'static str> {
        self.arguments_inside_charged(store, arguments, 0, charge)
    }
    pub(crate) fn arguments_inside_charged(
        &mut self,
        store: &mut Store,
        arguments: crate::types::GenericArgs,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<crate::types::GenericArgs, &'static str> {
        self.apply_arguments_observed::<true>(store, arguments, depth, charge)
    }
    pub(crate) fn argument_inside_charged(
        &mut self,
        store: &mut Store,
        argument: Argument,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Argument, &'static str> {
        self.apply_argument_observed::<true>(store, argument, depth, charge)
    }
    pub(crate) fn parameters_captured_charged(
        store: &Store,
        arguments: &[(ParameterId, Argument)],
        regions: Vec<crate::types::RegionId>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        charge(
            arguments.len(),
            size_of::<super::parameters::Bindings>()
                .saturating_add(retained::new_map::<ParameterId, Argument>(arguments.len())),
        )?;
        Self::parameters_captured(store, arguments, regions)
    }
    pub(crate) fn joint_charged(
        store: &Store,
        owner: crate::types::ItemId,
        arguments: crate::types::GenericArgs,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        let count = store
            .arguments(arguments)
            .ok_or("invalid charged substitution arguments")?
            .len();
        charge(
            count,
            size_of::<super::parameters::Bindings>().saturating_add(
                count.saturating_mul(size_of::<Argument>() + size_of::<crate::types::RegionId>()),
            ),
        )?;
        Self::joint(store, owner, arguments)
    }
    pub(in crate::types) fn apply_inside_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<TypeId, &'static str> {
        if !CHARGED {
            return self.apply_inside(store, ty, depth);
        }
        charge(1, 0)?;
        if !matches!(
            self.action,
            Action::Remove
                | Action::Shift(_)
                | Action::Lower(_)
                | Action::Shape { .. }
                | Action::Regions(_)
        ) {
            return Err("charged declaration fold requires an actual parameter/capture mapping");
        }
        if let Action::Regions(mapping) = &self.action
            && !mapping.permits_charged_observation()
        {
            return Err("charged region fold requires an actual observation or query opening");
        }
        let key = (ty, self.cache_depth(depth));
        if let Some(result) = self.result(store, key)? {
            return Ok(result);
        }
        let growth = Growth::vector::<(super::Key, bool)>(self.work.len(), self.work.capacity(), 1);
        charge(growth.work, growth.bytes)?;
        self.work.push((key, false));
        let result = self
            .drain_observed::<false, true>(store, &bounded::Budget::ordinary(), charge)
            .and_then(|()| {
                self.result(store, key)?
                    .ok_or("missing charged substituted root")
            });
        self.work.clear();
        result
    }
    pub(crate) fn apply_argument_charged(
        &mut self,
        store: &mut Store,
        argument: Argument,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Argument, &'static str> {
        self.apply_argument_observed::<true>(store, argument, 0, charge)
    }
    pub(super) fn apply_argument_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        argument: Argument,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Argument, &'static str> {
        if !CHARGED {
            return self.apply_argument_inside(store, argument, depth);
        }
        charge(1, 0)?;
        self.region_work(1)?;
        Ok(match argument {
            Argument::Lifetime(region) => {
                Argument::Lifetime(self.region_observed::<CHARGED>(region, depth, charge)?)
            }
            Argument::Type(ty) => {
                Argument::Type(self.apply_inside_observed::<true>(store, ty, depth, charge)?)
            }
            Argument::Const(id) => Argument::Const(self.constant(store, id)?),
        })
    }
}
