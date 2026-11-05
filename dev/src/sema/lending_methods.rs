//! Full async method descriptors identify a wrapper; its body still needs proof.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{model::Declarations, traits::callables::asynchronous},
    support::call_traits::CallTrait,
    types::{Kind, TypeId, instance::Instance},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Method {
    instance: Instance,
    future: TypeId,
}
const _: [(); 8] = [(); size_of::<Method>()];

pub(crate) struct Validated {
    pub definition: DefId,
    pub source: NodeKey,
    pub signature: TypeId,
    pub subject: TypeId,
    pub tuple: TypeId,
    pub future: TypeId,
    pub kind: CallTrait,
}

impl Method {
    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }

    pub(crate) fn validate(self, facts: &mut Declarations) -> Result<Validated, &'static str> {
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("async wrapper requires its full selected method instance");
        };
        let definition = DefId(item.0);
        let owner = facts
            .associated
            .trait_owner(definition)
            .ok_or("async wrapper lost its actual trait owner")?;
        let kind = asynchronous::kind(facts, crate::types::ItemId(owner.0))?
            .ok_or("async wrapper has a different source trait")?;
        let template = *facts
            .generics
            .trait_self_bounds
            .get(&owner)
            .ok_or("async wrapper lost its actual declared Self bound")?;
        let payments =
            crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
                .map_err(|_| "async wrapper lost its actual source payer")?;
        let mut fold = crate::sema::dispatch::callable_substitution_paid(
            facts,
            self.instance,
            &|work, bytes| payments.charge(work, bytes),
        )?;
        let goal = fold.predicate_charged(&mut facts.types, template, &|work, bytes| {
            payments.charge(work, bytes)
        })?;
        let future = asynchronous::future_goal(facts, self.future)?;
        // This reconstructs data correspondence, not an implementation proof.
        // Registry insertion is private to completed selected-method dispatch.
        let selected = asynchronous::Selected::new(facts, goal, future)?;
        selected.require_method(facts, self.instance)?;
        let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
            return Err("async wrapper lost its selected public signature");
        };
        if result != self.future {
            return Err("async wrapper public result differs from its selected future");
        }
        let parent = facts
            .types
            .trait_application(
                goal.bound
                    .trait_parts()
                    .ok_or("async wrapper has no full application")?
                    .0,
            )
            .ok_or("async wrapper lost its full application")?;
        let mut arguments = facts
            .types
            .arguments(parent.arguments)
            .ok_or("async wrapper has invalid tuple arguments")?;
        let Some(crate::types::arguments::Argument::Type(tuple)) = arguments.next() else {
            return Err("async wrapper requires its actual tuple argument");
        };
        if arguments.next().is_some() || !matches!(facts.types.kind(tuple), Some(Kind::Tuple(_))) {
            return Err("async wrapper requires exactly one full tuple argument");
        }
        let source = facts
            .function_header(definition)?
            .and_then(|header| header.declaration.source())
            .ok_or("async wrapper requires its actual source declaration")?;
        Ok(Validated {
            definition,
            source,
            signature,
            subject: goal.ty,
            tuple,
            future: self.future,
            kind,
        })
    }
}

pub(in crate::sema) fn retain(
    facts: &mut Declarations,
    instance: Instance,
    future: TypeId,
) -> Result<Instance, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("async wrapper requires its actual function item");
    };
    let Some(Kind::Function {
        bound_regions,
        parameters,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("async wrapper requires its actual callable signature");
    };
    let (types, payments) = facts.caller_input_store()?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let signature = types.intern_observed::<true>(
        Kind::Function {
            bound_regions,
            parameters,
            unsafe_,
            convention,
            result: future,
        },
        &pay,
    )?;
    let ty = types.intern_observed::<true>(
        Kind::FunctionItem {
            item,
            arguments,
            signature,
        },
        &pay,
    )?;
    let selected = Method {
        instance: Instance::new(&facts.types, ty)?,
        future,
    };
    selected.validate(facts)?;
    crate::sema::primitive_methods::retain_lending(facts, selected)?;
    Ok(selected.instance)
}
