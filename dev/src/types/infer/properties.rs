//! Built-in predicate possibility after isolated equations, under actual caller assumptions.
use super::{Constructor, Context, Error, Store, TermId, TypeId, Value};
use crate::types::{
    ItemId, Primitive,
    aggregate::Definitions,
    copy::{Copies, Properties},
    parameters::{environment::Environment, shape::Shapes},
};
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

#[derive(Clone, Copy)]
pub struct Possible {
    pub sized: bool,
    pub abilities: Properties,
    ty: Option<TypeId>,
}
impl Possible {
    const UNKNOWN: Self = Self {
        sized: true,
        abilities: Properties {
            copy: true,
            clone_: true,
        },
        ty: None,
    };
}

/// One context after its equations are fixed, with one immutable caller environment.
/// Unknowns remain possible; rigid parameters retain only their declared abilities.
/// These results filter candidates and never authorize operations in a body.
pub struct Query {
    cache: HashMap<TermId, Possible>,
    copies: Copies,
    shapes: Shapes,
}
impl Query {
    #[must_use]
    pub fn new(environment: Arc<Environment>) -> Self {
        Self {
            cache: HashMap::new(),
            copies: Copies::for_derive(Some(Arc::clone(&environment))),
            shapes: Shapes::new(environment),
        }
    }
    /// # Errors
    /// Rejects malformed graphs, missing assumptions and exhausted query resources.
    pub fn demand<S: BuildHasher>(
        &mut self,
        context: &mut Context,
        store: &mut Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        root: TermId,
    ) -> Result<Possible, &'static str> {
        let root = context.root(root).map_err(inference)?;
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((id, finish)) = work.pop() {
            context.account_work(1).map_err(inference)?;
            let id = context.root(id).map_err(inference)?;
            if self.cache.contains_key(&id) {
                continue;
            }
            let term = context.terms[id.0 as usize];
            let value = match term.value {
                Value::Variable(_) => Possible::UNKNOWN,
                Value::Known(ty) => self.known(store, definitions, selected, ty)?,
                Value::Construct(Constructor::Projection(_)) => {
                    return Err("candidate properties require associated type normalization");
                }
                Value::Construct(kind) => {
                    if !finish {
                        context
                            .account_work((term.end - term.start) as usize)
                            .map_err(inference)?;
                        work.push((id, true));
                        work.extend(
                            context.edges[term.start as usize..term.end as usize]
                                .iter()
                                .rev()
                                .map(|id| (*id, false)),
                        );
                        continue;
                    }
                    children.clear();
                    for offset in term.start..term.end {
                        let child = context
                            .root(context.edges[offset as usize])
                            .map_err(inference)?;
                        children.push(
                            *self
                                .cache
                                .get(&child)
                                .ok_or("missing candidate property child")?,
                        );
                    }
                    let canonical = children.iter().map(|c| c.ty).collect::<Option<Vec<_>>>();
                    let ty = if let Some(types) = canonical {
                        match context.canonical(store, kind, &types) {
                            Ok(ty) => Some(ty),
                            Err(Error::Unresolved) => None,
                            Err(e) => return Err(inference(e)),
                        }
                    } else {
                        None
                    };
                    if let Some(ty) = ty {
                        self.known(store, definitions, selected, ty)?
                    } else {
                        partial(kind, &children, definitions, selected)
                    }
                }
            };
            self.cache.insert(id, value);
        }
        self.cache
            .get(&root)
            .copied()
            .ok_or("missing candidate properties")
    }
    fn known<S: BuildHasher>(
        &mut self,
        store: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        ty: TypeId,
    ) -> Result<Possible, &'static str> {
        Ok(Possible {
            ty: Some(ty),
            sized: self.shapes.demand(store, ty)?.sized,
            abilities: self.copies.demand(store, definitions, selected, ty)?,
        })
    }
}

fn partial<S: BuildHasher>(
    kind: Constructor,
    children: &[Possible],
    definitions: &Definitions,
    selected: &HashMap<ItemId, Properties, S>,
) -> Possible {
    let mut value = Possible::UNKNOWN;
    match kind {
        Constructor::Primitive(Primitive::Str) | Constructor::Slice => {
            value.sized = false;
            value.abilities = Properties::default();
        }
        Constructor::Reference { mutable: true, .. } => value.abilities = Properties::default(),
        Constructor::Tuple
        | Constructor::Array(_)
        | Constructor::ArrayTerm(_)
        | Constructor::ArrayInfer(_) => {
            for child in children {
                value.sized &= child.sized;
                value.abilities.copy &= child.abilities.copy;
                value.abilities.clone_ &= child.abilities.clone_;
            }
        }
        Constructor::Adt(item, _) | Constructor::AdtArgs(item, _) => {
            value.abilities = selected.get(&item).copied().unwrap_or_default();
            let packed = definitions
                .get(item)
                .is_some_and(|d| d.repr.packed.is_some());
            for child in children {
                value.abilities.copy &= child.abilities.copy;
                value.abilities.clone_ &= child.abilities.clone_;
                if packed {
                    value.abilities.clone_ &= child.abilities.copy;
                }
            }
        }
        _ => {}
    }
    value
}
fn inference(error: Error) -> &'static str {
    match error {
        Error::Budget => "candidate property work budget exceeded",
        Error::Overflow => "candidate property arena exceeds 32 bits",
        _ => "invalid candidate property inference graph",
    }
}
