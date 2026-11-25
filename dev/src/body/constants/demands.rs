//! Each demanded constant is evaluated once, in one fixed order.
use super::{Evaluator, Rvalue, references};
use crate::{
    body::ir::{Body, BodyOrigin},
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{
        inline_constants::InlineInstance,
        values::{Instance, execution::SourceComputation},
    },
    types::TypeId,
};

/// One constant that some typed body requires before lowering.
#[derive(Clone, Copy)]
pub(super) enum Demand {
    Definition(DefId),
    Static(DefId),
    Inline(BodyOrigin),
    InlineInstance(InlineInstance, TypeId),
    Source(SourceComputation),
    Associated(Instance),
}
impl Demand {
    fn evaluate(self, evaluator: &mut Evaluator<'_>) -> Result<(), Error> {
        match self {
            Self::Definition(id) => evaluator.definition(id),
            Self::Static(id) => evaluator.initializer(id),
            Self::Inline(source) => evaluator.inline(source),
            Self::InlineInstance(instance, result) => evaluator.inline_instance(instance, result),
            Self::Source(source) => evaluator.source_associated(source),
            Self::Associated(instance) => evaluator.associated(instance),
        }
        .map(drop)
    }
    /// Whether this body requires the constant.
    fn demanded_by(self, body: &Body) -> bool {
        let value = |wanted: &dyn Fn(Rvalue, TypeId) -> bool| {
            body.values.iter().any(|value| wanted(value.kind, value.ty))
        };
        match self {
            Self::Definition(id) => references(body).any(|found| found == (id, false)),
            Self::Static(id) => references(body).any(|found| found == (id, true)),
            Self::Inline(source) => {
                value(&|kind, _| matches!(kind, Rvalue::InlineConstant(found) if found == source))
            }
            Self::InlineInstance(instance, result) => value(&|kind, ty| {
                ty == result && matches!(kind, Rvalue::InlineInstance(found) if found == instance)
            }),
            Self::Source(source) => {
                value(&|kind, _| matches!(kind, Rvalue::SourceAssociated(found) if found == source))
            }
            Self::Associated(instance) => value(
                &|kind, _| matches!(kind, Rvalue::AssociatedConstant(found) if found == instance),
            ),
        }
    }
}

/// Evaluate every demand in order. The bring-up profile sets a failed constant
/// aside at each body that requires it, wherever its evaluation failed, and
/// evaluates the rest; the stage then still fails.
pub(super) fn evaluate<'a>(
    report: &Report,
    evaluator: &mut Evaluator<'_>,
    demands: &[Demand],
    bodies: impl Iterator<Item = &'a Body>,
) -> Result<(), Error> {
    let mut deferred = None;
    // The bodies are only walked once a constant has failed.
    let mut unread = Some(bodies);
    let mut bodies = Vec::new();
    for &demand in demands {
        let Err(error) = demand.evaluate(evaluator) else {
            continue;
        };
        if crate::support::bringup::selected().is_none() {
            return Err(error);
        }
        bodies.extend(unread.take().into_iter().flatten());
        let mut users = bodies
            .iter()
            .filter(|body| demand.demanded_by(body))
            .map(|body| body.source.root_site())
            .filter(|&site| {
                crate::body::bringup::defer_site(
                    crate::support::bringup::Stage::Constants,
                    Some(site),
                    &error.message,
                )
            });
        let Some(first) = users.next() else {
            return Err(error);
        };
        users.for_each(drop);
        deferred = Some(report.body_error(first, error.message.clone()));
    }
    deferred.map_or(Ok(()), Err)
}
