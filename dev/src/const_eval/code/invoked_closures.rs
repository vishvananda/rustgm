//! Cold closure records bind one immutable proved Code and its actual operations.
use super::{Code, PreparedOperand};
use crate::{
    body::{
        invocation_entries::Entry,
        ir::{Action, Body, BodyOrigin, BodySite, Owner, Rvalue, Value, ValueId},
    },
    support::{
        call_traits::CallTrait,
        retained::{Growth, Payment},
    },
    types::{Kind, Store, TypeId, arena::Identity, instance::Instance, substitute::Bound},
};
use std::{
    cell::OnceCell,
    ops::Range,
    rc::{Rc, Weak},
};
#[cfg(test)]
#[path = "../../../tests/invariants/sealed_environments.rs"]
mod tests;

pub(crate) struct Context {
    arena: Identity,
    owner: Owner,
    source: BodyOrigin,
    instance: Option<Instance>,
    calls: Range<u32>,
    code: OnceCell<Weak<Code>>,
}
pub(crate) struct Call {
    context: Rc<Context>,
    statement: u32,
    source: BodySite,
    receiver: ValueId,
    instance: Instance,
    signature: TypeId,
    receiver_ty: TypeId,
    result: TypeId,
    entry: Option<Entry>,
}
pub(crate) struct Plan {
    calls: usize,
    owned: bool,
}
pub(crate) struct Preparation<'a> {
    pub plan: Plan,
    pub pay: Option<Payment<'a>>,
}
pub(super) struct Cost<'a, 'p> {
    pub budget: usize,
    pub work: &'a mut usize,
    pub pay: Option<Payment<'p>>,
}
const _: [(); 64] = [(); size_of::<Context>()];
const _: [(); 64] = [(); size_of::<Call>()];
impl Plan {
    pub(crate) fn collect(
        types: &Store,
        body: &Body,
        work: &mut usize,
    ) -> Result<Self, &'static str> {
        if !body.const_context {
            return Ok(Self {
                calls: 0,
                owned: false,
            });
        }
        let owned = matches!(body.owner, Owner::Closure { .. });
        let mut calls = 0usize;
        for statement in &body.statements {
            *work = work.saturating_add(1);
            if let Action::Evaluate(id) = statement.action
                && subject(types, &body.values, &body.arguments, id)?.is_some()
            {
                calls = calls
                    .checked_add(1)
                    .ok_or("constant closure call count overflow")?;
            }
        }
        Ok(Self { calls, owned })
    }
    pub(crate) const fn needed(&self) -> bool {
        self.owned || self.calls != 0
    }
    pub(super) fn install(
        self,
        code: &mut Code,
        types: &mut Store,
        body: &Body,
        fold: Option<&mut Bound>,
        cost: Cost<'_, '_>,
    ) -> Result<(), &'static str> {
        if !self.needed() {
            return Ok(());
        }
        let growth = Growth::vector::<PreparedOperand>(
            code.operands.len(),
            code.operands.capacity(),
            self.calls.saturating_add(1),
        );
        let bytes = growth
            .bytes
            .saturating_add(rc_bytes::<Context>())
            .saturating_add(self.calls.saturating_mul(rc_bytes::<Call>()))
            .saturating_add(if types.existing_arena_identity().is_none() {
                Identity::allocation_bytes()
            } else {
                0
            });
        let Cost { budget, work, pay } = cost;
        *work = work
            .saturating_add(growth.work)
            .saturating_add(self.calls + 1);
        if let Some(pay) = pay {
            pay(
                growth
                    .work
                    .saturating_add(self.calls.saturating_mul(3))
                    .saturating_add(1)
                    .saturating_add(body.statements.len().saturating_mul(2)),
                bytes,
            )?;
        }
        if bytes > budget {
            return Err("constant closure records exceed Code storage budget");
        }
        u32::try_from(
            code.operands
                .len()
                .saturating_add(self.calls)
                .saturating_add(1),
        )
        .ok()
        .filter(|count| *count < u32::MAX)
        .ok_or("constant closure operand range overflow")?;
        code.operands
            .try_reserve_exact(self.calls + 1)
            .map_err(|_| "constant closure operand allocation failed")?;
        let caller = matches!(code.operands.last(), Some(PreparedOperand::CallerSites(_)))
            .then(|| code.operands.pop())
            .flatten();
        let reborrows = matches!(code.operands.last(), Some(PreparedOperand::Reborrows(_)))
            .then(|| code.operands.pop())
            .flatten();
        let start = u32::try_from(code.operands.len())
            .map_err(|_| "constant closure range start overflow")?;
        let end = start
            .checked_add(
                u32::try_from(self.calls).map_err(|_| "constant closure range length overflow")?,
            )
            .ok_or("constant closure range end overflow")?;
        let context = Rc::new(Context {
            arena: types.arena_identity(),
            owner: code.owner,
            source: code.source,
            instance: code.instance,
            calls: start..end,
            code: OnceCell::new(),
        });
        append(code, types, body, &context, fold, work)?;
        if code.operands.len() != end as usize {
            return Err("constant closure call coverage changed during snapshot");
        }
        code.operands.push(PreparedOperand::ClosureContext(context));
        code.operands.extend(reborrows);
        code.operands.extend(caller);
        Ok(())
    }
}
fn append(
    code: &mut Code,
    types: &mut Store,
    body: &Body,
    context: &Rc<Context>,
    mut fold: Option<&mut Bound>,
    work: &mut usize,
) -> Result<(), &'static str> {
    for (position, statement) in body.statements.iter().enumerate() {
        *work = work.saturating_add(1);
        let Action::Evaluate(id) = statement.action else {
            continue;
        };
        let Some((ty, receiver)) = subject(types, &code.values, &code.arguments, id)? else {
            continue;
        };
        let value = &code.values[id.0 as usize];
        let Rvalue::Invoke { signature, .. } = value.kind else {
            return Err("constant closure operation changed during snapshot");
        };
        let instance = Instance::closure(types, ty)?;
        let mut entry = body.closure_calls.as_deref().and_then(|rows| rows.get(id));
        if let Some(fold) = fold.as_deref_mut() {
            entry = entry
                .map(|entry| super::proven_storage::closure_entry(types, fold, entry, work))
                .transpose()?;
        }
        if entry.is_some_and(|entry| entry.instance() != instance) {
            return Err("constant closure record changed its selected complete instance");
        }
        let call = Rc::new(Call {
            context: Rc::clone(context),
            statement: u32::try_from(position)
                .map_err(|_| "constant closure statement overflow")?,
            source: statement.source,
            receiver,
            instance,
            signature,
            result: value.ty,
            receiver_ty: code.values[receiver.0 as usize].ty,
            entry,
        });
        call.validate_operation(code, id)?;
        code.operands.push(PreparedOperand::ClosureCall {
            destination: id,
            call,
        });
    }
    Ok(())
}
pub(crate) fn subject(
    types: &Store,
    values: &[Value],
    operands: &[ValueId],
    operation: ValueId,
) -> Result<Option<(TypeId, ValueId)>, &'static str> {
    let value = values
        .get(operation.0 as usize)
        .ok_or("constant closure value is absent")?;
    let Rvalue::Invoke {
        kind, arguments, ..
    } = value.kind
    else {
        return Ok(None);
    };
    let receiver = *operands
        .get(arguments.start as usize..arguments.end as usize)
        .and_then(|args| args.first())
        .ok_or("constant closure has no receiver operand")?;
    let ty = values
        .get(receiver.0 as usize)
        .ok_or("constant closure receiver is absent")?
        .ty;
    let ty = if kind == CallTrait::FnOnce {
        ty
    } else {
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = types.kind(ty)
        else {
            return Ok(None);
        };
        if matches!(types.kind(referent), Some(Kind::Closure { .. }))
            && mutable != (kind == CallTrait::FnMut)
        {
            return Err("constant closure receiver has the wrong borrow capability");
        }
        referent
    };
    Ok(matches!(types.kind(ty), Some(Kind::Closure { .. })).then_some((ty, receiver)))
}
impl Context {
    fn header(&self, code: &Code) -> Result<(), &'static str> {
        if self.owner != code.owner
            || self.source != code.source
            || self.instance != code.instance
            || !code.const_context
        {
            return Err("constant closure context changed its proved owner/source/instance");
        }
        Ok(())
    }
    fn require(&self, types: &Store, code: &Code) -> Result<(), &'static str> {
        self.header(code)?;
        if !types.has_identity(&self.arena)
            || self
                .code
                .get()
                .and_then(Weak::upgrade)
                .is_none_or(|owner| !std::ptr::eq(owner.as_ref(), code))
        {
            return Err("constant closure context belongs to another Code or type store");
        }
        Ok(())
    }
    fn rows<'a>(&self, code: &'a Code) -> Result<&'a [PreparedOperand], &'static str> {
        if !matches!(code.operands.get(self.calls.end as usize),
            Some(PreparedOperand::ClosureContext(context)) if std::ptr::eq(context.as_ref(), self))
        {
            return Err("constant closure range lost its exact context footer");
        }
        code.operands
            .get(self.calls.start as usize..self.calls.end as usize)
            .ok_or("constant closure call range is outside its Code")
    }
}
impl Call {
    pub(crate) const fn instance(&self) -> Instance {
        self.instance
    }
    pub(crate) const fn entry(&self) -> Option<Entry> {
        self.entry
    }
    pub(crate) const fn receiver(&self) -> ValueId {
        self.receiver
    }
    fn validate_operation(&self, code: &Code, operation: ValueId) -> Result<(), &'static str> {
        let value = code
            .values
            .get(operation.0 as usize)
            .ok_or("constant closure operation disappeared")?;
        let Rvalue::Invoke {
            signature,
            arguments,
            kind,
        } = value.kind
        else {
            return Err("constant closure record no longer selects Invoke");
        };
        if !matches!(code.statements.get(self.statement as usize),
            Some(statement) if matches!(statement.action, Action::Evaluate(id) if id == operation)
                && statement.source == self.source)
            || signature != self.signature
            || value.ty != self.result
            || code
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .and_then(|args| args.first())
                .copied()
                != Some(self.receiver)
            || code.values.get(self.receiver.0 as usize).map(|v| v.ty) != Some(self.receiver_ty)
            || self
                .entry
                .is_some_and(|e| e.instance() != self.instance || e.requested() != kind)
        {
            return Err("constant closure changed its statement/destination/receiver/signature");
        }
        Ok(())
    }
}
impl Code {
    pub(in crate::const_eval) fn validate_closure_resume(
        &self,
        types: &Store,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        if let Some(context) = self.closure_context() {
            *work = work.saturating_add(8);
            return context.require(types, self);
        }
        if matches!(self.owner, Owner::Closure { .. }) {
            return Err("const closure frame lost its proved Code context");
        }
        Ok(())
    }
    pub(crate) fn validate_closure_family(
        &self,
        facts: &crate::sema::model::Declarations,
        instance: Instance,
    ) -> Result<(), &'static str> {
        self.require_closure_context(&facts.types)?;
        let source = crate::body::constants::const_closure_source(facts, instance)?;
        let Some(Kind::Closure {
            identity,
            signature: public,
            environment,
            ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err("const closure Code lost its complete application");
        };
        let Owner::Closure {
            identity: actual,
            signature,
            execution,
        } = self.owner
        else {
            return Err("const closure Code has another body owner");
        };
        let Some(Kind::Function {
            result,
            unsafe_: false,
            convention,
            ..
        }) = facts.types.kind(execution.unwrap_or(signature))
        else {
            return Err("const closure Code lost its proved execution signature");
        };
        if self.instance != Some(instance)
            || actual != identity
            || signature != public
            || self.source.source() != Some(source.semantic_source())
            || self.source_only
            || result != self.result
            || convention != crate::types::callable::Convention::RUST
            || execution.is_some() != (environment != TypeId::UNIT)
        {
            return Err("const closure Code changed its actual source/instance/signature family");
        }
        Ok(())
    }
    fn closure_context(&self) -> Option<&Rc<Context>> {
        let mut end = self.operands.len();
        for expected in [0, 1, 2] {
            let last = end.checked_sub(1).and_then(|i| self.operands.get(i));
            let skip = matches!(
                (expected, last),
                (0, Some(PreparedOperand::CallerSites(_)))
                    | (1, Some(PreparedOperand::Cleanup(_)))
                    | (2, Some(PreparedOperand::Reborrows(_)))
            );
            if skip {
                end -= 1;
            }
        }
        match end.checked_sub(1).and_then(|i| self.operands.get(i))? {
            PreparedOperand::ClosureContext(context) => Some(context),
            _ => None,
        }
    }
    pub(super) fn bind_closure_context(code: &Rc<Self>) -> Result<(), &'static str> {
        let Some(context) = code.closure_context() else {
            return Ok(());
        };
        context.header(code)?;
        let mut previous = None;
        for row in context.rows(code)? {
            let PreparedOperand::ClosureCall { destination, call } = row else {
                return Err("constant closure range contains an unrelated operand");
            };
            if !Rc::ptr_eq(context, &call.context)
                || previous.is_some_and(|before| before >= call.statement)
            {
                return Err("constant closure records have a foreign or duplicate statement");
            }
            call.validate_operation(code, *destination)?;
            previous = Some(call.statement);
        }
        if let Some(current) = context.code.get() {
            if !current.ptr_eq(&Rc::downgrade(code)) {
                return Err("constant closure context was bound to another Code");
            }
        } else {
            context
                .code
                .set(Rc::downgrade(code))
                .map_err(|_| "constant closure context was bound twice")?;
        }
        Ok(())
    }
    pub(in crate::const_eval) fn require_closure_context(
        &self,
        types: &Store,
    ) -> Result<(), &'static str> {
        self.closure_context()
            .ok_or("constant closure requires its proved Code context")?
            .require(types, self)
    }
    pub(in crate::const_eval) fn closure_call(
        &self,
        types: &Store,
        statement: u32,
        destination: ValueId,
        work: &mut usize,
    ) -> Result<Option<&Call>, &'static str> {
        let Some(context) = self.closure_context() else {
            return Ok(None);
        };
        context.require(types, self)?;
        let rows = context.rows(self)?;
        let index = rows.binary_search_by_key(&statement, |row| match row {
            PreparedOperand::ClosureCall { call, .. } => {
                *work = work.saturating_add(1);
                call.statement
            }
            _ => u32::MAX,
        });
        let Ok(index) = index else { return Ok(None) };
        let PreparedOperand::ClosureCall {
            destination: actual,
            call,
        } = &rows[index]
        else {
            return Err("constant closure range changed its typed row");
        };
        if *actual != destination || !Rc::ptr_eq(context, &call.context) {
            return Err("constant closure lookup changed its actual destination or owner");
        }
        call.validate_operation(self, destination)?;
        let Some((ty, _)) = subject(types, &self.values, &self.arguments, destination)? else {
            return Err("constant closure receiver lost its actual complete Self");
        };
        if ty != call.instance.ty() {
            return Err("constant closure lookup changed its full selected instance");
        }
        Ok(Some(call))
    }
    pub(super) fn closure_record_bytes(&self) -> usize {
        self.closure_context().map_or(0, |context| {
            rc_bytes::<Context>().saturating_add(
                context.calls.end.saturating_sub(context.calls.start) as usize * rc_bytes::<Call>(),
            )
        })
    }
}
fn rc_bytes<T>() -> usize {
    size_of::<T>().saturating_add(2 * size_of::<usize>())
}
