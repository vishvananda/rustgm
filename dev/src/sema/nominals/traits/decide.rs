//! Complete scoped decisions isolate failed inference without granting a body proof.
use super::{Context, Declarations, Predicate, State, select_requirement, selection};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    Proven,
    Absent,
}
impl State {
    /// Share the actual caller index/opening; isolate each complete canonical goal.
    /// Only successful evidence enters this owner's ordinary obligation frontier.
    pub(crate) fn decide(
        &mut self,
        facts: &mut Declarations,
        predicate: Predicate,
    ) -> Result<Decision, &'static str> {
        let fold = self
            .fold
            .as_mut()
            .ok_or("scoped decision requires its actual body opening")?;
        let before = fold.steps;
        let predicate = fold.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(|_| "scoped decision opening work exhausted")?;
        let first = self.context.steps;
        let mut query = Context::default();
        query
            .account_work(first)
            .map_err(|_| "scoped decision work exhausted")?;
        let selected = select_requirement(
            facts,
            &mut query,
            predicate,
            selection::Caller {
                assumptions: &self.assumptions,
                environment: self.environment.as_ref(),
                first_region: self.obligations.regions.end,
                retain_sized: self.retain_sized,
                nesting: self.nesting.nested(),
                placeholders: self.placeholders.as_ref(),
            },
            self.availability,
            false,
            None,
        );
        // Discarded and failed queries pay work too. Neither can mutate the
        // owner's inference graph or publish their incidental equalities.
        self.context
            .account_work(query.steps.saturating_sub(first))
            .map_err(|_| "scoped decision work exhausted")?;
        self.context
            .account_nested(query.nested())
            .map_err(|_| "scoped decision work exhausted")?;
        match selected? {
            selection::Selection::Absent => Ok(Decision::Absent),
            selection::Selection::Refuted(refutation) => {
                refutation.reject()?;
                Ok(Decision::Absent)
            }
            selection::Selection::Deferred => Err("scoped decision requires pending source proof"),
            selection::Selection::Proven(mut evidence) => {
                if evidence.provisional_size || !evidence.sized.is_empty() {
                    return Err("scoped decision requires completed sized evidence");
                }
                evidence
                    .proof
                    .equalities
                    .append(&mut query.region_equalities);
                self.accept_selection(selection::Selection::Proven(evidence))?
                    .ok_or("scoped decision lost its actual positive evidence")?;
                Ok(Decision::Proven)
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/ctqdc.rs"]
mod tests;
