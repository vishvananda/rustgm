//! Source callback selection data never substitutes for the proved reached call.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::bindings::{LangItem, SourceIntrinsic},
    types::{Kind, TypeId, arguments::Argument, callable::Convention, instance::Instance},
};

#[derive(Clone, Copy)]
pub(crate) struct Select {
    pub definition: DefId,
    pub instance: Instance,
    pub signature: TypeId,
    /// ARG/F/G/RET live in the actual caller argument namespace.
    pub types: [TypeId; 4],
}

/// Unadapted/root demand cannot execute a body-bearing selector placeholder.
pub(crate) fn check_const_select(
    facts: &Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Ok(());
    };
    if facts
        .source_bindings
        .as_deref()
        .map(|c| c.intrinsic_kind(DefId(item.0)))
        .transpose()?
        .flatten()
        == Some(SourceIntrinsic::ConstEvalSelect)
    {
        return Err("source const_eval_select requires its actual reached callback adapter");
    }
    Ok(())
}

/// Prepare only canonical relocation data; source selection/borrow proof pays F.
pub(crate) fn prepare_select(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<Option<Select>, &'static str> {
    let Some(selection) = inputs(facts, instance)? else {
        return Ok(None);
    };
    let types = selection.types;
    let mut work = 1;
    let result = (|| {
        let Some(Kind::Tuple(fields)) = facts.types.kind(types[0]) else {
            return Err("source selector requires its actual argument tuple");
        };
        let width = facts
            .types
            .list(fields)
            .ok_or("missing selector tuple inputs")?
            .len();
        work += width;
        if work > 1_000_000 {
            return Err("source selector preparation work budget exceeded");
        }
        for ty in types {
            work += 1;
            let before = facts.types.capture_shift_steps();
            let shifted = facts.types.shift_type(ty, 1);
            work += facts.types.capture_shift_steps().saturating_sub(before);
            shifted?;
        }
        source_select(facts, instance)
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result
}

pub(crate) fn source_select(
    facts: &Declarations,
    instance: Instance,
) -> Result<Option<Select>, &'static str> {
    let Some(selection) = inputs(facts, instance)? else {
        return Ok(None);
    };
    let Select {
        types, signature, ..
    } = selection;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("selected source selector has a different public signature");
    };
    let [a, f, g] = facts
        .types
        .list(parameters)
        .ok_or("missing selector public inputs")?
    else {
        return Err("source selector public input count changed");
    };
    for (actual, argument) in [*a, *f, *g, result].into_iter().zip(types) {
        if facts.types.prepared_shift(argument, 1)? != Some(actual) {
            return Err(
                "source selector public signature lacks its exact prepared argument relocation",
            );
        }
    }
    if !matches!(facts.types.kind(types[0]), Some(Kind::Tuple(_))) {
        return Err("source selector requires its actual argument tuple");
    }
    Ok(Some(selection))
}

fn inputs(facts: &Declarations, instance: Instance) -> Result<Option<Select>, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let definition = DefId(item.0);
    let Some((entry, kind, original)) = catalog.intrinsic_record(definition)? else {
        return Ok(None);
    };
    if kind != SourceIntrinsic::ConstEvalSelect {
        return Ok(None);
    }
    for role in [LangItem::Tuple, LangItem::FnOnce, LangItem::FnOnceOutput] {
        if catalog.validated_language(role)?.is_none() {
            return Err("source selector requires completed actual callable roles");
        }
    }
    catalog.require_source_destructors_prepared()?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&i| facts.functions.get(i))
        .ok_or("source selector has no actual function declaration")?;
    if function.definition != definition
        || function.node != entry.origin
        || function.signature != original
        || !facts.is_const_function(definition)
    {
        return Err("source selector declaration changed its exact source/signature identity");
    }
    let mut args = facts
        .types
        .arguments(arguments)
        .ok_or("invalid selector instance arguments")?;
    let [
        Some(Argument::Type(a)),
        Some(Argument::Type(f)),
        Some(Argument::Type(g)),
        Some(Argument::Type(r)),
        None,
    ] = [
        args.next(),
        args.next(),
        args.next(),
        args.next(),
        args.next(),
    ]
    else {
        return Err("source selector requires four complete type arguments");
    };
    let types = [a, f, g, r];
    if types.into_iter().any(|ty| {
        facts.types.contains_parameters(ty) != Some(false)
            || facts.types.contains_projections(ty) != Some(false)
    }) {
        return Err("source selector requires closed normalized actual type arguments");
    }
    callback_types(facts, f, g)?;
    Ok(Some(Select {
        definition,
        instance,
        signature,
        types,
    }))
}

fn callback_types(
    facts: &Declarations,
    constant: TypeId,
    runtime: TypeId,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        item: constant,
        signature,
        ..
    }) = facts.types.kind(constant)
    else {
        return Err("source selector constant callback requires an actual function item");
    };
    if !facts.is_const_function(DefId(constant.0))
        || !matches!(
            facts.types.kind(signature),
            Some(Kind::Function {
                unsafe_: false,
                convention: Convention::RUST,
                ..
            })
        )
    {
        return Err("source selector constant callback lacks actual const eligibility");
    }
    let runtime = match facts.types.kind(runtime) {
        Some(Kind::FunctionItem { signature, .. }) => signature,
        Some(Kind::Function { .. }) => runtime,
        _ => return Err("source selector runtime callback requires an item or pointer"),
    };
    if !matches!(
        facts.types.kind(runtime),
        Some(Kind::Function {
            unsafe_: false,
            convention: Convention::RUST,
            ..
        })
    ) {
        return Err("source selector runtime callback requires its safe Rust convention");
    }
    Ok(())
}
