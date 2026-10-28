//! Only proved adapters and factories may forward or omit a source caller scope.
use super::{Builder, borrow};
use crate::{
    body::ir::{Body, BodyOrigin, Owner},
    sema::model::Declarations,
    support::retained::Growth,
    types::{Kind, Store},
};

pub(super) enum Bridge {
    Factory(crate::body::primitive_methods::Permit),
    Adapter(usize),
}
impl Builder {
    pub(super) fn prepay_forwarding(
        &self,
        facts: &mut Declarations,
        budget: &mut usize,
        source: BodyOrigin,
    ) -> Result<(), borrow::Error> {
        let growth =
            Growth::map::<usize, Bridge>(self.forwarding.len(), self.forwarding.capacity(), 1);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(growth.work.saturating_add(1));
        *budget = budget.saturating_add(growth.bytes);
        if *budget > 64_000_000 || self.forwarding.contains_key(&self.bodies.len()) {
            return Err(borrow::Error {
                node: Some(source.into()),
                message: "caller forwarding exceeds its actual instance budget or repeats a body"
                    .into(),
            });
        }
        Ok(())
    }
    pub(in crate::lower::instances) fn caller_body(
        &self,
        types: &Store,
        index: usize,
    ) -> Result<Option<&Body>, &'static str> {
        let body = self.body(index)?;
        match self.forwarding.get(&index) {
            None => Ok(Some(body)),
            Some(Bridge::Factory(permit)) => {
                if body.instance != Some(permit.instance())
                    || !matches!((body.owner, types.kind(permit.instance().ty())),
                        (Owner::Function { definition, signature, .. }, Some(Kind::FunctionItem { item, signature: actual, .. }))
                        if definition.0 == item.0 && signature == actual)
                {
                    return Err("factory caller changed its proved selected instance");
                }
                // A compiler factory owns no lexical feature premises. Its
                // checked diagnostic anchor cannot confer source permissions.
                Ok(None)
            }
            Some(Bridge::Adapter(source)) => {
                if *source >= index {
                    return Err("adapter caller has no earlier proved source body");
                }
                let original = self.body(*source)?;
                let Owner::ClosureAdapter { adapter, .. } = body.owner else {
                    return Err("adapter caller changed its proved wrapper owner");
                };
                if body.instance != Some(adapter.instance())
                    || body.source != original.source
                    || !matches!((original.owner, types.kind(adapter.instance().ty())),
                        (Owner::Closure { identity, .. }, Some(Kind::Closure { identity: selected, .. }))
                        if identity == selected)
                {
                    return Err("adapter caller changed its actual proved closure source");
                }
                // Publication follows the existing adapter-template validation
                // and full original/adapter proof; source bodies stay immutable.
                Ok(Some(original))
            }
        }
    }
}
