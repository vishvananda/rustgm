//! Complete capture applications share the ordinary simultaneous substitution.
use super::{Declaration, InferredCaptures};
use crate::types::{
    GenericArgs, Kind, RegionId, Store, arguments::Argument,
    infer::arguments::Argument as Inferred, parameters::argument_id, substitute::Bound,
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

    pub(crate) fn inferred_arguments(
        &self,
        store: &Store,
        actual: &[Inferred],
    ) -> Result<InferredCaptures, &'static str> {
        let Some(Kind::Opaque(_, formal)) = store.kind(self.application) else {
            return Err("invalid inferred opaque capture template");
        };
        let formal = store
            .arguments(formal)
            .ok_or("invalid opaque capture arguments")?;
        if formal.len() != actual.len() {
            return Err("inferred opaque capture arity mismatch");
        }
        let mut regions = Vec::new();
        let mut bindings = Vec::new();
        for (formal, &actual) in formal.zip(actual) {
            match (formal, actual) {
                (Argument::Lifetime(formal), Inferred::Lifetime(actual)) => {
                    set_region(&mut regions, formal, actual)?;
                }
                (formal, actual) => bindings.push((argument_id(store, formal)?, actual)),
            }
        }
        Ok(InferredCaptures {
            regions: Bound::new(regions),
            parameters: bindings,
        })
    }

    pub(crate) fn substitution(
        &self,
        store: &Store,
        arguments: GenericArgs,
    ) -> Result<Bound, &'static str> {
        let Some(Kind::Opaque(_, captures)) = store.kind(self.application) else {
            return Err("invalid opaque declaration application");
        };
        Bound::captured_arguments(store, captures, arguments)
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
