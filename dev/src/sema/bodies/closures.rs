//! Closure bodies retain separate control state and exchange only interface constraints.
mod completion;
mod contextual;
mod interface;
mod universes;
use super::{
    Checker, Constructor, Domain, Error, Kind, NodeId, Scratch, State, TermId, TypeId, Work,
};
use crate::{
    syntax::ast::List,
    types::{
        closure::{CallKind, ClosureId},
        infer::{arguments::Argument, refinement::Refinement},
        substitute::Bound,
    },
};

/// All parked states have `closures: None`; nesting never owns a recursive tree.
#[derive(Default)]
pub(super) struct Scheduler {
    entries: Vec<Entry>,
    stack: Vec<(Option<usize>, Box<State>)>,
    active: Option<usize>,
}
pub(super) struct InputRefresh {
    source: NodeId,
    pairs: Vec<(Argument, Argument)>,
}
struct Entry {
    source: NodeId,
    identity: ClosureId,
    parameters: List,
    signature: TermId,
    inputs: Refinement,
    lifetimes: universes::Interface,
    state: Option<Box<State>>,
}
impl Checker<'_> {
    pub(super) fn enter_closure(
        &mut self,
        node: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let Kind::Closure {
            parameters,
            result,
            body,
            asynchronous,
            ..
        } = self.kind(node)
        else {
            return Err(self.error(node, "missing closure syntax"));
        };
        if asynchronous {
            return Err(self.error(node, "async closure bodies are not implemented"));
        }
        let nodes: Vec<_> = self
            .report
            .package
            .unit(self.state.file)
            .configured
            .ast
            .list(parameters)
            .iter()
            .copied()
            .filter(|&node| self.active(node))
            .collect();
        let count =
            u32::try_from(nodes.len()).map_err(|_| self.error(node, "too many closure inputs"))?;
        let mut terms = Vec::with_capacity(nodes.len() + 1);
        for _ in 0..=nodes.len() {
            terms.push(self.variable(node, Domain::Any)?);
        }
        let annotations = self
            .facts
            .closures
            .inputs
            .get(&self.key(node))
            .cloned()
            .unwrap_or_default();
        let expected = self.closure_expectation(node, expected, count)?;
        let bound_regions = expected
            .map_or_else(
                || u32::try_from(annotations.len()),
                |expected| Ok(expected.regions),
            )
            .map_err(|_| self.error(node, "too many closure lifetimes"))?;
        let signature = self
            .state
            .infer
            .construct(
                Constructor::Function {
                    bound_regions,
                    parameters: count,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &terms,
            )
            .map_err(|e| self.error(node, e))?;
        let identity = self
            .facts
            .types
            .reserve_closure()
            .map_err(|e| self.error(node, e))?;
        let environment = self.known(node, TypeId::UNIT)?;
        let arguments = self.closure_arguments(node)?;
        let closure = self
            .state
            .infer
            .closure(identity, signature, environment, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.save(node, closure);
        self.state.work.push(Work::Exit(node));
        let mut lifetimes = universes::Interface::new(
            &self.state.call_types.universal_arguments(),
            annotations,
            expected.map(|expected| expected.regions),
        )
        .map_err(|e| self.error(node, e))?;
        let (inputs, signature) =
            self.closure_body_interface(node, &terms, count, &mut lifetimes, expected)?;
        self.switch_closure_body(
            body,
            Entry {
                source: node,
                identity,
                parameters,
                signature,
                inputs,
                lifetimes,
                state: None,
            },
        )?;
        self.closure_parameters(node, signature, nodes, result)?;
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, signature)?;
        self.advance_projections(node)?;
        self.state.work.push(Work::CollectClosure);
        self.start_body(body)
    }

    fn closure_parameters(
        &mut self,
        node: NodeId,
        signature: TermId,
        nodes: Vec<NodeId>,
        result: Option<NodeId>,
    ) -> Result<(), Error> {
        let output = self.child(node, signature, nodes.len())?;
        self.state.result_term = Some(output);
        for (index, mut parameter) in nodes.into_iter().enumerate() {
            while let Kind::Attributed { value, .. } = self.kind(parameter) {
                parameter = value;
            }
            let Kind::Parameter { pattern, ty } = self.kind(parameter) else {
                return Err(self.error(parameter, "missing closure parameter"));
            };
            let term = self.child(parameter, signature, index)?;
            if let Some(ty) = ty {
                let annotation = self.annotation(ty)?;
                self.equate(parameter, term, annotation)?;
            }
            self.binding(pattern, term, None)?;
            self.save(parameter, term);
        }
        if let Some(result) = result {
            let annotation = self.annotation(result)?;
            self.equate(result, output, annotation)?;
        }
        Ok(())
    }

    fn switch_closure_body(&mut self, body: NodeId, mut entry: Entry) -> Result<(), Error> {
        let mut scratch = Scratch::default();
        scratch.resumable();
        let mut child = Checker::with_scratch(self.report, self.facts, scratch).state;
        child.infer = std::mem::take(&mut entry.inputs.context);
        child.body = self.key(body);
        child.module = self.state.module;
        child.in_function = true;
        child.safety = self.state.safety;
        child.body_binders = entry
            .lifetimes
            .count()
            .map_err(|e| self.error(entry.source, e))?;
        child.infer.region_equalities.extend(
            entry
                .lifetimes
                .equations()
                .map_err(|e| self.error(entry.source, e))?,
        );
        child.environment_owner = self.state.environment_owner;
        child
            .method_environment
            .clone_from(&self.state.method_environment);
        child.prepare_parameter_queries();
        child
            .trait_environment
            .clone_from(&self.state.trait_environment);
        child.completion = super::readiness::Completion::Ordinary;
        let mut scheduler = self.state.closures.take().unwrap_or_default();
        if scheduler.entries.len() >= 1_000_000 {
            return Err(self.error(entry.source, "closure body work limit exceeded"));
        }
        let index = scheduler.entries.len();
        scheduler.entries.push(entry);
        self.state.closure_children.push(index);
        self.state.retain_sparse();
        let parent = std::mem::replace(&mut self.state, child);
        scheduler.stack.push((scheduler.active, Box::new(parent)));
        scheduler.active = Some(index);
        self.state.closures = Some(scheduler);
        self.prepare_file(self.state.body.file);
        Ok(())
    }

    pub(super) fn collect_closure(&mut self) -> Result<(), Error> {
        let body = self.state.body.node;
        let mut scheduler = self
            .state
            .closures
            .take()
            .ok_or_else(|| self.error(body, "missing closure scheduler"))?;
        let index = scheduler
            .active
            .ok_or_else(|| self.error(body, "missing active closure"))?;
        let (active, mut parent) = scheduler
            .stack
            .pop()
            .ok_or_else(|| self.error(body, "missing enclosing closure state"))?;
        let entry = &mut scheduler.entries[index];
        // Until capture lowering is connected, any outer local use fails ordinary
        // value lookup in this isolated body. Reaching here proves an empty environment.
        self.facts
            .types
            .complete_closure(entry.identity, CallKind::Shared)
            .map_err(|e| self.error(body, e))?;
        self.closure_result_boundary(&entry.lifetimes)?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let mut identity = entry
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(body, e))?;
        let pairs = entry
            .inputs
            .outgoing_regions(&mut self.facts.types, &mut parent.infer, &mut identity)
            .map_err(|e| self.error(body, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let source = entry.source;
        entry.state = Some(Box::new(std::mem::replace(&mut self.state, *parent)));
        scheduler.active = active;
        self.state.closures = Some(scheduler);
        self.state
            .work
            .push(Work::ClosureInputs(Box::new(InputRefresh {
                source,
                pairs,
            })));
        // Importing interface constraints grants no normalization proof in the
        // receiving body. Register the new occurrences under this owner.
        self.defer_normalization(
            source,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.term(source)?,
        )?;
        Ok(())
    }
}
