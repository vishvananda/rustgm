//! Specializability belongs to the actual trait result, not the impl's written output.
use super::{Checker, DefId, Error};
use crate::sema::model::Function;

impl Checker<'_> {
    pub(super) fn reject_specializable_result(
        &self,
        declaration: DefId,
        actual: Function,
    ) -> Result<(), Error> {
        let Some(id) = self
            .facts
            .opaques
            .owners
            .get(&declaration)
            .and_then(|ids| ids.first())
        else {
            return Ok(());
        };
        let Some(modifier) = self.report.active_defaultness(actual.node)? else {
            return Ok(());
        };
        if self
            .facts
            .opaques
            .declarations
            .get(id)
            .is_none_or(|output| output.owner != declaration)
        {
            return Err(self.error(actual.node, "trait result declaration identity changed"));
        }
        Err(Error::at(
            actual.node.file,
            modifier.span.start,
            "associated function with return-position impl Trait in trait cannot be specialized",
        ))
    }
}
