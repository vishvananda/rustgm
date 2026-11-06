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
    let prepared = super::Formed::new(state, body, proof, work)?;
    if let Some(schedule) = schedule {
        for (index, cleanup) in schedule.cleanups().enumerate() {
            work.charge(1)?;
            if cleanup.access {
                let ty = cleanup
                    .live_type
                    .ok_or("active defining cleanup has no proved live subject")?;
                if let Some(selected) = schedule.selected() {
                    for subject in selected.for_cleanup(index)? {
                        if !subject.access {
                            continue;
                        }
                        prepared.check_for(types, subject.ty, subject.purpose, work)?;
                    }
                } else {
                    prepared.check(types, ty, work)?;
                }
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
