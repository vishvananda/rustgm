//! Hidden-type dependencies retain callers while each defining body is typed once.
use super::{Checker, Declarations, DefiningRequest, Error, Requirement, State};
use crate::support::hash::HashMap;
use crate::{
    resolve::expansion::Report,
    sema::census::{Census, Phase},
    types::opaque::OpaqueId,
};
#[cfg(test)]
#[path = "../../../tests/invariants/importscheduling.rs"]
mod importscheduling;

#[derive(Clone, Copy)]
enum Status {
    Unseen,
    Active,
    Complete,
    Checked,
    /// Census only: the body failed or reached a failed body. Names the
    /// root-cause record; the body is never typed, completed or resumed.
    Failed(usize),
}
struct Suspended {
    index: usize,
    state: Box<State>,
}
/// Hidden-type scheduling state shared by every step of one crate's bodies.
struct Schedule {
    owners: HashMap<super::DefId, usize>,
    states: Vec<Status>,
    patterns: Vec<Option<Box<super::pattern_constants::Pending>>>,
    stack: Vec<Suspended>,
    cursor: usize,
}
impl Schedule {
    fn new(facts: &Declarations) -> Self {
        let owners = facts
            .functions
            .iter()
            .enumerate()
            .filter(|(_, function)| facts.opaques.owners.contains_key(&function.definition))
            .map(|(index, function)| (function.definition, index))
            .collect();
        let states = initial_states(facts);
        let patterns = std::iter::repeat_with(|| None).take(states.len()).collect();
        Self {
            owners,
            states,
            patterns,
            stack: Vec::new(),
            cursor: 0,
        }
    }
    fn checked(&self) -> usize {
        self.states
            .iter()
            .filter(|state| matches!(state, Status::Checked))
            .count()
    }
}
/// The continuation of one scheduling step.
enum Step {
    Next(Option<usize>),
    /// Census only: a hidden-type dependency names a failed body.
    Blocked(usize),
    /// The hidden-type proof of a completed defining body failed.
    Hidden(usize, Error),
}

/// An authenticated import has no authored body to schedule. This does not
/// publish a typed-body proof or permit materialization of an absent body.
pub(super) fn imported(facts: &Declarations, function: super::Function) -> bool {
    function.body.is_none()
        && facts
            .foreign
            .as_deref()
            .and_then(|rows| rows.function(function.definition))
            .is_some_and(|function| function.linkage == crate::support::linkage::Linkage::Import)
}

/// Preserve functions already checked to satisfy an early constant demand.
fn initial_states(facts: &Declarations) -> Vec<Status> {
    facts
        .functions
        .iter()
        .map(|function| {
            if imported(facts, *function) {
                Status::Checked
            } else if facts.typed_function_bodies.contains(&function.definition) {
                Status::Complete
            } else {
                Status::Unseen
            }
        })
        .collect()
}

