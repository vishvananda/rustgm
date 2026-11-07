//! Builtin Field evidence selects genuine provided declarations without an invented impl.
use super::{Declarations, Dispatcher, Obligations, selection};
use crate::{
    resolve::model::DefId,
    sema::{compiler_items::field_protocol, values::Instance},
    types::{GenericArgs, parameters::predicates::Predicate},
};

impl Dispatcher {
    pub(super) fn field_constant_owner(
        &mut self,
        facts: &mut Declarations,
        predicate: Predicate,
    ) -> Result<Option<selection::Evidence>, String> {
        if !field_protocol::matches_goal(facts, predicate)? {
            return Ok(None);
        }
        self.source_evidence(facts, predicate, None)
            .map(Some)
            .map_err(super::Failure::message)
    }

    pub(super) fn constant_target(
        &mut self,
        facts: &mut Declarations,
        instance: Instance,
        predicate: Predicate,
    ) -> Result<(Instance, Obligations), String> {
        let (owner, arguments, proof) = if let Some(evidence) =
            self.field_constant_owner(facts, predicate)?
        {
            match evidence.source {
                selection::Source::Field(selected) => {
                    selected.provided_constant(facts, instance.definition)?;
                    return Ok((instance, evidence.proof));
                }
                selection::Source::Implementation { owner, arguments } => {
                    (owner, arguments, evidence.proof)
                }
                _ => return Err("concrete constant cannot use a symbolic Field assumption".into()),
            }
        } else {
            self.implementation(facts, predicate)?
        };
        Ok((
            implementation_constant(facts, instance, owner, arguments, self.availability)?,
            proof,
        ))
    }
}

fn implementation_constant(
    facts: &Declarations,
    instance: Instance,
    owner: DefId,
    arguments: GenericArgs,
    availability: crate::sema::traits::implementations::Availability,
) -> Result<Instance, String> {
    let definition = facts
        .generics
        .implementations
        .as_ref()
        .and_then(|registry| registry.available_constant(owner, instance.definition, availability))
        .ok_or("selected implementation lacks the trait constant")?;
    Ok(if definition == instance.definition {
        instance
    } else {
        Instance {
            definition,
            arguments,
        }
    })
}
