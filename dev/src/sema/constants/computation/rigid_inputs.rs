//! A completed caller owns rigid parameters; observing one grants no value proof.
use super::{Declarations, Instance, catalog};
use crate::{
    sema::callee_premises::Inputs,
    types::{Kind, arguments::Argument, parameters::ParameterId},
};
use std::{collections::HashSet, sync::Arc};

pub(super) struct Parameters {
    owner: catalog::CallerOwner,
    values: Vec<ParameterId>,
    index: HashSet<ParameterId>,
}
impl Parameters {
    pub(super) fn contains(&self, id: ParameterId) -> bool {
        self.index.contains(&id)
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(crate::support::retained::vector(&self.values))
            .saturating_add(crate::support::retained::new_map::<ParameterId, ()>(
                self.index.len(),
            ))
    }
    pub(super) fn validate(
        &self,
        facts: &Declarations,
        caller: catalog::CallerId,
        key: catalog::CallerKey,
    ) -> Result<(), &'static str> {
        facts
            .source_computations
            .as_deref()
            .ok_or("missing rigid source catalog")?
            .0
            .validate_caller_owner(self.owner, caller, key)
    }
}

pub(super) fn prepare(
    facts: &mut Declarations,
    key: catalog::CallerKey,
    publication: catalog::Publication,
    inputs: Inputs<'_>,
    selected: Instance,
) -> Result<Option<Arc<Parameters>>, &'static str> {
    let mut rigid = false;
    for argument in facts
        .types
        .arguments(selected.arguments)
        .ok_or("missing source arguments")?
    {
        facts.body_work.canonical_steps += 1;
        rigid |= matches!(argument, Argument::Type(ty) if matches!(facts.types.kind(ty), Some(Kind::Parameter(_))));
    }
    if !rigid {
        return Ok(None);
    }
    if !matches!(key.enclosing, catalog::Enclosing::Original)
        || inputs.source != key.source
        || inputs.universals != key.universals
    {
        return Err("rigid source parameters require their original prepared caller");
    }
    facts.body_work.canonical_steps += 2;
    let (signature, universals) =
        crate::sema::callee_premises::OriginalInputs::identity(facts, key.source, key.declaration)?;
    if signature != inputs.signature || universals != key.universals {
        return Err("rigid source parameters have another actual caller header");
    }
    if inputs.parameters.len() > 1_000_000 {
        return Err("rigid source parameter count budget exceeded");
    }
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source catalog")?
        .0;
    let (published, previous) = catalog.original(key.source)?;
    if published != key || previous != publication {
        return Err("rigid source parameters lack their exact completed caller publication");
    }
    let caller = catalog.caller(key, publication)?;
    if let Some(parameters) = catalog.parameters(caller)? {
        parameters.validate(facts, caller, key)?;
        return Ok(Some(parameters));
    }
    let owner = catalog.caller_owner(caller)?;
    let bytes = size_of::<Parameters>()
        .saturating_add(2 * size_of::<usize>())
        .saturating_add(
            inputs
                .parameters
                .len()
                .saturating_mul(2 * size_of::<ParameterId>()),
        )
        .saturating_add(crate::support::retained::new_map::<ParameterId, ()>(
            inputs.parameters.len(),
        ));
    catalog.namespace_budget(bytes)?;
    let declaration = key.declaration.ok_or("rigid caller has no declaration")?;
    let mut values = Vec::with_capacity(inputs.parameters.len());
    for parameter in facts.generics.parameters_in_scope(declaration) {
        facts.body_work.canonical_steps += 1;
        if let Argument::Type(ty) = parameter.argument {
            let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                return Err("rigid caller has an invalid declared parameter");
            };
            if values.len() >= inputs.parameters.len() {
                return Err("rigid caller input parameters omit a declaration");
            }
            values.push(id);
        }
    }
    if values != inputs.parameters {
        return Err("rigid caller input parameters belong to another declaration");
    }
    let index = values.iter().copied().collect();
    let parameters = Parameters {
        owner,
        values,
        index,
    };
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source catalog")?
        .0;
    catalog.retain_parameters(caller, parameters)?;
    let result = catalog.parameters(caller)?;
    facts.constant_work.source_context_bytes = catalog.retained_bytes();
    Ok(result)
}

#[cfg(test)]
#[path = "../../../../tests/invariants/rigid_callers.rs"]
mod tests;
