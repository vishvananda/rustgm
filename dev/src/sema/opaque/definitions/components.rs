//! One paid typed DAG walk finds exact applications without exposing hidden types.
use crate::{
    resolve::{expansion::Report, model::DefId},
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
    types::{Kind, Store, TypeId, arguments::Argument, opaque::OpaqueId},
};

fn walk(
    store: &Store,
    roots: &[TypeId],
    mut visit: impl FnMut(TypeId, Kind) -> Result<(), &'static str>,
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let mut work = Vec::new();
    let mut seen = HashSet::default();
    for &root in roots {
        push(&mut work, root, pay)?;
    }
    while let Some(ty) = work.pop() {
        pay(1, 0)?;
        if seen.contains(&ty) {
            continue;
        }
        let growth = Growth::map::<TypeId, ()>(seen.len(), seen.capacity(), 1);
        pay(growth.work, growth.bytes)?;
        seen.insert(ty);
        let kind = store
            .kind(ty)
            .ok_or("named opaque traversal reached an unknown type")?;
        visit(ty, kind)?;
        children(store, kind, &mut work, pay)?;
    }
    Ok(())
}
fn push(
    work: &mut Vec<TypeId>,
    ty: TypeId,
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let growth = Growth::vector::<TypeId>(work.len(), work.capacity(), 1);
    pay(growth.work, growth.bytes)?;
    work.push(ty);
    Ok(())
}
fn children(
    store: &Store,
    kind: Kind,
    work: &mut Vec<TypeId>,
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Reference { referent: ty, .. }
        | Kind::Pointer { pointee: ty, .. }
        | Kind::Slice(ty)
        | Kind::Array { element: ty, .. }
        | Kind::ArrayTerm { element: ty, .. }
        | Kind::Pattern { base: ty, .. } => push(work, ty, pay)?,
        Kind::Tuple(list) => {
            for &ty in store.list(list).ok_or("invalid named opaque tuple")? {
                push(work, ty, pay)?;
            }
        }
        Kind::Function {
            parameters, result, ..
        } => {
            push(work, result, pay)?;
            for &ty in store
                .list(parameters)
                .ok_or("invalid named opaque signature")?
            {
                push(work, ty, pay)?;
            }
        }
        Kind::FunctionItem {
            arguments,
            signature,
            ..
        }
        | Kind::Closure {
            arguments,
            signature,
            ..
        } => {
            push(work, signature, pay)?;
            arguments_children(store, arguments, work, pay)?;
            if let Kind::Closure { environment, .. } = kind {
                push(work, environment, pay)?;
            }
        }
        Kind::Adt(_, arguments)
        | Kind::Opaque(_, arguments)
        | Kind::Future { arguments, .. }
        | Kind::Dynamic { arguments, .. } => arguments_children(store, arguments, work, pay)?,
        Kind::Projection(application) => {
            push(work, application.subject, pay)?;
            let trait_ = store
                .trait_application(application.trait_application)
                .ok_or("invalid named opaque projection")?;
            arguments_children(store, trait_.arguments, work, pay)?;
            arguments_children(store, application.arguments, work, pay)?;
        }
        Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) | Kind::Foreign(_) => {}
    }
    Ok(())
}
fn arguments_children(
    store: &Store,
    arguments: crate::types::GenericArgs,
    work: &mut Vec<TypeId>,
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for argument in store
        .arguments(arguments)
        .ok_or("invalid named opaque arguments")?
    {
        pay(1, 0)?;
        if let Argument::Type(ty) = argument {
            push(work, ty, pay)?;
        }
    }
    Ok(())
}
pub(super) fn applications(
    report: &Report,
    store: &Store,
    root: TypeId,
) -> Result<HashMap<OpaqueId, TypeId>, &'static str> {
    let mut applications = HashMap::default();
    let pay = |steps, bytes| report.package.pay_named_opaque(steps, bytes);
    walk(
        store,
        &[root],
        |ty, kind| {
            if let Kind::Opaque(id, _) = kind {
                let growth =
                    Growth::map::<OpaqueId, TypeId>(applications.len(), applications.capacity(), 1);
                pay(growth.work, growth.bytes)?;
                if applications
                    .insert(id, ty)
                    .is_some_and(|previous| previous != ty)
                {
                    return Err("named opaque has distinct defining applications in one output");
                }
            }
            Ok(())
        },
        &pay,
    )?;
    Ok(applications)
}
/// The trait methods the types `roots` name as a body typed them, as
/// `<Input as From<&P>>::from` for `.map(Input::from)`, once their own
/// inputs are concrete: no parameter, projection, opaque or bound lifetime.
/// Each distinct method is listed once.
pub(crate) fn trait_methods(
    store: &Store,
    associated: &crate::sema::associated::Inventory,
    roots: &[TypeId],
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<TypeId>, &'static str> {
    let mut methods = Vec::new();
    walk(
        store,
        roots,
        |ty, kind| {
            if let Kind::FunctionItem { item, .. } = kind
                && associated.trait_owner(DefId(item.0)).is_some()
                && store.escaping_binders(ty) == Some(0)
                && store.contains_parameters(ty) == Some(false)
                && store.contains_projections(ty) == Some(false)
                && store.contains_opaques(ty) == Some(false)
            {
                let growth = Growth::vector::<TypeId>(methods.len(), methods.capacity(), 1);
                pay(growth.work, growth.bytes)?;
                methods.push(ty);
            }
            Ok(())
        },
        pay,
    )?;
    Ok(methods)
}
pub(super) fn defining_arguments(
    report: &Report,
    store: &Store,
    application: TypeId,
    owner: DefId,
) -> Result<(), &'static str> {
    let Some(Kind::Opaque(_, arguments)) = store.kind(application) else {
        return Err("invalid named defining application");
    };
    let mut seen = HashSet::default();
    for argument in store
        .arguments(arguments)
        .ok_or("invalid named defining arguments")?
    {
        let valid = match argument {
            Argument::Lifetime(region) => matches!(region.bound_parts(), Some((0, _))),
            Argument::Type(ty) => {
                matches!(store.kind(ty),Some(Kind::Parameter(id)) if store.parameter(id).is_some_and(|p|p.owner.0==owner.0))
            }
            Argument::Const(id) => {
                matches!(store.const_kind(id),Some(crate::types::arguments::ConstKind::Parameter(id)) if store.parameter(id).is_some_and(|p|p.owner.0==owner.0))
            }
        };
        let growth = Growth::map::<Argument, ()>(seen.len(), seen.capacity(), 1);
        report
            .package
            .pay_named_opaque(1 + growth.work, growth.bytes)?;
        if !valid || !seen.insert(argument) {
            return Err("named opaque defining arguments require distinct generic parameters");
        }
    }
    Ok(())
}
pub(super) fn require_alias_parameters(
    report: &Report,
    store: &Store,
    hidden: TypeId,
    owner: DefId,
) -> Result<(), &'static str> {
    walk(
        store,
        &[hidden],
        |_, kind| {
            if let Kind::Parameter(id) = kind {
                require_parameter(store, id, owner)?;
            }
            match kind {
                Kind::Adt(_, arguments)
                | Kind::Opaque(_, arguments)
                | Kind::Future { arguments, .. }
                | Kind::Dynamic { arguments, .. }
                | Kind::FunctionItem { arguments, .. }
                | Kind::Closure { arguments, .. } => {
                    require_const_arguments(report, store, arguments, owner)?;
                }
                Kind::ArrayTerm { length, .. } => require_const(store, length, owner)?,
                Kind::Projection(application) => {
                    require_const_arguments(report, store, application.arguments, owner)?;
                    let trait_ = store
                        .trait_application(application.trait_application)
                        .ok_or("invalid hidden projection")?;
                    require_const_arguments(report, store, trait_.arguments, owner)?;
                }
                _ => {}
            }
            Ok(())
        },
        &|steps, bytes| report.package.pay_named_opaque(steps, bytes),
    )
}

fn require_parameter(
    store: &Store,
    id: crate::types::parameters::ParameterId,
    owner: DefId,
) -> Result<(), &'static str> {
    if store.parameter(id).is_none_or(|p| p.owner.0 != owner.0) {
        return Err("named opaque hidden type captures a foreign generic parameter");
    }
    Ok(())
}
fn require_const(
    store: &Store,
    id: crate::types::arguments::ConstId,
    owner: DefId,
) -> Result<(), &'static str> {
    if let Some(crate::types::arguments::ConstKind::Parameter(id)) = store.const_kind(id) {
        require_parameter(store, id, owner)?;
    }
    Ok(())
}
fn require_const_arguments(
    report: &Report,
    store: &Store,
    arguments: crate::types::GenericArgs,
    owner: DefId,
) -> Result<(), &'static str> {
    for argument in store
        .arguments(arguments)
        .ok_or("invalid hidden arguments")?
    {
        report.package.pay_named_opaque(1, 0)?;
        if let Argument::Const(id) = argument {
            require_const(store, id, owner)?;
        }
    }
    Ok(())
}
