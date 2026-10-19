//! A ready inferred projection keeps its selected fold while declarations finish.
use super::{
    Checker, Completion, Error, Obligations, Query, ScopeId, Task, TypeId, WatchId,
    evidence::Evidence,
    selection::{Pending, ResultType},
};
use crate::{
    sema::projections::{Cursor, Progress},
    types::infer::deferred::Relation,
};

pub(super) struct Continuation {
    pub query: Query,
    pub watch: WatchId,
    cursor: Cursor,
    evidence: Evidence,
    cache: Option<TypeId>,
    selected: Option<Box<Pending>>,
    normalize_input: bool,
}
pub(super) enum Input {
    Resolved(TypeId),
    Selected(ResultType),
}
impl Checker<'_> {
    pub(super) fn start_projection_normalization(
        &mut self,
        query: Query,
        watch: WatchId,
        proof: Obligations,
        scope: ScopeId,
        input: Input,
    ) -> Result<Completion, Error> {
        let node = query.node;
        let normalize_input = matches!(input, Input::Selected(_));
        let (ty, selected) = match input {
            Input::Resolved(ty) | Input::Selected(ResultType::Type(ty)) => (Some(ty), None),
            Input::Selected(ResultType::Definition(selected)) => (None, Some(selected)),
        };
        let cache = ty.filter(|&ty| {
            !normalize_input
                && scope == ScopeId::ROOT
                && proof.is_empty()
                && self.facts.types.has_free_regions(ty, 0) == Some(false)
        });
        if let Some(input) = cache
            && let Some(result) = self
                .state
                .projections
                .data
                .as_ref()
                .and_then(|data| data.normal_forms.get(&input))
                .copied()
        {
            return self.publish_projection_query(&query, Evidence::new(proof), result, false);
        }
        let environment = self.projection_environment(node)?;
        let mut cursor = Cursor::new(environment, &proof, scope, self.availability)
            .map_err(|e| self.error(node, e))?;
        if let Some(ty) = ty {
            cursor
                .start(self.facts, ty)
                .map_err(|e| self.error(node, e))?;
        }
        self.advance_projection_normalization(Continuation {
            query,
            watch,
            cursor,
            evidence: Evidence::new(proof),
            cache,
            selected,
            normalize_input,
        })
    }

    pub(super) fn resume_projection(
        &mut self,
        mut continuation: Continuation,
    ) -> Result<Completion, Error> {
        continuation
            .evidence
            .resume_normalization(self.facts, &mut continuation.cursor)
            .map_err(|e| self.error(continuation.query.node, e))?;
        self.advance_projection_normalization(continuation)
    }

    fn advance_projection_normalization(
        &mut self,
        mut continuation: Continuation,
    ) -> Result<Completion, Error> {
        let node = continuation.query.node;
        if let Some(selected) = &mut continuation.selected {
            let Some((ty, steps)) = selected
                .result(self.facts)
                .map_err(|e| self.error(node, e))?
            else {
                let definition = selected.definition.id;
                return self.suspend_projection(
                    continuation,
                    super::super::Requirement::Definition(definition),
                );
            };
            self.state
                .infer
                .account_work(steps)
                .map_err(|e| self.error(node, e))?;
            continuation
                .cursor
                .start(self.facts, ty)
                .map_err(|e| self.error(node, e))?;
            continuation.selected = None;
        }
        let mut selections = 0;
        let progress = continuation.cursor.advance(
            self.facts,
            &mut continuation.evidence.proof,
            &mut |_, _| {
                selections += 1;
                Ok(None)
            },
        );
        self.facts.body_work.projection_selections += selections;
        match progress.map_err(|e| self.error(node, e))? {
            Progress::Definition(definition) => self.suspend_projection(
                continuation,
                super::super::Requirement::Definition(definition),
            ),
            Progress::Header(owner) => {
                self.suspend_projection(continuation, super::super::Requirement::ImplHeader(owner))
            }
            Progress::Complete { ty, steps } => {
                self.facts.body_work.projection_steps += steps;
                self.state
                    .infer
                    .account_work(steps)
                    .map_err(|e| self.error(node, e))?;
                if let Some(input) = continuation
                    .cache
                    .filter(|_| continuation.evidence.proof.is_empty())
                {
                    self.state
                        .projections
                        .data
                        .get_or_insert_with(Default::default)
                        .normal_forms
                        .insert(input, ty);
                }
                self.publish_projection_query(
                    &continuation.query,
                    continuation.evidence,
                    ty,
                    continuation.normalize_input,
                )
            }
        }
    }

    fn suspend_projection(
        &mut self,
        mut continuation: Continuation,
        requirement: super::super::Requirement,
    ) -> Result<Completion, Error> {
        self.facts.body_work.projection_waits += 1;
        let node = continuation.query.node;
        continuation
            .evidence
            .reserve_normalization(
                self.facts,
                self.state.body,
                self.key(node),
                &continuation.cursor,
            )
            .map_err(|e| self.error(node, e))?;
        Ok(Completion::Dependency(
            requirement,
            super::Continuation::Normalization(Box::new(continuation)),
        ))
    }

    fn publish_projection_query(
        &mut self,
        query: &Query,
        evidence: Evidence,
        ty: TypeId,
        normalize_input: bool,
    ) -> Result<Completion, Error> {
        let node = query.node;
        if normalize_input && let Task::Normalize { input, .. } = query.task {
            self.state
                .infer
                .apply_normalization(&mut self.facts.types, input, ty)
                .map_err(|e| self.error(node, e))?;
        }
        let (target, published, output) = match query.task {
            Task::Normalize { output, .. } => (output, None, Some(output)),
            Task::Symbolic { input, output } => (input, Some((input, output)), None),
            Task::Contract(_) => {
                return Err(self.error(node, "contract cannot publish a single projection result"));
            }
        };
        self.state
            .infer
            .apply_normalization(&mut self.facts.types, target, ty)
            .map_err(|e| self.error(node, e))?;
        if let Some((input, output)) = published {
            self.deferred_equation(node, input, output, Relation::Equality)?;
        }
        evidence
            .publish(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))?;
        if let Some(output) = output {
            self.published_normalization(node, output)?;
        }
        Ok(Completion::Complete)
    }
}
