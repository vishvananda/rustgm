//! Borrow actual successful originals into private neutral export staging.
use crate::{
    body::regions::Contract,
    borrow::{incremental::Proved, templates::Checked},
    resolve::package::Error,
    sema::model::Declarations,
};

pub(super) trait Collect {
    fn original(
        &mut self,
        facts: &mut Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), Error>;
}
/// This type deliberately depends on stage/callback only, never the final result.
pub(super) struct Binding<'a, S, F> {
    pub stage: &'a mut S,
    pub callback: F,
}
impl<S, F> Collect for Binding<'_, S, F>
where
    F: FnMut(&mut S, &mut Declarations, &Proved, &Contract) -> Result<(), Error>,
{
    fn original(
        &mut self,
        facts: &mut Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), Error> {
        (self.callback)(self.stage, facts, proved, input)
    }
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the shared legacy no-op implements the same fallible collector callback contract"
)]
pub(super) fn ignore(
    (): &mut (),
    _: &mut Declarations,
    _: &Proved,
    _: &Contract,
) -> Result<(), Error> {
    Ok(())
}

/// Stack-only borrowed state; the legacy stage is () and its collector has no state.
pub(super) struct Context<'a, 'checked, 'service, C> {
    pub checked: &'a Checked<'checked>,
    pub collector: &'a mut C,
    pub preparation: Option<&'service mut dyn crate::sema::dispatch::Preparation>,
}

#[cfg(test)]
#[path = "../../../tests/invariants/staged_exports.rs"]
mod tests;
