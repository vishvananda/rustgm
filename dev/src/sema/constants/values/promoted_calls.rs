//! Promoted `#[rustc_promotable]` calls execute through the checked constant service.
use super::{
    Constant, Error, Evaluator,
    execution::{Progress, Request, Target},
};

use crate::{
    resolve::body_origin::BodyOrigin,
    types::{Kind, TypeId, instance::Instance, substitute::Bound},
};

/// The body from which the promotion request was found. Its execution
/// signature includes regions introduced during specialization.
#[derive(Clone, Copy)]
pub(crate) struct PromotionCaller {
    pub body: BodyOrigin,
    pub instance: Option<Instance>,
    pub signature: Option<TypeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Scope {
    body: BodyOrigin,
    caller: Option<Instance>,
}

/// A constant evaluation target whose outer binder, if present, was
/// checked where the call was selected. A scoped result belongs to that body.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PromotedFunction {
    instance: Instance,
    scope: Option<Scope>,
}
impl PromotedFunction {
    pub(crate) const fn unscoped(instance: Instance) -> Self {
        Self {
            instance,
            scope: None,
        }
    }

    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }
}

impl Evaluator<'_> {
    /// Authenticate the exact resolved source instance again for fresh and cached
    /// promotion demands. The marker grants no conditional-call payment.
    pub(crate) fn check_promoted_instance(&self, target: PromotedFunction) -> Result<(), Error> {
        let instance = target.instance();
        let facts = self.facts();
        let error = Error::unlocated;
        self.check_source_call_policy().map_err(error)?;
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err(error("promoted call lost its complete function instance"));
        };
        let depth = u16::from(target.scope.is_some());
        if facts.types.contains_parameters(instance.ty()) != Some(false)
            || facts
                .types
                .escaping_binders(instance.ty())
                .is_none_or(|escaping| escaping > depth)
            || !facts
                .types
                .arguments_are_resolved_at(arguments, depth)
                .map_err(error)?
        {
            return Err(error("promoted call requires a closed complete instance"));
        }
        let declaration = crate::resolve::model::DefId(item.0);
        if !facts.is_const_function(declaration)
            || !matches!(
                facts.promotable_contract(self.queries.report, declaration)?,
                crate::sema::header_attributes::ContractKnowledge::KnownPresent(_)
            )
        {
            return Err(error(
                "promoted call requires its authentic source const promise",
            ));
        }
        Ok(())
    }

    fn promotion_target(
        &mut self,
        instance: Instance,
        caller: PromotionCaller,
    ) -> Result<PromotedFunction, Error> {
        let facts = &mut *self.queries.typing.facts;
        let error = Error::unlocated;
        let mut target = PromotedFunction::unscoped(instance);
        if facts.types.escaping_binders(instance.ty()) == Some(0) {
            return Ok(target);
        }
        let Some(Kind::Function { bound_regions, .. }) =
            caller.signature.and_then(|ty| facts.types.kind(ty))
        else {
            return Err(error(
                "scoped promotion requires its caller's execution signature",
            ));
        };
        let mut observed = Bound::observe_regions();
        let result = observed.apply(&mut facts.types, instance.ty());
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(observed.steps);
        if result.map_err(error)? != instance.ty()
            || observed.observed_regions().any(|region| {
                region
                    .bound_parts()
                    .is_some_and(|(depth, slot)| depth != 0 || u32::from(slot) >= bound_regions)
            })
        {
            return Err(error(
                "promoted function references a lifetime outside its caller's binder",
            ));
        }
        target.scope = Some(Scope {
            body: caller.body,
            caller: caller.instance,
        });
        Ok(target)
    }

    /// Execute one promoted call of a resolved const function instance with its
    /// constant arguments, as rustc evaluates a promoted constant. `None` when
    /// no checked execution service is installed, or the execution awaits a
    /// declaration fact that is not complete yet.
    /// # Errors
    /// Preserves the callee's const restrictions and evaluation faults.
    pub(crate) fn promoted_call(
        &mut self,
        instance: Instance,
        caller: PromotionCaller,
        arguments: Vec<Constant>,
    ) -> Result<Option<Constant>, Error> {
        let target = self.promotion_target(instance, caller)?;
        self.check_promoted_instance(target)?;
        let Some(mut executor) = self.queries.typing.facts.constant_executor.take() else {
            return Ok(None);
        };
        let progress = executor.advance(
            self.queries.report,
            self.queries.typing.facts,
            &Request {
                target: Target::PromotedFunction(target),
                arguments,
            },
        );
        self.queries.typing.facts.constant_executor = Some(executor);
        Ok(match progress? {
            Progress::Complete(value) => Some(value),
            Progress::Prepare(_) => None,
        })
    }
}
