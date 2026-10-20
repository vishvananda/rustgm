//! One-parameter summaries share type, list and argument graphs across default bounds.
use crate::types::{
    GenericArgs, Kind, ListId, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    parameters::{
        ParameterId,
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    traits::ApplicationId,
};
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../../../tests/invariants/default_occurrences.rs"]
mod tests;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Value {
    #[default]
    Empty,
    One(ParameterId),
    Multiple,
}
impl Value {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, value) | (value, Self::Empty) => value,
            (Self::One(left), Self::One(right)) if left == right => self,
            _ => Self::Multiple,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Type(TypeId),
    Types(ListId),
    Arguments(GenericArgs),
    Trait(ApplicationId),
}

#[derive(Default)]
pub(crate) struct Sole {
    completed: HashMap<Node, Value>,
    work: Vec<(Node, bool)>,
    pub steps: usize,
}

impl Sole {
    pub fn clause(
        &mut self,
        store: &Store,
        clause: Clause,
    ) -> Result<Option<ParameterId>, &'static str> {
        let predicate = match clause {
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid default parameter clause")?
                    .predicate
            }
        };
        self.predicate(store, predicate)
    }

    /// Parameter identities are not bound by lifetime quantifiers. This is an
    /// occurrence summary, including non-injective projection inputs, never proof.
    pub fn predicate(
        &mut self,
        store: &Store,
        predicate: Predicate,
    ) -> Result<Option<ParameterId>, &'static str> {
        self.charge(1)?;
        let subject = self.collect(store, Node::Type(predicate.ty))?;
        let bound = match predicate.bound {
            Bound::Trait(id) => self.collect(store, Node::Trait(id))?,
            Bound::ProjectionEqual(result) => self.collect(store, Node::Type(result))?,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Outlives(_) => Value::Empty,
        };
        Ok(match subject.merge(bound) {
            Value::One(id) => Some(id),
            Value::Empty | Value::Multiple => None,
        })
    }

    fn collect(&mut self, store: &Store, root: Node) -> Result<Value, &'static str> {
        self.work.push((root, false));
        while let Some((node, finish)) = self.work.pop() {
            self.charge(1)?;
            if self.completed.contains_key(&node) {
                continue;
            }
            if !finish {
                self.work.push((node, true));
            }
            let mut ready = true;
            let mut edges = 0;
            let value = parts(store, node, &mut edges, |child| {
                if let Some(value) = self.completed.get(&child) {
                    return Ok(*value);
                }
                if finish {
                    return Err("unfinished default parameter dependency");
                }
                if self.work.len() >= 1_000_000 {
                    return Err("default parameter occurrence work budget exceeded");
                }
                ready = false;
                self.work.push((child, false));
                Ok(Value::Empty)
            })?;
            self.charge(edges)?;
            if ready {
                if !finish {
                    self.work.pop();
                }
                self.completed.insert(node, value);
            }
        }
        self.completed
            .get(&root)
            .copied()
            .ok_or("missing default parameter summary")
    }

    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("default parameter occurrence work budget exceeded");
        }
        Ok(())
    }
}

fn constant(store: &Store, id: ConstId) -> Result<Value, &'static str> {
    Ok(
        match store
            .const_kind(id)
            .ok_or("invalid default parameter constant")?
        {
            ConstKind::Parameter(id) => Value::One(id),
            ConstKind::Value(_) => Value::Empty,
        },
    )
}

fn parts(
    store: &Store,
    node: Node,
    edges: &mut usize,
    mut child: impl FnMut(Node) -> Result<Value, &'static str>,
) -> Result<Value, &'static str> {
    let mut value = Value::Empty;
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
                    Argument::Const(id) => constant(store, id)?,
                    Argument::Lifetime(_) => Value::Empty,
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
                return Ok(Value::Empty);
            }
            let mut dependency = |node| {
                *edges += 1;
                child(node)
            };
            value = match store.kind(ty).ok_or("missing default parameter type")? {
                Kind::Parameter(id) => Value::One(id),
                Kind::ArrayTerm { element, length } => {
                    dependency(Node::Type(element))?.merge(constant(store, length)?)
                }
                Kind::Array { element, .. }
                | Kind::Slice(element)
                | Kind::Reference {
                    referent: element, ..
                }
                | Kind::Pointer {
                    pointee: element, ..
                } => dependency(Node::Type(element))?,
                Kind::Tuple(list) => dependency(Node::Types(list))?,
                Kind::Function {
                    parameters, result, ..
                } => dependency(Node::Types(parameters))?.merge(dependency(Node::Type(result))?),
                Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                    dependency(Node::Arguments(arguments))?
                }
                Kind::FunctionItem {
                    arguments,
                    signature,
                    ..
                } => dependency(Node::Arguments(arguments))?
                    .merge(dependency(Node::Type(signature))?),
                Kind::Projection(application) => dependency(Node::Type(application.subject))?
                    .merge(dependency(Node::Trait(application.trait_application))?)
                    .merge(dependency(Node::Arguments(application.arguments))?),
                Kind::Never | Kind::Primitive(_) => Value::Empty,
            };
        }
    }
    Ok(value)
}
