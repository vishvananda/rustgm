//! The new nested-bound producer suspends owned DFS work, not Rust call frames.
use super::{Binding, Checker, Error, NodeKey, constraint, proj};
use crate::{
    sema::{declarations::generics::bounds::Syntax, generics::Predicate},
    support::retained::Growth,
    types::TypeId,
};
type Edges = Vec<super::super::super::aliases::Edge>;
type Key = (NodeKey, Binding);
#[derive(Clone, Copy)]
enum Task {
    Binding(Binding),
    Constraint(NodeKey, Binding),
    Project(NodeKey, Binding, (TypeId, TypeId)),
    Written(NodeKey, Binding, (TypeId, TypeId), usize),
    Retain(NodeKey, Binding),
}
pub(super) struct Run {
    subject: (TypeId, TypeId),
    tasks: Vec<Task>,
    clauses: Vec<Predicate>,
    edges: Edges,
}
impl Run {
    fn new(subject: (TypeId, TypeId)) -> Self {
        Self {
            subject,
            tasks: Vec::new(),
            clauses: Vec::new(),
            edges: Vec::new(),
        }
    }
}
impl Checker<'_> {
    pub(super) fn complete_projected_bounds(
        &mut self,
        source: NodeKey,
        binding: Binding,
        subject: (TypeId, TypeId),
        completed: &mut Vec<Predicate>,
        dependencies: &mut Edges,
    ) -> Result<bool, Error> {
        self.prepare_projected_owner(binding.owner)?;
        let key = (source, binding);
        let mut retained = self
            .associated_bindings
            .projected
            .as_mut()
            .and_then(|p| p.runs.remove(&key));
        let mut fresh = Run::new(subject);
        let pending = {
            let resumed = retained.is_some();
            let run = retained.as_deref_mut().unwrap_or(&mut fresh);
            if run.subject != subject {
                return Err(self.error(
                    source,
                    "projected subject changed during pending declaration work",
                ));
            }
            if !resumed {
                self.push_bound_task(run, Task::Project(source, binding, subject))?;
            }
            self.advance_bound_run(run)?
        };
        if pending {
            let (length, capacity) = self
                .associated_bindings
                .projected
                .as_ref()
                .map_or((0, 0), |p| (p.runs.len(), p.runs.capacity()));
            let growth = Growth::map::<Key, Box<Run>>(length, capacity, 1);
            self.charge_bound_staging(
                growth.work.saturating_add(1),
                growth.bytes.saturating_add(if retained.is_none() {
                    size_of::<Run>()
                } else {
                    0
                }),
            )?;
            let run = retained.unwrap_or_else(|| Box::new(fresh));
            self.associated_bindings
                .projected
                .as_mut()
                .ok_or_else(|| Error::unlocated("nested bounds lost their owner"))?
                .runs
                .insert(key, run);
            return Ok(true);
        }
        let mut run = retained.map_or(fresh, |run| *run);
        let mut growth =
            Growth::vector::<Predicate>(completed.len(), completed.capacity(), run.clauses.len());
        growth.include(Growth::vector::<super::super::super::aliases::Edge>(
            dependencies.len(),
            dependencies.capacity(),
            run.edges.len(),
        ));
        self.charge_bound_staging(
            growth
                .work
                .saturating_add(run.clauses.len())
                .saturating_add(run.edges.len()),
            growth.bytes,
        )?;
        completed.append(&mut run.clauses);
        dependencies.append(&mut run.edges);
        Ok(false)
    }
    fn push_bound_task(&mut self, run: &mut Run, task: Task) -> Result<(), Error> {
        let growth = Growth::vector::<Task>(run.tasks.len(), run.tasks.capacity(), 1);
        self.charge_bound_staging(growth.work.saturating_add(1), growth.bytes)?;
        run.tasks.push(task);
        Ok(())
    }
    fn advance_bound_run(&mut self, run: &mut Run) -> Result<bool, Error> {
        while let Some(task) = run.tasks.pop() {
            self.charge_bound_staging(1, 0)?;
            let pending = match task {
                Task::Binding(binding) => {
                    self.queue_bound_binding(run, binding)?;
                    false
                }
                Task::Constraint(source, binding) => {
                    match self.complete_binding_constraint(source, binding)? {
                        constraint::Outcome::Pending => true,
                        constraint::Outcome::Equality(value) => {
                            self.prepay_bound_clause(run)?;
                            run.clauses.push(value);
                            false
                        }
                        constraint::Outcome::Bounds(subject) => {
                            self.push_bound_task(run, Task::Project(source, binding, subject))?;
                            false
                        }
                    }
                }
                Task::Project(source, binding, subject) => {
                    self.queue_projected_tasks(run, source, binding, subject)?
                }
                Task::Written(source, binding, subject, index) => {
                    let bound = self.cached_bound(source, binding, index)?;
                    self.prepay_bound_clause(run)?;
                    if let Some(child) = self.complete_written_bound(
                        (binding.owner, binding.context),
                        bound,
                        subject,
                        &mut run.clauses,
                        &mut run.edges,
                    )? {
                        self.push_bound_task(run, Task::Binding(child))?;
                    }
                    false
                }
                Task::Retain(source, binding) => {
                    self.finish_bound_roots(source, binding)?;
                    false
                }
            };
            if pending {
                self.push_bound_task(run, task)?;
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn prepay_bound_clause(&mut self, run: &Run) -> Result<(), Error> {
        let growth = Growth::vector::<Predicate>(run.clauses.len(), run.clauses.capacity(), 1);
        self.charge_bound_staging(growth.work.saturating_add(1), growth.bytes)
    }
    fn queue_bound_binding(&mut self, run: &mut Run, binding: Binding) -> Result<(), Error> {
        if crate::sema::lifetimes::call_arguments(self.report, binding.path)?.is_some() {
            self.prepay_bound_clause(run)?;
            self.call_output_binding(
                binding.path,
                binding.predicate,
                binding.original,
                binding.scope,
                &mut run.clauses,
            )?;
            return Ok(());
        }
        let Some(list) = self.associated_bindings.bindings(binding.path) else {
            return Ok(());
        };
        if self.associated_bindings.trait_headers.is_some() {
            return Err(self.error(
                binding.path,
                "nested bound continuation precedes its trait header phase",
            ));
        }
        for offset in (list.start..list.end).rev() {
            let node = self
                .report
                .package
                .unit(binding.path.file)
                .configured
                .ast
                .edges[offset as usize];
            self.push_bound_task(
                run,
                Task::Constraint(
                    NodeKey {
                        node,
                        ..binding.path
                    },
                    binding,
                ),
            )?;
        }
        Ok(())
    }
    fn queue_projected_tasks(
        &mut self,
        run: &mut Run,
        source: NodeKey,
        binding: Binding,
        subject: (TypeId, TypeId),
    ) -> Result<bool, Error> {
        let prepared = self
            .associated_bindings
            .projected
            .as_ref()
            .and_then(|p| p.interpretations.get(&(source, binding)));
        let Some(prepared) = prepared else {
            self.prepare_projected_bounds(source, binding)?;
            return Ok(true);
        };
        let count = prepared.syntax.len();
        self.push_bound_task(run, Task::Retain(source, binding))?;
        for index in (0..count).rev() {
            self.push_bound_task(run, Task::Written(source, binding, subject, index))?;
        }
        Ok(false)
    }
    fn cached_bound(
        &self,
        source: NodeKey,
        binding: Binding,
        index: usize,
    ) -> Result<Syntax, Error> {
        self.associated_bindings
            .projected
            .as_ref()
            .and_then(|p| p.interpretations.get(&(source, binding)))
            .and_then(|p| p.syntax.get(index))
            .copied()
            .ok_or_else(|| self.error(source, "nested bound lost its actual written ordinal"))
    }
    fn finish_bound_roots(&mut self, source: NodeKey, binding: Binding) -> Result<(), Error> {
        let key = (source, binding);
        let mut prepared = self
            .associated_bindings
            .projected
            .as_mut()
            .and_then(|p| p.interpretations.remove(&key))
            .ok_or_else(|| self.error(source, "nested bound lost its prepared syntax"))?;
        if !prepared.retained {
            self.retain_projected_raw_bounds(binding.owner, &prepared.syntax, binding.context)?;
            prepared.retained = true;
        }
        let cache = self
            .associated_bindings
            .projected
            .as_ref()
            .ok_or_else(|| self.error(source, "nested bound lost its cache"))?;
        let growth = Growth::map::<Key, proj::Prepared>(
            cache.interpretations.len(),
            cache.interpretations.capacity(),
            1,
        );
        self.charge_bound_staging(growth.work.saturating_add(1), growth.bytes)?;
        self.associated_bindings
            .projected
            .as_mut()
            .ok_or_else(|| Error::unlocated("nested bound lost its cache"))?
            .interpretations
            .insert(key, prepared);
        Ok(())
    }
}

#[cfg(test)]
pub(super) const fn task_bytes() -> usize {
    size_of::<Task>()
}
#[cfg(test)]
impl Run {
    pub(super) fn prefix_len(&self) -> usize {
        self.clauses.len()
    }
}
