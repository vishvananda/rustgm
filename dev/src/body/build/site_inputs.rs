//! The CFG transfers prepared data without constructing spans or constant arenas.
use super::{Builder, Error, NodeId, Operation, Rvalue, Type, ValueId};
use crate::{body::ir::CallerSite, support::bindings::SourceIntrinsic};
impl Builder<'_> {
    pub(super) fn record_caller_site(&mut self, node: NodeId, value: ValueId) -> Result<(), Error> {
        let Some((callable, signature, definition)) = self.typed_source_call(node, value)? else {
            return Ok(());
        };
        let tracked = self
            .facts
            .checked_caller_tracking(self.report, definition)?;
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "caller site lost its catalog"))?;
        let intrinsic = catalog
            .intrinsic_kind(definition)
            .map_err(|error| self.error(node, error))?;
        if intrinsic == Some(SourceIntrinsic::DiscriminantValue) {
            catalog
                .validate_discriminant_opening(
                    self.report,
                    self.facts,
                    self.source,
                    crate::resolve::unit::NodeKey {
                        node,
                        ..self.source
                    },
                    (callable, signature),
                )
                .map_err(|error| self.error(node, error))?;
        }
        if !tracked && intrinsic != Some(SourceIntrinsic::CallerLocation) {
            return Ok(());
        }
        let source_use = match self.fact(node)?.operation {
            Some(Operation::Call {
                signature: expected,
                source_use: Some(source),
            }) if expected == signature => source,
            Some(Operation::Try { plan }) => {
                let plan = plan
                    .current(
                        self.facts,
                        self.source,
                        crate::resolve::unit::NodeKey {
                            node,
                            ..self.source
                        },
                    )
                    .map_err(|error| self.error(node, error))?;
                if !plan
                    .calls()
                    .into_iter()
                    .flatten()
                    .any(|call| call.callable == callable && call.signature == signature)
                {
                    return Err(self.error(
                        node,
                        "caller site differs from the actual generated Try call",
                    ));
                }
                plan.source_use
            }
            _ => {
                return Err(self.error(
                    node,
                    "tracked generated call requires its prepared source-call context",
                ));
            }
        };
        let location = source_use
            .caller_site(self.facts, self.source, node, callable)
            .map_err(|error| self.error(node, error))?
            .ok_or_else(|| {
                self.error(
                    node,
                    "tracked call lacks its prepared actual caller location",
                )
            })?;
        crate::sema::values::Reader::new(self.facts)
            .caller_location_value(location)
            .map_err(|error| self.error(node, error))?;
        self.body
            .append_caller_site(
                CallerSite {
                    owner: self.source,
                    value,
                    callable,
                    location,
                },
                self.facts,
            )
            .map_err(|error| self.error(node, error))
    }
    fn typed_source_call(
        &self,
        node: NodeId,
        value: ValueId,
    ) -> Result<
        Option<(
            crate::types::TypeId,
            crate::types::TypeId,
            crate::resolve::model::DefId,
        )>,
        Error,
    > {
        let Rvalue::Call {
            signature, callee, ..
        } = self.body.values[value.0 as usize].kind
        else {
            return Ok(None);
        };
        let callable = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or_else(|| self.error(node, "caller site lost actual callee"))?
            .ty;
        let Some(Type::FunctionItem { item, .. }) = self.facts.types.kind(callable) else {
            return Ok(None);
        };
        let definition = crate::resolve::model::DefId(item.0);
        if self.facts.source_bindings.is_none() {
            return Ok(None);
        }
        Ok(Some((callable, signature, definition)))
    }
}
