//! Hidden-type dependencies retain callers while each defining body is typed once.
use super::{Checker, Declarations, Error, Requirement, State};
use crate::{resolve::expansion::Report, types::opaque::OpaqueId};
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum Status {
    Unseen,
    Active,
    Complete,
}
struct Suspended {
    index: usize,
    state: Box<State>,
}

/// Preserve functions already checked to satisfy an early constant demand.
fn initial_states(facts: &Declarations) -> Vec<Status> {
    facts
        .functions
        .iter()
        .map(|function| {
            if facts.typed_function_bodies.contains(&function.definition) {
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
        close: &mut impl FnMut(&Report, &mut Declarations, OpaqueId) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if self.facts.opaques.owners.is_empty() {
            for index in 0..self.facts.functions.len() {
                self.function(self.facts.functions[index])?;
            }
            return Ok(());
        }
        let owners: HashMap<_, _> = self
            .facts
            .functions
            .iter()
            .enumerate()
            .filter(|(_, function)| self.facts.opaques.owners.contains_key(&function.definition))
            .map(|(index, function)| (function.definition, index))
            .collect();
        let mut states = initial_states(self.facts);
        let mut patterns: Vec<_> = std::iter::repeat_with(|| None).take(states.len()).collect();
        let mut stack: Vec<Suspended> = Vec::new();
        let mut cursor = 0;
        let mut current = None;
        loop {
            let index = if let Some(index) = current {
                index
            } else {
                while states
                    .get(cursor)
                    .is_some_and(|state| matches!(state, Status::Complete))
                {
                    cursor += 1;
                }
                if cursor == states.len() {
                    break;
                }
                cursor
            };
            let progress = match states[index] {
                Status::Unseen => {
                    states[index] = Status::Active;
                    self.start_function(self.facts.functions[index])?
                }
                Status::Active => self.advance_body()?,
                Status::Complete => {
                    return Err(
                        self.error(self.state.body.node, "completed body was scheduled twice")
                    );
                }
            };
            if let Some(dependency) = progress {
                let Requirement::Opaque(id) = dependency.requirement else {
                    self.require_complete_body(Some(dependency))?;
                    continue;
                };
                let target = self.hidden_owner(id, dependency.source.node, &owners)?;
                match states[target] {
                    Status::Unseen => {
                        stack.push(Suspended {
                            index,
                            state: self.suspend_body(),
                        });
                        current = Some(target);
                    }
                    Status::Active => {
                        return Err(self.error(
                            dependency.source.node,
                            "cycle in opaque hidden-type proof dependencies",
                        ));
                    }
                    Status::Complete => {
                        self.close_hidden(
                            id,
                            dependency.source.node,
                            &mut patterns[target],
                            close,
                        )?;
                        current = Some(index);
                    }
                }
            } else {
                states[index] = Status::Complete;
                self.facts
                    .typed_function_bodies
                    .insert(self.facts.functions[index].definition);
                patterns[index] = self.state.pending_patterns.take();
                current = stack.pop().map(|saved| {
                    self.restore_body(*saved.state);
                    saved.index
                });
            }
        }
        for pending in patterns.into_iter().flatten() {
            self.state
                .pending_patterns
                .get_or_insert_with(Default::default)
                .append(*pending);
        }
        Ok(())
    }
    fn close_hidden(
        &mut self,
        id: OpaqueId,
        source: super::NodeId,
        patterns: &mut Option<Box<super::pattern_constants::Pending>>,
        close: &mut impl FnMut(&Report, &mut Declarations, OpaqueId) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Some(pending) = patterns.take() {
            self.prepare_defining_patterns(pending)?;
        }
        close(self.report, self.facts, id)?;
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
