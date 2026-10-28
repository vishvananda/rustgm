//! Non-type conditions finish directly, without manufacturing a trait frame.
use super::{Context, Decision, Declarations, Frame, Search, inference};
use crate::types::parameters::{clauses::Clause, features::FeatureId};
impl Search<'_> {
    pub(super) fn feature_condition(
        &self,
        facts: &Declarations,
        context: &mut Context,
        id: FeatureId,
    ) -> Result<bool, String> {
        let work = std::cell::Cell::new(0_usize);
        let pay = |steps, _| {
            work.set(work.get().saturating_add(steps));
            Ok(())
        };
        let result = facts.prove_feature(id, None, &pay).and_then(|enabled| {
            if enabled {
                Ok(true)
            } else {
                self.caller
                    .assumptions
                    .contains_feature(&facts.types, id, &pay)
            }
        });
        context.account_work(work.get()).map_err(inference)?;
        result.map_err(str::to_owned)
    }
    pub(super) fn feature_formation(
        &self,
        facts: &Declarations,
        frame: &mut Frame,
    ) -> Result<(), String> {
        loop {
            let Some(Clause::UnstableFeature(id)) = frame
                .formation
                .as_ref()
                .and_then(|formation| formation.requirements.clauses.get(formation.cursor))
                .copied()
            else {
                return Ok(());
            };
            let accepted = self.feature_condition(facts, &mut frame.inputs.context, id)?;
            let formation = frame
                .formation
                .as_mut()
                .ok_or("missing feature formation owner")?;
            if !accepted {
                formation.stopped = Some(Decision::Absent);
                return Ok(());
            }
            formation.cursor += 1;
        }
    }
}
