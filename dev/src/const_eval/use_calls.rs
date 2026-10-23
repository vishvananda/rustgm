//! One reached caller consumes its fresh const payment before invoking cached code.
use super::{
    Frame, Machine, State,
    code::{Catalog, Code},
    use_inputs::{Inputs, Row},
};
use crate::{
    sema::{
        compiler_items::const_payments::{self, Payment},
        model::Declarations,
    },
    types::instance::Instance,
};
use std::sync::Arc;
#[derive(Clone, Copy, PartialEq, Eq)]
struct Site {
    frame: usize,
    code: usize,
    block: crate::body::ir::BlockId,
    position: u32,
    target: Instance,
}
pub(crate) struct Reached {
    site: Site,
}
pub(super) struct Continuation {
    site: Site,
    caller: Arc<Row>,
    callee: Arc<Row>,
    target_code: usize,
    payment: Payment,
}
impl Reached {
    pub(crate) fn instance(&self) -> Instance {
        self.site.target
    }
}
fn inspect(
    facts: &Declarations,
    catalog: &Catalog,
    frame: &Frame,
    position: u32,
    target: Instance,
) -> Result<Site, &'static str> {
    catalog.validate_entry(frame.body)?;
    let body = catalog
        .code(frame.body)
        .ok_or("source const caller code disappeared")?;
    let (definition, _, _, primitive) = const_payments::profile(facts, target)?;
    super::reached_layout::checked_direct_call(
        facts,
        body,
        frame,
        position,
        target,
        (definition, primitive),
    )?;
    Ok(Site {
        frame: frame.instance,
        code: frame.body,
        block: frame.block,
        position,
        target,
    })
}
impl State {
    pub(crate) fn reached_const_call(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
    ) -> Result<Reached, &'static str> {
        if self.paid_call.is_some() {
            return Err("source const call was already paid");
        }
        let frame = self
            .frames
            .last()
            .ok_or("source const payment has no suspended caller")?;
        Ok(Reached {
            site: inspect(facts, catalog, frame, frame.next, target)?,
        })
    }
    pub(crate) fn pay_const_call(
        &mut self,
        facts: &mut Declarations,
        catalog: &Catalog,
        inputs: &Inputs,
        reached: &Reached,
        slots: usize,
    ) -> Result<(), &'static str> {
        if self.paid_call.is_some() {
            return Err("source const-call payment cannot be replayed");
        }
        let frame = self
            .frames
            .last()
            .ok_or("source const payment lost its caller")?;
        if reached.site != inspect(facts, catalog, frame, frame.next, reached.site.target)? {
            return Err("source const payment changed its reached statement");
        }
        let target_code = *catalog
            .functions
            .get(&reached.site.target)
            .ok_or("source const callee is not materialized for its complete instance")?;
        let origin = inputs.get(frame.body, catalog)?;
        let selected = inputs.get(target_code, catalog)?;
        if origin.contract().universals != 0
            || !origin.contract().parameters.is_empty()
            || selected.contract().universals != 0
            || !selected.contract().parameters.is_empty()
        {
            return Err(
                "source const-call payment for open contract namespaces is not implemented",
            );
        }
        let requirements = selected.requirements()?;
        requirements.matches(facts, reached.site.target)?;
        let payment = const_payments::pay(facts, requirements, origin.environment())?;
        let extra = size_of::<Continuation>().div_ceil(size_of::<super::Constant>());
        if self.retained_slots().saturating_add(extra) > slots {
            return Err("source const-call continuation exceeds retained slot budget");
        }
        self.paid_call = Some(Box::new(Continuation {
            site: reached.site,
            caller: origin,
            callee: selected,
            target_code,
            payment,
        }));
        Ok(())
    }
}
impl Machine<'_, '_> {
    pub(super) fn validate_const_resume(&mut self, frame: &Frame) -> Result<(), &'static str> {
        let Some(continuation) = self.paid_call.as_ref() else {
            return Ok(());
        };
        let checked = inspect(
            self.evaluator.facts(),
            &self.bodies,
            frame,
            frame.next,
            continuation.site.target,
        );
        if checked.is_err() || checked.ok() != Some(continuation.site) {
            self.paid_call = None;
            return Err("source const payment resumed outside its exact checked statement");
        }
        Ok(())
    }
    pub(super) fn consume_const_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
    ) -> Result<Option<usize>, &'static str> {
        let Some(continuation) = self.paid_call.take() else {
            return Ok(None);
        };
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("source const call has no executing statement")?;
        let catalog: &Catalog = &self.bodies;
        let site = inspect(self.evaluator.facts(), catalog, frame, position, target)?;
        if continuation.site != site {
            return Err("source const-call continuation belongs to another frame or statement");
        }
        continuation.caller.validate(frame.body, body)?;
        catalog.validate_entry(continuation.target_code)?;
        let callee = catalog
            .code(continuation.target_code)
            .ok_or("source const callee code disappeared")?;
        continuation
            .callee
            .validate(continuation.target_code, callee)?;
        let requirements = continuation.callee.requirements()?;
        requirements.matches(self.evaluator.facts(), target)?;
        continuation
            .payment
            .validate(requirements, continuation.caller.environment())?;
        if catalog.functions.get(&target) != Some(&continuation.target_code) {
            return Err("source const callee index changed after payment");
        }
        Ok(Some(continuation.target_code))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/const_continuation.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/invariants/reached_work.rs"]
mod work_tests;
