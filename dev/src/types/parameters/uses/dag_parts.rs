//! Canonical occurrence edges shared by summaries and capped path counting.
use crate::types::{
    GenericArgs, Kind, ListId, Store, TypeId,
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

pub(super) fn parts<V: Summary>(
    store: &Store,
    node: Node,
    edges: &mut usize,
    mut parameter: impl FnMut(ParameterId) -> Result<V, &'static str>,
    mut child: impl FnMut(Node) -> Result<V, &'static str>,
) -> Result<V, &'static str> {
    let mut value = V::empty();
    match node {
        Node::Types(list) => {
            for &ty in store.list(list).ok_or("invalid default parameter list")? {
                *edges += 1;
                value = value.merge(child(Node::Type(ty))?);
            }
        }
        Node::Arguments(arguments) => {
            for argument in store
                .arguments(arguments)
                .ok_or("invalid default parameter arguments")?
            {
                *edges += 1;
                value = value.merge(match argument {
                    Argument::Type(ty) => child(Node::Type(ty))?,
                    Argument::Const(id) => constant(store, id, &mut parameter)?,
                    Argument::Lifetime(_) => V::empty(),
                });
            }
        }
        Node::Trait(id) => {
            *edges += 1;
            value = child(Node::Arguments(
                store
                    .trait_application(id)
                    .ok_or("invalid default parameter trait")?
                    .arguments,
            ))?;
        }
        Node::Type(ty) => {
            if !store
                .contains_parameters(ty)
                .ok_or("invalid default parameter type")?
            {
                return Ok(V::empty());
            }
            let mut dependency = |node| {
                *edges += 1;
                child(node)
            };
            value = match store.kind(ty).ok_or("missing default parameter type")? {
                Kind::Parameter(id) => parameter(id)?,
                Kind::ArrayTerm { element, length } => {
                    dependency(Node::Type(element))?.merge(constant(store, length, &mut parameter)?)
                }
                Kind::Array { element, .. }
                | Kind::Slice(element)
                | Kind::Reference {
                    referent: element, ..
                }
                | Kind::Pointer {
                    pointee: element, ..
                } => dependency(Node::Type(element))?,
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => dependency(Node::Arguments(arguments))?
                    .merge(dependency(Node::Type(signature))?)
                    .merge(dependency(Node::Type(environment))?),
                Kind::Tuple(list) => dependency(Node::Types(list))?,
                Kind::Function {
                    parameters, result, ..
                } => dependency(Node::Types(parameters))?.merge(dependency(Node::Type(result))?),
                Kind::Adt(_, arguments)
                | Kind::Opaque(_, arguments)
                | Kind::Dynamic { arguments, .. } => dependency(Node::Arguments(arguments))?,
                Kind::FunctionItem {
                    arguments,
                    signature,
                    ..
                } => dependency(Node::Arguments(arguments))?
                    .merge(dependency(Node::Type(signature))?),
                Kind::Projection(application) => dependency(Node::Type(application.subject))?
                    .merge(dependency(Node::Trait(application.trait_application))?)
                    .merge(dependency(Node::Arguments(application.arguments))?),
                Kind::Never | Kind::Primitive(_) => V::empty(),
            };
        }
    }
    Ok(value)
}
