//! Paid calls and deferred Index contexts stay distinct until actual use is known.
use super::{Checker, Error, NodeId};
use crate::support::{crates::generics::source::CallSafety, retained};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema::bodies) enum PendingKind {
    Discharged(CallSafety),
    IndexContext(CallSafety),
}
#[derive(Clone, Copy)]
pub(in crate::sema::bodies) struct Record {
    pub node: NodeId,
    pub kind: PendingKind,
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
        let kind = self.lexical_call_context();
        if kind == CallSafety::Safe {
            return Err(self.error(node, "completed unsafe call lost its lexical context"));
        }
        self.retain_safety_record(node, PendingKind::Discharged(kind))
    }
    /// Version thirteen: a source provider retains each inline assembly's
    /// lexical unsafe context for export. A replay finds the same context.
    pub(in crate::sema::bodies) fn retain_assembly_context(
        &mut self,
        node: NodeId,
    ) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(());
        }
        let kind = self.lexical_call_context();
        if kind == CallSafety::Safe {
            return Err(self.error(node, "checked inline assembly lost its lexical context"));
        }
        self.state
            .infer
            .account_work(1)
            .map_err(|e| self.error(node, e))?;
        let key = self.key(node);
        let contexts = self.facts.assembly_contexts.get_or_insert_default();
        if *contexts.entry(key).or_insert(kind) != kind {
            return Err(self.error(
                node,
                "inline assembly replay changed its lexical unsafe context",
            ));
        }
        Ok(())
    }
    fn lexical_call_context(&self) -> CallSafety {
        if self.state.safety.blocks != 0 {
            CallSafety::ExplicitUnsafeBlock
        } else if self.state.safety.function {
            CallSafety::UnsafeFunction
        } else {
            CallSafety::Safe
        }
    }
    pub(in crate::sema::bodies) fn retain_index_unsafe_context(
        &mut self,
        node: NodeId,
    ) -> Result<(), Error> {
        if !matches!(self.kind(node), crate::syntax::ast::Kind::Index { .. }) {
            return Err(self.error(node, "deferred index safety changed its actual syntax"));
        }
        self.retain_safety_record(node, PendingKind::IndexContext(self.lexical_call_context()))
    }
    pub(in crate::sema::bodies) fn discharge_index_unsafe_context(
        &mut self,
        node: NodeId,
        kind: CallSafety,
    ) -> Result<(), Error> {
        let context = super::Context {
            blocks: u32::from(kind == CallSafety::ExplicitUnsafeBlock),
            function: kind == CallSafety::UnsafeFunction,
        };
        self.require_unsafe_context(
            node,
            "call to unsafe function requires an unsafe context",
            context,
        )
    }
    fn retain_safety_record(&mut self, node: NodeId, kind: PendingKind) -> Result<(), Error> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(());
        };
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(node, e))?;
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
