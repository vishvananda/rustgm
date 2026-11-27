//! The failing edge of a checked operation calls its prepared source hook.
use super::{Arithmetic, Lower, Operand, Type, Value, ir};
use crate::{
    lower::{
        arithmetic::{abort_guard, guard_edge},
        context::Context,
    },
    lowir::{
        builder::Builder,
        program::{BlockId, SignatureId, Terminator},
        vocabulary::Predicate,
    },
    support::bindings::LangItem,
};

impl Lower<'_, '_, '_> {
    pub(super) fn arithmetic<T>(
        &mut self,
        apply: impl FnOnce(&mut Arithmetic<'_>) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        self.with_guards(|builder, guards| {
            apply(&mut Arithmetic {
                builder,
                branch: &mut |builder, fault, role| guards.branch(builder, fault, role, &[]),
            })
        })
    }

    pub(super) fn bounds_check(&mut self, index: Value, length: Value) -> Result<(), &'static str> {
        self.with_guards(|builder, guards| {
            let fault = builder.compare(Predicate::Uge, index, length)?;
            guards.branch(builder, fault, LangItem::PanicBoundsCheck, &[index, length])
        })
    }

    fn with_guards<T>(
        &mut self,
        apply: impl FnOnce(&mut Builder, &mut Guards<'_, '_>) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        let mut guards = Guards {
            context: self.context,
            panic: &mut self.panic,
            caller: (self.body, self.source, self.native),
        };
        apply(&mut self.builder, &mut guards)
    }
}

struct Guards<'a, 'facts> {
    context: &'a mut Context<'facts>,
    panic: &'a mut Option<BlockId>,
    caller: (&'a ir::Body, ir::BodySite, SignatureId),
}

impl Guards<'_, '_> {
    fn branch(
        &mut self,
        builder: &mut Builder,
        fault: Value,
        cause: LangItem,
        arguments: &[Value],
    ) -> Result<(), &'static str> {
        let Some(route) = self.context.constants.arithmetic_guard(cause) else {
            self.context.constants.require_source_assertion(cause)?;
            return abort_guard(builder, self.panic, fault);
        };
        let (body, site, native) = self.caller;
        let symbol = self.context.demand(route.instance)?;
        let signature = self
            .context
            .caller_signature(route.signature, route.tracked)?;
        let actual = self.context.output.symbols[symbol.0 as usize].signature;
        if !self.context.output.compatible(actual, signature)? {
            return Err("source panic guard differs from its checked target ABI");
        }
        let blocked = builder.block()?;
        let next = guard_edge(builder, fault, blocked)?;
        builder.enter(blocked)?;
        let pointer = if route.tracked {
            Some(self.caller_pointer(builder, body, site, native)?)
        } else {
            None
        };
        let mut call =
            |inputs: &[Value]| builder.call(signature, Type::Void, Operand::Symbol(symbol), inputs);
        match (arguments, pointer) {
            ([], Some(caller)) => call(&[caller])?,
            ([index, length], Some(caller)) => call(&[*index, *length, caller])?,
            (inputs, None) => call(inputs)?,
            _ => return Err("source panic guard has unexpected runtime inputs"),
        };
        builder.terminate(Terminator::Unreachable)?;
        builder.enter(next)
    }

    fn caller_pointer(
        &mut self,
        builder: &mut Builder,
        body: &ir::Body,
        site: ir::BodySite,
        native: SignatureId,
    ) -> Result<Value, &'static str> {
        if let Some(value) = super::caller_location::forwarded_caller(self.context, body, native)? {
            return Ok(value);
        }
        let location = self
            .context
            .constants
            .arithmetic_guard_location(body.source.proof_site(site)?)
            .ok_or("source panic guard lacks its prepared caller location")?;
        let global = self.context.caller_location_global(location)?;
        builder.address(Operand::Global(global))
    }
}
