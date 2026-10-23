//! One lazy typed source tree describes every original proof component.
use crate::{
    body::ir::{BodyKey, NodeKey},
    resolve::model::DefId,
    sema::{model::Declarations, nested_outputs::Link},
    types::closure::ClosureId,
};
use std::{collections::HashMap, sync::Arc};
const LIMIT: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy)]
pub(super) enum Owner {
    Function(DefId),
    Initializer(DefId),
    Closure(ClosureId),
    Anonymous,
}
pub(super) struct Node {
    pub source: NodeKey,
    pub owner: Owner,
    pub declaration: Option<DefId>,
    pub link: Option<Arc<Link>>,
    header: Option<crate::types::TypeId>,
    universals: u32,
    parent: Option<usize>,
    children: Vec<usize>,
}
pub(super) struct Index {
    nodes: Vec<Node>,
    sources: HashMap<NodeKey, usize>,
    pub bytes: usize,
}
impl Index {
    pub fn new(
        facts: &Declarations,
        available: usize,
        work: &mut usize,
    ) -> Result<Self, &'static str> {
        let inline_count = facts
            .inline_constants
            .as_deref()
            .map_or(0, crate::sema::inline_constants::Registry::len);
        let count = facts
            .functions
            .len()
            .saturating_add(facts.initializers.len())
            .saturating_add(facts.closures.all().len())
            .saturating_add(inline_count);
        let bytes = size_of::<Self>()
            .saturating_add(crate::support::retained::new_map::<NodeKey, usize>(count))
            .saturating_add(count.saturating_mul(size_of::<Node>() + 6 * size_of::<usize>()));
        if count > LIMIT || bytes > available.min(BYTES) {
            return Err("original component index exceeds storage budget");
        }
        let mut index = Self {
            nodes: Vec::with_capacity(count),
            sources: HashMap::with_capacity(count),
            bytes,
        };
        for function in &facts.functions {
            visit(work)?;
            if let Some(source) = function.body {
                index.add(
                    facts,
                    source,
                    Owner::Function(function.definition),
                    Some(function.definition),
                )?;
            }
        }
        for initializer in &facts.initializers {
            visit(work)?;
            index.add(
                facts,
                initializer.semantic_source(),
                Owner::Initializer(initializer.definition),
                Some(initializer.definition),
            )?;
        }
        for closure in facts.closures.all() {
            visit(work)?;
            index.add(
                facts,
                closure.semantic_source(),
                Owner::Closure(closure.identity),
                closure.owner,
            )?;
        }
        for slot in 0..inline_count {
            visit(work)?;
            let inline = facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.at(slot))
                .ok_or("missing indexed inline original")?;
            index.add(facts, inline.source, Owner::Anonymous, inline.declaration)?;
        }
        index.connect(work)?;
        Ok(index)
    }
    fn add(
        &mut self,
        facts: &Declarations,
        source: NodeKey,
        owner: Owner,
        declaration: Option<DefId>,
    ) -> Result<(), &'static str> {
        if self.sources.insert(source, self.nodes.len()).is_some() {
            return Err("original component has duplicate semantic owners");
        }
        let (header, universals) = match owner {
            Owner::Function(id) => (
                facts
                    .function_indices
                    .get(&id)
                    .and_then(|&i| facts.functions.get(i))
                    .map(|f| f.signature),
                0,
            ),
            Owner::Initializer(id) => (
                facts
                    .initializer_index(id)
                    .and_then(|i| facts.initializers.get(i))
                    .map(|i| i.ty),
                0,
            ),
            Owner::Closure(id) => (facts.closures.identity(id).map(|c| c.signature), 0),
            Owner::Anonymous => facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .map_or((None, 0), |i| (i.result, i.universals)),
        };
        self.nodes.push(Node {
            source,
            owner,
            declaration,
            header,
            universals,
            link: facts
                .nested_outputs
                .as_deref()
                .and_then(|r| r.get(&source))
                .cloned(),
            parent: None,
            children: Vec::new(),
        });
        Ok(())
    }
    fn connect(&mut self, work: &mut usize) -> Result<(), &'static str> {
        for child in 0..self.nodes.len() {
            visit(work)?;
            let Some(link) = &self.nodes[child].link else {
                continue;
            };
            let Some(&parent) = self.sources.get(
                &link
                    .parent
                    .source()
                    .ok_or("source component has an imported parent link")?,
            ) else {
                continue;
            };
            self.nodes[child].parent = Some(parent);
            self.nodes[parent].children.push(child);
        }
        Ok(())
    }
    pub fn component(
        &self,
        facts: &Declarations,
        source: NodeKey,
        work: &mut usize,
    ) -> Result<Vec<usize>, &'static str> {
        let mut root = *self
            .sources
            .get(&source)
            .ok_or("anonymous original is outside its component index")?;
        let mut depth = 0;
        if self.nodes[root].link.is_some() && self.nodes[root].parent.is_none() {
            return Err("original output component is missing its typed parent");
        }
        while let Some(parent) = self.nodes[root].parent {
            visit(work)?;
            depth += 1;
            if depth > self.nodes.len() {
                return Err("cyclic original output component");
            }
            root = parent;
        }
        let mut members = vec![root];
        let mut cursor = 0;
        while cursor < members.len() {
            visit(work)?;
            let node = &self.nodes[members[cursor]];
            self.validate(facts, node)?;
            if members.len().saturating_add(node.children.len()) > self.nodes.len() {
                return Err("cyclic original output component");
            }
            members.extend_from_slice(&node.children);
            cursor += 1;
        }
        Ok(members)
    }
    pub fn node(&self, index: usize) -> Result<&Node, &'static str> {
        self.nodes
            .get(index)
            .ok_or("invalid original component member")
    }
    fn validate(&self, facts: &Declarations, node: &Node) -> Result<(), &'static str> {
        let current = facts
            .nested_outputs
            .as_deref()
            .and_then(|r| r.get(&node.source));
        if match (&node.link, current) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        } {
            return Err("original component output ownership changed");
        }
        if let Some(link) = &node.link {
            let parent = node
                .parent
                .and_then(|p| self.nodes.get(p))
                .ok_or("original output component is missing its typed parent")?;
            if link.source != node.source.into()
                || crate::body::ir::BodyOrigin::from(parent.source) != link.parent
                || parent.source == node.source
                || parent.declaration != node.declaration
            {
                return Err("original output component has a foreign parent owner");
            }
        }
        let valid = match node.owner {
            Owner::Function(definition) => facts
                .function_indices
                .get(&definition)
                .and_then(|&i| facts.functions.get(i))
                .is_some_and(|f| {
                    f.definition == definition
                        && f.body == Some(node.source)
                        && Some(f.signature) == node.header
                }),
            Owner::Initializer(definition) => facts
                .initializer_index(definition)
                .and_then(|i| facts.initializers.get(i))
                .is_some_and(|i| i.semantic_source() == node.source && Some(i.ty) == node.header),
            Owner::Closure(identity) => facts.closures.identity(identity).is_some_and(|c| {
                c.semantic_source() == node.source
                    && c.owner == node.declaration
                    && Some(c.signature) == node.header
            }),
            Owner::Anonymous => facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(node.source))
                .is_some_and(|i| {
                    i.declaration == node.declaration
                        && i.result.is_some()
                        && i.result == node.header
                        && i.universals == node.universals
                        && node
                            .link
                            .as_ref()
                            .is_none_or(|link| link.parent == i.parent_body.into())
                }),
        };
        if !valid {
            return Err("original component changed its typed source owner");
        }
        Ok(())
    }
}
fn visit(work: &mut usize) -> Result<(), &'static str> {
    *work = work.saturating_add(1);
    if *work > 4_000_000 {
        return Err("original component index work budget exceeded");
    }
    Ok(())
}
impl Node {
    pub fn key(&self) -> BodyKey {
        match self.owner {
            Owner::Function(id) | Owner::Initializer(id) => BodyKey::Declaration(id),
            Owner::Closure(id) => BodyKey::Closure(id),
            Owner::Anonymous => BodyKey::Anonymous(self.source),
        }
    }
}
