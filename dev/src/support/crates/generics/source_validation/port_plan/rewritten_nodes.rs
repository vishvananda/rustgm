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
    ObjectBound {
        depth: u16,
        slot: u32,
    },
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
    Nominal {
        declaration: s::DeclarationAddress,
        arguments: NodeId,
    },
    NominalEnd,
    Dynamic {
        bounds: NodeId,
        region: Region,
    },
    DynamicBounds {
        recipe: s::DynamicTypeRef,
        arguments: NodeId,
    },
    OpaqueFuture {
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        ordinal: u32,
        arguments: NodeId,
    },
    /// A declared result stays structural: its member names one trait, and the
    /// argument chain is its Self followed by that trait's complete arguments.
    /// Its own family parameters are a second chain. Selection still normalizes
    /// the projection against the declaration and use.
    Projection {
        member: s::DeclarationAddress,
        arguments: NodeId,
        family: NodeId,
    },
    NominalArgument {
        ty: NodeId,
        next: NodeId,
    },
    /// Version thirteen: one leading region argument of a nominal spine.
    NominalRegion {
        region: Region,
        next: NodeId,
    },
    Scalar(Scalar),
    F16,
    F128,
    Foreign(crate::support::crates::owner_catalog::DeclarationRef),
    /// Version thirteen: an evaluated const argument by its primitive type and
    /// canonical bits, held as four words so the node keeps its alignment. A
    /// rigid const term is a `Parameter` of its const slot.
    ConstValue {
        scalar: Scalar,
        bits: [u32; 4],
    },
    ConstVariant(crate::support::crates::owner_catalog::DeclarationRef),
    /// Version thirteen: the unsized `str` and the never type.
    Str,
    Never,
    Parameter(s::SlotRef),
    Reference {
        mutable: bool,
        region: Region,
        referent: NodeId,
    },
    /// Version thirteen: a raw pointer has no region and accepts any pointee.
    RawPointer {
        mutable: bool,
        pointee: NodeId,
    },
    /// Version thirteen: the unsized `[T]`; its element's sizedness is
    /// re-proved by every consumer.
    Slice(NodeId),
    /// Version thirteen: `[T; N]` over its element and its length's
    /// `ConstValue` or const `Parameter` node.
    Array {
        element: NodeId,
        length: NodeId,
    },
    /// Version thirteen: a pattern over a raw pointer or concrete scalar
    /// base. Signature matching compares range constraints by value.
    Pattern {
        base: NodeId,
        constraint: s::PatternConstraint,
    },
    FunctionItem(s::FunctionApplicationRef),
    SignatureResult(NodeId),
    SignatureInput {
        ty: NodeId,
        next: NodeId,
    },
    /// A non-unit tuple over a `NominalArgument` element chain.
    Tuple {
        elements: NodeId,
    },
    /// Only zero-own-slot, region-independent Rust/nonvariadic frames enter
    /// below version fourteen; version fourteen retains regional/higher-ranked
    /// callable frames and re-proves their formation.
    Function {
        signature: NodeId,
        bound_regions: u32,
        unsafe_: bool,
    },
    /// Version fourteen: an applied closure type; formation is re-proved on import.
    Closure {
        closure: s::ClosureRef,
        arguments: NodeId,
        signature: NodeId,
        environment: NodeId,
    },
}

