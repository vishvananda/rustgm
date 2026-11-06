//! A real required poll function item identifies data, never an authored original.
use super::protocol::{self, Declared};
use crate::{
    sema::{dispatch, header_attributes, model::Declarations},
    types::{
        Kind, TypeId,
        arguments::Argument,
        future::{EntryKey, Operation},
        instance::Instance,
        substitute::Bound,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FutureMethod {
    instance: Instance,
    key: EntryKey,
}
/// Signature and member data, never a checked body or generated-family permit.
pub(crate) struct Entry {
    pub method: FutureMethod,
    pub public: TypeId,
    pub execution: TypeId,
    pub pin_field: crate::sema::members::FieldId,
    pub ready: crate::sema::members::VariantId,
    pub result: TypeId,
}
impl FutureMethod {
    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }
    pub(crate) const fn key(self) -> EntryKey {
        self.key
    }
    /// Keep application coordinates outside the independently opened public binder.
    pub(crate) fn execution(
        self,
        facts: &mut Declarations,
        inputs: &mut Bound,
        first: u32,
    ) -> Result<Entry, &'static str> {
        let declared = self.validate(facts)?;
        let Some(Kind::FunctionItem {
            signature: public, ..
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("poll entry lost its real required function item");
        };
        let mut physical = {
            let (types, payments) = facts.caller_input_store()?;
            let pay = |work, bytes| payments.charge(work, bytes);
            let Argument::Type(frame) =
                inputs.apply_argument_charged(types, Argument::Type(self.key.future), &pay)?
            else {
                return Err("poll entry changed its actual application argument kind");
            };
            let formal = types
                .intern_arguments_observed::<true>(&[Argument::Type(declared.self_type)], &pay)?;
            let actual = types.intern_arguments_observed::<true>(&[Argument::Type(frame)], &pay)?;
            Bound::captured_arguments_charged(types, formal, actual, &pay)?
        };
        let signature = expected_signature(facts, &declared, &mut physical)?;
        let (execution, result) = open_entry(facts, signature, first)?;
        Ok(Entry {
            method: self,
            public,
            execution,
            pin_field: declared.pin_field,
            ready: declared.ready,
            result,
        })
    }
    pub(crate) fn validate(self, facts: &mut Declarations) -> Result<Declared, &'static str> {
        let declared = protocol::require(facts)?;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("future poll descriptor lost its actual function item");
        };
        if item.0 != declared.method.0 || self.key.operation != Operation::Poll {
            return Err("future descriptor differs from its actual required poll operation");
        }
        facts.checked_future_definition(self.key.future)?;
        let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|_| "future poll descriptor lost its actual source payer")?;
        let mut fold =
            dispatch::callable_substitution_paid(facts, self.instance, &|work, bytes| {
                payments.charge(work, bytes)
            })?;
        let Argument::Type(subject) = fold.apply_argument_charged(
            &mut facts.types,
            Argument::Type(declared.self_type),
            &|work, bytes| payments.charge(work, bytes),
        )?
        else {
            return Err("future poll descriptor changed its actual Self argument kind");
        };
        // The public item can still name its authenticated source opaque Self.
        // The separate entry key is its already selected complete physical future.
        if defining_future(facts, subject)? != self.key.future {
            return Err("future poll descriptor changed its actual full Self application");
        }
        let expected = expected_signature(facts, &declared, &mut fold)?;
        if signature != expected {
            return Err("future poll signature differs from its actual scoped source substitution");
        }
        Ok(declared)
    }
}

/// Remove exactly the public function binder; application lifetimes stay outside it.
fn open_entry(
    facts: &mut Declarations,
    signature: TypeId,
    first: u32,
) -> Result<(TypeId, TypeId), &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        bound_regions: 3,
    }) = facts.types.kind(signature)
    else {
        return Err("poll entry lost its actual three-slot public binder");
    };
    let [receiver, context] = facts
        .types
        .list(parameters)
        .ok_or("poll entry lost its actual parameters")?
    else {
        return Err("poll entry requires its two actual public inputs");
    };
    let parameters = [*receiver, *context];
    let end = first
        .checked_add(3)
        .filter(|&value| u16::try_from(value).is_ok())
        .ok_or("poll entry universal frame exceeds its slot encoding")?;
    facts.charge_caller_inputs(
        3,
        size_of::<Bound>() + 3 * size_of::<crate::types::RegionId>(),
    )?;
    let arguments = (first..end)
        .map(|slot| {
            crate::types::RegionId::bound(
                0,
                u16::try_from(slot).map_err(|_| "invalid poll entry slot")?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut opened = Bound::new(arguments);
    let (types, payments) = facts.caller_input_store()?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let mut mapped = [TypeId::UNIT; 2];
    for (slot, ty) in mapped.iter_mut().zip(parameters) {
        let Argument::Type(value) =
            opened.apply_argument_charged(types, Argument::Type(ty), &pay)?
        else {
            return Err("poll entry input changed its actual type kind");
        };
        *slot = value;
    }
    let Argument::Type(result) =
        opened.apply_argument_charged(types, Argument::Type(result), &pay)?
    else {
        return Err("poll entry result changed its actual type kind");
    };
    let parameters = types.intern_list_observed::<true>(&mapped, &pay)?;
    let execution = types.intern_observed::<true>(
        Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions: end,
        },
        &pay,
    )?;
    Ok((execution, result))
}

pub(in crate::sema) fn retain(
    facts: &mut Declarations,
    instance: Instance,
    future: TypeId,
) -> Result<(), &'static str> {
    let selected = FutureMethod {
        instance,
        key: EntryKey {
            future,
            operation: Operation::Poll,
        },
    };
    selected.validate(facts)?;
    crate::sema::primitive_methods::retain_future(facts, selected)
}

