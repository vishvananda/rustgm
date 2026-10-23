//! Encode exact selected items after reserving every original header/formal.
use super::{BodyRef, Checked, Export, GenericRequest, ItemId, Rvalue, TypeId, g, signatures};
use crate::types::{Kind, arguments::Argument};
pub(super) fn prepare(
    export: &mut Export<'_>,
    checked: &Checked<'_>,
    requests: &[GenericRequest],
) -> Result<(), &'static str> {
    for (index, request) in requests.iter().enumerate() {
        let (template, input, _) = checked
            .function_proof(ItemId(request.definition.0))
            .ok_or("call dependency lacks its actual complete original proof")?;
        signatures::validate(
            export,
            request,
            &template.body,
            input,
            &template.environment,
        )?;
        let header = signatures::emit(export, request, BodyRef(Export::ordinal(index)?))?;
        export.reserve(4 * size_of::<g::Function>())?;
        export.rows.functions.push(header);
    }
    // All ordinary body types and complete actual arguments precede recipes.
    for request in requests {
        let body = &checked
            .get(ItemId(request.definition.0))
            .ok_or("call original disappeared")?
            .body;
        for ty in body
            .locals
            .iter()
            .map(|l| l.ty)
            .chain(body.values.iter().map(|v| v.ty))
            .chain(std::iter::once(body.result))
        {
            export.charge(1)?;
            if let Some(Kind::FunctionItem { arguments, .. }) = export.facts.types.kind(ty) {
                for argument in export
                    .facts
                    .types
                    .arguments(arguments)
                    .ok_or("missing complete function arguments")?
                {
                    let Argument::Type(ty) = argument else {
                        return Err("helper application requires type-only arguments");
                    };
                    export.ty(ty)?;
                }
            } else {
                export.ty(ty)?;
            }
        }
    }
    let header_types = Export::ordinal(export.rows.types.len())?;
    export.reserve(size_of::<g::Calls>())?;
    export.rows.calls = Some(Box::new(g::Calls {
        header_types,
        applications: Vec::new(),
        arguments: Vec::new(),
    }));
    for request in requests {
        let body = &checked
            .get(ItemId(request.definition.0))
            .ok_or("call original disappeared")?
            .body;
        for value in &body.values {
            export.charge(1)?;
            if let Rvalue::Function(target) = value.kind {
                if export.applications.contains_key(&value.ty) {
                    continue;
                }
                application(export, target, value.ty)?;
            }
        }
    }
    Ok(())
}
fn application(
    export: &mut Export<'_>,
    target: super::DefId,
    ty: TypeId,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = export.facts.types.kind(ty)
    else {
        return Err("call application lost its full item type");
    };
    if item != ItemId(target.0) {
        return Err("call application changed its exact declaration");
    }
    let arguments = export
        .facts
        .types
        .arguments(arguments)
        .ok_or("missing selected complete arguments")?;
    let count = arguments.len();
    export.charge(count)?;
    export.reserve(4 * count.saturating_mul(size_of::<Argument>() + size_of::<g::TypeRef>()))?;
    let args: Vec<_> = arguments.collect();
    validate_signature(export, target, signature, &args)?;
    let mut roots = Vec::with_capacity(args.len());
    for argument in args {
        let Argument::Type(ty) = argument else {
            return Err("helper application has a non-type actual");
        };
        if export.facts.types.escaping_binders(ty) != Some(0) {
            return Err("helper application has an escaping binder");
        }
        roots.push(export.ty(ty)?);
    }
    let target = export.declaration(target)?;
    export.reserve(
        4 * (size_of::<g::Application>()
            + 2 * size_of::<g::Type>()
            + size_of::<(TypeId, g::ApplicationRef)>()),
    )?;
    let calls = export
        .rows
        .calls
        .as_mut()
        .ok_or("call tables disappeared")?;
    let reference = g::ApplicationRef(Export::ordinal(calls.applications.len())?);
    let first = Export::ordinal(calls.arguments.len())?;
    calls.arguments.extend(roots);
    let end = Export::ordinal(calls.arguments.len())?;
    calls.applications.push(g::Application {
        target,
        arguments: g::Range { start: first, end },
    });
    let root = calls.item_type(reference)?;
    export.rows.types.extend([
        g::Type::FunctionItem(reference),
        g::Type::ApplicationSignature(reference),
    ]);
    export.types.insert(ty, root);
    export.applications.insert(ty, reference);
    Ok(())
}
fn validate_signature(
    export: &mut Export<'_>,
    target: super::DefId,
    selected: TypeId,
    args: &[Argument],
) -> Result<(), &'static str> {
    let header = export
        .facts
        .function_header(target)?
        .ok_or("helper application has no actual header")?;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention,
    }) = export.facts.types.kind(header.signature)
    else {
        return Err("helper requires an actual safe zero-region signature");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters: actual,
        result: actual_result,
        unsafe_: false,
        convention: actual_convention,
    }) = export.facts.types.kind(selected)
    else {
        return Err("selected helper has a different callable contract");
    };
    if convention != actual_convention {
        return Err("helper application changed calling convention");
    }
    let inputs = export
        .facts
        .types
        .list(parameters)
        .ok_or("helper inputs disappeared")?;
    let actual = export
        .facts
        .types
        .list(actual)
        .ok_or("selected helper inputs disappeared")?;
    if inputs.len() != actual.len() {
        return Err("helper application changed physical arity");
    }
    let count = export.facts.generics.parameters_in_scope(target).count();
    export.charge(count.saturating_add(inputs.len()).saturating_add(1))?;
    if args.len() != count {
        return Err("helper application omitted a complete argument");
    }
    let leaf = |ty| match export.facts.types.kind(ty) {
        Some(Kind::Parameter(id)) => {
            let p = export
                .facts
                .types
                .parameter(id)
                .ok_or("helper parameter disappeared")?;
            if p.owner != ItemId(target.0) {
                return Err("helper signature parameter belongs to another declaration");
            }
            match args.get(p.index as usize) {
                Some(Argument::Type(actual)) => Ok(*actual),
                _ => Err("helper application has incomplete type arguments"),
            }
        }
        Some(Kind::Primitive(_)) => Ok(ty),
        _ if ty == TypeId::UNIT => Ok(ty),
        _ => Err("helper physical ports require scalar or parameter roots"),
    };
    if leaf(result)? != actual_result {
        return Err("helper application changed its exact result");
    }
    for (&formal, &actual) in inputs.iter().zip(actual) {
        if leaf(formal)? != actual {
            return Err("helper application changed its exact input");
        }
    }
    Ok(())
}
