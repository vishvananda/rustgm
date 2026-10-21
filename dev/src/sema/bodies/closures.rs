//! Closure bodies retain separate control state and exchange only interface constraints.
mod completion;
mod contextual;
mod interface;
pub(super) mod universes;
use super::{Checker, Constructor, Domain, Error, Kind, NodeId, Scratch, TermId, TypeId, Work};
use crate::{
    syntax::ast::List,
    types::{
        closure::{CallKind, ClosureId},
        infer::{arguments::Argument, refinement::Refinement},
        substitute::Bound,
    },
};

pub(super) type Scheduler = super::nested::Scheduler<super::nested::Payload>;
type Entry = super::nested::Entry<super::nested::Payload>;
use super::nested::{InputRefresh, Payload};
pub(super) struct Metadata {
    identity: ClosureId,
    parameters: List,
    signature: TermId,
    lifetimes: universes::Interface,
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
        self.check_closure_body_owner(body)?;
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
                inputs,
                state: None,
                payload: Payload::Closure(Metadata {
                    identity,
                    parameters,
                    signature,
                    lifetimes,
                }),
            },
        )?;
        self.closure_parameters(node, signature, nodes, result)?;
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, signature)?;
        self.advance_projections(node)?;
        self.state.work.push(Work::CollectClosure);
        self.start_body(body)
    }

    fn check_closure_body_owner(&self, body: NodeId) -> Result<(), Error> {
        if matches!(
            self.kind(body),
            Kind::Block {
                flavor: crate::syntax::ast::BlockFlavor::Const,
                ..
            }
        ) {
            return Err(self.error(
                body,
                "inline constant as a closure body requires distinct checked body ownership",
            ));
        }
        Ok(())
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
        let metadata = entry
            .payload
            .closure()
            .map_err(|e| self.error(entry.source, e))?;
        child.body_binders = metadata
            .lifetimes
            .count()
            .map_err(|e| self.error(entry.source, e))?;
        child.infer.region_equalities.extend(
            metadata
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
        self.park_nested(child, entry)
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
        let metadata = entry
            .payload
            .closure_mut()
            .map_err(|e| self.error(entry.source, e))?;
        // Until capture lowering is connected, any outer local use fails ordinary
        // value lookup in this isolated body. Reaching here proves an empty environment.
        self.facts
            .types
            .complete_closure(metadata.identity, CallKind::Shared)
            .map_err(|e| self.error(body, e))?;
        self.closure_result_boundary(&metadata.lifetimes)?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let mut identity = metadata
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
            .push(Work::BodyInputs(Box::new(InputRefresh { source, pairs })));
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
