//! Recursive drop requirements use the least closure of the reached field equations.
use super::{
    Arc, Definitions, HashMap, HashSet, ItemId, Key, Parts, Regions, State, Store, Values,
    dependencies,
};

const BYTE_LIMIT: usize = 64_000_000;

struct Parent {
    index: usize,
    indirect: bool,
}

struct Node {
    key: Key,
    parents: Vec<Parent>,
    owned_children: usize,
    values: Values,
    queued: bool,
}

#[derive(Default)]
pub(super) struct Graph {
    index: HashMap<Key, usize>,
    nodes: Vec<Node>,
    scratch_bytes: usize,
}

impl Graph {
    pub(super) fn parts(&self, key: Key) -> Option<Parts<'_>> {
        let index = *self.index.get(&key)?;
        self.nodes.get(index).map(|node| node.values.parts())
    }

    fn reserve(&mut self, retained: usize, bytes: usize) -> Result<(), &'static str> {
        self.scratch_bytes = self.scratch_bytes.saturating_add(bytes);
        if retained.saturating_add(self.scratch_bytes) > BYTE_LIMIT {
            return Err("recursive drop region equations exceed 64 MB budget");
        }
        Ok(())
    }

    fn insert(&mut self, retained: usize, key: Key) -> Result<usize, &'static str> {
        if let Some(&index) = self.index.get(&key) {
            return Ok(index);
        }
        self.reserve(
            retained,
            2 * (size_of::<Node>() + size_of::<(Key, usize)>()),
        )?;
        let index = self.nodes.len();
        self.nodes.push(Node {
            key,
            parents: Vec::new(),
            owned_children: 0,
            values: Values::default(),
            queued: false,
        });
        self.index.insert(key, index);
        Ok(index)
    }

    fn discover(
        &mut self,
        query: &mut Regions,
        types: &Store,
        definitions: &Definitions,
        root: Key,
    ) -> Result<(), &'static str> {
        self.insert(query.retained, root)?;
        let custom = Arc::clone(&query.custom);
        let mut owner = 0;
        while let Some(key) = self.nodes.get(owner).map(|node| node.key) {
            query.charge(1)?;
            if let Some(mut values) = query.terminal(types, definitions, key)? {
                query.retain(2 * size_of::<(Key, State)>())?;
                query.publish(key, &mut values)?;
            } else {
                self.dependencies(query, types, definitions, &custom, owner)?;
            }
            owner += 1;
        }
        Ok(())
    }

    fn dependencies(
        &mut self,
        query: &mut Regions,
        types: &Store,
        definitions: &Definitions,
        custom: &HashSet<ItemId>,
        owner: usize,
    ) -> Result<(), &'static str> {
        dependencies(
            types,
            definitions,
            custom,
            self.nodes[owner].key,
            |child, indirect| {
                query.charge(1)?;
                if let Some(state) = query.states.get(&child) {
                    match state {
                        State::Ready(_) => return Ok(()),
                        State::Failed(message) => return Err(message),
                        State::Active => {
                            return Err("recursive drop region query retained an active visit");
                        }
                    }
                }
                let child = self.insert(query.retained, child)?;
                self.reserve(query.retained, 2 * size_of::<Parent>())?;
                self.nodes[child].parents.push(Parent {
                    index: owner,
                    indirect,
                });
                self.nodes[owner].owned_children += usize::from(!indirect);
                Ok(())
            },
        )
    }

    /// Direct storage still has to be finite. The graph may only be recursive
    /// through a checked owning pointer or a zero-sized logical field.
    fn check_owned_edges(&mut self, query: &mut Regions) -> Result<(), &'static str> {
        self.reserve(query.retained, 2 * self.nodes.len() * size_of::<usize>())?;
        let mut ready = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.owned_children == 0).then_some(index))
            .collect::<Vec<_>>();
        let mut done = 0;
        while let Some(index) = ready.pop() {
            done += 1;
            for edge in 0..self.nodes[index].parents.len() {
                let parent = &self.nodes[index].parents[edge];
                query.charge(1)?;
                if !parent.indirect {
                    let owner = parent.index;
                    self.nodes[owner].owned_children -= 1;
                    if self.nodes[owner].owned_children == 0 {
                        ready.push(owner);
                    }
                }
            }
        }
        if done != self.nodes.len() {
            return Err("recursive owned drop region requirement");
        }
        Ok(())
    }

    fn solve(
        &mut self,
        query: &mut Regions,
        types: &Store,
        definitions: &Definitions,
    ) -> Result<(), &'static str> {
        let mut pending = Vec::with_capacity(self.nodes.len());
        for (index, node) in self.nodes.iter_mut().enumerate().rev() {
            if !query.states.contains_key(&node.key) {
                node.queued = true;
                pending.push(index);
            }
        }
        while let Some(index) = pending.pop() {
            query.charge(1)?;
            let node = &mut self.nodes[index];
            node.queued = false;
            let key = node.key;
            let mut next = query.finish(types, definitions, key, Some(self))?;
            query.charge(next.parts().len())?;
            next.canonicalize();
            let node = &self.nodes[index];
            let old = node.values.parts();
            let new = next.parts();
            if old.regions == new.regions && old.parameters == new.parameters {
                continue;
            }
            self.reserve_values(index, query.retained, &next)?;
            self.nodes[index].values = next;
            for edge in 0..self.nodes[index].parents.len() {
                let owner = self.nodes[index].parents[edge].index;
                query.charge(1)?;
                if !self.nodes[owner].queued {
                    self.nodes[owner].queued = true;
                    pending.push(owner);
                }
            }
        }
        Ok(())
    }

    fn reserve_values(
        &mut self,
        index: usize,
        retained: usize,
        values: &Values,
    ) -> Result<(), &'static str> {
        fn bytes(values: &Values) -> usize {
            values.regions.capacity() * size_of::<crate::types::RegionId>()
                + values.parameters.capacity() * size_of::<crate::types::parameters::ParameterId>()
        }

        self.reserve(retained, bytes(values))?;
        self.scratch_bytes -= bytes(&self.nodes[index].values);
        Ok(())
    }
}

impl Regions {
    pub(super) fn resolve_cycles(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        root: Key,
    ) -> Result<(), &'static str> {
        let mut graph = Graph::default();
        graph.discover(self, types, definitions, root)?;
        graph.check_owned_edges(self)?;
        graph.solve(self, types, definitions)?;
        for node in &mut graph.nodes {
            if !self.states.contains_key(&node.key) {
                self.retain(2 * size_of::<(Key, State)>())?;
                self.publish(node.key, &mut node.values)?;
            }
        }
        Ok(())
    }
}
