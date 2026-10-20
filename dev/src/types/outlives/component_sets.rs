//! Share canonical component sets across targets without copying type prefixes.
use super::{
    Argument, Budget, ComponentKind, GenericArgs, HashMap, HashSet, Kind, ParameterId, Projection,
    RegionId, Store, TypeId, free,
};
use crate::{support::radix_map, types::ListId};
use radix_map::{Id, Map, Node};
#[cfg(test)]
#[path = "../../../tests/invariants/component_sets.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Atom {
    Region(RegionId),
    Parameter(ParameterId),
    Projection(Projection),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Input {
    Type(TypeId),
    List(ListId),
    Arguments(GenericArgs),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    input: Input,
    depth: u16,
    kind: ComponentKind,
}

/// Structural results depend on depth and policy, never on a lifetime target.
/// Target cursors retain visited set nodes, so repeated or growing sets share
/// work without retaining a flattened component vector for every type prefix.
#[derive(Default)]
pub(super) struct Sets {
    completed: HashMap<Key, Id>,
    atoms: HashMap<Atom, u32>,
    map: Map<Atom>,
    work: Vec<(Key, bool)>,
    visited: HashSet<(RegionId, Id)>,
    pending: Vec<Id>,
}
impl Sets {
    pub(super) fn extend(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
        kind: ComponentKind,
        shorter: RegionId,
        budget: &mut Budget,
    ) -> Result<Vec<Atom>, &'static str> {
        if !matches!(kind, ComponentKind::Outlives | ComponentKind::TypeBound) {
            return Err("formation components require a type-bound policy");
        }
        let key = Key {
            input: Input::Type(ty),
            depth,
            kind,
        };
        let root = self.collect(types, key, budget)?;
        self.pending.push(root);
        let mut found = Vec::new();
        while let Some(id) = self.pending.pop() {
            budget.charge()?;
            if id == Id::EMPTY || !self.visited.insert((shorter, id)) {
                continue;
            }
            match self.map.node(id).map_err(map_error)? {
                Node::Leaf(_, atom) => found.push(atom),
                Node::Branch { left, right, .. } => {
                    budget.pending(self.pending.len().saturating_add(2))?;
                    self.pending.push(right);
                    self.pending.push(left);
                }
            }
        }
        Ok(found)
    }

    fn collect(
        &mut self,
        types: &Store,
        key: Key,
        budget: &mut Budget,
    ) -> Result<Id, &'static str> {
        self.work.push((key, false));
        while let Some((key, finish)) = self.work.pop() {
            budget.charge()?;
            if self.completed.contains_key(&key) {
                continue;
            }
            if !finish {
                budget.pending(self.work.len().saturating_add(1))?;
                self.work.push((key, true));
            }
            let set = self.inspect(types, key, finish, budget)?;
            if finish {
                self.completed.insert(key, set);
            }
        }
        self.completed
            .get(&key)
            .copied()
            .ok_or("missing component set")
    }

    fn child(&mut self, key: Key, finish: bool, budget: &mut Budget) -> Result<Id, &'static str> {
        budget.charge()?;
        if let Some(&found) = self.completed.get(&key) {
            return Ok(found);
        }
        if finish {
            return Err("missing component set dependency");
        }
        budget.pending(self.work.len().saturating_add(1))?;
        self.work.push((key, false));
        Ok(Id::EMPTY)
    }

    fn inspect(
        &mut self,
        types: &Store,
        key: Key,
        finish: bool,
        budget: &mut Budget,
    ) -> Result<Id, &'static str> {
        let ty = match key.input {
            Input::Type(ty) => ty,
            Input::List(list) => return self.list(types, list, key, finish, budget),
            Input::Arguments(args) => return self.arguments(types, args, key, finish, budget),
        };
        match types.kind(ty).ok_or("invalid component type")? {
            Kind::Parameter(id) => self.atom(Atom::Parameter(id), budget),
            Kind::Reference {
                region, referent, ..
            } => {
                let set = self.region(region, key.depth, budget)?;
                if key.kind == ComponentKind::Outlives {
                    return Ok(set);
                }
                let child = self.child(
                    Key {
                        input: Input::Type(referent),
                        ..key
                    },
                    finish,
                    budget,
                )?;
                self.merge(set, child, budget)
            }
            Kind::Pointer {
                pointee: element, ..
            }
            | Kind::Slice(element)
            | Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. } => self.child(
                Key {
                    input: Input::Type(element),
                    ..key
                },
                finish,
                budget,
            ),
            Kind::Tuple(list) => self.child(
                Key {
                    input: Input::List(list),
                    ..key
                },
                finish,
                budget,
            ),
            Kind::Function {
                parameters, result, ..
            } => {
                let depth = key
                    .depth
                    .checked_add(1)
                    .ok_or("component binder depth overflow")?;
                let result = self.child(
                    Key {
                        input: Input::Type(result),
                        depth,
                        ..key
                    },
                    finish,
                    budget,
                )?;
                let inputs = self.child(
                    Key {
                        input: Input::List(parameters),
                        depth,
                        ..key
                    },
                    finish,
                    budget,
                )?;
                self.merge(result, inputs, budget)
            }
            Kind::Adt(_, args)
            | Kind::FunctionItem {
                arguments: args, ..
            } => self.child(
                Key {
                    input: Input::Arguments(args),
                    ..key
                },
                finish,
                budget,
            ),
            Kind::Opaque(..) | Kind::Projection(_) => self.atom(
                Atom::Projection(Projection {
                    ty,
                    depth: key.depth,
                }),
                budget,
            ),
            Kind::Never | Kind::Primitive(_) => Ok(Id::EMPTY),
        }
    }

    fn list(
        &mut self,
        types: &Store,
        list: ListId,
        key: Key,
        finish: bool,
        budget: &mut Budget,
    ) -> Result<Id, &'static str> {
        let mut set = Id::EMPTY;
        for &ty in types.list(list).ok_or("missing component type list")? {
            let child = self.child(
                Key {
                    input: Input::Type(ty),
                    ..key
                },
                finish,
                budget,
            )?;
            set = self.merge(set, child, budget)?;
        }
        Ok(set)
    }

    fn arguments(
        &mut self,
        types: &Store,
        args: GenericArgs,
        key: Key,
        finish: bool,
        budget: &mut Budget,
    ) -> Result<Id, &'static str> {
        let mut set = Id::EMPTY;
        for argument in types.arguments(args).ok_or("missing component arguments")? {
            budget.charge()?;
            let child = match argument {
                Argument::Lifetime(region) => self.region(region, key.depth, budget)?,
                Argument::Type(ty) => self.child(
                    Key {
                        input: Input::Type(ty),
                        ..key
                    },
                    finish,
                    budget,
                )?,
                Argument::Const(_) => Id::EMPTY,
            };
            set = self.merge(set, child, budget)?;
        }
        Ok(set)
    }

    fn region(
        &mut self,
        region: RegionId,
        depth: u16,
        budget: &mut Budget,
    ) -> Result<Id, &'static str> {
        match free(region, depth)? {
            Some(region) if region != RegionId::STATIC => self.atom(Atom::Region(region), budget),
            _ => Ok(Id::EMPTY),
        }
    }
    fn atom(&mut self, atom: Atom, budget: &mut Budget) -> Result<Id, &'static str> {
        let next = u32::try_from(self.atoms.len()).map_err(|_| "too many formation components")?;
        let key = *self.atoms.entry(atom).or_insert(next);
        let before = self.map.steps;
        let result = self.map.singleton(key, atom).map_err(map_error)?;
        budget.work(self.map.steps.saturating_sub(before))?;
        Ok(result)
    }
    fn merge(&mut self, a: Id, b: Id, budget: &mut Budget) -> Result<Id, &'static str> {
        let before = self.map.steps;
        let result = self.map.union(a, b).map_err(map_error)?;
        budget.work(self.map.steps.saturating_sub(before))?;
        result.ok_or("conflicting formation component identities")
    }
}
fn map_error(error: radix_map::Error) -> &'static str {
    match error {
        radix_map::Error::Identity => "invalid formation component identity",
        radix_map::Error::Budget => "type outlives work budget exceeded",
    }
}
