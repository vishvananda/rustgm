//! Active proved cleanup candidates retain defining Conditional Destruct legality.
use super::{Body, Store, regions};
use crate::body::defining_cleanups::Work;
use crate::borrow::destruction::schedule::Schedule;

pub(super) fn check(
    types: &Store,
    body: &Body,
    proof: &regions::Body,
    schedule: Option<Schedule<'_>>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(requirements) = proof.builtins.as_deref() else {
        return Ok(());
    };
    let Some(state) = requirements.defining() else {
        return Ok(());
    };
    let prepared = state.checked(body, requirements.environment.as_ref(), work)?;
    if let Some(schedule) = schedule {
        for cleanup in schedule.cleanups() {
            work.charge(1)?;
            if cleanup.access {
                let ty = cleanup
                    .live_type
                    .ok_or("active defining cleanup has no proved live subject")?;
                prepared.check(types, ty, work)?;
            }
        }
    }
    Ok(())
}

pub(super) fn steps(
    types: &Store,
    body: &Body,
    proof: &regions::Body,
    schedule: Option<Schedule<'_>>,
) -> Result<usize, &'static str> {
    let mut work = Work::new(1_000_000);
    check(types, body, proof, schedule, &mut work)?;
    Ok(work.steps)
}
