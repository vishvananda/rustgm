//! One explicit equality lookup preserves the same proved parent and opened goal.
use super::{Completion, Declarations, Normalization, Opened, Search, Source, equalities, inference};
use crate::types::{TypeId, infer::projections::Application};
#[derive(Clone, Copy)]
pub(super) enum ResultEquality { Type(TypeId), Absent, Ambiguous }
impl Search<'_> {
    pub(super) fn resolve_result_equality(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::super::Context,
        equality: &Normalization,
        completed: Completion,
    ) -> Result<(ResultEquality, Completion), String> {
        let Completion {
            opened,
            mut node,
            equations_start,
        } = completed;

        let Some(application) = opened.application.associated() else {
            return Ok((ResultEquality::Absent, Completion { opened, node, equations_start }));
        };
        let application = Application {
            item: application.item,
            predicate: opened.goal,
            arguments: context
                .known_arguments(&facts.types, opened.application.arguments())
                .map_err(inference)?,
        };
        let resolution = match equalities::for_projection(
            facts,
            context,
            &application,
            self.caller.environment.map(AsRef::as_ref),
            self.query.nodes[node]
                .end
                .max(equality.end)
                .max(self.query.allocations.floor()),
        )? {
            equalities::Selection::Proven(evidence) => {
                let ty = evidence.ty;
                let mut equations = context.region_equalities.split_off(equations_start);
                node = self
                    .query
                    .equality_use(node, equality.scope, evidence, &mut equations)?;
                context.region_equalities.extend(equations);
                ResultEquality::Type(ty)
            }
            equalities::Selection::Absent => {
                #[cfg(test)]
                super::copy_trace::record(|| {
                    let source = self.query.nodes[node].source.as_ref();
                    format!(
                        "opaque-associated-equality-absent query={:p} scope={:?} node={node} node_scope={:?} node_goal={:?} application={:?} assumption={} result_bound={} quantified={}",
                        &self.query,
                        equality.scope,
                        self.query.node_scope(node),
                        self.query.nodes[node].goal,
                        opened.application.associated(),
                        matches!(source, Some(Source::Assumption(_))),
                        matches!(source, Some(Source::ResultBound(_))),
                        matches!(
                            source,
                            Some(
                                Source::QuantifiedAssumption { .. }
                                    | Source::QuantifiedResultBound { .. }
                            )
                        )
                    )
                });
                ResultEquality::Absent
            }
            equalities::Selection::Ambiguous => ResultEquality::Ambiguous,
        };
        Ok((resolution, Completion {
            opened: Opened { application: opened.application, goal: application.predicate },
            node, equations_start,
        }))
    }
}
