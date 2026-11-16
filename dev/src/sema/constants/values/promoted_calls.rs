//! Promoted `#[rustc_promotable]` calls execute through the checked constant service.
use super::{
    Constant, Error, Evaluator,
    execution::{Progress, Request, Target},
};

impl Evaluator<'_> {
    /// Authenticate the exact closed source instance again for fresh and cached
    /// promotion demands. The marker grants no conditional-call payment.
    pub(crate) fn check_promoted_instance(
        &self,
        instance: crate::types::instance::Instance,
    ) -> Result<(), Error> {
        let facts = self.facts();
        let error = Error::unlocated;
        self.check_source_call_policy().map_err(error)?;
        let Some(crate::types::Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err(error("promoted call lost its complete function instance"));
        };
        if facts.types.contains_parameters(instance.ty()) != Some(false)
            || !facts
                .types
                .arguments_are_concrete(arguments)
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

    /// Execute one promoted call of a closed const function instance with its
    /// constant arguments, as rustc evaluates a promoted constant. `None` when
    /// no checked execution service is installed, or the execution awaits a
    /// declaration fact that is not complete yet.
    /// # Errors
    /// Preserves the callee's const restrictions and evaluation faults.
    pub(crate) fn promoted_call(
        &mut self,
        instance: crate::types::instance::Instance,
        arguments: Vec<Constant>,
    ) -> Result<Option<Constant>, Error> {
        self.check_promoted_instance(instance)?;
        let Some(mut executor) = self.queries.typing.facts.constant_executor.take() else {
            return Ok(None);
        };
        let progress = executor.advance(
            self.queries.report,
            self.queries.typing.facts,
            &Request {
                target: Target::PromotedFunction(instance),
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
