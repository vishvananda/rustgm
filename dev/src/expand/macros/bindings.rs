//! Replay only the winning trace into flat repetition instances and sparse bindings.
use super::{
    capture::CaptureId,
    matcher::{Event, Matched, NO_TRACE},
    model::{Definition, NodeId, Rule, VariableId},
    nfa::Action,
};
use crate::support::hash::HashMap;
use crate::{lex::token::LexError, syntax::token_tree::error};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IterationId(pub u32);
#[derive(Clone, Copy, Debug)]
pub enum Binding<'a> {
    Capture(CaptureId),
    Sequence(&'a [IterationId]),
}
#[derive(Default)]
pub struct Bindings {
    groups: Vec<Range<usize>>,
    iterations: Vec<IterationId>,
    repetitions: HashMap<(IterationId, NodeId), usize>,
    values: HashMap<(IterationId, VariableId), CaptureId>,
}
struct Frame {
    node: NodeId,
    group: usize,
    parent: IterationId,
}
impl Bindings {
    /// # Errors
    /// Reports inconsistent event nesting or capture identity overflow.
    pub fn from_match(matched: &Matched) -> Result<Self, LexError> {
        let mut events = Vec::new();
        let mut trace = matched.last_trace;
        while trace != NO_TRACE {
            let entry = matched
                .traces
                .get(trace as usize)
                .ok_or_else(|| error(0, "invalid macro capture trace"))?;
            events.push(entry.event);
            if entry.parent != NO_TRACE && entry.parent >= trace {
                return Err(error(0, "cyclic macro capture trace"));
            }
            trace = entry.parent;
        }
        let mut result = Self::default();
        let mut frames: Vec<Frame> = Vec::new();
        let mut current = IterationId(0);
        let mut owners = Vec::new();
        for event in events.into_iter().rev() {
            match event {
                Event::Action(Action::Begin(node)) => {
                    let group = result.groups.len();
                    result.groups.push(0..0);
                    if result.repetitions.insert((current, node), group).is_some() {
                        return Err(error(0, "duplicate repetition instance"));
                    }
                    frames.push(Frame {
                        node,
                        group,
                        parent: current,
                    });
                }
                Event::Action(Action::Iteration(node)) => {
                    let frame = frames
                        .last()
                        .filter(|f| f.node == node)
                        .ok_or_else(|| error(0, "iteration outside its repetition"))?;
                    current = IterationId(
                        u32::try_from(owners.len() + 1)
                            .map_err(|_| error(0, "too many macro iterations"))?,
                    );
                    owners.push(frame.group);
                    result.groups[frame.group].end += 1;
                }
                Event::Action(Action::End(node)) => {
                    let frame = frames
                        .pop()
                        .filter(|f| f.node == node)
                        .ok_or_else(|| error(0, "mismatched repetition end"))?;
                    current = frame.parent;
                }
                Event::Capture { variable, value } => {
                    if result.values.insert((current, variable), value).is_some() {
                        return Err(error(0, "duplicate metavariable capture"));
                    }
                }
            }
        }
        if !frames.is_empty() {
            return Err(error(0, "unterminated repetition trace"));
        }
        let mut total = 0;
        for range in &mut result.groups {
            let count = range.end;
            *range = total..total + count;
            total += count;
        }
        let mut next: Vec<_> = result.groups.iter().map(|r| r.start).collect();
        result.iterations.resize(total, IterationId(0));
        for (index, owner) in owners.into_iter().enumerate() {
            result.iterations[next[owner]] = IterationId(
                u32::try_from(index + 1).map_err(|_| error(0, "too many macro iterations"))?,
            );
            next[owner] += 1;
        }
        Ok(result)
    }

    /// Locate a binding at the current template repetition indices. Outer
    /// scalar bindings remain available inside a repetition driven by another variable.
    ///
    /// # Errors
    /// Reports malformed traces and incompatible template repetition shapes.
    pub fn at_path(
        &self,
        definition: &Definition,
        rule: &Rule,
        variable: VariableId,
        indices: &[usize],
    ) -> Result<Binding<'_>, LexError> {
        let repeats = definition.list(rule.variables[variable.0 as usize].repetitions);
        let mut iteration = IterationId(0);
        for (depth, repeat) in repeats.iter().enumerate() {
            let group = self
                .repetitions
                .get(&(iteration, *repeat))
                .ok_or_else(|| error(0, "missing repetition capture"))?;
            let values = &self.iterations[self.groups[*group].clone()];
            let Some(index) = indices.get(depth) else {
                return Ok(Binding::Sequence(values));
            };
            iteration = *values
                .get(*index)
                .ok_or_else(|| error(0, "incompatible macro repetition counts"))?;
        }
        self.values
            .get(&(iteration, variable))
            .copied()
            .map(Binding::Capture)
            .ok_or_else(|| error(0, "missing metavariable capture"))
    }
}
