//! Reuse one append-only substitution for unreduced defaults at a type application.
use crate::types::{GenericArgs, ItemId, Store, arguments::Argument, substitute::Bound};

#[derive(Default)]
pub(in crate::sema::declarations) struct Arguments {
    state: Option<Box<State>>,
}
struct State {
    values: Vec<Argument>,
    defaults: Option<Bound>,
}
impl Arguments {
    pub fn apply_default(
        &mut self,
        store: &mut Store,
        owner: ItemId,
        preceding: &[Argument],
        template: Argument,
        normal: Argument,
    ) -> Result<Argument, &'static str> {
        let projected =
            matches!(template, Argument::Type(ty) if store.contains_projections(ty) == Some(true));
        if self.state.is_none() && !projected {
            return Ok(normal);
        }
        let state = self.state.get_or_insert_with(|| {
            Box::new(State {
                values: preceding.to_vec(),
                defaults: None,
            })
        });
        if state.defaults.is_none() {
            let mut fold = Bound::prefix(owner);
            for &argument in &state.values {
                fold.push_argument(store, argument)?;
            }
            state.defaults = Some(fold);
        }
        state
            .defaults
            .as_mut()
            .ok_or("missing unreduced default substitution")?
            .apply_argument(store, template)
    }
    pub fn push(
        &mut self,
        store: &Store,
        preceding: &[Argument],
        normal: Argument,
        original: Argument,
    ) -> Result<(), &'static str> {
        if original != normal && self.state.is_none() {
            self.state = Some(Box::new(State {
                values: preceding.to_vec(),
                defaults: None,
            }));
        }
        if let Some(state) = &mut self.state {
            state.values.push(original);
            if let Some(fold) = &mut state.defaults {
                fold.push_argument(store, original)?;
            }
        }
        Ok(())
    }
    pub fn finish(self, store: &mut Store) -> Result<Option<GenericArgs>, &'static str> {
        self.state
            .map(|state| store.intern_arguments(&state.values))
            .transpose()
    }
}
