//! Builtin instantiation recipes preserve slot correlations without copying prefixes.
use crate::support::hash::HashMap;
use crate::support::retained::Growth;
use crate::{
    support::radix_map::{Id, Map},
    types::{
        GenericArgs, ListId, RegionId, Store, TypeId, region_graph::Error,
        relations::RegionEqualities, structural::Compare,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Pattern {
    pub ty: TypeId,
    pub quantified: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Inputs {
    Types(TypeId, TypeId),
    Lists(ListId, ListId),
    Arguments(GenericArgs, GenericArgs),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    inputs: Inputs,
    nested: u16,
    surrounding: u16,
    quantified: bool,
}
#[derive(Default)]
struct Node {
    slots: Id,
    children: Vec<Key>,
}
#[derive(Default)]
pub(super) struct Matches {
    failure: Option<Error>,
    completed: HashMap<Key, Option<Id>>,
    pending: HashMap<Key, Node>,
    work: Vec<(Key, bool)>,
    slots: Map<RegionId>,
    steps: usize,
}
impl Matches {
    pub fn steps(&self) -> usize {
        self.steps.saturating_add(self.slots.steps)
    }
    pub fn accepts(
        &mut self,
        types: &Store,
        actual: TypeId,
        pattern: Pattern,
        surrounding: u16,
        equalities: Option<&RegionEqualities>,
    ) -> Result<bool, Error> {
        self.accepts_observed::<false>(types, actual, pattern, surrounding, equalities, &|_, _| {
            Ok(())
        })
    }
    pub(super) fn accepts_charged(
        &mut self,
        types: &Store,
        actual: TypeId,
        pattern: Pattern,
        surrounding: u16,
        equalities: Option<&RegionEqualities>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        let failure = std::cell::Cell::new(None);
        let pay = |work, bytes| {
            charge(work, bytes).map_err(|message| {
                failure.set(Some(message));
                Error::Budget
            })
        };
        self.accepts_observed::<true>(types, actual, pattern, surrounding, equalities, &pay)
            .map_err(|error| {
                failure.get().unwrap_or(match error {
                    Error::Budget => "builtin assumption matching work budget exceeded",
                    _ => "invalid builtin assumption matching input",
                })
            })
    }
    fn accepts_observed<const CHARGED: bool>(
        &mut self,
        types: &Store,
        actual: TypeId,
        pattern: Pattern,
        surrounding: u16,
        equalities: Option<&RegionEqualities>,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<bool, Error> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.run::<CHARGED>(types, actual, pattern, surrounding, equalities, pay);
        if let Err(error) = result {
            self.failure = Some(error);
            self.work.clear();
            self.pending.clear();
        }
        result
    }
    fn run<const CHARGED: bool>(
        &mut self,
        types: &Store,
        actual: TypeId,
        pattern: Pattern,
        surrounding: u16,
        equalities: Option<&RegionEqualities>,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<bool, Error> {
        let root = Key {
            inputs: Inputs::Types(actual, pattern.ty),
            nested: 0,
            surrounding,
            quantified: pattern.quantified,
        };
        self.schedule::<CHARGED>(&[(root, false)], pay)?;
        while let Some((key, finish)) = self.work.pop() {
            if CHARGED {
                pay(1, 0)?;
            }
            self.steps = self.steps.saturating_add(1);
            if self.steps().saturating_add(self.work.len()) > 1_000_000 {
                return Err(Error::Budget);
            }
            if self.completed.contains_key(&key) {
                continue;
            }
            if finish {
                let node = self.pending.remove(&key).ok_or(Error::Identity)?;
                let mut result = Some(node.slots);
                for child in node.children {
                    let Some(left) = result else {
                        break;
                    };
                    result = match self.completed.get(&child).ok_or(Error::Identity)? {
                        Some(right) => {
                            if CHARGED {
                                self.slots.union_charged(left, *right, &|work, bytes| {
                                    pay(work, bytes)
                                        .map_err(|_| crate::support::radix_map::Error::Budget)
                                })?
                            } else {
                                self.slots.union(left, *right)?
                            }
                        }
                        None => None,
                    };
                }
                self.complete::<CHARGED>(key, result, pay)?;
            } else {
                self.start::<CHARGED>(types, key, equalities, pay)?;
            }
        }
        self.completed
            .get(&root)
            .map(Option::is_some)
            .ok_or(Error::Identity)
    }
    fn start<const CHARGED: bool>(
        &mut self,
        types: &Store,
        key: Key,
        equalities: Option<&RegionEqualities>,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Inputs::Types(actual, pattern) = key.inputs
            && actual == pattern
            && types.escaping_binders(actual).ok_or(Error::Identity)? <= key.nested
        {
            self.complete::<CHARGED>(key, Some(Id::EMPTY), pay)?;
            return Ok(());
        }
        let mut frame = Frame::<CHARGED> {
            types,
            key,
            node: Node::default(),
            slots: &mut self.slots,
            steps: &mut self.steps,
            equalities,
            pay,
        };
        let matched = match key.inputs {
            Inputs::Types(a, b) => frame.compare(a, b, key.nested)?,
            Inputs::Lists(a, b) => frame.compare_list(a, b, key.nested)?,
            Inputs::Arguments(a, b) => frame.compare_arguments(a, b, key.nested)?,
        };
        if matched {
            if CHARGED {
                let mut growth = Growth::vector::<(Key, bool)>(
                    self.work.len(),
                    self.work.capacity(),
                    1 + frame.node.children.len(),
                );
                growth.include(Growth::map::<Key, Node>(
                    self.pending.len(),
                    self.pending.capacity(),
                    1,
                ));
                pay(1 + frame.node.children.len() + growth.work, growth.bytes)?;
            }
            self.work.push((key, true));
            self.work
                .extend(frame.node.children.iter().rev().map(|&key| (key, false)));
            self.pending.insert(key, frame.node);
        } else {
            self.complete::<CHARGED>(key, None, pay)?;
        }
        Ok(())
    }
    fn schedule<const CHARGED: bool>(
        &mut self,
        values: &[(Key, bool)],
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if CHARGED {
            let growth =
                Growth::vector::<(Key, bool)>(self.work.len(), self.work.capacity(), values.len());
            pay(values.len() + growth.work, growth.bytes)?;
        }
        self.work.extend_from_slice(values);
        Ok(())
    }
    fn complete<const CHARGED: bool>(
        &mut self,
        key: Key,
        value: Option<Id>,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if CHARGED {
            let growth =
                Growth::map::<Key, Option<Id>>(self.completed.len(), self.completed.capacity(), 1);
            pay(1 + growth.work, growth.bytes)?;
        }
        self.completed.insert(key, value);
        Ok(())
    }
}
struct Frame<'a, 't, const CHARGED: bool> {
    types: &'t Store,
    key: Key,
    node: Node,
    slots: &'a mut Map<RegionId>,
    steps: &'a mut usize,
    equalities: Option<&'a RegionEqualities>,
    pay: &'a dyn Fn(usize, usize) -> Result<(), Error>,
}
impl<const CHARGED: bool> Frame<'_, '_, CHARGED> {
    fn canonical(&self, region: RegionId, depth: u16) -> Result<RegionId, Error> {
        self.equalities
            .map_or(Ok(region), |e| e.canonical(region, depth))
    }
    fn enqueue(&mut self, inputs: Inputs, nested: u16) -> Result<(), Error> {
        self.charge(1)?;
        if CHARGED {
            let growth =
                Growth::vector::<Key>(self.node.children.len(), self.node.children.capacity(), 1);
            (self.pay)(growth.work, growth.bytes)?;
        }
        self.node.children.push(Key {
            inputs,
            nested,
            ..self.key
        });
        Ok(())
    }
}
impl<'t, const CHARGED: bool> Compare<'t> for Frame<'_, 't, CHARGED> {
    fn types(&self) -> &'t Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        if CHARGED {
            (self.pay)(count, 0)?;
        }
        *self.steps = self.steps.saturating_add(count);
        if *self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, actual: TypeId, pattern: TypeId, nested: u16) -> Result<(), Error> {
        self.enqueue(Inputs::Types(actual, pattern), nested)
    }
    fn list(&mut self, a: ListId, b: ListId, nested: u16) -> Result<bool, Error> {
        self.enqueue(Inputs::Lists(a, b), nested)?;
        Ok(true)
    }
    fn arguments(&mut self, a: GenericArgs, b: GenericArgs, nested: u16) -> Result<bool, Error> {
        self.enqueue(Inputs::Arguments(a, b), nested)?;
        Ok(true)
    }
    fn regions(&mut self, actual: RegionId, pattern: RegionId, nested: u16) -> Result<bool, Error> {
        let local = |region: RegionId| {
            region
                .bound_parts()
                .is_some_and(|(binder, _)| binder < nested)
        };
        if let Some((binder, slot)) = pattern.bound_parts()
            && self.key.quantified
            && binder == nested
        {
            // A premise substitution cannot capture a lifetime introduced by a
            // nested callable. Query-surrounding universals are valid arguments.
            if local(actual) {
                return Ok(false);
            }
            let actual = match actual.bound_parts() {
                Some((binder, slot)) => {
                    RegionId::bound(binder - nested, slot).map_err(|_| Error::Identity)?
                }
                None => actual,
            };
            let actual = self.canonical(actual, self.key.surrounding)?;
            let pay = |work, bytes| {
                (self.pay)(work, bytes).map_err(|_| crate::support::radix_map::Error::Budget)
            };
            let leaf = if CHARGED {
                self.slots
                    .singleton_charged(u32::from(slot), actual, &pay)?
            } else {
                self.slots.singleton(u32::from(slot), actual)?
            };
            let slots = if CHARGED {
                self.slots.union_charged(self.node.slots, leaf, &pay)?
            } else {
                self.slots.union(self.node.slots, leaf)?
            };
            let Some(slots) = slots else {
                return Ok(false);
            };
            self.node.slots = slots;
            return Ok(true);
        }
        if local(actual) || local(pattern) {
            return Ok(actual == pattern);
        }
        let expected = match pattern.bound_parts() {
            Some((binder, slot)) => RegionId::bound(
                (binder - u16::from(self.key.quantified))
                    .checked_add(self.key.surrounding)
                    .ok_or(Error::Budget)?,
                slot,
            )
            .map_err(|_| Error::Identity)?,
            None => pattern,
        };
        // No unproved lifetime equality can authorize a copy or omit destruction.
        let depth = nested
            .checked_add(self.key.surrounding)
            .ok_or(Error::Budget)?;
        Ok(self.canonical(actual, depth)? == self.canonical(expected, depth)?)
    }
}
