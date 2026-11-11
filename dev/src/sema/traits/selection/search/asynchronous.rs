//! Automatic async callable evidence proves the actual returned Future and trait inputs.
use super::ambient::Cursor;
use super::mechanical::Pending as Language;
use super::{Action, Decision, Declarations, Frame, Node, Predicate, Search, Source, inference};
use crate::{
    sema::traits::{
        Requirements,
        callables::{self, Matched, asynchronous},
    },
    support::constness::Constness,
    types::{
        infer::{self, Constructor},
        parameters::clauses::Clause,
    },
};
use std::{ops::Range, sync::Arc};

pub(super) struct Pending {
    matched: Matched,
    inputs: Option<Box<Inputs>>,
}
const _: [(); 32] = [(); size_of::<Pending>()];
const _: [(); 96] = [(); size_of::<Language>()];
const _: [(); 1312] = [(); size_of::<Frame>()];
const _: [(); 160] = [(); size_of::<Node>()];
struct Inputs {
    goal: Predicate,
    future: Predicate,
    requirements: Arc<Requirements>,
    cursor: Cursor,
}
impl Pending {
    pub(super) fn regions(&self) -> Range<u32> {
        self.matched.regions.clone()
    }
    pub(super) fn retained_child(&self) -> Option<usize> {
        self.inputs
            .as_deref()
            .and_then(|inputs| inputs.cursor.suspended)
    }
    pub(super) fn condition(&mut self) -> Result<(Option<Clause>, &mut Cursor), String> {
        let inputs = self
            .inputs
            .as_deref_mut()
            .ok_or("missing async conditions")?;
        Ok((inputs.clause(), &mut inputs.cursor))
    }
}
impl Inputs {
    fn clause(&self) -> Option<Clause> {
        let required = &self.requirements.required;
        let implied = &self.requirements.implied;
        if self.cursor.index < required.len() {
            return required.get(self.cursor.index).copied();
        }
        let index = self.cursor.index - required.len();
        if index < implied.len() {
            return implied.get(index).copied();
        }
        if index == implied.len() {
            return Some(
                Predicate {
                    ty: self.future.ty,
                    bound: crate::types::parameters::predicates::Bound::Sized,
                }
                .into(),
            );
        }
        (index == implied.len() + 1).then_some(self.future.into())
    }
}
impl Frame {
    fn async_callable(&mut self) -> Result<&mut Pending, String> {
        match self.language.as_deref_mut() {
            Some(Language::Async(pending)) => Ok(pending),
            _ => Err("missing async callable proof frontier".into()),
        }
    }
}
impl Search<'_> {
    pub(super) fn start_async_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some((item, _)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(kind) = asynchronous::kind(facts, item)? else {
            return Ok(None);
        };
        // User implementations and the real source reference forwarders retain
        // their ordinary candidate, conformance and method-body proof path.
        match frame
            .inputs
            .context
            .head(&facts.types, frame.goal.subject())
            .map_err(inference)?
        {
            Some(
                Constructor::Function { .. }
                | Constructor::FunctionItem(..)
                | Constructor::FunctionItemArgs(..)
                | Constructor::Closure(..),
            ) => {}
            _ => return Ok(None),
        }
        if frame.goal.constness() != Some(Constness::Ordinary) {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        let first = frame.available_region().max(self.query.allocations.floor());
        let matched = match callables::match_signature_as(
            facts,
            &mut frame.inputs.context,
            &frame.goal,
            kind,
            first,
        ) {
            Ok(matched) => matched,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                return Ok(Some(Action::Complete(Decision::Absent)));
            }
            Err(error) => return Err(inference(error)),
        };
        facts.charge_caller_inputs(1, size_of::<Language>())?;
        self.query.allocations.record(matched.regions.end);
        self.query.reserve(frame.scope, matched.regions.clone())?;
        frame.language = Some(Box::new(Language::Async(Pending {
            matched,
            inputs: None,
        })));
        self.complete_async_callable(facts, frame).map(Some)
    }

    fn prepare_async_inputs(facts: &mut Declarations, frame: &mut Frame) -> Result<bool, String> {
        if frame.async_callable()?.inputs.is_some() {
            return Ok(true);
        }
        let output = frame.async_callable()?.matched.output;
        let resolved = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .and_then(|goal| {
                frame
                    .inputs
                    .context
                    .resolved(&mut facts.types, output)
                    .map(|future| (goal, future))
            });
        let (goal, future) = match resolved {
            Ok(value) => value,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => return Ok(false),
            Err(error) => return Err(inference(error)),
        };
        let future = asynchronous::future_goal(facts, future)?;
        let payments =
            crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
                .map_err(|e| e.to_string())?;
        let requirements = facts
            .generics
            .traits
            .as_deref_mut()
            .ok_or("missing async trait contracts")?
            .requirements_counted(&mut facts.types, goal, &|work, bytes| {
                payments.charge(work, bytes)
            })?;
        let count = requirements
            .required
            .len()
            .saturating_add(requirements.implied.len())
            .saturating_add(2);
        payments.charge(
            count,
            size_of::<Inputs>().saturating_add(count.saturating_mul(size_of::<usize>())),
        )?;
        frame.async_callable()?.inputs = Some(Box::new(Inputs {
            goal,
            future,
            requirements,
            cursor: Cursor::with_capacity(count),
        }));
        Ok(true)
    }

    pub(super) fn complete_async_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let Some(Language::Async(pending)) = frame.language.as_deref() else {
            return Err("missing async callable equations".into());
        };
        match pending
            .matched
            .permitted(facts, &mut frame.inputs.context, &frame.goal)?
        {
            callables::Eligibility::Allowed => {}
            callables::Eligibility::Rejected => return Ok(Action::Complete(Decision::Absent)),
            callables::Eligibility::Pending => return Ok(self.pending_async_callable(frame)),
            _ => return Err("async callable requires ordinary capture permission".into()),
        }
        if !Self::prepare_async_inputs(facts, frame)? {
            return Ok(self.pending_async_callable(frame));
        }
        self.advance_callable_conditions(facts, frame)
    }

    fn pending_async_callable(&self, frame: &Frame) -> Action {
        if self.query.refining && frame.scope == self.query.input_scope() {
            Action::Yield
        } else {
            Action::Complete(Decision::Deferred)
        }
    }

    pub(super) fn finish_async_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let regions = frame.async_callable()?.regions();
        let inputs = frame
            .async_callable()?
            .inputs
            .take()
            .ok_or("missing completed async inputs")?;
        let future = inputs
            .cursor
            .children
            .last()
            .ok_or("async callable has no Future proof")?;
        if self.query.nodes[*future].goal != inputs.future {
            return Err("async callable child differs from its actual returned Future".into());
        }
        let growth = self.query.node_growth();
        facts.charge_caller_inputs(
            growth.work.saturating_add(1),
            size_of::<asynchronous::Selected>()
                + 2 * size_of::<usize>()
                + inputs.requirements.regions.len() * size_of::<crate::types::outlives::Pair>()
                + growth.bytes,
        )?;
        let selected = asynchronous::Selected::new(facts, inputs.goal, inputs.future)?;
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal: inputs.goal,
                source: Some(Source::AsyncCallable(Arc::new(selected))),
                children: inputs.cursor.children,
                first: frame.first,
                end: regions.end.max(self.query.allocations.end()),
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: inputs.requirements.regions.clone(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            regions,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
}
