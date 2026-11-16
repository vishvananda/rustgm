//! A call operator on a nominal receiver executes the source call-trait member
//! that ordinary dispatch selects, passing its operands as one tuple `Args`.
use super::{CallTrait, Type, TypeId, ValueId, borrowed};
use crate::{
    body::ir::{Body, Rvalue},
    sema::model::Declarations,
    types::{Store, callable::Convention, instance::Instance},
};

/// Construction is private to a selected source implementation member.
#[derive(Clone, Copy)]
pub struct Member {
    instance: Instance,
    execution: TypeId,
    tuple: TypeId,
}
impl Member {
    #[must_use]
    pub const fn instance(self) -> Instance {
        self.instance
    }
    /// The opened `fn(receiver, Args) -> Output` the call site passes.
    #[must_use]
    pub const fn execution(self) -> TypeId {
        self.execution
    }
    #[must_use]
    pub const fn tuple(self) -> TypeId {
        self.tuple
    }
}

/// One call operator whose member the caller's dispatcher must still select.
pub(crate) struct Request {
    value: ValueId,
    item: TypeId,
    execution: TypeId,
    tuple: TypeId,
}
impl Request {
    pub(crate) const fn item(&self) -> TypeId {
        self.item
    }
    /// The selection must be an implementation's own method, never the
    /// trait declaration or a compiler-provided callable.
    pub(crate) fn finish(
        self,
        facts: &Declarations,
        selected: TypeId,
    ) -> Result<(ValueId, Member), &'static str> {
        let Some(Type::FunctionItem { item, .. }) = facts.types.kind(selected) else {
            return Err("call operator member selected no function item");
        };
        if selected == self.item
            || facts
                .associated
                .trait_owner(crate::resolve::model::DefId(item.0))
                .is_some()
        {
            return Err("call operator member selected no source implementation body");
        }
        Ok((
            self.value,
            Member {
                instance: Instance::new(&facts.types, selected)?,
                execution: self.execution,
                tuple: self.tuple,
            },
        ))
    }
}

/// The member's `Self` when a call needs one: closures, function values and
/// objects keep their own executable paths, and a symbolic Self stays fenced.
fn subject(
    types: &Store,
    receiver: TypeId,
    kind: CallTrait,
) -> Result<Option<TypeId>, &'static str> {
    let subject = borrowed::subject(types, receiver, kind)?;
    let mut leaf = subject;
    let mut visits = 0_usize;
    while let Some(Type::Reference { referent, .. }) = types.kind(leaf) {
        visits += 1;
        if visits > 1_000_000 {
            return Err("call operator receiver exceeds its work budget");
        }
        leaf = types.storage_head(referent)?;
    }
    Ok((matches!(types.kind(leaf), Some(Type::Adt(..)))
        && types.contains_parameters(subject) == Some(false))
    .then_some(subject))
}

/// Whether any call operator of this concrete body needs its member selected.
pub(crate) fn needed(facts: &Declarations, body: &Body) -> Result<bool, &'static str> {
    for value in &body.values {
        if let Rvalue::Invoke {
            kind, arguments, ..
        } = value.kind
            && subject(&facts.types, receiver(body, arguments)?, kind)?.is_some()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The trait members to select, in value order, each with the exact tuple and
/// opened signature its call site passes to the selected body.
pub(crate) fn requests(
    facts: &mut Declarations,
    body: &Body,
) -> Result<Vec<Request>, &'static str> {
    let mut requests = Vec::new();
    for (index, value) in body.values.iter().enumerate() {
        let Rvalue::Invoke {
            kind,
            signature,
            arguments,
        } = value.kind
        else {
            continue;
        };
        let receiver = receiver(body, arguments)?;
        let Some(subject) = subject(&facts.types, receiver, kind)? else {
            continue;
        };
        let Some(Type::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("call operator member requires its opened safe signature");
        };
        if result != value.ty {
            return Err("call operator signature differs from its typed result");
        }
        let tuple = facts.types.intern(Type::Tuple(parameters))?;
        facts.aggregates.prepare(&mut facts.types, tuple)?;
        let inputs = facts.types.intern_list(&[receiver, tuple])?;
        let execution = facts.types.intern(Type::Function {
            parameters: inputs,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: Convention::RUST_CALL,
        })?;
        requests.push(Request {
            value: ValueId(u32::try_from(index).map_err(|_| "too many call operator values")?),
            item: crate::sema::compiler_items::callable_operator(facts, kind, subject, tuple)?,
            execution,
            tuple,
        });
    }
    Ok(requests)
}

fn receiver(body: &Body, arguments: crate::body::ir::ValueList) -> Result<TypeId, &'static str> {
    let receiver = *body
        .arguments(arguments)
        .first()
        .ok_or("call operator has no receiver")?;
    Ok(body
        .values
        .get(receiver.0 as usize)
        .ok_or("invalid call operator receiver")?
        .ty)
}
