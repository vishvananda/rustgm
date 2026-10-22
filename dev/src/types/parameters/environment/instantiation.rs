//! Builtin instantiation recipes preserve slot correlations without copying prefixes.
use crate::support::hash::HashMap;
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
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.run(types, actual, pattern, surrounding, equalities);
        if let Err(error) = result {
            self.failure = Some(error);
            self.work.clear();
            self.pending.clear();
        }
        result
    }
    fn run(
        &mut self,
        types: &Store,
        actual: TypeId,
        pattern: Pattern,
        surrounding: u16,
        equalities: Option<&RegionEqualities>,
    ) -> Result<bool, Error> {
        let root = Key {
            inputs: Inputs::Types(actual, pattern.ty),
            nested: 0,
            surrounding,
            quantified: pattern.quantified,
        };
        self.work.push((root, false));
        while let Some((key, finish)) = self.work.pop() {
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
                        Some(right) => self.slots.union(left, *right)?,
                        None => None,
                    };
                }
                self.completed.insert(key, result);
            } else {
                self.start(types, key, equalities)?;
            }
        }
        self.completed
            .get(&root)
            .map(Option::is_some)
            .ok_or(Error::Identity)
    }
    fn start(
        &mut self,
        types: &Store,
        key: Key,
        equalities: Option<&RegionEqualities>,
    ) -> Result<(), Error> {
        if let Inputs::Types(actual, pattern) = key.inputs
            && actual == pattern
            && types.escaping_binders(actual).ok_or(Error::Identity)? <= key.nested
        {
            self.completed.insert(key, Some(Id::EMPTY));
            return Ok(());
        }
        let mut frame = Frame {
            types,
            key,
            node: Node::default(),
            slots: &mut self.slots,
            steps: &mut self.steps,
            equalities,
        };
        let matched = match key.inputs {
            Inputs::Types(a, b) => frame.compare(a, b, key.nested)?,
            Inputs::Lists(a, b) => frame.compare_list(a, b, key.nested)?,
            Inputs::Arguments(a, b) => frame.compare_arguments(a, b, key.nested)?,
        };
        if matched {
            self.work.push((key, true));
            self.work
                .extend(frame.node.children.iter().rev().map(|&key| (key, false)));
            self.pending.insert(key, frame.node);
        } else {
            self.completed.insert(key, None);
        }
        Ok(())
    }
}
struct Frame<'a, 't> {
    types: &'t Store,
    key: Key,
    node: Node,
    slots: &'a mut Map<RegionId>,
    steps: &'a mut usize,
    equalities: Option<&'a RegionEqualities>,
}
impl Frame<'_, '_> {
    fn canonical(&self, region: RegionId, depth: u16) -> Result<RegionId, Error> {
        self.equalities
            .map_or(Ok(region), |e| e.canonical(region, depth))
    }
    fn enqueue(&mut self, inputs: Inputs, nested: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.node.children.push(Key {
            inputs,
            nested,
            ..self.key
        });
        Ok(())
    }
}
impl<'t> Compare<'t> for Frame<'_, 't> {
    fn types(&self) -> &'t Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
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
            let leaf = self.slots.singleton(u32::from(slot), actual)?;
            let Some(slots) = self.slots.union(self.node.slots, leaf)? else {
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
