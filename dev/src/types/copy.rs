//! Closed-profile Copy/Clone obligations, cached independently of storage layout.
use super::{ItemId, Kind, Primitive, Store, TypeId, aggregate::Definitions};
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Properties {
    pub copy: bool,
    pub clone_: bool,
}
impl Properties {
    const BOTH: Self = Self {
        copy: true,
        clone_: true,
    };
    fn intersect(self, other: Self) -> Self {
        Self {
            copy: self.copy && other.copy,
            clone_: self.clone_ && other.clone_,
        }
    }
}
#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(Properties),
    Failed(&'static str),
}
/// One canonical store and immutable completed definitions/derive selections.
/// Appending canonical types is permitted. Nominal types never gain Copy from layout.
#[derive(Default)]
pub struct Copies {
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl Copies {
    /// # Errors
    /// Rejects invalid type graphs and by-value cycles; caches failed queries too.
    pub fn demand<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        ty: TypeId,
    ) -> Result<Properties, &'static str> {
        match self.states.get(ty.0 as usize) {
            Some(State::Ready(value)) => return Ok(*value),
            Some(State::Failed(message)) => return Err(message),
            _ => {}
        }
        self.states.resize(types.len(), State::Unseen);
        self.work.clear();
        self.work.push((ty, false));
        let result = self.run(types, definitions, selected);
        if let Err(message) = result {
            // A failed dependency also fails every suspended parent, without
            // leaving active entries that poison independent later queries.
            for state in &mut self.states {
                if matches!(state, State::Active) {
                    *state = State::Failed(message);
                }
            }
            self.work.clear();
            return Err(message);
        }
        self.get(ty)
    }
    fn get(&self, ty: TypeId) -> Result<Properties, &'static str> {
        match self.states.get(ty.0 as usize) {
            Some(State::Ready(value)) => Ok(*value),
            Some(State::Failed(message)) => Err(message),
            _ => Err("missing completed Copy obligation"),
        }
    }
    fn run<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
    ) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            let state = self.states.get(ty.0 as usize).ok_or("invalid Copy type")?;
            match state {
                State::Ready(_) => continue,
                State::Failed(message) => return Err(message),
                State::Active if !finish => return Err("recursive by-value Copy obligation"),
                _ => {}
            }
            let kind = types.kind(ty).ok_or("invalid Copy type")?;
            let initial = match kind {
                Kind::Adt(item, _) => selected.get(&item).copied().unwrap_or_default(),
                Kind::Primitive(Primitive::Str)
                | Kind::Slice(_)
                | Kind::Reference { mutable: true, .. } => Properties::default(),
                _ => Properties::BOTH,
            };
            let mut result = initial;
            let packed = matches!(kind, Kind::Adt(item, _)
                if definitions.get(item).is_some_and(|d| d.repr.packed.is_some()));
            if !finish {
                self.states[ty.0 as usize] = State::Active;
                self.work.push((ty, true));
                self.steps += 1;
            }
            if initial != Properties::default() {
                children(types, definitions, kind, |child| {
                    if finish {
                        let properties = self.get(child)?;
                        result = result.intersect(properties);
                        if packed {
                            result.clone_ &= properties.copy;
                        }
                    } else {
                        self.work.push((child, false));
                    }
                    Ok(())
                })?;
            }
            if finish {
                self.states[ty.0 as usize] = State::Ready(result);
            }
        }
        Ok(())
    }
}
fn children(
    types: &Store,
    definitions: &Definitions,
    kind: Kind,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Array { element, .. } => visit(element)?,
        Kind::Tuple(list) => {
            for ty in types.list(list).ok_or("invalid Copy tuple")? {
                visit(*ty)?;
            }
        }
        Kind::Adt(item, _) => {
            let definition = definitions.get(item).ok_or("missing Copy aggregate")?;
            for variant in definitions.variants(definition) {
                for ty in types.list(variant.fields).ok_or("invalid Copy fields")? {
                    visit(*ty)?;
                }
            }
        }
        // Reference and pointer abilities do not depend on their referent.
        _ => {}
    }
    Ok(())
}
