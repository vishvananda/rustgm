//! Signature-local wire ordinals and lifetime slots never denote frontend arena IDs.
use super::Scalar;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeRef(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Static,
    Bound(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeNode {
    Scalar(Scalar),
    Reference {
        region: Lifetime,
        mutable: bool,
        referent: TypeRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeGraph {
    nodes: Box<[TypeNode]>,
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
    fn validate(nodes: &[TypeNode], bound_regions: u16) -> Result<(), &'static str> {
        if nodes.is_empty() || nodes.len() > Self::MAX_NODES || bound_regions > 256 {
            return Err("metadata type graph exceeds profile budget");
        }
        let mut depths = Vec::with_capacity(nodes.len());
        let mut unique = HashSet::with_capacity(nodes.len());
        for (index, &node) in nodes.iter().enumerate() {
            if !unique.insert(node) {
                return Err("duplicate metadata type graph node");
            }
            let depth = match node {
                TypeNode::Scalar(_) => 0usize,
                TypeNode::Reference {
                    region, referent, ..
                } => {
                    if matches!(region, Lifetime::Bound(slot) if slot >= bound_regions) {
                        return Err("metadata reference uses an unbound lifetime slot");
                    }
                    if usize::from(referent.0) >= index {
                        return Err("metadata type graph has a forward or cyclic edge");
                    }
                    depths[usize::from(referent.0)] + 1
                }
            };
            if depth > Self::MAX_DEPTH {
                return Err("metadata reference nesting exceeds profile budget");
            }
            depths.push(depth);
        }
        Ok(())
    }
    fn canonical(&self, roots: impl Iterator<Item = TypeRef>) -> Result<(), &'static str> {
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
                if !finish && let TypeNode::Reference { referent, .. } = node {
                    work.push((id, true));
                    work.push((referent, false));
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
    fn regions(&self, roots: impl Iterator<Item = TypeRef>) -> HashSet<u16> {
        let mut seen = vec![false; self.nodes.len()];
        let mut regions = HashSet::new();
        for mut id in roots {
            while !seen[usize::from(id.0)] {
                seen[usize::from(id.0)] = true;
                match self.nodes[usize::from(id.0)] {
                    TypeNode::Scalar(_) => break,
                    TypeNode::Reference {
                        region, referent, ..
                    } => {
                        if let Lifetime::Bound(slot) = region {
                            regions.insert(slot);
                        }
                        id = referent;
                    }
                }
            }
        }
        regions
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    types: TypeGraph,
    parameters: Box<[TypeRef]>,
    result: TypeRef,
    bound_regions: u16,
    unsafe_: bool,
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
        let mut indices = HashMap::new();
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
        if parameters.len() > 256 {
            return Err("metadata signature exceeds parameter budget");
        }
        TypeGraph::validate(&nodes, bound_regions)?;
        let types = TypeGraph {
            nodes: nodes.into_boxed_slice(),
        };
        types.canonical(parameters.iter().copied().chain(std::iter::once(result)))?;
        if parameters
            .iter()
            .any(|&id| types.node(id) == Some(TypeNode::Scalar(Scalar::Unit)))
        {
            return Err("unit parameters are outside the metadata profile");
        }
        let inputs = types.regions(parameters.iter().copied());
        let output = types.regions(std::iter::once(result));
        if !output.is_subset(&inputs) {
            return Err("metadata output lifetime is unconstrained by its inputs");
        }
        Ok(Self {
            types,
            parameters: parameters.into_boxed_slice(),
            result,
            bound_regions,
            unsafe_,
        })
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
                .types
                .nodes
                .iter()
                .all(|node| matches!(node, TypeNode::Scalar(_)))
    }
    pub(super) fn atoms(&self) -> usize {
        self.types.nodes.len() + self.parameters.len() + usize::from(self.bound_regions) + 1
    }
}
