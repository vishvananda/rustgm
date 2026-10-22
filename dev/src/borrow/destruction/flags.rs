//! Sparse runtime writes observe the same initialization transfers as the proof.
use super::{Effect, Event, Flag, Plan, glue::Range};
use crate::{
    body::storage::Location,
    borrow::ownership::initialization::{self, Transition},
};

pub(super) fn elaborate(
    plan: &mut Plan,
    facts: &initialization::Facts<'_>,
    recording: &mut Recording,
    retained: usize,
) -> Result<(), &'static str> {
    Recorder {
        events: &mut plan.events,
        flags: &plan.flags,
        steps: &mut plan.steps,
        retained,
    }
    .record_from(facts, recording, plan.cleanups.iter().map(|c| c.candidate))
}
/// Transitions captured by the selection replay so elaboration need not replay
/// the same initialization transfers again. An oversized body keeps no copy and
/// elaboration replays its transfers instead.
#[derive(Default)]
pub(super) struct Recording {
    transitions: Vec<Transition>,
    overflow: bool,
}
impl Recording {
    const LIMIT: usize = 1 << 20;
    /// Reserve for a body's typical transition count: roughly a point, a
    /// statement and an initialization write per statement. `buffer` is an
    /// earlier body's emptied recording, reused for its capacity.
    pub(super) fn for_body(body: &crate::body::ir::Body, mut buffer: Vec<Transition>) -> Self {
        let estimate = body
            .statements
            .len()
            .saturating_mul(3)
            .saturating_add(body.blocks.len())
            .min(Self::LIMIT);
        buffer.clear();
        buffer.reserve(estimate);
        Self {
            transitions: buffer,
            overflow: false,
        }
    }
    /// The emptied transition buffer, for the next body's recording.
    pub(super) fn into_buffer(mut self) -> Vec<Transition> {
        self.transitions.clear();
        self.transitions
    }
    pub(super) fn push(&mut self, transition: Transition) {
        if self.overflow {
            return;
        }
        if self.transitions.len() >= Self::LIMIT {
            self.overflow = true;
            self.transitions = Vec::new();
            return;
        }
        self.transitions.push(transition);
    }
}
pub(super) struct Recorder<'a> {
    pub events: &'a mut Vec<Event>,
    pub flags: &'a [Flag],
    pub steps: &'a mut usize,
    pub retained: usize,
}
impl Recorder<'_> {
    /// Elaborate recorded transitions, replaying them only if none were kept.
    pub(super) fn record_from(
        &mut self,
        facts: &initialization::Facts<'_>,
        recording: &mut Recording,
        mut candidates: impl Iterator<Item = initialization::Candidate>,
    ) -> Result<(), &'static str> {
        if recording.overflow {
            return self.record(facts, candidates);
        }
        let mut cleanup = 0_usize;
        self.events.reserve(recording.transitions.len());
        for transition in recording.transitions.drain(..) {
            self.transition(transition, &mut cleanup, &mut candidates)?;
        }
        if candidates.next().is_some() {
            return Err("selected cleanup was not elaborated");
        }
        Ok(())
    }
    pub(super) fn record(
        &mut self,
        facts: &initialization::Facts<'_>,
        mut candidates: impl Iterator<Item = initialization::Candidate>,
    ) -> Result<(), &'static str> {
        let mut cleanup = 0_usize;
        facts.visit_transfers(|transition, _| {
            self.transition(transition, &mut cleanup, &mut candidates)
        })?;
        if candidates.next().is_some() {
            return Err("selected cleanup was not elaborated");
        }
        Ok(())
    }
    fn transition(
        &mut self,
        transition: Transition,
        cleanup: &mut usize,
        candidates: &mut impl Iterator<Item = initialization::Candidate>,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let event = match transition {
            Transition::Observe(initialization::Event::Point(location)) => Event {
                location,
                effect: Effect::Point,
            },
            Transition::Observe(initialization::Event::Cleanup(candidate)) => {
                let expected = candidates.next().ok_or("missing selected cleanup")?;
                if expected.location != candidate.location
                    || expected.place != candidate.place
                    || expected.cause != candidate.cause
                    || expected.path != candidate.path
                {
                    return Err("cleanup selection and elaboration disagree");
                }
                let index = u32::try_from(*cleanup).map_err(|_| "too many cleanup events")?;
                *cleanup += 1;
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
                    self.push(Event {
                        location,
                        effect: Effect::Deinitialize { start, end },
                    })?;
                }
                let Some(event) = self.update(location, start, end, initialized)? else {
                    return Ok(());
                };
                event
            }
        };
        self.push(event)
    }
    fn push(&mut self, event: Event) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(size_of::<Event>())
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction plan exceeds 64 MB budget")?;
        self.events.push(event);
        Ok(())
    }
    fn update(
        &mut self,
        location: Location,
        start: u32,
        end: u32,
        initialized: bool,
    ) -> Result<Option<Event>, &'static str> {
        if self.flags.is_empty() || start == end {
            return Ok(None);
        }
        self.charge(2 * (self.flags.len().ilog2() as usize + 1))?;
        let first = self.flags.partition_point(|f| f.0 < start);
        let last = self.flags.partition_point(|f| f.0 < end);
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
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        *self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= 32_000_000)
            .ok_or("destruction planning work budget exceeded")?;
        Ok(())
    }
}
