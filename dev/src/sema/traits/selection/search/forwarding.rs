//! Reference call implementations retain the referent's complete child proof.
use super::callables::Pending as Callable;
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, Source, inference};
use crate::{
    sema::traits::heads::{Head, Subject},
    support::call_traits::CallTrait,
    types::{
        GenericArgs, ItemId, Kind,
        infer::{self, Constructor},
        parameters::predicates::Bound,
        projection,
        traits::Application,
    },
};

pub(super) struct Pending {
    goal: Goal,
    selected: Option<usize>,
    stopped: Option<Decision>,
    suspended: Option<usize>,
    publish: bool,
}

impl Frame {
    fn forwarding(&mut self) -> Result<&mut Pending, String> {
        match self.callable.as_deref_mut() {
            Some(Callable::Forward(pending)) => Ok(pending),
            _ => Err("missing reference callable proof".into()),
        }
    }
}

impl Search<'_> {
    pub(super) fn start_forwarding(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        required: CallTrait,
    ) -> Result<Option<Action>, String> {
        let mut referent = frame.goal.subject();
        let mut required = required;
        let mut forwarded = false;
        loop {
            let head = match frame.inputs.context.head(&facts.types, referent) {
                Ok(head) => head,
                Err(infer::Error::Normalization) => break,
                Err(error) => return Err(inference(error)),
            };
            let Some(Constructor::Reference { mutable, .. }) = head else {
                break;
            };
            frame.inputs.context.account_work(1).map_err(inference)?;
            let item = facts
                .associated
                .callable_definition(required)
                .ok_or("missing forwarded call trait")?;
            // A caller may promise that an intermediate reference is callable
            // without promising anything about its referent. Keep that query.
            if forwarded
                && self
                    .caller
                    .assumptions
                    .may_have_subject(ItemId(item.0), Subject::Rigid(Head::Reference(mutable)))
            {
                break;
            }
            let Some(next) = required.referent(mutable) else {
                return Ok(Some(Action::Complete(Decision::Absent)));
            };
            required = next;
            referent = frame
                .inputs
                .context
                .child(&facts.types, referent, 0)
                .map_err(inference)?;
            forwarded = true;
        }
        if !forwarded {
            return Ok(None);
        }
        let item = facts
            .associated
            .callable_definition(required)
            .ok_or("missing forwarded call trait")?;
        let (_, arguments) = frame
            .goal
            .trait_arguments()
            .ok_or("missing call tuple arguments")?;
        frame
            .inputs
            .context
            .account_work(arguments.len())
            .map_err(inference)?;
        let goal = Goal::trait_application(referent, ItemId(item.0), arguments.to_vec());
        frame.callable = Some(Box::new(Callable::Forward(Pending {
            goal,
            selected: None,
            stopped: None,
            suspended: None,
            publish: false,
        })));
        Ok(Some(Action::Descend))
    }

    pub(super) fn callable_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        let first = frame.available_region().max(self.query.allocations.floor());
        let Some(Callable::Forward(pending)) = frame.callable.as_deref_mut() else {
            return Ok(None);
        };
        let mut child = if let Some(slot) = pending.suspended.take() {
            let mut child = self
                .query
                .suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained callable child")?;
            child.refresh(facts, &mut frame.inputs.context)?;
            return Ok(Some(child));
        } else {
            Frame::new(facts, &mut frame.inputs.context, &pending.goal, first)?
        };
        child.scope = frame.scope;
        child.inference = if frame.inference == super::frames::Inference::Disabled {
            super::frames::Inference::Disabled
        } else {
            super::frames::Inference::Selecting
        };
        Ok(Some(child))
    }

    pub(super) fn accept_forwarding(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let Some(Callable::Forward(pending)) = frame.callable.as_deref_mut() else {
            return Err("missing forwarded call parent".into());
        };
        if let Decision::Proven(node) = decision {
            frame
                .inputs
                .context
                .equate_proved_predicate(
                    &mut facts.types,
                    &pending.goal,
                    self.query.nodes[node].goal,
                )
                .map_err(inference)?;
            pending.selected = Some(node);
        } else {
            pending.stopped = Some(decision);
        }
        Ok(())
    }

    pub(super) fn accept_partial_forwarding(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        child.publish(facts, &mut frame.inputs.context)?;
        let slot = self.query.retain_partial(child)?;
        let pending = frame.forwarding()?;
        pending.suspended = Some(slot);
        pending.publish = true;
        Ok(())
    }

    pub(super) fn complete_forwarding(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let pending = frame.forwarding()?;
        if let Some(decision) = pending.stopped {
            return Ok(Action::Complete(decision));
        }
        if std::mem::take(&mut pending.publish) {
            return Ok(Action::Yield);
        }
        let Some(child) = pending.selected else {
            return Ok(Action::Descend);
        };
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved)
                if self.query.refining && frame.scope == self.query.input_scope() =>
            {
                return Ok(Action::Yield);
            }
            Err(infer::Error::Unresolved) => return Ok(Action::Complete(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        };
        let output = self.forwarded_output(facts, child)?;
        let node = self.query.push(
            Node {
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Callable { output }),
                children: vec![child],
                first: frame.first,
                end: self.query.nodes[child].end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }

    fn forwarded_output(
        &self,
        facts: &mut Declarations,
        child: usize,
    ) -> Result<crate::types::TypeId, String> {
        let proof = &self.query.nodes[child];
        match proof.source {
            Some(Source::Callable { output }) => return Ok(output),
            Some(
                Source::Assumption(_)
                | Source::ResultBound(_)
                | Source::QuantifiedAssumption { .. }
                | Source::QuantifiedResultBound { .. },
            ) => {}
            _ => return Err("reference callable requires callable or caller evidence".into()),
        }
        let Bound::Trait(application) = proof.goal.bound else {
            return Err("forwarded call proof has no application".into());
        };
        let arguments = facts
            .types
            .trait_application(application)
            .ok_or("missing forwarded call inputs")?
            .arguments;
        let owner = facts
            .associated
            .callable_definition(CallTrait::FnOnce)
            .ok_or("missing FnOnce identity")?;
        let member = facts
            .associated
            .callable_output()
            .ok_or("missing FnOnce Output identity")?;
        let trait_application = facts.types.intern_trait_application(Application {
            item: ItemId(owner.0),
            arguments,
        })?;
        facts
            .types
            .intern(Kind::Projection(projection::Application {
                item: ItemId(member.0),
                subject: proof.goal.ty,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))
            .map_err(str::to_owned)
    }
}
