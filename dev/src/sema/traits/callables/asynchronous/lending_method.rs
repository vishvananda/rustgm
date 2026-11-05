//! Selected async method data cannot stand in for an authored or generated body.
use super::{CallTrait, Declarations, DefId, GenericArgs, ItemId, Kind, LangItem, Selected, role};
use crate::{
    support::constness::Constness,
    types::{arguments::Argument, callable::Convention, instance::Instance},
};

impl Selected {
    pub(in crate::sema) fn retain_method(
        &self,
        facts: &mut Declarations,
        instance: Instance,
    ) -> Result<Instance, &'static str> {
        self.require_method(facts, instance)?;
        crate::sema::lending_methods::retain(facts, instance, self.future)
    }

    pub(in crate::sema) fn require_method(
        &self,
        facts: &mut Declarations,
        instance: Instance,
    ) -> Result<(), &'static str> {
        let parent = facts
            .types
            .trait_application(self.application)
            .ok_or("missing async method parent")?;
        let original = self.require_application(facts, instance, DefId(parent.item.0))?;
        let kind =
            super::kind(facts, parent.item)?.ok_or("async method has a different actual trait")?;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err("async method requires its actual full function item");
        };
        let method = DefId(item.0);
        let header = facts
            .function_header(method)?
            .ok_or("async method has no actual header")?;
        if facts.associated.trait_owner(method) != Some(DefId(parent.item.0))
            || header.declaration.source().is_none()
            || header.body.is_some()
        {
            return Err("automatic async method requires its actual required source declaration");
        }
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST_CALL,
        }) = facts.types.kind(signature)
        else {
            return Err("async method requires its actual safe rust-call signature");
        };
        let [receiver, tuple] = facts
            .types
            .list(parameters)
            .ok_or("missing async method parameters")?
        else {
            return Err("async method requires receiver and tuple arguments");
        };
        let (receiver, tuple) = (*receiver, *tuple);
        let Some(Kind::Function {
            bound_regions: expected_binders,
            parameters: expected_parameters,
            result: expected_result,
            unsafe_: false,
            convention: Convention::RUST_CALL,
        }) = facts.types.kind(original)
        else {
            return Err("async method lost its actual canonical signature");
        };
        if bound_regions != expected_binders || parameters != expected_parameters {
            return Err("async method changed its canonical parameter substitution");
        }
        self.method_result(facts, kind, parent.arguments, expected_result)?;
        let mut arguments = facts
            .types
            .arguments(parent.arguments)
            .ok_or("missing async method trait arguments")?;
        if arguments.len() != 1 || arguments.next() != Some(Argument::Type(tuple)) {
            return Err("async method changed its actual argument tuple");
        }
        let valid_receiver = match kind {
            CallTrait::FnOnce => receiver == self.subject && bound_regions == 0,
            CallTrait::Fn | CallTrait::FnMut => {
                bound_regions == 1
                    && matches!(facts.types.kind(receiver), Some(Kind::Reference { referent, mutable, region })
                    if referent == self.subject && mutable == (kind == CallTrait::FnMut)
                        && region.bound_parts() == Some((0, 0)))
            }
        };
        if !valid_receiver {
            return Err("async method changed its selected receiver permission");
        }
        self.method_result(facts, kind, parent.arguments, result)
    }

    fn require_application(
        &self,
        facts: &mut Declarations,
        instance: Instance,
        parent: DefId,
    ) -> Result<crate::types::TypeId, &'static str> {
        let template = *facts
            .generics
            .trait_self_bounds
            .get(&parent)
            .ok_or("async callable method lost its actual declared Self bound")?;
        let payments =
            crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
                .map_err(|_| "async callable method lost its actual source payer")?;
        let mut fold =
            crate::sema::dispatch::callable_substitution_paid(facts, instance, &|work, bytes| {
                payments.charge(work, bytes)
            })?;
        let goal = fold.predicate_charged(&mut facts.types, template, &|work, bytes| {
            payments.charge(work, bytes)
        })?;
        if goal.ty != self.subject
            || goal.bound.trait_parts() != Some((self.application, Constness::Ordinary))
        {
            return Err("async method differs from its selected full callable application");
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err("async method lost its canonical function identity");
        };
        let signature = facts
            .lifetimes
            .callables
            .get(item)
            .ok_or("async method lost its canonical callable template")?
            .signature;
        if facts
            .function_header(DefId(item.0))?
            .is_none_or(|header| header.signature != signature)
        {
            return Err("async method changed its actual canonical declaration signature");
        }
        let signature = fold.apply_argument_charged(
            &mut facts.types,
            Argument::Type(signature),
            &|work, bytes| payments.charge(work, bytes),
        )?;
        let Argument::Type(signature) = signature else {
            return Err("async method canonical substitution changed its argument kind");
        };
        Ok(signature)
    }

    fn method_result(
        &self,
        facts: &Declarations,
        kind: CallTrait,
        arguments: GenericArgs,
        result: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        if result == self.future {
            return Ok(());
        }
        let Some(Kind::Projection(application)) = facts.types.kind(result) else {
            return Err("async method result differs from its selected returned future");
        };
        let (owner, member) = if kind == CallTrait::FnOnce {
            (LangItem::AsyncFnOnce, LangItem::CallOnceFuture)
        } else {
            (LangItem::AsyncFnMut, LangItem::CallRefFuture)
        };
        let owner = role(facts, owner)?;
        let member = role(facts, member)?;
        let parent = facts
            .types
            .trait_application(application.trait_application)
            .ok_or("missing async result parent")?;
        let mut family = facts
            .types
            .arguments(application.arguments)
            .ok_or("missing async result family")?;
        let valid_family = if kind == CallTrait::FnOnce {
            family.len() == 0
        } else {
            family.len() == 1
                && matches!(family.next(), Some(Argument::Lifetime(region)) if region.bound_parts() == Some((0, 0)))
        };
        if application.subject != self.subject
            || application.item != ItemId(member.0)
            || parent.item != ItemId(owner.0)
            || parent.arguments != arguments
            || !valid_family
        {
            return Err("async method changed its actual returned future family");
        }
        Ok(())
    }
}
