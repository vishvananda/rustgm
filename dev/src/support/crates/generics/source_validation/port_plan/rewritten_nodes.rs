//! Pure type structure is shared; no node contains caller proof or permission.
use super::{Context, DomainIndex, Work, g, matching::Pair, s, signatures, uses};
use crate::support::{
    crates::{self, Scalar, owner_catalog::BodyRef},
    hash::{HashMap, HashSet},
    retained,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct NodeId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Region {
    Static,
    Bound {
        binder: g::BinderRef,
        slot: u32,
    },
    Variable {
        body: BodyRef,
        slot: u32,
    },
    /// A template slot of this exact target's public callable. An actual Call
    /// must supply its own checked opening before comparison can resolve it.
    Late {
        binder: g::BinderRef,
        slot: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Node {
    Scalar(Scalar),
    Parameter(s::SlotRef),
    Reference {
        mutable: bool,
        region: Region,
        referent: NodeId,
    },
    FunctionItem(s::FunctionApplicationRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Rewrite {
    Original,
    /// Exact original-to-public relocation, not an inferred matching arity.
    Public(crate::support::crates::owner_catalog::DeclarationRef),
    Application(s::FunctionApplicationRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key {
    pub root: g::TypeRef,
    pub rewrite: Rewrite,
    pub caller: Option<Context>,
}

pub(super) struct Arena {
    pub nodes: Vec<Node>,
    trivial_formation: Vec<bool>,
    pub relocations: Vec<u32>,
    interned: HashMap<Node, NodeId>,
    pub completed: HashMap<Key, NodeId>,
    pub matched: HashSet<Pair>,
    pub templates: HashMap<signatures::Key, signatures::Template>,
    pub signature_roots: Vec<NodeId>,
    pub reserved: usize,
    pub peak: usize,
    pub visits: usize,
    pub hits: usize,
    scratch: usize,
    held: usize,
}
impl Arena {
    pub(super) fn new(
        domains: &DomainIndex<'_, '_>,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let reserved = domains
            .retained_bytes()
            .saturating_add(base)
            .saturating_add(size_of::<Self>());
        if reserved > crates::MAX_BYTES {
            return Err("source input-plan arena exceeds the combined retained budget");
        }
        work.charge(1)?;
        let mut arena = Self {
            nodes: Vec::new(),
            trivial_formation: Vec::new(),
            relocations: Vec::new(),
            interned: HashMap::default(),
            completed: HashMap::default(),
            matched: HashSet::default(),
            templates: HashMap::default(),
            signature_roots: Vec::new(),
            reserved,
            peak: domains.peak_bytes().max(reserved),
            visits: 0,
            hits: 0,
            scratch: 0,
            held: 0,
        };
        uses::relocations(domains, &mut arena, work)?;
        Ok(arena)
    }
    pub(super) fn reserve(
        &mut self,
        bytes: usize,
        scratch: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let reserved = self.reserved.saturating_add(bytes);
        let peak = self.peak.max(
            reserved
                .saturating_add(self.held)
                .saturating_add(self.scratch.max(scratch)),
        );
        if peak > crates::MAX_BYTES {
            return Err("source input-plan growth exceeds the combined retained/scratch budget");
        }
        self.reserved = reserved;
        self.peak = peak;
        Ok(())
    }
    pub(super) fn set_scratch(
        &mut self,
        bytes: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.reserve(0, bytes, work)?;
        self.scratch = bytes;
        Ok(())
    }
    pub(super) fn held_scratch(
        &mut self,
        bytes: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let peak = self
            .reserved
            .saturating_add(bytes)
            .saturating_add(self.scratch);
        work.charge(1)?;
        if peak > crates::MAX_BYTES {
            return Err("source operation scratch exceeds the combined input budget");
        }
        self.held = bytes;
        self.peak = self.peak.max(peak);
        Ok(())
    }
    pub(super) fn intrinsically_formed(
        &self,
        node: NodeId,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        self.trivial_formation
            .get(node.0 as usize)
            .copied()
            .ok_or("source formation node is unknown")
    }
    pub(super) fn intern(&mut self, node: Node, work: &mut Work) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        if let Some(&id) = self.interned.get(&node) {
            self.hits = self.hits.saturating_add(1);
            return Ok(id);
        }
        self.reserve(
            4 * size_of::<Node>() + 8 * size_of::<bool>() + retained::new_map::<Node, NodeId>(1),
            0,
            work,
        )?;
        let id = NodeId(
            u32::try_from(self.nodes.len()).map_err(|_| "source input type-node index overflow")?,
        );
        let formed = match node {
            Node::Scalar(_) | Node::Parameter(_) => true,
            Node::FunctionItem(_) => false,
            Node::Reference { referent, .. } => {
                let child = referent.0 as usize;
                !matches!(self.nodes.get(child), Some(Node::Parameter(_)))
                    && self.trivial_formation.get(child) == Some(&true)
            }
        };
        self.nodes.push(node);
        self.trivial_formation.push(formed);
        self.interned.insert(node, id);
        self.visits = self.visits.saturating_add(1);
        Ok(id)
    }
    pub(super) fn remember(
        &mut self,
        key: Key,
        value: NodeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.reserve(retained::new_map::<Key, NodeId>(1), 0, work)?;
        if self.completed.insert(key, value).is_some() {
            return Err("source input type recipe was completed twice");
        }
        Ok(())
    }
    pub(super) fn retained_bytes(&self, domains: &DomainIndex<'_, '_>, base: usize) -> usize {
        retained::sum([
            domains.retained_bytes(),
            base,
            size_of::<Self>(),
            retained::vector(&self.nodes),
            retained::vector(&self.trivial_formation),
            retained::vector(&self.relocations),
            retained::map(&self.interned),
            retained::map(&self.completed),
            retained::map(&self.templates),
            retained::vector(&self.signature_roots),
            self.matched.capacity().saturating_mul(size_of::<Pair>()),
        ])
    }
}
