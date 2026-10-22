//! Inline bodies exchange their result interface without capturing enclosing locals.
mod completion;
use super::{
    Checker, Domain, Error, NodeId, Scratch, TermId, Work, closures::universes::Interface,
};
use crate::{
    sema::{
        bodies::nested::{Entry, InputRefresh, Payload},
        inline_constants::Id,
    },
    types::infer::{arguments::Argument, refinement::Refinement},
};

pub(super) struct Metadata {
    pub id: Id,
    pub result: TermId,
    pub lifetimes: Interface,
}

impl Checker<'_> {
    pub(super) fn enter_inline(
        &mut self,
        node: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let result = match expected {
            Some(result) => result,
            None => self.variable(node, Domain::Any)?,
        };
        self.save(node, result);
        // The inline body inherits every declared enclosing slot, even when no
        // earlier expression has needed the parent's lazy universal aliases.
        self.prepare_universals(node)?;
        let (mut inputs, roots) = Refinement::new(
            &self.facts.types,
            &mut self.state.infer,
            &[Argument::Type(result)],
        )
        .map_err(|e| self.error(node, e))?;
        let [Argument::Type(result)] = roots.as_slice() else {
            return Err(self.error(node, "missing inline constant result interface"));
        };
        let result = *result;
        let mut lifetimes = Interface::new(
            &self.state.call_types.universal_arguments(),
            Vec::new(),
            None,
        )
        .map_err(|e| self.error(node, e))?;
        let environment = self.nested_projection_environment(node, &mut lifetimes)?;
        let mapped_environment = environment.is_some();
        let generic = self.state.body_binders != 0
            || self
                .state
                .environment_owner
                .is_some_and(|owner| self.facts.generics.has_parameters(owner));
        let source = self.key(node);
        let id = self
            .facts
            .inline_constants
            .get_or_insert_with(Default::default)
            .reserve(
                source,
                self.state.body,
                self.state.environment_owner,
                generic,
            )
            .map_err(|e| self.error(node, e))?;
        let mut scratch = Scratch::default();
        scratch.resumable();
        let mut child = Checker::with_scratch(self.report, self.facts, scratch).state;
        child.infer = std::mem::take(&mut inputs.context);
        child.body = source;
        child.module = self.state.module;
        child.result_term = Some(result);
        child.in_function = false;
        child.safety = self.state.safety;
        child.body_binders = lifetimes.count().map_err(|e| self.error(node, e))?;
        child.environment_owner = self.state.environment_owner;
        if let Some(environment) = environment {
            child
                .projections
                .inherit_environment(environment)
                .map_err(|e| self.error(node, e))?;
        }
        child
            .method_environment
            .clone_from(&self.state.method_environment);
        if !mapped_environment {
            child
                .trait_environment
                .clone_from(&self.state.trait_environment);
        }
        child.prepare_parameter_queries();
        child.completion = super::readiness::Completion::Ordinary;
        self.park_nested(
            child,
            Entry {
                source: node,
                inputs,
                state: None,
                payload: Payload::Inline(Metadata {
                    id,
                    result,
                    lifetimes,
                }),
            },
        )?;
        if mapped_environment {
            self.prepare_universals(node)?;
        }
        self.state.work.push(Work::CollectInline);
        self.start_body(node)
    }

    pub(super) fn collect_inline(&mut self) -> Result<(), Error> {
        let body = self.state.body.node;
        let mut scheduler = self
            .state
            .closures
            .take()
            .ok_or_else(|| self.error(body, "missing inline body scheduler"))?;
        let index = scheduler
            .active
            .ok_or_else(|| self.error(body, "missing active inline body"))?;
        let (active, mut parent) = scheduler
            .stack
            .pop()
            .ok_or_else(|| self.error(body, "missing enclosing inline state"))?;
        let entry = &mut scheduler.entries[index];
        let metadata = entry
            .payload
            .inline_mut()
            .map_err(|e| self.error(body, e))?;
        self.prepare_nested_output(&mut metadata.lifetimes, parent.body)?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let mut closing = metadata
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(body, e))?;
        let pairs = entry
            .inputs
            .outgoing_regions(&mut self.facts.types, &mut parent.infer, &mut closing)
            .map_err(|e| self.error(body, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let source = entry.source;
        entry.state = Some(Box::new(std::mem::replace(&mut self.state, *parent)));
        scheduler.active = active;
        self.state.closures = Some(scheduler);
        self.state
            .work
            .push(Work::BodyInputs(Box::new(InputRefresh::outgoing(
                source, pairs,
            ))));
        self.defer_normalization(
            source,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.term(source)?,
        )
    }
}