// A const value keeps the node at its version-twelve width.
#[cfg(test)]
const _: [(); 20] = [(); size_of::<Node>()];
#[derive(Clone, Copy)]
struct NodeFacts(u8);
#[cfg(test)]
const _: [(); size_of::<bool>()] = [(); size_of::<NodeFacts>()];
impl NodeFacts {
    const FORMED: u8 = 1;
    const REGIONAL: u8 = 2;
    const ITEM: u8 = 4;
    const PARAMETER: u8 = 8;
    const fn formed(self) -> bool {
        self.0 & Self::FORMED != 0
    }
    const fn combine(self, other: Self) -> Self {
        Self(((self.0 | other.0) & !Self::FORMED) | ((self.0 & other.0) & Self::FORMED))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Rewrite {
    Original,
    Nominal {
        owner: crate::support::crates::owner_catalog::DeclarationRef,
        arguments: NodeId,
    },
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DynamicShape {
    principal: Option<s::DeclarationAddress>,
    principal_arguments: u32,
    bound_regions: u32,
    bindings: Vec<s::DynamicBinding>,
    markers: Vec<s::DynamicMarker>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MappedSlot {
    Unused,
    Early(u32),
    Late(u32),
}

pub(super) struct Arena {
    pub nodes: Vec<Node>,
    facts: Vec<NodeFacts>,
    pub relocations: Vec<u32>,
    pub fn_slot_base: Vec<u32>,
    pub slot_map: Vec<MappedSlot>,
    pub nominal_arguments: HashMap<NodeId, Vec<NodeId>>,
    pub nominal_regions: HashMap<NodeId, Vec<Region>>,
    pub nominal_argument_bytes: usize,
    dynamic_shapes: HashMap<DynamicShape, s::DynamicTypeRef>,
    dynamic_shape_bytes: usize,
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
    declaration_policies: bool,
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
            facts: Vec::new(),
            relocations: Vec::new(),
            fn_slot_base: Vec::new(),
            slot_map: Vec::new(),
            nominal_arguments: HashMap::default(),
            nominal_regions: HashMap::default(),
            nominal_argument_bytes: 0,
            dynamic_shapes: HashMap::default(),
            dynamic_shape_bytes: 0,
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
            declaration_policies: domains.declaration_policies,
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
        self.facts
            .get(node.0 as usize)
            .copied()
            .map(NodeFacts::formed)
            .ok_or("source formation node is unknown")
    }
    pub(super) fn intern(&mut self, node: Node, work: &mut Work) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        if let Some(&id) = self.interned.get(&node) {
            self.hits = self.hits.saturating_add(1);
            return Ok(id);
        }
        self.reserve(
            4 * size_of::<Node>()
                + 8 * size_of::<NodeFacts>()
                + retained::new_map::<Node, NodeId>(1),
            0,
            work,
        )?;
        let id = NodeId(
            u32::try_from(self.nodes.len()).map_err(|_| "source input type-node index overflow")?,
        );
        let facts = self.node_facts(node, work)?;
        self.nodes.push(node);
        self.facts.push(facts);
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
    fn facts_of(&self, node: NodeId, work: &mut Work) -> Result<NodeFacts, &'static str> {
        work.charge(1)?;
        self.facts
            .get(node.0 as usize)
            .copied()
            .ok_or("source input type child facts are absent")
    }
    fn node_facts(&self, node: Node, work: &mut Work) -> Result<NodeFacts, &'static str> {
        work.charge(1)?;
        Ok(match node {
            Node::Dynamic {
                bounds: child,
                region,
            }
            | Node::NominalRegion {
                region,
                next: child,
            } => {
                let mut facts = self.facts_of(child, work)?;
                if region != Region::Static {
                    facts.0 |= NodeFacts::REGIONAL;
                }
                facts
            }
            Node::Nominal { arguments, .. }
            | Node::DynamicBounds { arguments, .. }
            | Node::OpaqueFuture { arguments, .. } => {
                let mut facts = self.facts_of(arguments, work)?;
                facts.0 &= !NodeFacts::FORMED;
                facts
            }
            Node::Projection {
                arguments, family, ..
            } => {
                let mut facts = self
                    .facts_of(arguments, work)?
                    .combine(self.facts_of(family, work)?);
                facts.0 &= !NodeFacts::FORMED;
                facts
            }
            Node::NominalEnd
            | Node::Scalar(_)
            | Node::F16
            | Node::F128
            | Node::Foreign(_)
            | Node::ConstValue { .. }
            | Node::ConstVariant(_)
            | Node::Str
            | Node::Never => NodeFacts(NodeFacts::FORMED),
            Node::NominalArgument { ty, next } => {
                self.facts_of(ty, work)?.combine(self.facts_of(next, work)?)
            }
            // Preserve the old parameter formation rule outside Functions.
            // A Function explicitly retains opaque parameter conditionality.
            Node::Parameter(_) => NodeFacts(NodeFacts::FORMED | NodeFacts::PARAMETER),
            Node::FunctionItem(_) => NodeFacts(NodeFacts::ITEM),
            Node::Reference {
                region, referent, ..
            } => {
                let mut facts = self.facts_of(referent, work)?;
                if matches!(
                    self.nodes.get(referent.0 as usize),
                    Some(Node::Parameter(_))
                ) {
                    facts.0 &= !NodeFacts::FORMED;
                }
                if region != Region::Static {
                    facts.0 |= NodeFacts::REGIONAL;
                }
                facts
            }
            // `*const T`, tuples, patterns, and signature results inherit their child's facts.
            Node::RawPointer { pointee: child, .. }
            | Node::Pattern { base: child, .. }
            | Node::Tuple { elements: child }
            | Node::SignatureResult(child) => self.facts_of(child, work)?,
            Node::Slice(element) => {
                let mut facts = self.facts_of(element, work)?;
                if matches!(self.nodes.get(element.0 as usize), Some(Node::Parameter(_))) {
                    facts.0 &= !NodeFacts::FORMED;
                }
                facts
            }
            // `[T; N]` needs `T: Sized` as `[T]` does; its length adds no
            // formation condition.
            Node::Array { element, length } => {
                let mut facts = self
                    .facts_of(element, work)?
                    .combine(self.facts_of(length, work)?);
                if matches!(self.nodes.get(element.0 as usize), Some(Node::Parameter(_))) {
                    facts.0 &= !NodeFacts::FORMED;
                }
                facts
            }
            Node::SignatureInput { ty, next } => {
                self.signature_tail(next, work)?;
                self.facts_of(ty, work)?.combine(self.facts_of(next, work)?)
            }
            Node::Function { signature, .. } => self.function_facts(signature, work)?,
            Node::Closure {
                arguments,
                signature,
                environment,
                ..
            } => {
                let mut facts = self
                    .facts_of(arguments, work)?
                    .combine(self.facts_of(signature, work)?)
                    .combine(self.facts_of(environment, work)?);
                facts.0 &= !NodeFacts::FORMED;
                facts
            }
        })
    }
    fn function_facts(
        &self,
        signature: NodeId,
        work: &mut Work,
    ) -> Result<NodeFacts, &'static str> {
        self.signature_tail(signature, work)?;
        let mut facts = self.facts_of(signature, work)?;
        if facts.0 & NodeFacts::ITEM != 0
            || (!self.declaration_policies && facts.0 & NodeFacts::REGIONAL != 0)
        {
            return Err("source Function substitution captured regional or function-item inputs");
        }
        if facts.0 & (NodeFacts::PARAMETER | NodeFacts::REGIONAL) != 0 {
            facts.0 &= !NodeFacts::FORMED;
        }
        Ok(facts)
    }
    fn signature_tail(&self, node: NodeId, work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        if !matches!(
            self.nodes.get(node.0 as usize),
            Some(Node::SignatureResult(_) | Node::SignatureInput { .. })
        ) {
            return Err("source Function lost its canonical signature-cell tail");
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn observe(&self, steps: usize) {
        eprintln!(
            "source planner node={} flags={} nodes={}/{} flags_cap={} memo={}/{} steps={} reserved={} peak={}",
            size_of::<Node>(),
            size_of::<NodeFacts>(),
            self.nodes.len(),
            self.nodes.capacity(),
            self.facts.capacity(),
            self.completed.len(),
            self.completed.capacity(),
            steps,
            self.reserved,
            self.peak,
        );
    }
    /// Intern a trait object's structural bounds so two declarations naming the
    /// same principal, equalities, auto traits and inner binder share one node.
    pub(super) fn canonical_dynamic(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        reference: s::DynamicTypeRef,
        work: &mut Work,
    ) -> Result<s::DynamicTypeRef, &'static str> {
        work.charge(1)?;
        let rows = domains.headers.rows();
        let columns = &rows.cold().dynamics;
        let row = columns
            .objects
            .get(reference.0 as usize)
            .ok_or("source dynamic shape is unknown")?;
        let bindings = row.bindings.slice(&columns.bindings)?;
        let markers = row.markers.slice(&columns.markers)?;
        work.charge(bindings.len().saturating_add(markers.len()))?;
        let key = DynamicShape {
            principal: row.principal,
            principal_arguments: row.principal_arguments,
            bound_regions: rows
                .binders
                .get(row.binder.0 as usize)
                .ok_or("source dynamic shape lost its inner binder")?
                .count,
            bindings: bindings.to_vec(),
            markers: markers.to_vec(),
        };
        if let Some(&canonical) = self.dynamic_shapes.get(&key) {
            self.hits = self.hits.saturating_add(1);
            return Ok(canonical);
        }
        let extra = retained::vector(&key.bindings).saturating_add(retained::vector(&key.markers));
        self.reserve(
            retained::new_map::<DynamicShape, s::DynamicTypeRef>(1).saturating_add(extra),
            0,
            work,
        )?;
        self.dynamic_shape_bytes = self.dynamic_shape_bytes.saturating_add(extra);
        self.dynamic_shapes.insert(key, reference);
        Ok(reference)
    }

    pub(super) fn retained_bytes(&self, domains: &DomainIndex<'_, '_>, base: usize) -> usize {
        retained::sum([
            domains.retained_bytes(),
            base,
            size_of::<Self>(),
            retained::vector(&self.nodes),
            retained::vector(&self.facts),
            retained::vector(&self.relocations),
            retained::vector(&self.fn_slot_base),
            retained::vector(&self.slot_map),
            retained::map(&self.nominal_arguments),
            retained::map(&self.nominal_regions),
            self.nominal_argument_bytes,
            retained::map(&self.dynamic_shapes),
            self.dynamic_shape_bytes,
            retained::map(&self.interned),
            retained::map(&self.completed),
            retained::map(&self.templates),
            retained::vector(&self.signature_roots),
            self.matched.capacity().saturating_mul(size_of::<Pair>()),
        ])
    }
}
