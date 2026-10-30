//! A method goal follows its genuine trait Self slot and complete callable formals.
use super::{Declarations, DefId, Stage, canonical_nodes::Context, s};
use crate::{
    support::{hash::HashMap, retained},
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{ParameterId, ParameterKind},
        traits::ApplicationId,
    },
};

impl Stage {
    pub(crate) fn method_request(
        &mut self,
        facts: &mut Declarations,
        callable: TypeId,
        context: Context,
    ) -> Result<Option<s::TraitApplicationRef>, &'static str> {
        self.charge(1)?;
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(callable)
        else {
            return Err("source method request has no actual callable");
        };
        self.member_request(facts, item, arguments, callable, context)
    }
    fn member_request(
        &mut self,
        facts: &mut Declarations,
        item: ItemId,
        arguments: GenericArgs,
        callable: TypeId,
        context: Context,
    ) -> Result<Option<s::TraitApplicationRef>, &'static str> {
        let definition = DefId(item.0);
        let Some(owner) = facts.associated.trait_owner(definition) else {
            if let Some(application) = self
                .inputs
                .selected_methods
                .as_deref()
                .and_then(|map| map.get(&definition))
                .copied()
            {
                if self.conversion_roles() {
                    return self
                        .selected_request(facts, callable, application, context)
                        .map(Some);
                }
                return Ok(Some(application));
            }
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
        if actual.item.0 != owner.0 {
            return Err("source method Self request changed its actual trait owner");
        }
        let count = facts
            .types
            .arguments(actual.arguments)
            .ok_or("source method trait arguments are absent")?
            .len();
        if count != 0 && !self.conversion_roles() {
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
        let supplied = facts
            .types
            .arguments(arguments)
            .ok_or("source method application inputs are absent")?;
        if supplied.len() != formals.len() {
            return Err("source method application changed its complete formal count");
        }
        let mut subject = None;
        for (&formal, argument) in formals.iter().zip(supplied) {
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
        if count == 0 {
            self.trait_application(facts, subject, application, context)
        } else {
            self.applied_method_request(facts, subject, application, (formals, arguments), context)
        }
        .map(Some)
    }

    fn selected_request(
        &mut self,
        facts: &mut Declarations,
        callable: TypeId,
        witness: s::TraitApplicationRef,
        context: Context,
    ) -> Result<s::TraitApplicationRef, &'static str> {
        let goal = self
            .paying(|pay| crate::sema::dispatch::callable_requirement(facts, callable, pay))?
            .ok_or("source selected method has no actual caller trait goal")?;
        let (application, _) = goal
            .bound
            .trait_parts()
            .ok_or("source selected method goal is not a trait predicate")?;
        let actual = facts
            .types
            .trait_application(application)
            .ok_or("source selected method trait goal disappeared")?;
        let owner = DefId(actual.item.0);
        let declaration = self
            .rows
            .trait_applications
            .get(witness.0 as usize)
            .ok_or("source selected method linkage disappeared")?
            .declaration;
        if declaration != self.address(owner)? {
            return Err("source selected method linkage changed its trait owner");
        }
        self.trait_application(facts, goal.ty, application, context)
    }

    /// The implicit trait predicate uses the genuine inherited callable formals.
    /// Actual arguments are simultaneous replacements, then ordinary recipe roots.
    fn applied_method_request(
        &mut self,
        facts: &Declarations,
        subject: TypeId,
        application: ApplicationId,
        supplied: (&[ParameterId], GenericArgs),
        context: Context,
    ) -> Result<s::TraitApplicationRef, &'static str> {
        let (formals, arguments) = supplied;
        self.reserve(retained::new_map::<ParameterId, TypeId>(formals.len()))?;
        let mut mapped = HashMap::default();
        for (&formal, argument) in formals.iter().zip(
            facts
                .types
                .arguments(arguments)
                .ok_or("source method inputs disappeared")?,
        ) {
            self.charge(3)?;
            let Argument::Type(ty) = argument else {
                return Err("source method argument is outside its actual type formal domain");
            };
            if !facts
                .types
                .parameter(formal)
                .is_some_and(|parameter| matches!(parameter.kind, ParameterKind::Type { .. }))
                || facts.types.kind(ty).is_none()
            {
                return Err("source method changed its actual type formal or argument identity");
            }
            if mapped.insert(formal, ty).is_some() {
                return Err("source method repeats an actual callable formal");
            }
        }
        let actual = facts
            .types
            .trait_application(application)
            .ok_or("source method trait application disappeared")?;
        let arguments = facts
            .types
            .arguments(actual.arguments)
            .ok_or("source method inherited trait arguments disappeared")?;
        self.reserve(arguments.len().saturating_mul(4 * size_of::<Argument>()))?;
        let mut completed = Vec::with_capacity(arguments.len());
        for argument in arguments {
            self.charge(3)?;
            let Argument::Type(ty) = argument else {
                return Err("source method inherited non-type arguments require wider transport");
            };
            let Some(Kind::Parameter(parameter)) = facts.types.kind(ty) else {
                return Err("source method implicit trait argument is not an actual formal");
            };
            let ty = *mapped
                .get(&parameter)
                .ok_or("source method omitted an inherited trait formal")?;
            completed.push(Argument::Type(ty));
        }
        let declaration = self.address(DefId(actual.item.0))?;
        let subject = self.ty(facts, subject, context)?;
        let arguments = self.argument_rows(facts, completed.into_iter(), context)?;
        self.push_trait(s::TraitApplication {
            declaration,
            subject,
            arguments,
        })
    }
}
