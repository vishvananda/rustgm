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
            value = type_parts::<SET, V>(
                store,
                ty,
                depth,
                edges,
                &mut parameter,
                &mut region,
                &mut child,
            )?
        }
    }
    Ok(value)
}

fn edge<V>(
    edges: &mut usize,
    child: &mut impl FnMut(Node, u16) -> Result<V, &'static str>,
    node: Node,
    depth: u16,
) -> Result<V, &'static str> {
    *edges += 1;
    child(node, depth)
}
fn type_parts<const SET: bool, V: Summary>(
    store: &Store,
    ty: TypeId,
    depth: u16,
    edges: &mut usize,
    parameter: &mut impl FnMut(ParameterId) -> Result<V, &'static str>,
    region: &mut impl FnMut(RegionId, u16) -> Result<V, &'static str>,
    child: &mut impl FnMut(Node, u16) -> Result<V, &'static str>,
) -> Result<V, &'static str> {
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
    let kind = store.kind(ty).ok_or("missing default parameter type")?;
    let value = match kind {
        Kind::Closure { .. } | Kind::Function { .. } | Kind::FunctionItem { .. } => {
            callable_parts::<SET, V>(kind, depth, edges, child)?
        }
        Kind::Parameter(id) => parameter(id)?,
        Kind::ArrayTerm { element, length } => edge(edges, child, Node::Type(element), depth)?
            .merge(constant(store, length, parameter)?),
        Kind::Array { element, .. }
        | Kind::Slice(element)
        | Kind::Pointer {
            pointee: element, ..
        } => edge(edges, child, Node::Type(element), depth)?,
        Kind::Reference {
            region: value,
            referent,
            ..
        } => region(value, depth)?.merge(edge(edges, child, Node::Type(referent), depth)?),
        Kind::Tuple(list) => edge(edges, child, Node::Types(list), depth)?,
        Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
            edge(edges, child, Node::Arguments(arguments), depth)?
        }
        Kind::Dynamic {
            arguments,
            region: value,
            ..
        } => region(value, depth)?.merge(edge(
            edges,
            child,
            Node::Arguments(arguments),
            depth
                .checked_add(1)
                .ok_or("occurrence binder depth overflow")?,
        )?),
        Kind::Projection(application) => {
            edge(edges, child, Node::Type(application.subject), depth)?
                .merge(edge(
                    edges,
                    child,
                    Node::Trait(application.trait_application),
                    depth,
                )?)
                .merge(edge(
                    edges,
                    child,
                    Node::Arguments(application.arguments),
                    depth,
                )?)
        }
        Kind::Never | Kind::Primitive(_) => V::empty(),
    };
    Ok(value)
}
fn callable_parts<const SET: bool, V: Summary>(
    kind: Kind,
    depth: u16,
    edges: &mut usize,
    child: &mut impl FnMut(Node, u16) -> Result<V, &'static str>,
) -> Result<V, &'static str> {
    Ok(match kind {
        Kind::Closure {
            arguments,
            signature,
            environment,
            ..
        } => edge(edges, child, Node::Arguments(arguments), depth)?
            .merge(edge(edges, child, Node::Type(signature), depth)?)
            .merge(edge(edges, child, Node::Type(environment), depth)?),
        Kind::Function {
            parameters, result, ..
        } => {
            let nested = depth
                .checked_add(1)
                .ok_or("occurrence binder depth overflow")?;
            edge(edges, child, Node::Types(parameters), nested)?.merge(edge(
                edges,
                child,
                Node::Type(result),
                nested,
            )?)
        }
        Kind::FunctionItem {
            arguments,
            signature,
            ..
        } => {
            let value = edge(edges, child, Node::Arguments(arguments), depth)?;
            if SET {
                value.merge(edge(edges, child, Node::Type(signature), depth)?)
            } else {
                value
            }
        }

        _ => return Err("canonical callable occurrence kind changed"),
    })
}