impl Checker<'_> {
    pub(super) fn functions(
        &mut self,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
        mut census: Option<&mut Census>,
    ) -> Result<(), Error> {
        if self.facts.opaques.owners.is_empty() {
            return self.plain_functions(close, census);
        }
        let mut schedule = Schedule::new(self.facts);
        let imported = schedule.checked();
        let mut current = None;
        loop {
            let index = if let Some(index) = current {
                index
            } else {
                if let Err(error) = self.advance_completed(&mut schedule, close) {
                    let Some(census) = census.as_deref_mut() else {
                        return Err(error);
                    };
                    let owner = self.facts.functions[schedule.cursor].definition;
                    schedule.states[schedule.cursor] =
                        Status::Failed(self.isolate_body(census, owner, error)?);
                    continue;
                }
                if schedule.cursor == schedule.states.len() {
                    break;
                }
                schedule.cursor
            };
            let step = self.schedule(index, current, &mut schedule, close);
            current = match (step, census.as_deref_mut()) {
                (Ok(Step::Next(next)), _) => next,
                (Err(error) | Ok(Step::Hidden(_, error)), None) => return Err(error),
                (Ok(Step::Blocked(_)), None) => {
                    return Err(self.error(
                        self.state.body.node,
                        "a census-failed body was reached outside a census",
                    ));
                }
                (Err(error), Some(census)) => {
                    let owner = self.facts.functions[index].definition;
                    let cause = self.isolate_body(census, owner, error)?;
                    self.unwind(census, &mut schedule, index, cause);
                    None
                }
                (Ok(Step::Blocked(cause)), Some(census)) => {
                    self.block_body(census, self.facts.functions[index].definition, cause);
                    self.discard_body();
                    self.unwind(census, &mut schedule, index, cause);
                    None
                }
                (Ok(Step::Hidden(target, error)), Some(census)) => {
                    let owner = self.facts.functions[target].definition;
                    let cause = self.isolate_body(census, owner, error)?;
                    schedule.states[target] = Status::Failed(cause);
                    schedule.patterns[target] = None;
                    self.block_body(census, self.facts.functions[index].definition, cause);
                    self.unwind(census, &mut schedule, index, cause);
                    None
                }
            };
        }
        if let Some(census) = census {
            census.checked(Phase::Bodies, schedule.checked().saturating_sub(imported));
        }
        for pending in schedule.patterns.into_iter().flatten() {
            self.state
                .pending_patterns
                .get_or_insert_with(Default::default)
                .append(*pending);
        }
        Ok(())
    }
    /// Advance one scheduled body until it completes or waits on a hidden type.
    fn schedule(
        &mut self,
        index: usize,
        current: Option<usize>,
        schedule: &mut Schedule,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<Step, Error> {
        let progress = match schedule.states[index] {
            Status::Unseen => {
                schedule.states[index] = Status::Active;
                self.start_function(self.facts.functions[index])?
            }
            Status::Active => self.advance_body()?,
            Status::Complete | Status::Checked | Status::Failed(_) => {
                return Err(self.error(self.state.body.node, "completed body was scheduled twice"));
            }
        };
        let Some(dependency) = progress else {
            let function = self.facts.functions[index];
            self.facts.typed_function_bodies.insert(function.definition);
            self.facts
                .prepare_authored_future_shape(self.report, &function)?;
            schedule.patterns[index] = self.state.pending_patterns.take();
            self.complete_comptime(
                self.facts.functions[index],
                &mut schedule.patterns[index],
                close,
            )?;
            schedule.states[index] = Status::Checked;
            return Ok(Step::Next(schedule.stack.pop().map(|saved| {
                self.restore_body(*saved.state);
                saved.index
            })));
        };
        let Requirement::Opaque(id) = dependency.requirement else {
            self.require_complete_body(Some(dependency))?;
            return Ok(Step::Next(current));
        };
        let target = self.hidden_owner(id, dependency.source.node, &schedule.owners)?;
        match schedule.states[target] {
            Status::Unseen => {
                schedule.stack.push(Suspended {
                    index,
                    state: self.suspend_body(),
                });
                Ok(Step::Next(Some(target)))
            }
            Status::Active => Err(self.error(
                dependency.source.node,
                "cycle in opaque hidden-type proof dependencies",
            )),
            Status::Complete | Status::Checked => {
                match self.close_hidden(
                    id,
                    dependency.source.node,
                    &mut schedule.patterns[target],
                    close,
                ) {
                    Ok(()) => Ok(Step::Next(Some(index))),
                    Err(error) => Ok(Step::Hidden(target, error)),
                }
            }
            Status::Failed(cause) => Ok(Step::Blocked(cause)),
        }
    }
    /// Every suspended body waits on the failed one through its hidden types.
    fn unwind(&mut self, census: &mut Census, schedule: &mut Schedule, index: usize, cause: usize) {
        schedule.states[index] = Status::Failed(cause);
        schedule.patterns[index] = None;
        for suspended in std::mem::take(&mut schedule.stack) {
            schedule.states[suspended.index] = Status::Failed(cause);
            let owner = self.facts.functions[suspended.index].definition;
            self.block_body(census, owner, cause);
        }
    }
    fn plain_functions(
        &mut self,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
        mut census: Option<&mut Census>,
    ) -> Result<(), Error> {
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            let result = self.plain_function(function, close);
            let Some(census) = census.as_deref_mut() else {
                result?;
                continue;
            };
            match result {
                Ok(()) => census.checked(Phase::Bodies, 1),
                Err(error) => {
                    self.isolate_body(census, function.definition, error)?;
                }
            }
        }
        Ok(())
    }
    fn plain_function(
        &mut self,
        function: super::Function,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let eager = if let Some(body) = function.body
            && self.facts.is_const_function(function.definition)
        {
            self.facts
                .source_function_context(self.report, function.definition, body)
                .map_err(|error| Declarations::error(self.report, body, error))?
                == crate::sema::function_context::FunctionContext::Comptime
        } else {
            false
        };
        if eager {
            let previous = self.state.pending_patterns.take();
            self.function(function)?;
            let mut pending = self.state.pending_patterns.take();
            self.complete_comptime(function, &mut pending, close)?;
            self.state.pending_patterns = match (previous, pending) {
                (Some(mut previous), Some(pending)) => {
                    previous.append(*pending);
                    Some(previous)
                }
                (previous, None) => previous,
                (None, pending) => pending,
            };
        } else {
            self.function(function)?;
            self.complete_comptime(function, &mut None, close)?;
        }
        Ok(())
    }
    fn advance_completed(
        &mut self,
        schedule: &mut Schedule,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<(), Error> {
        while let Some(state) = schedule.states.get(schedule.cursor).copied() {
            match state {
                Status::Complete => {
                    self.complete_comptime(
                        self.facts.functions[schedule.cursor],
                        &mut schedule.patterns[schedule.cursor],
                        close,
                    )?;
                    schedule.states[schedule.cursor] = Status::Checked;
                }
                Status::Checked | Status::Failed(_) => {}
                Status::Unseen | Status::Active => break,
            }
            schedule.cursor += 1;
        }
        Ok(())
    }
    fn complete_comptime(
        &mut self,
        function: super::Function,
        pending: &mut Option<Box<super::pattern_constants::Pending>>,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let Some(body) = function.body else {
            return Ok(());
        };
        if !self.facts.is_const_function(function.definition) {
            return Ok(());
        }
        let context = self
            .facts
            .source_function_context(self.report, function.definition, body)
            .map_err(|error| Declarations::error(self.report, body, error))?;
        if !matches!(
            context,
            crate::sema::function_context::FunctionContext::Comptime
                | crate::sema::function_context::FunctionContext::ConstFunction
        ) {
            return Ok(());
        }
        if context == crate::sema::function_context::FunctionContext::Comptime
            && let Some(pending) = pending.take()
        {
            self.prepare_defining_patterns(pending)?;
        }
        close(
            self.report,
            self.facts,
            if context == crate::sema::function_context::FunctionContext::Comptime {
                DefiningRequest::ComptimeDefinition(function.definition)
            } else {
                DefiningRequest::ConstDefinition(function.definition)
            },
        )
    }
    fn close_hidden(
        &mut self,
        id: OpaqueId,
        source: super::NodeId,
        patterns: &mut Option<Box<super::pattern_constants::Pending>>,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Some(pending) = patterns.take() {
            self.prepare_defining_patterns(pending)?;
        }
        close(self.report, self.facts, DefiningRequest::Hidden(id))?;
        if self
            .facts
            .opaques
            .declarations
            .get(&id)
            .is_none_or(|d| d.hidden.is_none())
        {
            return Err(self.error(source, "opaque region proof did not close its hidden type"));
        }
        Ok(())
    }
    fn hidden_owner(
        &self,
        id: OpaqueId,
        source: super::NodeId,
        owners: &HashMap<super::DefId, usize>,
    ) -> Result<usize, Error> {
        let declaration = self
            .facts
            .opaques
            .declarations
            .get(&id)
            .ok_or_else(|| self.error(source, "missing demanded opaque declaration"))?;
        let target = *owners
            .get(&declaration.owner)
            .ok_or_else(|| self.error(source, "opaque hidden type has no defining body"))?;
        if self.facts.functions[target].body.is_none() {
            return Err(self.error(source, "abstract method has no hidden auto-trait contract"));
        }
        Ok(target)
    }
}
