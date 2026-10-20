//! Built-in struct unsizing eligibility follows declaration tails, not pointer layout.
mod eligibility;
use super::{
    ItemId, Kind, Store, TypeId,
    aggregate::{self, Definitions},
    arguments::Argument,
    parameters::{ParameterId, uses::Uses},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Item(ItemId),
    Type(TypeId),
}

/// Completed field declarations are immutable. Missing declarations are errors,
/// never cached negative answers. Parameter identities are unique across owners.
#[derive(Default)]
pub struct Rules {
    completed: HashMap<Key, Option<ParameterId>>,
    excluded: Uses,
    projected: eligibility::Candidates,
    pub steps: usize,
}
impl Rules {
    /// Find the only argument that can change under built-in struct unsizing.
    /// This supplies structural eligibility, not an argument relation or proof.
    /// # Errors
    /// Rejects incomplete declarations, malformed types and exhausted work.
    pub fn parameter(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        item: ItemId,
    ) -> Result<Option<u32>, &'static str> {
        let root = Key::Item(item);
        let mut work = vec![(root, false)];
        let mut active = HashSet::new();
        while let Some((key, finish)) = work.pop() {
            self.charge(1)?;
            if self.completed.contains_key(&key) {
                continue;
            }
            if !finish && !active.insert(key) {
                return Err("cycle in struct unsizing declarations");
            }
            let dependency = self.dependency(types, definitions, key)?;
            if let Some(child) = dependency
                && !self.completed.contains_key(&child)
            {
                work.push((key, true));
                work.push((child, false));
                continue;
            }
            let result = match key {
                Key::Type(ty) => match types.kind(ty) {
                    Some(Kind::Parameter(id)) => Some(id),
                    _ => dependency.and_then(|child| self.completed[&child]),
                },
                Key::Item(owner) => {
                    let candidate = dependency.and_then(|child| self.completed[&child]);
                    candidate.filter(|&id| {
                        types.parameter(id).is_some_and(|p| p.owner == owner)
                            && !self.excluded.parameters.contains(&id)
                    })
                }
            };
            self.completed.insert(key, result);
            active.remove(&key);
        }
        self.completed[&root]
            .map(|id| {
                types
                    .parameter(id)
                    .map(|p| p.index)
                    .ok_or("invalid struct unsizing parameter")
            })
            .transpose()
    }

    fn dependency(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
    ) -> Result<Option<Key>, &'static str> {
        match key {
            Key::Item(item) => {
                let definition = definitions
                    .get(item)
                    .ok_or("missing unsizing declaration")?;
                if definition.kind != aggregate::Kind::Struct {
                    return Ok(None);
                }
                let fields = definitions
                    .variants(definition)
                    .first()
                    .and_then(|v| types.list(v.fields))
                    .ok_or("missing unsizing fields")?;
                let Some((&tail, prefix)) = fields.split_last() else {
                    return Ok(None);
                };
                self.charge(prefix.len())?;
                for &field in prefix {
                    self.exclude(types, field)?;
                }
                Ok(Some(Key::Type(tail)))
            }
            Key::Type(ty) => {
                let Some(Kind::Adt(item, arguments)) = types.kind(ty) else {
                    return Ok(None);
                };
                let Some(&parameter) = self.completed.get(&Key::Item(item)) else {
                    return Ok(Some(Key::Item(item)));
                };
                let Some(parameter) = parameter else {
                    return Ok(None);
                };
                let slot = types
                    .parameter(parameter)
                    .ok_or("invalid nested unsizing parameter")?
                    .index as usize;
                let arguments = types
                    .arguments(arguments)
                    .ok_or("missing unsizing arguments")?;
                self.charge(arguments.len())?;
                let mut selected = None;
                for (index, argument) in arguments.enumerate() {
                    if let Argument::Type(ty) = argument {
                        if index == slot {
                            selected = Some(Key::Type(ty));
                        } else {
                            self.exclude(types, ty)?;
                        }
                    }
                }
                Ok(selected)
            }
        }
    }
    fn exclude(&mut self, types: &Store, ty: TypeId) -> Result<(), &'static str> {
        let before = self.excluded.steps;
        self.excluded.collect(types, ty)?;
        self.charge(self.excluded.steps - before)
    }
    fn charge(&mut self, steps: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(steps);
        if self.steps > 1_000_000 {
            return Err("struct unsizing declaration work budget exceeded");
        }
        Ok(())
    }
}
