//! Hidden-type dependencies retain callers while each defining body is typed once.
use super::{
    Checker, Declarations, DefiningRequest, Error, Requirement, State, isolation::Isolation,
};
use crate::support::hash::HashMap;
use crate::{resolve::expansion::Report, types::opaque::OpaqueId};
#[cfg(test)]
#[path = "../../../tests/invariants/importscheduling.rs"]
mod importscheduling;

#[derive(Clone, Copy)]
enum Status {
    Unseen,
    Active,
    Complete,
    Checked,
    /// Isolated only: the body failed or reached a failed body. Names the
    /// root cause; the body is never typed, completed or resumed.
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
    /// Isolated only: a hidden-type dependency names a failed body.
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
        mut isolation: Option<Isolation<'_>>,
    ) -> Result<(), Error> {
        if self.facts.opaques.owners.is_empty() {
            return self.plain_functions(close, isolation);
        }
        let mut schedule = Schedule::new(self.facts);
        let imported = schedule.checked();
        let mut current = None;
        loop {
            let index = if let Some(index) = current {
                index
            } else {
                if let Err(error) = self.advance_completed(&mut schedule, close) {
                    let Some(isolation) = isolation.as_mut() else {
                        return Err(error);
                    };
                    let function = self.facts.functions[schedule.cursor];
                    schedule.states[schedule.cursor] =
                        Status::Failed(self.isolate_body(isolation, function, error)?);
                    continue;
                }
                if schedule.cursor == schedule.states.len() {
                    break;
                }
                schedule.cursor
            };
            if matches!(schedule.states[index], Status::Unseen)
                && let Some(isolation) = isolation.as_mut()
            {
                let function = self.facts.functions[index];
                isolation.progress_item(self.report, function.definition, function.body)?;
            }
            let step = self.schedule(index, current, &mut schedule, close);
            current = match (step, isolation.as_mut()) {
                (Ok(Step::Next(next)), _) => next,
                (Err(error) | Ok(Step::Hidden(_, error)), None) => return Err(error),
                (Ok(Step::Blocked(_)), None) => {
                    return Err(self.error(
                        self.state.body.node,
                        "an isolated failed body was reached without isolation",
                    ));
                }
                (Err(error), Some(isolation)) => {
                    let function = self.facts.functions[index];
                    let cause = self.isolate_body(isolation, function, error)?;
                    self.unwind(isolation, &mut schedule, index, cause);
                    None
                }
                (Ok(Step::Blocked(cause)), Some(isolation)) => {
                    self.block_body(isolation, self.facts.functions[index], cause);
                    self.discard_body();
                    self.unwind(isolation, &mut schedule, index, cause);
                    None
                }
                (Ok(Step::Hidden(target, error)), Some(isolation)) => {
                    let failed = self.facts.functions[target];
                    let cause = self.isolate_body(isolation, failed, error)?;
                    schedule.states[target] = Status::Failed(cause);
                    schedule.patterns[target] = None;
                    self.block_body(isolation, self.facts.functions[index], cause);
                    self.unwind(isolation, &mut schedule, index, cause);
                    None
                }
            };
        }
        if let Some(isolation) = isolation.as_mut() {
            isolation.checked(schedule.checked().saturating_sub(imported));
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
            self.close_named_body(index, schedule, close)?;
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
    fn close_named_body(
        &mut self,
        index: usize,
        schedule: &mut Schedule,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let owner = self.facts.functions[index].definition;
        let count = self.facts.opaques.owners.get(&owner).map_or(0, Vec::len);
        for position in 0..count {
            let id = self.facts.opaques.owners[&owner][position];
            if self.facts.is_named_opaque(id)
                && self.facts.opaques.declarations[&id].hidden.is_none()
            {
                self.close_hidden(
                    id,
                    self.facts.functions[index].node.node,
                    &mut schedule.patterns[index],
                    close,
                )?;
            }
        }
        Ok(())
    }

    /// Every suspended body waits on the failed one through its hidden types.
    fn unwind(
        &mut self,
        isolation: &mut Isolation<'_>,
        schedule: &mut Schedule,
        index: usize,
        cause: usize,
    ) {
        schedule.states[index] = Status::Failed(cause);
        schedule.patterns[index] = None;
        for suspended in std::mem::take(&mut schedule.stack) {
            schedule.states[suspended.index] = Status::Failed(cause);
            self.block_body(isolation, self.facts.functions[suspended.index], cause);
        }
    }
    fn plain_functions(
        &mut self,
        close: &mut impl FnMut(&Report, &mut Declarations, DefiningRequest) -> Result<(), Error>,
        mut isolation: Option<Isolation<'_>>,
    ) -> Result<(), Error> {
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            if let Some(isolation) = isolation.as_mut() {
                isolation.progress_item(self.report, function.definition, function.body)?;
            }
            let result = self.plain_function(function, close);
            let Some(isolation) = isolation.as_mut() else {
                result?;
                continue;
            };
            match result {
                Ok(()) => isolation.checked(1),
                Err(error) => {
                    self.isolate_body(isolation, function, error)?;
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
        declaration
            .require_current(self.facts, id)
            .map_err(|e| self.error(source, e))?;
        let owner = self
            .facts
            .opaque_defining_owner(id)
            .ok_or_else(|| self.error(source, "opaque has no authenticated defining owner"))?;
        let target = *owners
            .get(&owner)
            .ok_or_else(|| self.error(source, "opaque hidden type has no defining body"))?;
        if self.facts.functions[target].body.is_none() {
            return Err(self.error(source, "abstract method has no hidden auto-trait contract"));
        }
        Ok(target)
    }
}
