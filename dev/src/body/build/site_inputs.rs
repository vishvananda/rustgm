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
        self.validate_source_opening(node, callable, signature, intrinsic)?;
        let required = tracked || intrinsic == Some(SourceIntrinsic::CallerLocation);
        if !required && self.facts.associated.trait_owner(definition).is_none() {
            return Ok(());
        }
        let source_use = match self.fact(node)?.operation {
            Some(Operation::Call {
                signature: expected,
                source_use: Some(source),
            }) if expected == signature => source,
            Some(
                Operation::Field {
                    plan: Some(plan), ..
                }
                | Operation::MethodReceiver { plan }
                | Operation::Try { plan }
                | Operation::Iteration { plan },
            ) => self.protocol_site_source(node, plan, callable, signature)?,
            _ if !required => return Ok(()),
            _ => {
                return Err(self.error(
                    node,
                    "tracked generated call requires its prepared source-call context",
                ));
            }
        };
        let location = source_use
            .caller_site(self.facts, self.source, node, callable)
            .map_err(|error| self.error(node, error))?;
        // A symbolic untracked trait member may select a tracked implementation.
        // Its authenticated semantic producer prepared the same original callable;
        // concrete dispatch rewrites that identity with the actual operand later.
        let Some(location) = location else {
            if required {
                return Err(self.error(
                    node,
                    "tracked call lacks its prepared actual caller location",
                ));
            }
            return Ok(());
        };
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
    fn protocol_site_source(
        &self,
        node: NodeId,
        id: crate::sema::bodies::ProtocolPlanId,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
    ) -> Result<crate::sema::bodies::SourceCallUse, Error> {
        let key = crate::resolve::unit::NodeKey {
            node,
            ..self.source
        };
        let plan = id
            .current(self.facts, self.source, key)
            .map_err(|error| self.error(node, error))?;
        let final_call = matches!(plan.kind, crate::sema::protocol_plans::Kind::Receiver { call, .. } if call.callable == callable && call.signature == signature);
        if plan.kind.access().is_some() && !final_call {
            crate::sema::protocol_plans::access::lookup(
                self.report,
                self.facts,
                plan,
                self.source,
                crate::sema::iteration::Call {
                    callable,
                    signature,
                },
            )?;
        } else {
            let mut calls = plan.calls().map_err(|error| self.error(node, error))?;
            let mut present = false;
            while let Some(call) = plan
                .next_call(self.facts, self.source, &mut calls)
                .map_err(|error| self.error(node, error))?
            {
                present |= call.callable == callable && call.signature == signature;
            }
            if !present {
                return Err(self.error(
                    node,
                    "caller site differs from its actual generated protocol call",
                ));
            }
        }
        Ok(plan.source_use)
    }
    fn validate_source_opening(
        &self,
        node: NodeId,
        callable: crate::types::TypeId,
        signature: crate::types::TypeId,
        intrinsic: Option<SourceIntrinsic>,
    ) -> Result<(), Error> {
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "source call lost its catalog"))?;
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
        if matches!(intrinsic,Some(SourceIntrinsic::Declaration(identity)) if identity.name()=="va_arg")
        {
            catalog
                .validate_vararg_opening(
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
        Ok(())
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
