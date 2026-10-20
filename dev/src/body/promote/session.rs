//! One lazy preparation cache spans demanded bodies in the same declaration result.
use crate::{
    resolve::expansion::Report,
    sema::{model::Declarations, values},
};

#[derive(Default)]
pub(crate) struct Scratch {
    evaluation: Option<Box<values::Scratch>>,
}
impl Scratch {
    pub(crate) fn preparation_steps(&self) -> usize {
        self.evaluation
            .as_ref()
            .map_or(0, |s| s.preparation_steps())
    }
    pub(super) fn evaluator<'a>(
        &mut self,
        report: &'a Report,
        facts: &'a mut Declarations,
    ) -> values::Evaluator<'a> {
        values::Evaluator::with_scratch(report, facts, *self.evaluation.take().unwrap_or_default())
    }
    pub(super) fn retain(&mut self, evaluator: values::Evaluator<'_>) {
        self.evaluation = Some(Box::new(evaluator.into_scratch()));
    }
}
