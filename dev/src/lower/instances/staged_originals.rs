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
        facts: &Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), Error>;
}
impl<F> Collect for F
where
    F: FnMut(&Declarations, &Proved, &Contract) -> Result<(), Error>,
{
    fn original(
        &mut self,
        facts: &Declarations,
        proved: &Proved,
        input: &Contract,
    ) -> Result<(), Error> {
        self(facts, proved, input)
    }
}

/// Stack-only borrowed state; the legacy stage is () and its collector has no state.
pub(super) struct Context<'a, 'checked, C> {
    pub checked: &'a Checked<'checked>,
    pub collector: &'a mut C,
}

#[cfg(test)]
#[path = "../../../tests/invariants/staged_exports.rs"]
mod tests;
