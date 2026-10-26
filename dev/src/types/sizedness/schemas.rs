//! Reduce tail conditions once per declaration, composing typed argument substitutions.
use super::{Condition, Rule, Rules};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
    aggregate::{self, Definitions},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Item(ItemId),
    Type(ItemId, TypeId),
}
#[derive(Clone, Copy)]
pub(super) enum Term {
    Sized,
    Assumed,
    Unsized,
    Type(TypeId),
}
#[derive(Clone)]
pub(super) struct Entry {
    pub rule: Rule,
    term: Term,
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Complete(Term),
}
enum Step {
    Complete(Term),
    Child(Key),
    Fields(ItemId),
}
#[derive(Clone, Default)]
pub(super) struct Pending {
    states: HashMap<Key, State>,
    work: Vec<(Key, bool)>,
}
struct Builder<'a, F> {
    rules: &'a mut Rules,
    definitions: &'a Definitions,
    assumed: F,
    states: HashMap<Key, State>,
    work: Vec<(Key, bool)>,
}

impl Rules {
    /// Complete a rule from declaration templates, not expanded storage instances.
    /// `assumed` supplies only the owner's independently formed Sized premises.
    /// A returned item requests its field declarations before resuming this query.
    ///
    /// This query proves no finite storage or field well-formedness. Those gates
    /// remain mandatory, including when a positive recursive condition is reduced.
    /// # Errors
    /// Rejects invalid templates/substitutions, foreign parameters and exhausted work.
    pub fn prepare(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        item: ItemId,
        assumed: impl FnMut(&Store, ItemId, TypeId) -> Result<bool, &'static str>,
    ) -> Result<Option<ItemId>, &'static str> {
        if self.entries.contains_key(&item) {
            self.pending.remove(&item);
            return Ok(None);
        }
        let pending = self.pending.remove(&item).unwrap_or_else(|| Pending {
            states: HashMap::default(),
            work: vec![(Key::Item(item), false)],
        });
        let pending = Builder {
            rules: self,
            definitions,
            assumed,
            states: pending.states,
            work: pending.work,
        }
        .run(types)?;
        if let Some((missing, pending)) = pending {
            self.pending.insert(item, pending);
            Ok(Some(missing))
        } else {
            Ok(None)
        }
    }
}

