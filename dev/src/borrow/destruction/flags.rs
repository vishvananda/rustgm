//! Sparse runtime writes observe the same initialization transfers as the proof.
use super::{Effect, Event, Plan, glue::Range};
use crate::{
    body::storage::Location,
    borrow::ownership::initialization::{self, Transition},
};

pub(super) fn elaborate(
    plan: &mut Plan,
    facts: &initialization::Facts<'_>,
    mut retained: usize,
) -> Result<(), &'static str> {
    let mut cleanup = 0;
    facts.visit_transfers(|transition, _| {
        charge(plan, 1)?;
        let event = match transition {
            Transition::Observe(initialization::Event::Point(location)) => Event {
                location,
                effect: Effect::Point,
            },
            Transition::Observe(initialization::Event::Cleanup(candidate)) => {
                let expected = &plan
                    .cleanups
                    .get(cleanup)
                    .ok_or("missing selected cleanup")?
                    .candidate;
                if expected.location != candidate.location
                    || expected.place != candidate.place
                    || expected.cause != candidate.cause
                    || expected.path != candidate.path
                {
                    return Err("cleanup selection and elaboration disagree");
                }
                let index = u32::try_from(cleanup).map_err(|_| "too many cleanup events")?;
                cleanup += 1;
                Event {
                    location: candidate.location,
                    effect: Effect::Cleanup(index),
                }
            }
            Transition::Statement(location) => Event {
                location,
                effect: Effect::Statement,
            },
            Transition::Birth { location, value } => Event {
                location,
                effect: Effect::Birth(value),
            },
            Transition::StorageEnd { location, end } => Event {
                location,
                effect: Effect::StorageEnd(end),
            },
            Transition::Initialize {
                location,
                start,
                end,
                initialized,
            } => {
                if !initialized && start != end {
                    push(
                        plan,
                        &mut retained,
                        Event {
                            location,
                            effect: Effect::Deinitialize { start, end },
                        },
                    )?;
                }
                let Some(event) = update(plan, location, start, end, initialized)? else {
                    return Ok(());
                };
                event
            }
        };
        push(plan, &mut retained, event)?;
        Ok(())
    })?;
    if cleanup != plan.cleanups.len() {
        return Err("selected cleanup was not elaborated");
    }
    Ok(())
}
fn push(plan: &mut Plan, retained: &mut usize, event: Event) -> Result<(), &'static str> {
    *retained = retained
        .checked_add(size_of::<Event>())
        .filter(|n| *n <= 64_000_000)
        .ok_or("destruction plan exceeds 64 MB budget")?;
    plan.events.push(event);
    Ok(())
}
fn update(
    plan: &mut Plan,
    location: Location,
    start: u32,
    end: u32,
    initialized: bool,
) -> Result<Option<Event>, &'static str> {
    if plan.flags.is_empty() || start == end {
        return Ok(None);
    }
    charge(plan, 2 * (plan.flags.len().ilog2() as usize + 1))?;
    let first = plan.flags.partition_point(|f| f.0 < start);
    let last = plan.flags.partition_point(|f| f.0 < end);
    if first == last {
        return Ok(None);
    }
    let range = Range {
        start: u32::try_from(first).map_err(|_| "too many drop flags")?,
        end: u32::try_from(last).map_err(|_| "too many drop flags")?,
    };
    Ok(Some(Event {
        location,
        effect: Effect::Flags { range, initialized },
    }))
}
fn charge(plan: &mut Plan, cost: usize) -> Result<(), &'static str> {
    plan.steps = plan
        .steps
        .checked_add(cost)
        .filter(|n| *n <= 32_000_000)
        .ok_or("destruction planning work budget exceeded")?;
    Ok(())
}
