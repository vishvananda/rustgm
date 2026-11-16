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
    loads: u32,
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
    /// The pointer loads from the call operand to the member's receiver.
    #[must_use]
    pub const fn loads(self) -> u32 {
        self.loads
    }
}

/// One candidate member of a call operator: its unselected trait item, the
/// opened signature its call site passes, and the receiver loads before it.
#[derive(Clone, Copy)]
struct Call {
    item: TypeId,
    execution: TypeId,
    loads: u32,
}

/// One call operator whose member the caller's dispatcher must still select.
pub(crate) struct Request {
    value: ValueId,
    tuple: TypeId,
    call: Call,
    /// The referent's member, as rustc's core `&F` and `&mut F` call impls
    /// call it, when the receiver is a reference to a nominal callable.
    forwarded: Option<Call>,
}
impl Request {
    pub(crate) const fn item(&self) -> TypeId {
        self.call.item
    }
    pub(crate) const fn forwards(&self) -> bool {
        self.forwarded.is_some()
    }
    /// Execute the referent's member, once dispatch proves the reference's
    /// call forwards to its referent rather than to an impl of its own.
    pub(crate) fn forward(&mut self) -> Result<TypeId, &'static str> {
        self.call = self
            .forwarded
            .take()
            .ok_or("call operator has no referent member")?;
        Ok(self.call.item)
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
        if selected == self.call.item
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
                execution: self.call.execution,
                tuple: self.tuple,
                loads: self.call.loads,
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
        let call = Call {
            item: crate::sema::compiler_items::callable_operator(facts, kind, subject, tuple)?,
            execution: execution(facts, receiver, tuple, result)?,
            loads: 0,
        };
        let forwarded = match referent(&facts.types, subject, kind)? {
            Some(referent) => Some(Call {
                item: crate::sema::compiler_items::callable_operator(
                    facts,
                    referent.kind,
                    referent.subject,
                    tuple,
                )?,
                execution: execution(facts, referent.receiver, tuple, result)?,
                loads: referent.loads,
            }),
            None => None,
        };
        requests.push(Request {
            value: ValueId(u32::try_from(index).map_err(|_| "too many call operator values")?),
            tuple,
            call,
            forwarded,
        });
    }
    Ok(requests)
}

fn execution(
    facts: &mut Declarations,
    receiver: TypeId,
    tuple: TypeId,
    result: TypeId,
) -> Result<TypeId, &'static str> {
    let inputs = facts.types.intern_list(&[receiver, tuple])?;
    facts.types.intern(Type::Function {
        parameters: inputs,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST_CALL,
    })
}

/// A nominal referent's member reached through references.
struct Referent {
    subject: TypeId,
    kind: CallTrait,
    /// The innermost reference, which the referent's `call` or `call_mut` takes.
    receiver: TypeId,
    loads: u32,
}

/// The referent call rustc's core `&F` and `&mut F` impls make, as trait
/// selection forwards a reference's call trait: `&mut F` calls `F: FnMut`
/// and `&F` calls `F: Fn`, through every reference to the nominal leaf.
/// A borrowed call operator's receiver is one more reference to load.
fn referent(
    types: &Store,
    subject: TypeId,
    mut kind: CallTrait,
) -> Result<Option<Referent>, &'static str> {
    let mut loads = u32::from(kind != CallTrait::FnOnce);
    let mut found = None;
    let mut ty = subject;
    while let Some(Type::Reference {
        referent, mutable, ..
    }) = types.kind(ty)
    {
        // Selection answers that reference absent; its own request reports it.
        let Some(next) = kind.referent(mutable) else {
            return Ok(None);
        };
        if found.is_some() {
            loads = loads
                .checked_add(1)
                .ok_or("call operator receiver exceeds its work budget")?;
        }
        found = Some(ty);
        kind = next;
        ty = types.storage_head(referent)?;
    }
    Ok(found.map(|receiver| Referent {
        subject: ty,
        kind,
        receiver,
        loads,
    }))
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
