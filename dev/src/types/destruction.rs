//! Destruction requirements for the closed, lifetime-parameterized type vocabulary.
pub mod regions;
use super::{ItemId, Kind, Store, TypeId, aggregate::Definitions};
use std::collections::HashSet;

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(bool),
    Failed(&'static str),
}
/// One canonical store, completed nominal definitions and immutable custom Drop set.
/// Lifetime arguments do not change whether this closed profile needs destruction.
pub struct NeedsDrop {
    custom: HashSet<ItemId>,
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl NeedsDrop {
    #[must_use]
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Self {
        Self {
            custom: custom.into_iter().collect(),
            states: Vec::new(),
            work: Vec::new(),
            steps: 0,
        }
    }
    /// # Errors
    /// Rejects invalid identities, by-value cycles and exhausted query work.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        if matches!(
            self.states.get(ty.0 as usize),
            Some(State::Ready(_) | State::Failed(_))
        ) {
            return self.get(ty);
        }
        types.kind(ty).ok_or("invalid destruction type")?;
        self.states.resize(types.len(), State::Unseen);
        self.work.push((ty, false));
        if let Err(message) = self.run(types, definitions) {
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
    fn get(&self, ty: TypeId) -> Result<bool, &'static str> {
        match self.states.get(ty.0 as usize) {
            Some(State::Ready(value)) => Ok(*value),
            Some(State::Failed(message)) => Err(message),
            _ => Err("missing destruction requirement"),
        }
    }
    fn charge(&mut self) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(1)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destruction query work budget exceeded")?;
        Ok(())
    }
    fn run(&mut self, types: &Store, definitions: &Definitions) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            self.charge()?;
            match self
                .states
                .get(ty.0 as usize)
                .ok_or("invalid destruction type")?
            {
                State::Ready(_) => continue,
                State::Failed(message) => return Err(message),
                State::Active if !finish => {
                    return Err("recursive by-value destruction requirement");
                }
                _ => {}
            }
            let kind = types.kind(ty).ok_or("invalid destruction type")?;
            if let Kind::Adt(item, _) = kind {
                definitions
                    .get(item)
                    .ok_or("missing destruction aggregate")?;
                if self.custom.contains(&item) {
                    self.states[ty.0 as usize] = State::Ready(true);
                    continue;
                }
            }
            let mut needed = false;
            if !finish {
                self.states[ty.0 as usize] = State::Active;
                self.work.push((ty, true));
            }
            children(types, definitions, kind, |child| {
                self.charge()?;
                if finish {
                    needed |= self.get(child)?;
                } else {
                    self.work.push((child, false));
                }
                Ok(())
            })?;
            if finish {
                self.states[ty.0 as usize] = State::Ready(needed);
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
        Kind::Array { length: 0, .. } => {}
        Kind::Array { element, .. } | Kind::Slice(element) => visit(element)?,
        Kind::Tuple(list) => {
            for &ty in types.list(list).ok_or("invalid destruction tuple")? {
                visit(ty)?;
            }
        }
        Kind::Adt(item, _) => {
            let definition = definitions
                .get(item)
                .ok_or("missing destruction aggregate")?;
            for variant in definitions.variants(definition) {
                for &ty in types
                    .list(variant.fields)
                    .ok_or("invalid destruction fields")?
                {
                    visit(ty)?;
                }
            }
        }
        // References and raw pointers do not own their referents.
        _ => {}
    }
    Ok(())
}
