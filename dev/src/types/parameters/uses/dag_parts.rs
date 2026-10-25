//! Canonical occurrence edges shared by summaries and capped path counting.
use crate::types::{
    GenericArgs, Kind, ListId, RegionId, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    parameters::ParameterId,
    traits::ApplicationId,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Node {
    Type(TypeId),
    Types(ListId),
    Arguments(GenericArgs),
    Trait(ApplicationId),
}

pub(super) trait Summary: Copy {
    fn empty() -> Self;
    fn merge(self, other: Self) -> Self;
}
impl Summary for () {
    fn empty() -> Self {}
    fn merge(self, _other: Self) -> Self {}
}

fn constant<V: Summary>(
    store: &Store,
    id: ConstId,
    parameter: &mut impl FnMut(ParameterId) -> Result<V, &'static str>,
) -> Result<V, &'static str> {
    Ok(
        match store
            .const_kind(id)
            .ok_or("invalid default parameter constant")?
        {
            ConstKind::Parameter(id) => parameter(id)?,
            ConstKind::Value(_) => V::empty(),
        },
    )
}

pub(super) fn parts<const SET: bool, V: Summary>(
    store: &Store,
    node: Node,
    depth: u16,
    edges: &mut usize,
    mut parameter: impl FnMut(ParameterId) -> Result<V, &'static str>,
    mut region: impl FnMut(RegionId, u16) -> Result<V, &'static str>,
    mut child: impl FnMut(Node, u16) -> Result<V, &'static str>,
) -> Result<V, &'static str> {
    let mut value = V::empty();
    match node {
        Node::Types(list) => {
            for &ty in store.list(list).ok_or("invalid default parameter list")? {
                *edges += 1;
                value = value.merge(child(Node::Type(ty), depth)?);
            }
        }
        Node::Arguments(arguments) => {
            for argument in store
                .arguments(arguments)
                .ok_or("invalid default parameter arguments")?
            {
                *edges += 1;
                value = value.merge(match argument {
                    Argument::Type(ty) => child(Node::Type(ty), depth)?,
                    Argument::Const(id) => constant(store, id, &mut parameter)?,
                    Argument::Lifetime(value) => region(value, depth)?,
                });
            }
        }
        Node::Trait(id) => {
            *edges += 1;
            value = child(
                Node::Arguments(
                    store
                        .trait_application(id)
                        .ok_or("invalid default parameter trait")?
                        .arguments,
                ),
                depth,
            )?;
        }
        Node::Type(ty) => {
            if !store
                .contains_parameters(ty)
                .ok_or("invalid default parameter type")?
                && (SET
                    || store
                        .escaping_binders(ty)
                        .ok_or("invalid occurrence binder summary")?
                        <= depth)
            {
                return Ok(V::empty());
            }
            let mut dependency = |node, inside| {
                *edges += 1;
                child(node, inside)
            };
            value = match store.kind(ty).ok_or("missing default parameter type")? {
                Kind::Parameter(id) => parameter(id)?,
                Kind::ArrayTerm { element, length } => dependency(Node::Type(element), depth)?
                    .merge(constant(store, length, &mut parameter)?),
                Kind::Array { element, .. }
                | Kind::Slice(element)
                | Kind::Pointer {
                    pointee: element, ..
                } => dependency(Node::Type(element), depth)?,
                Kind::Reference {
                    region: value,
                    referent,
                    ..
                } => region(value, depth)?.merge(dependency(Node::Type(referent), depth)?),
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => dependency(Node::Arguments(arguments), depth)?
                    .merge(dependency(Node::Type(signature), depth)?)
                    .merge(dependency(Node::Type(environment), depth)?),
                Kind::Tuple(list) => dependency(Node::Types(list), depth)?,
                Kind::Function {
                    parameters, result, ..
                } => {
                    let nested = depth
                        .checked_add(1)
                        .ok_or("occurrence binder depth overflow")?;
                    dependency(Node::Types(parameters), nested)?
                        .merge(dependency(Node::Type(result), nested)?)
                }
                Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                    dependency(Node::Arguments(arguments), depth)?
                }
                Kind::Dynamic {
                    arguments,
                    region: value,
                    ..
                } => region(value, depth)?.merge(dependency(
                    Node::Arguments(arguments),
                    depth
                        .checked_add(1)
                        .ok_or("occurrence binder depth overflow")?,
                )?),
                Kind::FunctionItem {
                    arguments,
                    signature,
                    ..
                } => {
                    let value = dependency(Node::Arguments(arguments), depth)?;
                    if SET {
                        value.merge(dependency(Node::Type(signature), depth)?)
                    } else {
                        value
                    }
                }
                Kind::Projection(application) => {
                    dependency(Node::Type(application.subject), depth)?
                        .merge(dependency(
                            Node::Trait(application.trait_application),
                            depth,
                        )?)
                        .merge(dependency(Node::Arguments(application.arguments), depth)?)
                }
                Kind::Never | Kind::Primitive(_) => V::empty(),
            };
        }
    }
    Ok(value)
}
