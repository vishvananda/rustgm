//! Snapshot only the currently selected retained subtree, never earlier yields.
#[cfg(test)]
#[path = "../../../../../tests/invariants/numeric_snapshot.rs"]
mod tests;
use super::{Context, Declarations, Frame, Query, inference};
use crate::{
    support::{hash::HashSet, retained::Growth},
    types::{infer::TermId, lifetime_proof::ScopeId, substitute::Bound},
};

#[derive(Clone, Copy)]
enum Edge {
    Trial,
    Frame,
}
struct Row {
    slot: Option<usize>,
    parent: Option<(usize, Edge)>,
    trial: Vec<TermId>,
    frame: Vec<TermId>,
}
#[derive(Default)]
struct Snapshot {
    rows: Vec<Row>,
    seen: HashSet<usize>,
    bytes: usize,
}
fn charge(caller: &mut Context, bytes: &mut usize, growth: Growth) -> Result<(), String> {
    caller
        .account_work(growth.work.saturating_add(1))
        .map_err(inference)?;
    *bytes = bytes
        .checked_add(growth.bytes)
        .ok_or("numeric hint scratch overflow")?;
    if *bytes > 64 * 1024 * 1024 {
        return Err("numeric hint scratch budget exceeded".into());
    }
    Ok(())
}
fn reserve<T>(
    values: &mut Vec<T>,
    count: usize,
    bytes: &mut usize,
    caller: &mut Context,
) -> Result<(), String> {
    charge(
        caller,
        bytes,
        Growth::vector::<T>(values.len(), values.capacity(), count),
    )?;
    values
        .try_reserve(count)
        .map_err(|_| "numeric hint scratch allocation failed".into())
}
fn append(
    values: &mut Vec<TermId>,
    hints: &[TermId],
    bytes: &mut usize,
    caller: &mut Context,
) -> Result<(), String> {
    if hints.is_empty() {
        return Ok(());
    }
    reserve(values, hints.len(), bytes, caller)?;
    caller.account_work(hints.len()).map_err(inference)?;
    values.extend_from_slice(hints);
    Ok(())
}
fn unique(values: &mut Vec<TermId>, caller: &mut Context) -> Result<(), String> {
    let count = values.len();
    caller
        .account_work(count.saturating_mul(count.max(1).ilog2() as usize + 1))
        .map_err(inference)?;
    values.sort_unstable_by_key(|term| term.0);
    values.dedup();
    Ok(())
}
fn eligible(frame: &Frame, scope: ScopeId) -> bool {
    frame.scope == scope
        && !frame.normalizing()
        && !frame.deferred
        && frame.selected.is_none()
        && frame.partial.is_none()
}
fn trial_allowed(frame: &Frame, scope: ScopeId) -> bool {
    frame.trial.as_ref().is_some_and(|trial| {
        trial.scope == scope
            && !trial
                .instance
                .quantified
                .as_ref()
                .is_some_and(|conditions| conditions.waiting())
    })
}
impl Query {
    fn hint_frame(&self, slot: Option<usize>) -> Result<&Frame, String> {
        match slot {
            None => self.stack.first(),
            Some(slot) => self.suspended.get(slot).and_then(Option::as_ref),
        }
        .ok_or_else(|| "numeric hint snapshot lost its live frame".into())
    }
    fn hint_frame_mut(&mut self, slot: Option<usize>) -> Result<&mut Frame, String> {
        match slot {
            None => self.stack.first_mut(),
            Some(slot) => self.suspended.get_mut(slot).and_then(Option::as_mut),
        }
        .ok_or_else(|| "numeric hint snapshot lost its live frame".into())
    }
    pub(super) fn float_hints(
        &mut self,
        facts: &mut Declarations,
        caller: &mut Context,
    ) -> Result<Vec<TermId>, String> {
        caller.account_work(1).map_err(inference)?;
        if !self.publication.published() || self.stack.len() != 1 {
            return Err("numeric hints require the current inferred selection".into());
        }
        if !self.publication.has_hints() {
            return Ok(Vec::new());
        }
        let mut snapshot = Snapshot::default();
        snapshot.add(None, None, caller)?;
        snapshot.discover(self, caller)?;
        for index in (0..snapshot.rows.len()).rev() {
            snapshot.finish(self, facts, caller, index)?;
        }
        let hints = std::mem::take(&mut snapshot.rows[0].frame);
        if hints.is_empty() {
            return Ok(hints);
        }
        charge(
            caller,
            &mut snapshot.bytes,
            Growth::vector::<TermId>(0, 0, hints.len()),
        )?;
        let frame = self.stack.first_mut().ok_or("missing numeric hint root")?;
        let result = if let Some(universal) = &mut self.universal {
            universal.publish_numeric(
                facts,
                caller,
                frame,
                &mut self.allocations,
                self.scopes
                    .as_mut()
                    .ok_or("missing numeric hint universal scopes")?,
                &hints,
            )
        } else {
            let mut closing = Bound::parameters(&facts.types, &[])?;
            frame
                .inputs
                .publish_with_numeric_hints(&mut facts.types, caller, &mut closing, &hints)
                .map_err(inference)
        };
        let charged = caller
            .account_work(std::mem::take(&mut frame.inputs.context.steps))
            .map_err(inference);
        result.and_then(|(_, hints)| charged.map(|()| hints))
    }
}
impl Snapshot {
    fn add(
        &mut self,
        slot: Option<usize>,
        parent: Option<(usize, Edge)>,
        caller: &mut Context,
    ) -> Result<(), String> {
        caller.account_work(1).map_err(inference)?;
        if let Some(slot) = slot {
            if self.seen.contains(&slot) {
                return Err("numeric hint retained subtree is not a tree".into());
            }
            charge(
                caller,
                &mut self.bytes,
                Growth::map::<usize, ()>(self.seen.len(), self.seen.capacity(), 1),
            )?;
            self.seen
                .try_reserve(1)
                .map_err(|_| "numeric hint visited allocation failed")?;
            self.seen.insert(slot);
        }
        reserve(&mut self.rows, 1, &mut self.bytes, caller)?;
        self.rows.push(Row {
            slot,
            parent,
            trial: Vec::new(),
            frame: Vec::new(),
        });
        Ok(())
    }
    fn child(
        &mut self,
        query: &Query,
        slot: usize,
        parent: usize,
        edge: Edge,
        caller: &mut Context,
    ) -> Result<(), String> {
        caller.account_work(1).map_err(inference)?;
        if eligible(query.hint_frame(Some(slot))?, query.input_scope()) {
            self.add(Some(slot), Some((parent, edge)), caller)?;
        }
        Ok(())
    }
    fn discover(&mut self, query: &Query, caller: &mut Context) -> Result<(), String> {
        let mut index = 0;
        while index < self.rows.len() {
            caller.account_work(1).map_err(inference)?;
            let frame = query.hint_frame(self.rows[index].slot)?;
            if eligible(frame, query.input_scope()) {
                if trial_allowed(frame, query.input_scope()) {
                    let trial = frame.trial.as_ref().ok_or("missing numeric hint trial")?;
                    for (_, _, child) in trial.wakeups.live() {
                        caller.account_work(1).map_err(inference)?;
                        if let Some(slot) = child {
                            self.child(query, slot, index, Edge::Trial, caller)?;
                        }
                    }
                }
                let child = match frame.language.as_deref() {
                    Some(super::mechanical::Pending::Forward(pending)) => pending.retained_child(),
                    Some(super::mechanical::Pending::Unsize(pending)) => pending.retained_child(),
                    _ => None,
                };
                if let Some(slot) = child {
                    self.child(query, slot, index, Edge::Frame, caller)?;
                }
            }
            index += 1;
        }
        Ok(())
    }
    fn finish(
        &mut self,
        query: &mut Query,
        facts: &mut Declarations,
        caller: &mut Context,
        index: usize,
    ) -> Result<(), String> {
        let scope = query.input_scope();
        let row = &mut self.rows[index];
        let frame = query.hint_frame_mut(row.slot)?;
        if !eligible(frame, scope) {
            return Ok(());
        }
        if trial_allowed(frame, scope) {
            collect_trial(frame, row, facts, caller, &mut self.bytes)?;
        }
        unique(&mut row.frame, caller)?;
        let Some((parent, edge)) = row.parent else {
            return Ok(());
        };
        if row.frame.is_empty() {
            return Ok(());
        }
        let hints = std::mem::take(&mut row.frame);
        let slot = row.slot.ok_or("numeric child has no retained slot")?;
        let parent_slot = self.rows[parent].slot;
        charge(
            caller,
            &mut self.bytes,
            Growth::vector::<TermId>(0, 0, hints.len()),
        )?;
        let mapped = publish_child(query, facts, caller, slot, parent_slot, edge, &hints)?;
        let values = match edge {
            Edge::Trial => &mut self.rows[parent].trial,
            Edge::Frame => &mut self.rows[parent].frame,
        };
        append(values, &mapped, &mut self.bytes, caller)
    }
}
fn collect_trial(
    frame: &mut Frame,
    row: &mut Row,
    facts: &mut Declarations,
    caller: &mut Context,
    bytes: &mut usize,
) -> Result<(), String> {
    let result = trial_hints(frame, row, facts, caller, bytes);
    let steps = frame
        .trial
        .as_mut()
        .map_or(0, |trial| std::mem::take(&mut trial.inputs.context.steps))
        .saturating_add(std::mem::take(&mut frame.inputs.context.steps));
    let charged = caller.account_work(steps).map_err(inference);
    result.and_then(|mapped| charged.and_then(|()| append(&mut row.frame, &mapped, bytes, caller)))
}
fn trial_hints(
    frame: &mut Frame,
    row: &mut Row,
    facts: &mut Declarations,
    caller: &mut Context,
    bytes: &mut usize,
) -> Result<Vec<TermId>, String> {
    let trial = frame
        .trial
        .as_mut()
        .ok_or("missing numeric hint selected trial")?;
    for (index, watch, _) in trial.wakeups.live() {
        caller.account_work(1).map_err(inference)?;
        if let Some(hint) = trial
            .inputs
            .context
            .preferred_float(watch)
            .map_err(inference)?
        {
            let goal = trial
                .instance
                .goals
                .get(index)
                .ok_or("numeric hint lost its actual condition")?;
            let Some(current) =
                crate::sema::compiler_items::float_default(facts, &mut trial.inputs.context, goal)?
            else {
                continue;
            };
            let current = trial.inputs.context.root(current).map_err(inference)?;
            if trial.inputs.context.root(hint).map_err(inference)? != current {
                return Err("numeric hint changed its authenticated From argument".into());
            }
            append(&mut row.trial, &[current], bytes, caller)?;
        }
    }
    unique(&mut row.trial, caller)?;
    let result = if row.trial.is_empty() {
        Ok((false, Vec::new()))
    } else {
        charge(
            caller,
            bytes,
            Growth::vector::<TermId>(0, 0, row.trial.len()),
        )?;
        let mut closing = Bound::parameters(&facts.types, &[])?;
        trial
            .inputs
            .publish_with_numeric_hints(
                &mut facts.types,
                &mut frame.inputs.context,
                &mut closing,
                &row.trial,
            )
            .map_err(inference)
    };
    result.map(|(_, mapped)| mapped)
}
fn publish_child(
    query: &mut Query,
    facts: &mut Declarations,
    caller: &mut Context,
    slot: usize,
    parent: Option<usize>,
    edge: Edge,
    hints: &[TermId],
) -> Result<Vec<TermId>, String> {
    let mut child = query
        .suspended
        .get_mut(slot)
        .and_then(Option::take)
        .ok_or("missing numeric hint child frame")?;
    let result = (|| {
        let parent = query.hint_frame_mut(parent)?;
        let source = match edge {
            Edge::Frame => &mut parent.inputs.context,
            Edge::Trial => {
                &mut parent
                    .trial
                    .as_mut()
                    .ok_or("missing numeric hint parent trial")?
                    .inputs
                    .context
            }
        };
        let mut closing = Bound::parameters(&facts.types, &[])?;
        let result = child
            .inputs
            .publish_with_numeric_hints(&mut facts.types, source, &mut closing, hints)
            .map_err(inference);
        let steps = std::mem::take(&mut source.steps)
            .saturating_add(std::mem::take(&mut child.inputs.context.steps));
        let charged = caller.account_work(steps).map_err(inference);
        result.and_then(|(_, hints)| charged.map(|()| hints))
    })();
    query.suspended[slot] = Some(child);
    result
}
