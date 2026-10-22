//! Declaration scheduling marks the exact lazily installed slice metadata ready.
use super::{Checker, Error, State};
impl Checker<'_> {
    pub(super) fn prepare_slice_methods(&mut self) -> Result<(), Error> {
        let declaration = self
            .report
            .resolution
            .library_types
            .slice_length()
            .copied()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing slice metadata"))?;
        super::super::methods::slices::prepare(self.report, &mut self.facts)
            .map_err(|e| super::Declarations::error(self.report, declaration.origin, e))?;
        for id in [
            declaration.owner,
            declaration.parameter,
            declaration.definition,
        ] {
            self.states[id.0 as usize] = State::Complete;
        }
        Ok(())
    }
}
