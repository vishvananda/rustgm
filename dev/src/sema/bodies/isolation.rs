//! Census isolation of failed bodies. Ordinary checking keeps its first error.
use super::{Checker, Error, Scratch, State};
use crate::{
    resolve::model::DefId,
    sema::census::{self, Census, Phase},
};

impl Checker<'_> {
    /// Record a failed body, keep it unusable and discard its partial state.
    /// Budget exhaustion stays sticky: the unchanged error ends the census.
    pub(super) fn isolate_body(
        &mut self,
        census: &mut Census,
        owner: DefId,
        error: Error,
    ) -> Result<usize, Error> {
        if census::exhausts_budget(&error.message) {
            return Err(error);
        }
        let error = self.report.diagnostic(error);
        let cause = if let Some(cause) = census.repeated(&error) {
            self.block_body(census, owner, cause);
            cause
        } else {
            census.failed(Phase::Bodies, census::describe(self.report, owner), &error)
        };
        self.facts.typed_function_bodies.remove(&owner);
        self.discard_body();
        Ok(cause)
    }
    /// A body that reached a failed body is recorded once and never completed.
    pub(super) fn block_body(&mut self, census: &mut Census, owner: DefId, cause: usize) {
        census.blocked(
            Phase::Bodies,
            census::describe(self.report, owner),
            census::site(self.report, owner),
            cause,
        );
        self.facts.typed_function_bodies.remove(&owner);
    }
    /// Replace every per-body field; a failed body's slots are never reused.
    pub(super) fn discard_body(&mut self) {
        let mut scratch = Scratch::default();
        scratch.terms.files(self.facts.nodes.len());
        self.state = State::fresh(scratch);
    }
    pub(super) fn census_initializer(
        &mut self,
        index: usize,
        mut census: Option<&mut Census>,
    ) -> Result<(), Error> {
        if let Some(census) = census.as_deref_mut() {
            let initializer = self.facts.initializers[index];
            if !matches!(
                initializer.typing,
                crate::sema::model::InitializerTyping::Complete
            ) {
                census.progress_item(
                    self.report,
                    census::ItemKind::Initializer,
                    initializer.definition,
                    Some(initializer.value),
                )?;
            }
        }
        let result = self.initializer(index);
        let Some(census) = census else {
            return result;
        };
        match result {
            Ok(()) => {
                census.checked(Phase::Bodies, 1);
                Ok(())
            }
            Err(error) => {
                let owner = self.facts.initializers[index].definition;
                self.isolate_body(census, owner, error).map(drop)
            }
        }
    }
}
