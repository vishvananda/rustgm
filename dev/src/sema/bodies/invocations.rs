//! Call syntax selects a receiver permission before argument checking and retains its proof.
use super::{Checker, Constructor, Error, List, NodeId, TermId};
use crate::{
    sema::{
        invocations::Invocation,
        model::BodyRegionOrigin,
        traits::selection::{self, Caller, Selection},
    },
    support::call_traits::CallTrait,
    types::{
        ItemId,
        infer::{Context, Domain, Solved, arguments::Argument, predicates::Predicate},
    },
};

pub(super) struct Pending {
    kind: Option<CallTrait>,
    dereferences: u32,
    borrow: Option<TermId>,
    pub receiver: TermId,
}
impl Checker<'_> {
    pub(super) fn invocation_signature(
        &mut self,
        node: NodeId,
        callee: NodeId,
        arguments: List,
    ) -> Result<TermId, Error> {
        let mut subject = self.term(callee)?;
        let mut dereferences = 0_u32;
        loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            let head = self.head(node, subject)?;
            if matches!(head, Some(Constructor::Reference { .. })) {
                // Call syntax follows the referent; a bound on the reference
                // alone is not a callable permission on that referent.
                subject = self.child(node, subject, 0)?;
                dereferences = dereferences
                    .checked_add(1)
                    .ok_or_else(|| self.error(node, "call receiver depth exhausted"))?;
                continue;
            }

            if let Some(Constructor::Closure(identity, _)) = head {
                let kind = match self.facts.types.closure_kind(identity) {
                    Some(crate::types::closure::CallKind::Shared) => CallTrait::Fn,
                    Some(crate::types::closure::CallKind::Mutable) => CallTrait::FnMut,
                    Some(crate::types::closure::CallKind::Once) => CallTrait::FnOnce,
                    None => return Err(self.error(node, "closure call permission is unresolved")),
                };
                let signature = self.child(node, subject, 0)?;
                self.retain_invocation(node, callee, subject, dereferences, Some(kind))?;
                return self.call_signature(node, signature);
            }

            let signature = match head {
                Some(Constructor::Function { .. }) => Some(subject),
                Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..)) => {
                    Some(self.child(node, subject, 0)?)
                }
                _ => None,
            };
            if let Some(signature) = signature {
                self.retain_invocation(node, callee, subject, dereferences, None)?;
                return self.call_signature(node, signature);
            }
            if head.is_none() {
                return Err(self.error(node, "call requires a known function type"));
            }
            if let Some((kind, tuple)) = self.select_invocation(node, subject, head)? {
                let signature = self.invocation_inputs(node, subject, tuple, arguments)?;
                self.retain_invocation(node, callee, subject, dereferences, Some(kind))?;
                return Ok(signature);
            }
            return Err(self.error(node, "call requires a known function type"));
        }
    }

    fn select_invocation(
        &mut self,
        node: NodeId,
        subject: TermId,
        head: Option<Constructor>,
    ) -> Result<Option<(CallTrait, TermId)>, Error> {
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        for kind in CallTrait::ALL {
            let Some(item) = self.facts.associated.callable_definition(kind) else {
                continue;
            };
            if let Some(Constructor::Parameter(id)) = head
                && !assumptions.parameter_traits(id).contains(&ItemId(item.0))
            {
                continue;
            }
            if !self.possible_call_kind(node, subject, ItemId(item.0), &assumptions)? {
                continue;
            }
            // Only the chosen kind creates caller variables. Rejected alternatives
            // must not leave unconstrained tuple holes in the body's inference graph.
            let tuple = self.variable(node, Domain::Any)?;
            let goal =
                Predicate::trait_application(subject, ItemId(item.0), vec![Argument::Type(tuple)]);
            let first_region = self.trait_region_start(node)?;
            let selected = selection::select_available(
                self.facts,
                &mut self.state.infer,
                &goal,
                Caller {
                    assumptions: &assumptions,
                    environment: self.state.method_environment.as_ref(),
                    first_region,
                    retain_sized: false,
                },
                self.availability,
            )
            .map_err(|e| self.error(node, e))?;
            match selected {
                Selection::Proven(evidence) => {
                    self.retain_trait_evidence(node, evidence)?;
                }
                Selection::Deferred => {}
                Selection::Absent => {
                    return Err(self.error(node, "selected call permission changed"));
                }
            }
            self.state.call_types.obligations.push((node, goal));
            return Ok(Some((kind, tuple)));
        }
        Ok(None)
    }

    fn possible_call_kind(
        &mut self,
        node: NodeId,
        subject: TermId,
        item: ItemId,
        assumptions: &crate::sema::traits::assumptions::Assumptions,
    ) -> Result<bool, Error> {
        let mut probe = Context::default();
        let subject = probe
            .import(&self.facts.types, &mut self.state.infer, subject)
            .map_err(|e| self.error(node, e))?;
        let tuple = probe
            .variable(Domain::Any)
            .map_err(|e| self.error(node, e))?;
        let goal = Predicate::trait_application(subject, item, vec![Argument::Type(tuple)]);
        let first_region = self.trait_region_start(node)?;
        let selected = selection::select_available(
            self.facts,
            &mut probe,
            &goal,
            Caller {
                assumptions,
                environment: self.state.method_environment.as_ref(),
                first_region,
                retain_sized: false,
            },
            self.availability,
        );
        self.state
            .infer
            .account_work(probe.steps)
            .map_err(|e| self.error(node, e))?;
        Ok(!matches!(
            selected.map_err(|e| self.error(node, e))?,
            Selection::Absent
        ))
    }

    fn invocation_inputs(
        &mut self,
        node: NodeId,
        subject: TermId,
        tuple: TermId,
        arguments: List,
    ) -> Result<TermId, Error> {
        if self.head(node, tuple)?.is_none() {
            let count = self
                .report
                .package
                .unit(self.state.file)
                .configured
                .ast
                .list(arguments)
                .iter()
                .filter(|&&node| self.active(node))
                .count();
            self.state
                .infer
                .account_work(count)
                .map_err(|e| self.error(node, e))?;
            let mut fields = Vec::with_capacity(count);
            for _ in 0..count {
                fields.push(self.variable(node, Domain::Any)?);
            }
            let inferred = self
                .state
                .infer
                .construct(Constructor::Tuple, &fields)
                .map_err(|e| self.error(node, e))?;
            self.equate(node, tuple, inferred)?;
        }
        if self.head(node, tuple)? != Some(Constructor::Tuple) {
            return Err(self.error(node, "call trait arguments are not a tuple"));
        }
        let count = self
            .state
            .infer
            .arity(&self.facts.types, tuple)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "missing call tuple arity"))?;
        let mut fields = Vec::with_capacity(count + 1);
        for index in 0..count {
            fields.push(self.child(node, tuple, index)?);
        }
        let output = self
            .facts
            .associated
            .callable_output()
            .ok_or_else(|| self.error(node, "missing call Output"))?;
        let result = self
            .state
            .infer
            .projected(
                &self.facts.types,
                ItemId(output.0),
                subject,
                &[Argument::Type(tuple)],
                &[],
            )
            .map_err(|e| self.error(node, e))?;
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, result)?;
        fields.push(result);
        let parameters =
            u32::try_from(count).map_err(|_| self.error(node, "too many call arguments"))?;
        self.state
            .infer
            .construct(
                Constructor::Function {
                    parameters,
                    bound_regions: 0,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &fields,
            )
            .map_err(|e| self.error(node, e))
    }

    fn retain_invocation(
        &mut self,
        node: NodeId,
        source: NodeId,
        subject: TermId,
        dereferences: u32,
        kind: Option<CallTrait>,
    ) -> Result<(), Error> {
        let borrow = match kind {
            Some(CallTrait::Fn | CallTrait::FnMut) => {
                let mutable = kind == Some(CallTrait::FnMut);
                let region = self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?;
                let term = self
                    .state
                    .infer
                    .construct(Constructor::Reference { mutable, region }, &[subject])
                    .map_err(|e| self.error(node, e))?;
                if dereferences == 0 {
                    self.state.alignment.borrows.push((node, source));
                }
                Some(term)
            }
            _ => None,
        };
        self.state.call_types.field_receivers.push(subject);
        self.state.call_types.invocations.push((
            node,
            Pending {
                kind,
                dereferences,
                borrow,
                receiver: borrow.unwrap_or(subject),
            },
        ));
        Ok(())
    }

    pub(super) fn publish_invocations(&mut self, solved: &Solved) {
        for (node, pending) in &self.state.call_types.invocations {
            self.facts.invocations.insert(
                self.key(*node),
                Invocation {
                    kind: pending.kind,
                    dereferences: pending.dereferences,
                    borrow: pending.borrow.map(|term| solved.types[term.0 as usize]),
                    receiver: solved.types[pending.receiver.0 as usize],
                },
            );
        }
    }
}
