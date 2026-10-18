//! Normalization requests descend on the existing proof stack and retain their evidence.
use super::{Action, Decision, Declarations, Frame, Goal, Search, Source, inference};
use crate::sema::{projections::implementation_result, traits::equalities};
use crate::types::{
    GenericArgs, TypeId,
    infer::{Instantiation, projections::Application},
    parameters::predicates::{Bound, Predicate},
    substitute::{Normalizer, Resolution, Step},
};

pub(super) enum Purpose {
    Equality(super::equalities::Equality),
    Inputs(super::inputs::Inputs),
}
pub(super) struct Normalization {
    pub fold: Normalizer,
    pub step: Step,
    pub purpose: Purpose,
    pub goal: Option<Goal>,
    pub end: u32,
    pub proof: Option<usize>,
    pub stopped: Option<Decision>,
}
impl Normalization {
    pub fn pending(&self) -> bool {
        !matches!(&self.purpose, Purpose::Inputs(inputs) if inputs.ready)
    }
    pub fn new(
        facts: &mut Declarations,
        frame: &mut Frame,
        root: TypeId,
        purpose: Purpose,
    ) -> Result<Self, String> {
        let mut fold = Normalizer::default();
        let step = fold.start(&mut facts.types, root)?;
        frame
            .context
            .account_work(fold.steps())
            .map_err(inference)?;
        Ok(Self {
            fold,
            step,
            purpose,
            goal: None,
            end: frame.first,
            proof: None,
            stopped: None,
        })
    }
}
impl Frame {
    pub(super) fn child(&mut self, facts: &mut Declarations, floor: u32) -> Result<Self, String> {
        if let Some(normalization) = &self.normalization
            && normalization.pending()
        {
            let mut child = Self::new(
                facts,
                &mut self.context,
                normalization
                    .goal
                    .as_ref()
                    .ok_or("missing normalization trait premise")?,
                normalization.end.max(floor),
            )?;
            child.scope = self.scope;
            child.normalized_inputs = Some(super::frames::NormalizedInputs {
                proof: normalization.proof,
            });
            return Ok(child);
        }
        let trial = self.trial.as_mut().ok_or("missing descending candidate")?;
        let index = trial.pending[trial.cursor];
        let mut child = Self::new(
            facts,
            &mut trial.context,
            &trial.instance.goals[index],
            trial.end.max(floor),
        )?;
        child.scope = self.scope;
        Ok(child)
    }
    pub(super) fn available_region(&self) -> u32 {
        self.normalization
            .as_ref()
            .map_or(self.first, |normalization| normalization.end)
    }
    pub(super) fn normalizing(&self) -> bool {
        self.normalization
            .as_ref()
            .is_some_and(|normalization| normalization.pending())
    }
}
impl Search<'_> {
    pub(super) fn advance_normalization(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing normalization query")?;
        if let Some(decision) = normalization.stopped {
            return Ok(Action::Complete(decision));
        }
        let value = match normalization.step {
            Step::Complete(value) => value,
            Step::Resolve(request) => {
                if request.depth != 0 {
                    return Err(
                        "quantified trait premises require scoped selection evidence".into(),
                    );
                }
                if request.application.arguments != GenericArgs::EMPTY {
                    return Err("associated families require advanced generic relationships".into());
                }
                let mut substitution =
                    Instantiation::new(&mut frame.context, &facts.types, &[]).map_err(inference)?;
                normalization.goal = Some(
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
        match normalization.purpose {
            Purpose::Equality(_) => self.advance_equality(facts, frame, value),
            Purpose::Inputs(_) => self.advance_inputs(facts, frame, value),
        }
    }
    pub(super) fn accept_normalization(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let equality = frame
            .normalization
            .as_mut()
            .ok_or("missing parent normalization")?;
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
            .equate_trait_predicate(&mut facts.types, &goal, self.query.nodes[node].goal)
            .map_err(inference)?;
        let source = self.query.nodes[node]
            .source
            .ok_or("missing equality premise evidence")?;
        let resolution = match source {
            Source::Implementation { .. } => Resolution::Type(implementation_result(
                facts,
                request.application.item,
                source,
            )?),
            Source::Assumption(_)
            | Source::ResultBound(_)
            | Source::QuantifiedAssumption { .. } => {
                let application = Application {
                    item: request.application.item,
                    predicate: goal,
                    arguments: Vec::new(),
                };
                match equalities::for_projection(
                    facts,
                    &mut frame.context,
                    &application,
                    self.caller.environment.map(AsRef::as_ref),
                )? {
                    equalities::Selection::Proven(ty) => Resolution::Type(ty),
                    equalities::Selection::Absent => Resolution::Opaque,
                    equalities::Selection::Ambiguous => {
                        equality.stopped = Some(Decision::Deferred);
                        return Ok(());
                    }
                }
            }
            Source::Equality => return Err("equality evidence cannot prove a trait premise".into()),
            Source::Builtin => return Err("builtin evidence cannot prove a trait premise".into()),
        };
        if self.query.nodes[node].first != self.query.nodes[node].end {
            equality.end = self.query.nodes[node].end;
        }
        equality.proof = Some(self.inherit_inputs(node, equality.proof));
        frame.context.account_work(2).map_err(inference)?;
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
