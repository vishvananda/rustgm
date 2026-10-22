//! Shared wire graphs summarize lifetime slots once; roots never denote frontend IDs.
use super::{RecordRef, Scalar};
use crate::support::hash::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeRef(pub u16);
/// Calling convention is retained independently of function safety and linkage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Abi {
    #[default]
    Rust,
    C,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Static,
    Bound(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeNode {
    Scalar(Scalar),
    Record(RecordRef),
    Array {
        element: TypeRef,
        length: u64,
    },
    Reference {
        region: Lifetime,
        mutable: bool,
        referent: TypeRef,
    },
}
impl TypeNode {
    pub(super) const fn child(self) -> Option<TypeRef> {
        match self {
            Self::Reference { referent, .. } => Some(referent),
            Self::Array { element, .. } => Some(element),
            Self::Scalar(_) | Self::Record(_) => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeGraph {
    nodes: Arc<[TypeNode]>,
    regions: Arc<[[u64; 4]]>,
}
impl TypeGraph {
    pub const MAX_NODES: usize = 4096;
    pub const MAX_DEPTH: usize = 64;
    #[must_use]
    pub fn nodes(&self) -> &[TypeNode] {
        &self.nodes
    }
    #[must_use]
    pub fn node(&self, reference: TypeRef) -> Option<TypeNode> {
        self.nodes.get(usize::from(reference.0)).copied()
    }
    /// Validate storage once. Signature binders and record ordinals are checked
    /// against their owning roots and artifact before publication.
    /// # Errors
    /// Rejects oversized, duplicate, forward/cyclic and overdeep graph nodes.
    pub fn new(nodes: Vec<TypeNode>) -> Result<Self, &'static str> {
        if nodes.len() > Self::MAX_NODES {
            return Err("metadata type graph exceeds profile budget");
        }
        let mut depths = Vec::with_capacity(nodes.len());
        let mut regions: Vec<[u64; 4]> = Vec::with_capacity(nodes.len());
        let mut unique = HashSet::with_capacity_and_hasher(
            nodes.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for (index, &node) in nodes.iter().enumerate() {
            if !unique.insert(node) {
                return Err("duplicate metadata type graph node");
            }
            let (depth, mut slots) = if let Some(child) = node.child() {
                let child = usize::from(child.0);
                if child >= index {
                    return Err("metadata type graph has a forward or cyclic edge");
                }
                (depths[child] + 1, regions[child])
            } else {
                (0usize, [0; 4])
            };
            if let TypeNode::Reference {
                region: Lifetime::Bound(slot),
                ..
            } = node
            {
                if slot >= 256 {
                    return Err("metadata reference uses an unbound lifetime slot");
                }
                slots[usize::from(slot / 64)] |= 1u64 << (slot % 64);
            }
            if depth > Self::MAX_DEPTH {
                return Err("metadata reference nesting exceeds profile budget");
            }
            depths.push(depth);
            regions.push(slots);
        }
        Ok(Self {
            nodes: nodes.into(),
            regions: regions.into(),
        })
    }
    /// Cloning a graph retains the same immutable storage for all signature roots.
    #[must_use]
    pub fn shares_storage(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.nodes, &other.nodes) && Arc::ptr_eq(&self.regions, &other.regions)
    }
    pub(super) fn slots(&self, root: TypeRef) -> Result<[u64; 4], &'static str> {
        self.regions
            .get(usize::from(root.0))
            .copied()
            .ok_or("unknown metadata type graph root")
    }
    pub(super) fn canonical(
        &self,
        roots: impl Iterator<Item = TypeRef>,
    ) -> Result<(), &'static str> {
        let mut seen = vec![false; self.nodes.len()];
        let mut next = 0usize;
        for root in roots {
            let mut work = vec![(root, false)];
            while let Some((id, finish)) = work.pop() {
                let index = usize::from(id.0);
                let node = self.node(id).ok_or("unknown metadata type graph root")?;
                if seen[index] {
                    continue;
                }
                if !finish && let Some(child) = node.child() {
                    work.push((id, true));
                    work.push((child, false));
                    continue;
                }
                if index != next {
                    return Err("metadata type graph is not in first-use postorder");
                }
                seen[index] = true;
                next += 1;
            }
        }
        if next != self.nodes.len() {
            return Err("metadata type graph contains unreachable nodes");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    types: TypeGraph,
    parameters: Box<[TypeRef]>,
    result: TypeRef,
    bound_regions: u16,
    unsafe_: bool,
    abi: Abi,
}
impl Signature {
    /// Preserve the original closed scalar constructor and its wire encoding.
    /// # Errors
    /// Rejects unit parameters, oversized signatures and malformed type graphs.
    pub fn new(
        parameters: Vec<Scalar>,
        result: Scalar,
        unsafe_: bool,
    ) -> Result<Self, &'static str> {
        let mut nodes = Vec::new();
        let mut indices = HashMap::default();
        let mut intern = |scalar| -> Result<_, &'static str> {
            if let Some(&id) = indices.get(&scalar) {
                return Ok(id);
            }
            let id = TypeRef(u16::try_from(nodes.len()).map_err(|_| "too many metadata types")?);
            nodes.push(TypeNode::Scalar(scalar));
            indices.insert(scalar, id);
            Ok(id)
        };
        if parameters.len() > 256 || parameters.contains(&Scalar::Unit) {
            return Err("unsupported metadata scalar parameters");
        }
        let parameters = parameters
            .into_iter()
            .map(&mut intern)
            .collect::<Result<_, _>>()?;
        let result = intern(result)?;
        Self::with_types(nodes, 0, parameters, result, unsafe_)
    }
    /// Validate a complete late-bound signature before any semantic interning.
    /// # Errors
    /// Rejects invalid graph identities, lifetime slots, canonical order and quotas.
    pub fn with_types(
        nodes: Vec<TypeNode>,
        bound_regions: u16,
        parameters: Vec<TypeRef>,
        result: TypeRef,
        unsafe_: bool,
    ) -> Result<Self, &'static str> {
        if nodes.is_empty() || nodes.iter().any(|node| matches!(node, TypeNode::Record(_))) {
            return Err("signature-local metadata requires a nonempty nonnominal graph");
        }
        let types = TypeGraph::new(nodes)?;
        types.canonical(parameters.iter().copied().chain(std::iter::once(result)))?;
        Self::with_shared_types(types, bound_regions, parameters, result, unsafe_)
    }
    /// Bind roots into one validated artifact graph without copying or rescanning it.
    /// # Errors
    /// Rejects invalid roots, unsupported parameters, unbound or escaping lifetimes.
    pub fn with_shared_types(
        types: TypeGraph,
        bound_regions: u16,
        parameters: Vec<TypeRef>,
        result: TypeRef,
        unsafe_: bool,
    ) -> Result<Self, &'static str> {
        if parameters.len() > 256 || bound_regions > 256 {
            return Err("metadata signature exceeds parameter or lifetime budget");
        }
        let mut inputs = [0u64; 4];
        for &id in &parameters {
            if types.node(id) == Some(TypeNode::Scalar(Scalar::Unit)) {
                return Err("unit parameters are outside the metadata profile");
            }
            for (all, next) in inputs.iter_mut().zip(types.slots(id)?) {
                *all |= next;
            }
        }
        let output = types.slots(result)?;
        for (index, (&input, out)) in inputs.iter().zip(output).enumerate() {
            let available = usize::from(bound_regions)
                .saturating_sub(index * 64)
                .min(64);
            let allowed = u64::MAX
                .checked_shr(
                    u32::try_from(64 - available).map_err(|_| "invalid metadata lifetime mask")?,
                )
                .unwrap_or(0);
            if (input | out) & !allowed != 0 {
                return Err("metadata reference uses an unbound lifetime slot");
            }
            if out & !input != 0 {
                return Err("metadata output lifetime is unconstrained by its inputs");
            }
        }
        Ok(Self {
            types,
            parameters: parameters.into_boxed_slice(),
            result,
            bound_regions,
            unsafe_,
            abi: Abi::Rust,
        })
    }
    /// Select a fixed calling convention without changing the validated type graph.
    #[must_use]
    pub const fn with_abi(mut self, abi: Abi) -> Self {
        self.abi = abi;
        self
    }
    #[must_use]
    pub const fn abi(&self) -> Abi {
        self.abi
    }
    #[must_use]
    pub const fn types(&self) -> &TypeGraph {
        &self.types
    }
    #[must_use]
    pub fn parameters(&self) -> &[TypeRef] {
        &self.parameters
    }
    #[must_use]
    pub const fn result(&self) -> TypeRef {
        self.result
    }
    #[must_use]
    pub const fn bound_regions(&self) -> u16 {
        self.bound_regions
    }
    #[must_use]
    pub const fn unsafe_(&self) -> bool {
        self.unsafe_
    }
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        self.bound_regions == 0
            && self
                .parameters
                .iter()
                .copied()
                .chain(std::iter::once(self.result))
                .all(|id| matches!(self.types.node(id), Some(TypeNode::Scalar(_))))
    }
    pub(super) fn atoms(&self) -> usize {
        self.types.nodes.len() + self.root_atoms()
    }
    pub(super) fn root_atoms(&self) -> usize {
        self.parameters.len() + usize::from(self.bound_regions) + 1
    }
}
