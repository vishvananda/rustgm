//! Async callable results retain the actual trait, callable and returned future.
mod lending_method;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::{bindings::LangItem, call_traits::CallTrait},
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        projection::Application,
        traits::ApplicationId,
    },
};

/// Data correspondence for one completed selection; its proof stays in the
/// original search graph. This carries no method/body execution permission.
pub(in crate::sema) struct Selected {
    subject: TypeId,
    application: ApplicationId,
    future: TypeId,
    output: TypeId,
}
const _: [(); 16] = [(); size_of::<Selected>()];

pub(in crate::sema) fn kind(
    facts: &Declarations,
    item: ItemId,
) -> Result<Option<CallTrait>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    for (role, kind) in [
        (LangItem::AsyncFnOnce, CallTrait::FnOnce),
        (LangItem::AsyncFnMut, CallTrait::FnMut),
        (LangItem::AsyncFn, CallTrait::Fn),
    ] {
        if catalog
            .role(role)?
            .is_some_and(|actual| actual.definition == DefId(item.0))
        {
            catalog
                .validated_language(role)?
                .ok_or("async callable role is incomplete")?;
            return Ok(Some(kind));
        }
    }
    Ok(None)
}

pub(in crate::sema) fn future_goal(
    facts: &mut Declarations,
    future: TypeId,
) -> Result<Predicate, &'static str> {
    let trait_ = role(facts, LangItem::Future)?;
    let application = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: ItemId(trait_.0),
            arguments: GenericArgs::EMPTY,
        })?;
    Ok(Predicate {
        ty: future,
        bound: Bound::Trait(application),
    })
}

impl Selected {
    pub(in crate::sema) fn new(
        facts: &mut Declarations,
        goal: Predicate,
        future: Predicate,
    ) -> Result<Self, &'static str> {
        if future_goal(facts, future.ty)? != future {
            return Err("async callable lost its actual returned Future proof");
        }
        let (application, _) = goal
            .bound
            .trait_parts()
            .ok_or("missing async callable application")?;
        let future_application = future
            .bound
            .trait_parts()
            .ok_or("missing Future application")?
            .0;
        let member = role(facts, LangItem::FutureOutput)?;
        let schema = facts
            .types
            .projection_schema(ItemId(member.0))
            .ok_or("async callable future Output has no canonical schema")?;
        let actual = facts
            .source_bindings
            .as_deref()
            .ok_or("missing async source catalog")?
            .validated_language(LangItem::FutureOutput)?
            .ok_or("missing actual Future Output")?;
        let parent = role(facts, LangItem::Future)?;
        if actual.parent != Some(parent) || schema.trait_item != ItemId(parent.0) {
            return Err("async callable result changed its actual Future Output owner");
        }
        let output = facts.types.intern(Kind::Projection(Application {
            item: ItemId(member.0),
            subject: future.ty,
            trait_application: future_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        Ok(Self {
            subject: goal.ty,
            application,
            future: future.ty,
            output,
        })
    }

    pub(in crate::sema) fn output(
        &self,
        facts: &Declarations,
        application: Application,
    ) -> Result<TypeId, &'static str> {
        if application.subject != self.subject || application.trait_application != self.application
        {
            return Err("async callable projection changed its complete selected application");
        }
        let parent = facts
            .types
            .trait_application(self.application)
            .ok_or("missing async callable parent")?;
        let kind =
            kind(facts, parent.item)?.ok_or("async callable projection has a different trait")?;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("missing async callable catalog")?;
        for (member_role, owner_role, expected_kind) in [
            (
                LangItem::AsyncFnOnceOutput,
                LangItem::AsyncFnOnce,
                CallTrait::FnOnce,
            ),
            (
                LangItem::CallOnceFuture,
                LangItem::AsyncFnOnce,
                CallTrait::FnOnce,
            ),
            (
                LangItem::CallRefFuture,
                LangItem::AsyncFnMut,
                CallTrait::FnMut,
            ),
        ] {
            let Some(member) = catalog.validated_language(member_role)? else {
                continue;
            };
            if member.definition != DefId(application.item.0) {
                continue;
            }
            let owner = role(facts, owner_role)?;
            let schema = facts
                .types
                .projection_schema(application.item)
                .ok_or("missing async member schema")?;
            if kind != expected_kind
                || parent.item != ItemId(owner.0)
                || member.parent != Some(owner)
                || schema.trait_item != parent.item
            {
                return Err("async callable member changed its actual owner");
            }
            let mut family = facts
                .types
                .arguments(application.arguments)
                .ok_or("missing async family arguments")?;
            let valid = if member_role == LangItem::CallRefFuture {
                family.len() == 1 && matches!(family.next(), Some(Argument::Lifetime(_)))
            } else {
                family.len() == 0
            };
            if !valid {
                return Err("async callable member has invalid family arguments");
            }
            return Ok(if member_role == LangItem::AsyncFnOnceOutput {
                self.output
            } else {
                self.future
            });
        }
        Err("async callable evidence does not own this associated member")
    }
}

fn role(facts: &Declarations, role: LangItem) -> Result<DefId, &'static str> {
    facts
        .source_bindings
        .as_deref()
        .ok_or("async callable requires actual source roles")?
        .validated_language(role)?
        .map(|d| d.definition)
        .ok_or("async callable source role is absent")
}