fn expected_signature(
    facts: &mut Declarations,
    declared: &Declared,
    fold: &mut Bound,
) -> Result<TypeId, &'static str> {
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "future poll substitution lost its actual source payer")?;
    let Argument::Type(expected) = fold.apply_argument_charged(
        &mut facts.types,
        Argument::Type(declared.signature),
        &|work, bytes| payments.charge(work, bytes),
    )?
    else {
        return Err("future poll substitution changed its actual callable type kind");
    };
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        bound_regions,
    }) = facts.types.kind(expected)
    else {
        return Err("future poll substitution lost its actual public signature");
    };
    let Some(Kind::Adt(poll, arguments)) = facts.types.kind(result) else {
        return Err("future poll substitution lost its actual Poll result");
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("future poll result arguments are invalid")?;
    let Some(Argument::Type(projected)) = arguments.next() else {
        return Err("future poll result has no actual Output type");
    };
    if arguments.next().is_some() || poll != declared.poll {
        return Err("future poll result has incorrect actual nominal arguments");
    }
    let Some(Kind::Projection(application)) = facts.types.kind(projected) else {
        return Err("future poll template lost its exact Output projection");
    };
    let subject = defining_future(facts, application.subject)?;
    let output = super::output(
        facts,
        crate::types::projection::Application {
            subject,
            ..application
        },
        subject,
    )?;
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "future poll result lost its actual source payer")?;
    let arguments = facts
        .types
        .intern_arguments_observed::<true>(&[Argument::Type(output)], &|work, bytes| {
            payments.charge(work, bytes)
        })?;
    let result = facts
        .types
        .intern_observed::<true>(Kind::Adt(poll, arguments), &|work, bytes| {
            payments.charge(work, bytes)
        })?;
    facts.types.intern_observed::<true>(
        Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions,
        },
        &|work, bytes| payments.charge(work, bytes),
    )
}

fn defining_future(facts: &mut Declarations, ty: TypeId) -> Result<TypeId, &'static str> {
    let future = match facts.types.kind(ty) {
        Some(Kind::Future { .. }) => ty,
        Some(Kind::Opaque(identity, arguments)) => {
            facts
                .opaques
                .declarations
                .get(&identity)
                .ok_or("poll Self opaque lost its actual declaration")?
                .require_current(facts, identity)?;
            let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
                .map_err(|_| "poll Self mapping lost its actual source payer")?;
            facts.types.intern_observed::<true>(
                Kind::Future {
                    identity,
                    arguments,
                },
                &|work, bytes| payments.charge(work, bytes),
            )?
        }
        _ => return Err("poll Self requires its actual authored future representation"),
    };
    facts.checked_future_definition(future)?;
    Ok(future)
}

/// Reveal only an actual checked authored primary future for this Future query.
/// Earlier declaration bounds remain usable while defining publication is pending.
pub(in crate::sema) fn concrete_self(
    facts: &mut Declarations,
    ty: TypeId,
) -> Result<Option<TypeId>, &'static str> {
    let identity = match facts.types.kind(ty) {
        Some(Kind::Future { identity, .. }) => identity,
        Some(Kind::Opaque(identity, _)) => {
            if facts.types.authored_future_template(identity)?.is_none() {
                return Ok(None);
            }
            identity
        }
        _ => return Ok(None),
    };
    let declaration = facts
        .opaques
        .declarations
        .get(&identity)
        .ok_or("future Self lost its actual opaque declaration")?;
    declaration.require_current(facts, identity)?;
    let function = facts
        .function_indices
        .get(&declaration.owner)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("future Self lost its actual defining function")?;
    if facts
        .authored_async_body(function)?
        .is_none_or(|body| body.identity != identity)
    {
        return Err("future Self does not belong to its actual authored primary opaque");
    }
    if declaration.hidden.is_none() {
        return Ok(None);
    }
    defining_future(facts, ty).map(Some)
}
