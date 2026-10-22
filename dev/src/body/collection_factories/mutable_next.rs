//! The private nonempty kernel restores the disjoint remainder before exposing first.
use super::{
    Action, Body, Declarations, Kind, LocalKind, Original, Place, Rvalue, TypeId, ValueList,
    dereference, local, statement, value,
};
use crate::{
    support::intrinsics::Intrinsic,
    types::{GenericArgs, ItemId, arguments::Argument, instance::Instance, substitute::Bound},
};
pub(super) fn snapshot(
    facts: &mut Declarations,
    original: Original,
    body: &mut Body,
    stored: Place,
    reference: TypeId,
) -> Result<Place, &'static str> {
    original.validate(facts)?;
    let instance = read_instance(facts, original, reference)?;
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: reference,
    })?;
    let address = value(body, pointer, Rvalue::Address(stored))?;
    let start =
        u32::try_from(body.arguments.len()).map_err(|_| "too many mutable iterator operands")?;
    body.arguments.push(address);
    let copied = value(
        body,
        reference,
        Rvalue::Intrinsic {
            intrinsic: Intrinsic::Read,
            instance,
            arguments: ValueList {
                start,
                end: start + 1,
            },
        },
    )?;
    let snapshot = local(body, reference, LocalKind::Binding, false)?;
    statement(
        body,
        Action::Store {
            place: Place::Local(snapshot),
            value: copied,
        },
    );
    dereference(facts, body, Place::Local(snapshot))
}
fn read_instance(
    facts: &mut Declarations,
    original: Original,
    reference: TypeId,
) -> Result<Instance, &'static str> {
    let registry = facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .and_then(|r| r.mutable.as_deref())
        .ok_or("missing mutable iterator contract")?;
    let declaration = registry.canonical.read;
    if declaration.kind != Intrinsic::Read
        || declaration.owner.is_some()
        || facts.intrinsics.get(Intrinsic::Read).is_none_or(|d| {
            d.definition != declaration.definition
                || d.parameter != declaration.parameter
                || d.owner.is_some()
        })
    {
        return Err("mutable next requires actual canonical ptr::read");
    }
    if !matches!(facts.types.kind(reference),Some(Kind::Reference {mutable:true,referent,..}) if facts.types.kind(referent)==Some(Kind::Slice(original.parameter)))
    {
        return Err("mutable next source has wrong reference shape");
    }
    let parameter = declaration.parameter.ok_or("missing read parameter")?;
    let Some(Kind::Parameter(formal)) =
        facts.definitions[parameter.0 as usize].and_then(|t| facts.types.kind(t))
    else {
        return Err("invalid read parameter identity");
    };
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments: GenericArgs::EMPTY,
    }) = facts.definitions[declaration.definition.0 as usize].and_then(|t| facts.types.kind(t))
    else {
        return Err("missing canonical read callable");
    };
    if item != ItemId(declaration.definition.0) {
        return Err("read callable differs from registered definition");
    }
    let function = facts
        .function_indices
        .get(&declaration.definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("missing canonical read function record")?;
    if function.definition != declaration.definition
        || function.signature != signature
        || function.body.is_some()
    {
        return Err("canonical read function differs from its registered contract");
    }
    let formal_type = facts.types.intern(Kind::Parameter(formal))?;
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: true,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: 0,
    }) = facts.types.kind(signature)
    else {
        return Err("invalid canonical read signature");
    };
    let [pointer] = facts
        .types
        .list(parameters)
        .ok_or("missing read signature inputs")?
    else {
        return Err("invalid read signature arity");
    };
    if result != formal_type
        || facts.types.kind(*pointer)
            != Some(Kind::Pointer {
                mutable: false,
                pointee: formal_type,
            })
        || facts.types.parameter(formal).is_none_or(|p| {
            p.owner != item
                || p.index != 0
                || p.kind != (crate::types::parameters::ParameterKind::Type { sized: true })
        })
    {
        return Err("canonical read has mismatched parameter/result contract");
    }
    let mut fold = Bound::parameter_only(&facts.types, formal, Argument::Type(reference))?;
    let signature = fold.apply(&mut facts.types, signature)?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(reference)])?;
    let ty = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    Instance::new(&facts.types, ty)
}
