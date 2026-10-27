//! Pure counts share actual binding paths and one lazy preorder interval index.
mod driver_sets;
mod indexed;
use super::{
    Binding, Bindings, Definition, IterationId, LexError, NodeId, Rule, VariableId, error,
};
use crate::support::hash::HashMap;
use crate::support::names::NameId;
use driver_sets::DriverSets;
use indexed::Index;

#[derive(Clone, Copy)]
pub(in crate::expand::macros) struct PathIndex {
    pub index: u32,
    pub length: u32,
    /// Unique active transcription frame; zero is reserved for the root.
    pub key: u32,
}
pub(in crate::expand::macros) struct Count<'a> {
    pub definition: &'a Definition,
    pub rule: &'a Rule,
    pub variable: VariableId,
    pub path: &'a [PathIndex],
    pub depth: u64,
}
pub(in crate::expand::macros) struct Template<'a> {
    pub definition: &'a Definition,
    pub rule: &'a Rule,
    pub variables: &'a HashMap<NameId, VariableId>,
    pub path: &'a [PathIndex],
}
#[derive(Clone, Copy)]
struct First {
    next: Option<IterationId>,
    observed: usize,
}
#[derive(Default)]
pub(in crate::expand::macros) struct Cardinalities {
    first: HashMap<NodeId, First>,
    current: HashMap<(NodeId, u32), IterationId>,
    index: Option<Box<Index>>,
    drivers: DriverSets,
}
impl Cardinalities {
    pub(in crate::expand::macros) fn count(
        &mut self,
        bindings: &Bindings,
        query: &Count<'_>,
        work: &mut usize,
    ) -> Result<u32, LexError> {
        let repeats = query.definition.list(
            query
                .rule
                .variables
                .get(query.variable.0 as usize)
                .ok_or_else(|| error(0, "count variable has no actual declaration"))?
                .repetitions,
        );
        let maximum = self
            .observed(bindings, repeats, work)?
            .saturating_sub(1)
            .saturating_sub(query.path.len());
        let depth = usize::try_from(query.depth)
            .map_err(|_| error(0, "count depth exceeds its actual repetition shape"))?;
        if depth > maximum {
            return Err(error(0, "count depth exceeds its actual repetition shape"));
        }
        let Binding::Sequence(values) = binding(&mut self.current, bindings, query, work)? else {
            return Err(error(0, "count cannot consume an unrepeated metavariable"));
        };
        let target = query.path.len() + maximum - depth;
        if target == query.path.len() || values.is_empty() {
            return u32::try_from(values.len()).map_err(|_| error(0, "count exceeds u32"));
        }
        if self.index.is_none() {
            self.index = Some(Box::new(Index::new(bindings, work)?));
        }
        self.index
            .as_ref()
            .ok_or_else(|| error(0, "missing repetition count index"))?
            .count(
                values,
                *repeats
                    .get(target)
                    .ok_or_else(|| error(0, "count lost its actual repetition target"))?,
                work,
            )
    }

    /// The pinned empty-first-branch rule follows observed, not declared depth.
    fn observed(
        &mut self,
        bindings: &Bindings,
        repeats: &[NodeId],
        work: &mut usize,
    ) -> Result<usize, LexError> {
        let mut start = repeats.len();
        let mut first = First {
            next: Some(IterationId(0)),
            observed: 0,
        };
        while start != 0 {
            charge(work, 1)?;
            if let Some(cached) = self.first.get(&repeats[start - 1]) {
                first = *cached;
                break;
            }
            start -= 1;
        }
        for (index, &repeat) in repeats.iter().enumerate().skip(start) {
            charge(work, 2)?;
            if let Some(parent) = first.next {
                first = First {
                    next: bindings.sequence(parent, repeat)?.first().copied(),
                    observed: index + 1,
                };
            }
            self.first.insert(repeat, first);
        }
        Ok(first.observed)
    }

    pub(in crate::expand::macros) fn repetition_count(
        &mut self,
        bindings: &Bindings,
        template: &Template<'_>,
        list: super::super::model::List,
        work: &mut usize,
    ) -> Result<usize, LexError> {
        let mask = self
            .drivers
            .get(template.definition, template.variables, list, work)?;
        let mut count = None;
        for variable in mask.variables() {
            charge(work, 1)?;
            let query = Count {
                definition: template.definition,
                rule: template.rule,
                variable: variable?,
                path: template.path,
                depth: 0,
            };
            if let Binding::Sequence(values) = binding(&mut self.current, bindings, &query, work)? {
                if count.is_some_and(|old| old != values.len()) {
                    return Err(error(0, "metavariables repeat different numbers of times"));
                }
                count = Some(values.len());
            }
        }
        count.ok_or_else(|| error(0, "template repetition has no repeating metavariable"))
    }
}
fn binding<'a>(
    current: &mut HashMap<(NodeId, u32), IterationId>,
    bindings: &'a Bindings,
    query: &Count<'_>,
    work: &mut usize,
) -> Result<Binding<'a>, LexError> {
    let repeats = query.definition.list(
        query
            .rule
            .variables
            .get(query.variable.0 as usize)
            .ok_or_else(|| error(0, "count variable has no actual declaration"))?
            .repetitions,
    );
    let selected = repeats.len().min(query.path.len());
    let mut start = selected;
    let mut iteration = IterationId(0);
    while start != 0 {
        charge(work, 1)?;
        if let Some(cached) = self
            .current
            .get(&(repeats[start - 1], query.path[start - 1].key))
        {
            iteration = *cached;
            break;
        }
        start -= 1;
    }
    for (&repeat, frame) in repeats[start..selected]
        .iter()
        .zip(&query.path[start..selected])
    {
        charge(work, 2)?;
        iteration = *bindings
            .sequence(iteration, repeat)?
            .get(frame.index as usize)
            .ok_or_else(|| error(0, "incompatible macro repetition counts"))?;
        current.insert((repeat, frame.key), iteration);
    }
    if selected < repeats.len() {
        return Ok(Binding::Sequence(
            bindings.sequence(iteration, repeats[selected])?,
        ));
    }
    bindings
        .values
        .get(&(iteration, query.variable))
        .copied()
        .map(Binding::Capture)
        .ok_or_else(|| error(0, "missing metavariable capture"))
}

fn charge(work: &mut usize, steps: usize) -> Result<(), LexError> {
    if steps > 1_000_000_usize.saturating_sub(*work) {
        return Err(error(0, "macro counting work budget exceeded"));
    }
    *work += steps;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/cardinality_payments.rs"]
mod tests;
