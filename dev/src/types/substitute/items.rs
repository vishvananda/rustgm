//! Replace selected callable identities throughout a type DAG without capturing binders.
#[cfg(test)]
mod tests;
use super::{Action, Bound, Key};
use crate::types::{Kind, Store, TypeId, bound_uses::captures::Captures};
use std::collections::HashMap;

pub(super) struct Replacements {
    values: HashMap<TypeId, TypeId>,
    scopes: HashMap<u16, Scope>,
    recursive: bool,
}
struct Scope {
    captures: Captures,
    lower: Bound,
    raise: Bound,
}
impl Bound {
    /// Replace only defining-result identities while constructing their obligations.
    /// This does not publish representation or permit caller-side normalization.
    pub(crate) fn defining_results(
        store: &Store,
        values: HashMap<TypeId, TypeId>,
    ) -> Result<Self, &'static str> {
        if values.iter().any(|(&source, &target)| {
            source == target
                || !matches!(store.kind(source), Some(Kind::Opaque(..)))
                || store.kind(target).is_none()
        }) {
            return Err("invalid defining result replacement");
        }
        Ok(Self::replacements(values, false))
    }

    pub(crate) fn selected_items(
        store: &Store,
        values: HashMap<TypeId, TypeId>,
    ) -> Result<Self, &'static str> {
        for (&source, &target) in &values {
            if source == target
                || ![source, target]
                    .iter()
                    .all(|&ty| matches!(store.kind(ty), Some(Kind::FunctionItem { .. })))
            {
                return Err("invalid selected callable replacement");
            }
        }
        Ok(Self::replacements(values, true))
    }
    fn replacements(values: HashMap<TypeId, TypeId>, recursive: bool) -> Self {
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Items(Box::new(Replacements {
            values,
            scopes: HashMap::new(),
            recursive,
        }));
        fold
    }
    pub(super) fn replace_item(
        &mut self,
        store: &mut Store,
        key: Key,
        finish: bool,
    ) -> Result<bool, &'static str> {
        let Action::Items(items) = &mut self.action else {
            return Ok(false);
        };
        if !matches!(
            store.kind(key.0),
            Some(Kind::FunctionItem { .. } | Kind::Opaque(..))
        ) {
            return Ok(false);
        }
        let (replacement, steps) = items.lookup(store, key)?;
        let recursive = items.recursive;
        self.account_arguments(steps)?;
        let Some(replacement) = replacement else {
            return Ok(false);
        };
        if !recursive {
            self.completed.insert(key, replacement);
            return Ok(true);
        }
        let child = (replacement, key.1);
        if let Some(result) = self.result(store, child)? {
            self.completed.insert(key, result);
        } else {
            if finish {
                return Err("missing selected callable dependency");
            }
            self.work.push((key, true));
            self.queue(child.0, child.1)?;
        }
        Ok(true)
    }
}
impl Replacements {
    fn lookup(
        &mut self,
        store: &mut Store,
        (ty, depth): Key,
    ) -> Result<(Option<TypeId>, usize), &'static str> {
        if depth == 0 {
            return Ok((self.values.get(&ty).copied(), 0));
        }
        let scope = self.scopes.entry(depth).or_insert_with(|| Scope {
            captures: Captures::new(depth),
            lower: Bound::lower(depth),
            raise: Bound::shift(depth),
        });
        let before = scope.captures.steps + scope.lower.steps + scope.raise.steps;
        let result = if scope.captures.collect(store, ty)? {
            None
        } else {
            let original = scope.lower.apply(store, ty)?;
            self.values
                .get(&original)
                .map(|&replacement| scope.raise.apply(store, replacement))
                .transpose()?
        };
        let after = scope.captures.steps + scope.lower.steps + scope.raise.steps;
        Ok((result, after - before))
    }
}
