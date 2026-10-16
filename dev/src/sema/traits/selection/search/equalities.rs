//! Equality premises suspend normalization on the same iterative trait proof stack.
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, Source, inference};
use crate::{
    sema::{projections::implementation_result, traits::equalities},
    types::{
        GenericArgs, TypeId,
        infer::{self, Instantiation, projections::Application},
        parameters::predicates::{Bound, Predicate},
        substitute::{Normalizer, Resolution, Step},
    },
};

pub(super) struct Equality {
    fold: Normalizer,
    step: Step,
    right: Option<TypeId>,
    left: Option<TypeId>,
    goal: Option<Goal>,
    end: u32,
    children: Vec<usize>,
    stopped: Option<Decision>,
}

impl Frame {
    pub(super) fn child(&mut self, facts: &mut Declarations) -> Result<Self, String> {
        if let Some(equality) = &self.equality {
            return Self::new(
                facts,
                &mut self.context,
                equality
                    .goal
                    .as_ref()
                    .ok_or("missing equality trait premise")?,
                equality.end,
            );
        }
        let trial = self.trial.as_mut().ok_or("missing descending candidate")?;
        let index = trial.pending[trial.cursor];
        Self::new(
            facts,
            &mut trial.context,
            &trial.instance.goals[index],
            trial.end,
        )
    }
}

impl Search<'_> {
    pub(super) fn start_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if let Some(decision) = self.cached_query(facts, frame)? {
            return Ok(Action::Complete(decision));
        }
        let left = match frame
            .context
            .resolved(&mut facts.types, frame.goal.subject())
        {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved) => return Ok(Action::Complete(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        };
        let result = frame
            .goal
            .projection_result()
            .ok_or("missing equality result")?;
        let right = match frame.context.resolved(&mut facts.types, result) {
            Ok(ty) => Some(ty),
            Err(infer::Error::Unresolved) => None,
            Err(error) => return Err(inference(error)),
        };
        let mut fold = Normalizer::default();
        let step = fold.start(&mut facts.types, left)?;
        frame
            .context
            .account_work(fold.steps())
            .map_err(inference)?;
        frame.equality = Some(Box::new(Equality {
            fold,
            step,
            right,
            left: None,
            goal: None,
            end: frame.first,
            children: Vec::new(),
            stopped: None,
        }));
        Ok(Action::Continue)
    }

    pub(super) fn advance_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let equality = frame.equality.as_mut().ok_or("missing equality query")?;
        if let Some(decision) = equality.stopped {
            return Ok(Action::Complete(decision));
        }
        let value = match equality.step {
            Step::Complete(value) => value,
            Step::Resolve(request) => {
                if request.depth != 0 {
                    return Err(
                        "quantified equality premises require scoped selection evidence".into(),
                    );
                }
                if request.application.arguments != GenericArgs::EMPTY {
                    return Err("associated families require advanced generic relationships".into());
                }
                let mut substitution =
                    Instantiation::new(&mut frame.context, &facts.types, &[]).map_err(inference)?;
                equality.goal = Some(
                    frame
                        .context
                        .instantiate_predicate(
                            &mut facts.types,
                            Predicate {
                                ty: request.application.subject,
                                bound: Bound::Trait(request.application.trait_application),
                            },
                            &mut substitution,
                        )
                        .map_err(inference)?,
                );
                return Ok(Action::Descend);
            }
        };
        if equality.left.is_none() {
            equality.left = Some(value);
            if let Some(right) = equality.right {
                let before = equality.fold.steps();
                equality.step = equality.fold.start(&mut facts.types, right)?;
                frame
                    .context
                    .account_work(equality.fold.steps().saturating_sub(before))
                    .map_err(inference)?;
                return Ok(Action::Continue);
            }
        }
        self.complete_equality(facts, frame, value)
            .map(Action::Complete)
    }

    fn complete_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        value: TypeId,
    ) -> Result<Decision, String> {
        let equality = frame
            .equality
            .as_mut()
            .ok_or("missing equality completion")?;
        let left = frame
            .context
            .known(
                &facts.types,
                equality.left.ok_or("missing normalized equality subject")?,
            )
            .map_err(inference)?;
        let right = if equality.right.is_some() {
            frame
                .context
                .known(&facts.types, value)
                .map_err(inference)?
        } else {
            frame
                .goal
                .projection_result()
                .ok_or("missing equality result")?
        };
        match frame.context.equate_scoped_terms(&facts.types, left, right) {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(Decision::Absent),
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Decision::Deferred);
            }
            Err(error) => return Err(inference(error)),
        }
        let goal = match frame
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved) => return Ok(Decision::Deferred),
            Err(error) => return Err(inference(error)),
        };
        let node = self.nodes.len();
        self.nodes.push(Node {
            goal,
            source: Some(Source::Equality),
            children: std::mem::take(&mut equality.children),
            first: frame.first,
            end: equality.end,
            equalities: std::mem::take(&mut frame.context.region_equalities),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        });
        Ok(Decision::Proven(node))
    }

    pub(super) fn accept_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let equality = frame.equality.as_mut().ok_or("missing parent equality")?;
        let Decision::Proven(node) = decision else {
            equality.stopped = Some(decision);
            return Ok(());
        };
        let Step::Resolve(request) = equality.step else {
            return Err("missing pending equality normalization".into());
        };
        let goal = equality
            .goal
            .take()
            .ok_or("missing equality trait premise")?;
        frame
            .context
            .equate_trait_predicate(&facts.types, &goal, self.nodes[node].goal)
            .map_err(inference)?;
        let source = self.nodes[node]
            .source
            .ok_or("missing equality premise evidence")?;
        let resolution = match source {
            Source::Implementation { .. } => Resolution::Type(implementation_result(
                facts,
                request.application.item,
                source,
            )?),
            Source::Assumption(_) => {
                let application = Application {
                    item: request.application.item,
                    predicate: goal,
                    arguments: Vec::new(),
                };
                match equalities::select(
                    &facts.types,
                    &mut frame.context,
                    &application,
                    self.caller.environment.map(AsRef::as_ref),
                )
                .map_err(inference)?
                {
                    equalities::Selection::Proven(ty) => Resolution::Type(ty),
                    equalities::Selection::Absent => Resolution::Opaque,
                    equalities::Selection::Ambiguous => {
                        equality.stopped = Some(Decision::Deferred);
                        return Ok(());
                    }
                }
            }
            Source::Equality => return Err("equality evidence cannot prove a trait premise".into()),
        };
        if self.nodes[node].first != self.nodes[node].end {
            equality.end = self.nodes[node].end;
        }
        equality.children.push(node);
        let before = equality.fold.steps();
        equality.step = equality
            .fold
            .resume(&mut facts.types, request, resolution)?;
        frame
            .context
            .account_work(equality.fold.steps().saturating_sub(before))
            .map_err(inference)
    }
}
