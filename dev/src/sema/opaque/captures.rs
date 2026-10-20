//! Complete capture applications share the ordinary simultaneous substitution.
use super::Declaration;
use crate::types::{
    GenericArgs, Kind, RegionId, Store, arguments::Argument, parameters::argument_id,
    substitute::Bound,
};

impl Declaration {
    pub(crate) fn check_hidden_captures(
        &self,
        store: &Store,
        hidden: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        let mut captured = crate::types::bound_uses::Uses::default();
        captured.collect(store, self.application)?;
        let mut required = crate::types::bound_uses::Uses::default();
        required.collect(store, hidden)?;
        if !required.indices.is_subset(&captured.indices) {
            return Err("opaque hidden type uses an excluded lifetime capture");
        }
        Ok(())
    }

    pub(crate) fn substitution(
        &self,
        store: &Store,
        arguments: GenericArgs,
    ) -> Result<Bound, &'static str> {
        let Some(Kind::Opaque(_, captures)) = store.kind(self.application) else {
            return Err("invalid opaque declaration application");
        };
        let captures = store.arguments(captures).ok_or("invalid opaque captures")?;
        let arguments = store
            .arguments(arguments)
            .ok_or("invalid opaque arguments")?;
        if captures.len() != arguments.len() {
            return Err("opaque capture arity mismatch");
        }
        let mut regions = Vec::new();
        let mut bindings = Vec::new();
        for (parameter, actual) in captures.zip(arguments) {
            match (parameter, actual) {
                (Argument::Lifetime(parameter), Argument::Lifetime(actual)) => {
                    let Some((0, slot)) = parameter.bound_parts() else {
                        return Err("invalid opaque lifetime capture");
                    };
                    regions.resize(regions.len().max(usize::from(slot) + 1), RegionId::STATIC);
                    regions[usize::from(slot)] = actual;
                }
                (parameter, actual) => {
                    bindings.push((argument_id(store, parameter)?, actual));
                }
            }
        }
        Bound::parameters_captured(store, &bindings, regions)
    }
}

impl super::Implementation {
    /// Canonical and inferred selections share the same complete method-capture map.
    pub(super) fn method_inputs<A: Copy>(
        &self,
        store: &Store,
        declaration: &Declaration,
        actual: impl ExactSizeIterator<Item = A>,
        bindings: &mut Vec<(crate::types::parameters::ParameterId, A)>,
        regions: &mut Vec<RegionId>,
        lifetime: impl Fn(A) -> Option<RegionId>,
    ) -> Result<(), &'static str> {
        let Some(Kind::Opaque(_, formal)) = store.kind(declaration.application) else {
            return Err("invalid trait result template");
        };
        let formal = store
            .arguments(formal)
            .ok_or("invalid result capture template")?;
        if formal.len() != actual.len() {
            return Err("selected result capture arity mismatch");
        }
        for (formal, actual) in formal.zip(actual) {
            if let Argument::Lifetime(formal) = formal {
                let Some((0, slot)) = formal.bound_parts() else {
                    return Err("invalid captured method lifetime");
                };
                if u32::from(slot) >= self.parent_regions {
                    let target = *self
                        .regions
                        .get(usize::from(slot))
                        .ok_or("missing result lifetime coordinate")?;
                    let actual =
                        lifetime(actual).ok_or("result lifetime argument kind mismatch")?;
                    set_region(regions, target, actual)?;
                }
            } else {
                let parameter = argument_id(store, formal)?;
                if store
                    .parameter(parameter)
                    .ok_or("invalid captured method parameter")?
                    .owner
                    == crate::types::ItemId(declaration.owner.0)
                {
                    bindings.push((parameter, actual));
                }
            }
        }
        Ok(())
    }
}

pub(super) fn set_region(
    regions: &mut Vec<RegionId>,
    formal: RegionId,
    actual: RegionId,
) -> Result<(), &'static str> {
    let Some((0, slot)) = formal.bound_parts() else {
        return Err("invalid result lifetime mapping");
    };
    regions.resize(regions.len().max(usize::from(slot) + 1), RegionId::STATIC);
    regions[usize::from(slot)] = actual;
    Ok(())
}
