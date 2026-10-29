//! Sparse call facts are captured only after the actual lexical check succeeds.
use super::{Checker, Error, NodeId};
use crate::support::{crates::generics::source::CallSafety, retained};

#[derive(Clone, Copy)]
pub(in crate::sema::bodies) struct Record {
    pub node: NodeId,
    pub kind: CallSafety,
}
#[derive(Default)]
pub(in crate::sema::bodies) struct Pending {
    pub rows: Vec<Record>,
}
impl Pending {
    pub(in crate::sema::bodies) fn bytes(&self) -> usize {
        size_of::<Self>().saturating_add(retained::vector(&self.rows))
    }
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn require_unsafe_call(
        &mut self,
        node: NodeId,
    ) -> Result<(), Error> {
        self.require_unsafe(node, "call to unsafe function requires an unsafe context")?;
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(());
        };
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(node, e))?;
        let kind = if self.state.safety.blocks != 0 {
            CallSafety::ExplicitUnsafeBlock
        } else if self.state.safety.function {
            CallSafety::UnsafeFunction
        } else {
            return Err(self.error(node, "completed unsafe call lost its lexical context"));
        };
        let (length, capacity, old) = self
            .state
            .call_types
            .safety
            .as_deref()
            .map_or((0, 0, 0), |p| (p.rows.len(), p.rows.capacity(), p.bytes()));
        let mut growth = retained::Growth::vector::<Record>(length, capacity, 1);
        growth.bytes = growth.bytes.saturating_add(
            usize::from(self.state.call_types.safety.is_none()) * size_of::<Pending>(),
        );
        self.state
            .infer
            .account_work(growth.work.saturating_add(1))
            .map_err(|e| self.error(node, e))?;
        super::super::effect_facts::call_safety_scratch(
            self.facts,
            old.saturating_add(growth.bytes),
            length.saturating_add(1),
        )
        .map_err(|e| self.error(node, e))?;
        let pending = self
            .state
            .call_types
            .safety
            .get_or_insert_with(Default::default);
        pending
            .rows
            .try_reserve(1)
            .map_err(|_| Error::unlocated("unsafe call scratch allocation failed"))?;
        if pending.bytes() > old.saturating_add(growth.bytes) {
            return Err(Error::unlocated(
                "unsafe call scratch exceeded prepaid capacity",
            ));
        }
        pending.rows.push(Record { node, kind });
        Ok(())
    }
}