impl<F> Builder<'_, F>
where
    F: FnMut(&Store, ItemId, TypeId) -> Result<bool, &'static str>,
{
    fn run(mut self, types: &mut Store) -> Result<Option<(ItemId, Pending)>, &'static str> {
        while let Some((key, finish)) = self.work.pop() {
            self.rules.charge(1)?;
            if self.completed(key).is_some() {
                continue;
            }
            if !finish && matches!(self.states.get(&key), Some(State::Active)) {
                // Sized constraints are positive. Infinite by-value storage is
                // rejected separately, before any concrete layout expansion.
                self.complete(types, key, Term::Sized)?;
                continue;
            }
            self.states.insert(key, State::Active);
            match self.inspect(types, key)? {
                Step::Complete(term) => self.complete(types, key, term)?,
                Step::Fields(item) => {
                    self.work.push((key, true));
                    return Ok(Some((
                        item,
                        Pending {
                            states: self.states,
                            work: self.work,
                        },
                    )));
                }
                Step::Child(child) => {
                    if let Some(term) = self.completed(child) {
                        self.complete(types, key, term)?;
                    } else {
                        self.work.push((key, true));
                        self.work.push((child, false));
                    }
                }
            }
        }
        Ok(None)
    }
    fn completed(&self, key: Key) -> Option<Term> {
        if let Key::Type(_, ty) = key
            && let Some(&term) = self.rules.closed.get(&ty)
        {
            return Some(term);
        }
        if let Key::Item(item) = key
            && let Some(entry) = self.rules.entries.get(&item)
        {
            return Some(entry.term);
        }
        match self.states.get(&key) {
            Some(State::Complete(term)) => Some(*term),
            _ => None,
        }
    }
    fn complete(&mut self, types: &Store, key: Key, term: Term) -> Result<(), &'static str> {
        if let Key::Item(item) = key {
            let rule = match term {
                Term::Sized | Term::Assumed => Rule::Sized,
                Term::Unsized => Rule::Unsized,
                Term::Type(ty) => {
                    if let Some(Kind::Parameter(id)) = types.kind(ty) {
                        let parameter = types.parameter(id).ok_or("missing sizedness parameter")?;
                        if parameter.owner != item {
                            return Err("sizedness condition contains a foreign parameter");
                        }
                        Rule::Argument(parameter.index)
                    } else {
                        Rule::Projection(ty)
                    }
                }
            };
            if self
                .rules
                .entries
                .insert(
                    item,
                    Entry {
                        rule,
                        // A nominal's own formation gate proves its declared
                        // premises. A caller still must prove that formation.
                        term: if matches!(term, Term::Assumed) {
                            Term::Sized
                        } else {
                            term
                        },
                    },
                )
                .is_some_and(|old| old.rule != rule)
            {
                return Err("completed nominal sizedness rule changed");
            }
        }
        if let Key::Type(_, ty) = key
            && matches!(term, Term::Sized | Term::Unsized)
            && types.contains_parameters(ty) == Some(false)
            && types.contains_projections(ty) == Some(false)
            && types.contains_opaques(ty) == Some(false)
        {
            // Only physical answers without caller premises can be shared.
            // Unknown and assumed terminals remain in this owner's scratch.
            self.rules.closed.insert(ty, term);
        }
        self.states.insert(key, State::Complete(term));
        Ok(())
    }
    fn inspect(&mut self, types: &mut Store, key: Key) -> Result<Step, &'static str> {
        let (owner, ty) = match key {
            Key::Item(item) => {
                let Some(definition) = self.definitions.get(item) else {
                    return Ok(Step::Fields(item));
                };
                if definition.kind == aggregate::Kind::Enum {
                    return Ok(Step::Complete(Term::Sized));
                }
                let fields = self
                    .definitions
                    .variants(definition)
                    .first()
                    .ok_or("missing sizedness declaration fields")?
                    .fields;
                return Self::last(types, item, fields);
            }
            Key::Type(owner, ty) => (owner, ty),
        };
        if (self.assumed)(types, owner, ty)? {
            return Ok(Step::Complete(Term::Assumed));
        }
        Ok(
            match types.kind(ty).ok_or("invalid sizedness template type")? {
                Kind::Foreign(_)
                | Kind::Slice(_)
                | Kind::Primitive(Primitive::Str)
                | Kind::Dynamic { .. } => Step::Complete(Term::Unsized),
                Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => {
                    Step::Complete(if types.is_sized(ty) == Some(true) {
                        Term::Sized
                    } else {
                        Term::Type(ty)
                    })
                }
                Kind::Tuple(fields) => return Self::last(types, owner, fields),
                Kind::Adt(item, arguments) => {
                    return self.application(types, owner, item, arguments);
                }
                Kind::Never
                | Kind::Primitive(_)
                | Kind::Array { .. }
                | Kind::ArrayTerm { .. }
                | Kind::Reference { .. }
                | Kind::Pattern { .. }
                | Kind::Pointer { .. }
                | Kind::Function { .. }
                | Kind::FunctionItem { .. }
                | Kind::Closure { .. } => Step::Complete(Term::Sized),
            },
        )
    }
    fn last(
        types: &Store,
        owner: ItemId,
        fields: crate::types::ListId,
    ) -> Result<Step, &'static str> {
        Ok(types
            .list(fields)
            .ok_or("invalid sizedness field list")?
            .last()
            .map_or(Step::Complete(Term::Sized), |&ty| {
                Step::Child(Key::Type(owner, ty))
            }))
    }
    fn application(
        &mut self,
        types: &mut Store,
        owner: ItemId,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<Step, &'static str> {
        if self.rules.get(item).is_none() {
            return Ok(Step::Child(Key::Item(item)));
        }
        Ok(match self.rules.apply(types, item, arguments)? {
            Condition::Sized => Step::Complete(Term::Sized),
            Condition::Unsized => Step::Complete(Term::Unsized),
            Condition::Type(ty) => Step::Child(Key::Type(owner, ty)),
        })
    }
}
