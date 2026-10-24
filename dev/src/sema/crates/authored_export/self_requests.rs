//! A method goal follows its genuine trait Self slot and complete callable formals.
use super::{Declarations, DefId, Stage, canonical_nodes::Context, s};
use crate::types::{GenericArgs, ItemId, Kind, arguments::Argument};

impl Stage {
    pub(crate) fn method_request(
        &mut self,
        facts: &Declarations,
        item: ItemId,
        arguments: GenericArgs,
        context: Context,
    ) -> Result<Option<s::TraitApplicationRef>, &'static str> {
        self.charge(1)?;
        let definition = DefId(item.0);
        let Some(owner) = facts.associated.trait_owner(definition) else {
            if facts.associated.owner(definition).is_some() {
                return Err(
                    "source impl/inherent call requests need their exact selected trait-use producer",
                );
            }
            return Ok(None);
        };
        let implicit = facts
            .generics
            .trait_self_bounds
            .get(&owner)
            .copied()
            .ok_or("source method has no actual implicit Self trait application")?;
        let (application, _) = implicit
            .bound
            .trait_parts()
            .ok_or("source method Self request is not a trait predicate")?;
        let actual = facts
            .types
            .trait_application(application)
            .ok_or("source method trait application is unknown")?;
        if actual.item.0 != owner.0
            || facts
                .types
                .arguments(actual.arguments)
                .is_none_or(|arguments| arguments.len() != 0)
        {
            return Err(
                "parameterized source method trait arguments require complete wider transport",
            );
        }
        let Some(Kind::Parameter(self_parameter)) = facts.types.kind(implicit.ty) else {
            return Err("source method implicit Self has no genuine parameter identity");
        };
        let formals = facts
            .lifetimes
            .callables
            .generic_parameters(item)
            .ok_or("source method has no completed callable formal order")?;
        let template = facts
            .lifetimes
            .callables
            .get(item)
            .ok_or("source method callable header is absent")?;
        if template.parameters != 0 {
            return Err("source method early arguments require a wider complete request profile");
        }
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("source method application inputs are absent")?;
        if arguments.len() != formals.len() {
            return Err("source method application changed its complete formal count");
        }
        let mut subject = None;
        for (&formal, argument) in formals.iter().zip(arguments) {
            self.charge(1)?;
            let Argument::Type(ty) = argument else {
                return Err("source method formal kind is outside the supported type domain");
            };
            if formal == self_parameter && subject.replace(ty).is_some() {
                return Err("source method repeats its actual Self formal");
            }
        }
        let subject =
            subject.ok_or("source method application omitted its exact actual Self parameter")?;
        self.trait_application(facts, subject, application, context)
            .map(Some)
    }
}
