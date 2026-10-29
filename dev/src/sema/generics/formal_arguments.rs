//! Selected formals retain their separate implementation lifetime capture prefix.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{GenericArgs, arguments::Argument, parameters::ParameterId, substitute::Bound},
};

impl Declarations {
    pub(in crate::sema) fn selected_implementation_substitution(
        &self,
        owner: DefId,
        arguments: GenericArgs,
        pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(Bound, Vec<ParameterId>), &'static str> {
        let captures = self
            .associated
            .schema(owner)
            .ok_or("defining implementation schema is missing")?
            .captures as usize;
        let values = self
            .types
            .arguments(arguments)
            .ok_or("defining implementation arguments are missing")?;
        let count = values.len();
        let parameters = self.generics.parameters.get(&owner).map_or(0, Vec::len);
        pay(
            1 + count + parameters,
            count
                .saturating_mul(4 * size_of::<Argument>())
                .saturating_add(parameters.saturating_mul(4 * size_of::<ParameterId>())),
        )?;
        let values: Vec<_> = values.collect();
        let ids = self.generics.parameter_ids(&self.types, owner)?;
        if count != captures.saturating_add(ids.len()) {
            return Err("defining implementation argument arity changed");
        }
        let regions = values[..captures]
            .iter()
            .map(|argument| match argument {
                Argument::Lifetime(region) => Ok(*region),
                _ => Err("defining capture is not a lifetime"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bindings: Vec<_> = ids
            .iter()
            .copied()
            .zip(values[captures..].iter().copied())
            .collect();
        Ok((
            Bound::parameters_captured_charged(&self.types, &bindings, regions, pay)?,
            ids,
        ))
    }
}
