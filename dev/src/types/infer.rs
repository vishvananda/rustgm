//! Per-body constraints over flat terms, with path-compressed variable identities.
//! A failed constraint may retain prior progress; callers discard that body context.
mod allocation;
pub mod arguments;
mod batch;
mod compare;
mod completed;
pub mod constants;
pub mod deferred;
mod defining;
pub mod dependencies;
mod freshen;
pub use freshen::FreshRegions;
mod import;
mod indices;
pub mod objects;
pub mod predicates;
pub mod probe;
pub mod projections;
pub mod properties;
pub mod refinement;
mod regions;
pub(crate) use regions::RegionMapping;
mod resolve;
mod scoped;
mod signature_slots;
mod specialize;
mod substitute;
use super::{GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId};
use crate::support::hash::{HashMap, HashSet};
pub use substitute::Instantiation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TermId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    Any,
    Integer,
    Float,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constructor {
    Primitive(Primitive),
    Parameter(super::parameters::ParameterId),
    Adt(ItemId, GenericArgs),
    Never,
    Tuple,
    Array(u64),
    ArrayTerm(super::arguments::ConstId),
    ArrayInfer(constants::ConstTermId),
    AdtArgs(ItemId, arguments::ArgumentsId),
    Opaque(super::opaque::OpaqueId, arguments::ArgumentsId),
    Dynamic {
        shape: super::dynamic::ShapeId,
        arguments: arguments::ArgumentsId,
        lifetimes: objects::LifetimesId,
    },
    FunctionItemArgs(ItemId, arguments::ArgumentsId),
    Projection(projections::Projection),
    Slice,
    Reference {
        mutable: bool,
        region: RegionId,
    },
    Pointer {
        mutable: bool,
    },
    Function {
        bound_regions: u32,
        parameters: u32,
        unsafe_: bool,
        convention: super::callable::Convention,
    },
    FunctionItem(ItemId, GenericArgs),
    /// Signature and environment tuple precede the enclosing type argument edges.
    Closure(super::closure::ClosureId, arguments::ArgumentsId),
}
#[derive(Clone, Copy)]
enum Value {
    Variable(Domain),
    Known(TypeId),
    Construct(Constructor),
}
#[derive(Clone, Copy)]
struct Term {
    parent: TermId,
    rank: u8,
    was_variable: bool,
    value: Value,
    start: u32,
    end: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidIdentity,
    Overflow,
    Mismatch,
    Occurs,
    Unresolved,
    Budget,
    Normalization,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(match self {
            Self::InvalidIdentity => "invalid inference identity",
            Self::Overflow => "inference arena exceeds 32 bits",
            Self::Mismatch => "incompatible types",
            Self::Occurs => "type variable occurs in its own type",
            Self::Unresolved => "type annotations needed",
            Self::Budget => "type constraint work budget exceeded",
            Self::Normalization => "associated type projection requires normalization",
        })
    }
}
impl std::error::Error for Error {}
#[derive(Default)]
pub struct Context {
    defining: Option<Box<defining::Definitions>>,
    signature_slots: Option<Box<signature_slots::Slots>>,
    arguments: Option<Box<arguments::Storage>>,
    dependencies: Option<Box<dependencies::Storage>>,
    resolution: Option<Box<resolve::Resolution>>,
    /// Completed graph facts, dense by term identity.
    #[expect(
        clippy::box_collection,
        reason = "search frames embed two contexts; one pointer keeps their qualified layout"
    )]
    settled: Option<Box<Vec<Option<completed::Completed>>>>,
    generalization: freshen::Scratch,
    terms: Vec<Term>,
    edges: Vec<TermId>,
    known: HashMap<TypeId, TermId>,
    pairs: Vec<(TermId, TermId)>,
    seen: HashSet<(TermId, TermId)>,
    occurs_work: Vec<TermId>,
    child_scratch: Vec<TermId>,
    marks: Vec<u32>,
    epoch: u32,
    /// Equalities are retained for the later region checker, not silently erased.
    pub region_equalities: Vec<(RegionId, RegionId)>,
    pub steps: usize,
}
impl Context {
    /// A context equivalent to `Context::default()` that keeps this finished
    /// context's flat storage, bounded so one large body cannot make every
    /// later small body clear a large table.
    #[must_use]
    pub fn recycled(self) -> Self {
        const RETAINED: usize = 1 << 12;
        fn keep<T>(mut values: Vec<T>) -> Vec<T> {
            values.clear();
            if values.capacity() > RETAINED {
                Vec::new()
            } else {
                values
            }
        }
        // Clearing a table visits its capacity: keep only tables this body
        // actually filled, so a small body after a large one stays cheap.
        let retained =
            |len: usize, capacity: usize| capacity <= RETAINED.min(len.saturating_mul(4).max(64));
        let Self {
            terms,
            edges,
            mut known,
            pairs,
            mut seen,
            occurs_work,
            child_scratch,
            marks,
            region_equalities,
            settled,
            ..
        } = self;
        let known = if retained(known.len(), known.capacity()) {
            known.clear();
            known
        } else {
            HashMap::default()
        };
        let seen = if retained(seen.len(), seen.capacity()) {
            seen.clear();
            seen
        } else {
            HashSet::default()
        };
        Self {
            terms: keep(terms),
            edges: keep(edges),
            known,
            pairs: keep(pairs),
            seen,
            occurs_work: keep(occurs_work),
            child_scratch: keep(child_scratch),
            marks: keep(marks),
            region_equalities: keep(region_equalities),
            settled: settled.map(|values| Box::new(keep(*values))),
            ..Self::default()
        }
    }
    fn settled_facts(&self, id: TermId) -> Option<completed::Completed> {
        self.settled.as_ref()?.get(id.0 as usize).copied().flatten()
    }
    fn set_settled(&mut self, id: TermId, facts: completed::Completed) {
        let settled = self.settled.get_or_insert_with(Box::default);
        let index = id.0 as usize;
        if index >= settled.len() {
            settled.resize(index + 1, None);
        }
        settled[index] = Some(facts);
    }
    fn append_edges(&mut self, children: &[TermId]) -> Result<(u32, u32), Error> {
        let start = u32::try_from(self.edges.len()).map_err(|_| Error::Overflow)?;
        let end = self
            .edges
            .len()
            .checked_add(children.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Error::Overflow)?;
        self.edges.extend_from_slice(children);
        Ok((start, end))
    }
    fn push(&mut self, value: Value, children: &[TermId]) -> Result<TermId, Error> {
        if children.iter().any(|id| id.0 as usize >= self.terms.len()) {
            return Err(Error::InvalidIdentity);
        }
        let id = TermId(u32::try_from(self.terms.len()).map_err(|_| Error::Overflow)?);
        let (start, end) = self.append_edges(children)?;
        self.terms.push(Term {
            parent: id,
            rank: 0,
            was_variable: matches!(value, Value::Variable(_)),
            value,
            start,
            end,
        });
        self.marks.push(0);
        Ok(id)
    }
    /// # Errors
    /// Rejects compact identity overflow.
    pub fn variable(&mut self, domain: Domain) -> Result<TermId, Error> {
        self.push(Value::Variable(domain), &[])
    }
    /// # Errors
    /// Rejects invalid canonical identities and compact arena overflow.
    pub fn known(&mut self, store: &Store, id: TypeId) -> Result<TermId, Error> {
        if store.kind(id).is_none() {
            return Err(Error::InvalidIdentity);
        }
        if let Some(term) = self.known.get(&id) {
            return Ok(*term);
        }
        let term = self.push(Value::Known(id), &[])?;
        self.known.insert(id, term);
        Ok(term)
    }
    /// # Errors
    /// Rejects invalid arity, child identities and compact arena overflow.
    pub fn construct(&mut self, kind: Constructor, children: &[TermId]) -> Result<TermId, Error> {
        let valid = match kind {
            Constructor::AdtArgs(_, id) | Constructor::Opaque(_, id) => {
                children.len() == self.argument_arity(id)?
            }
            Constructor::Dynamic {
                arguments,
                lifetimes,
                ..
            } => {
                self.object_lifetimes(lifetimes)?;
                children.len() == self.argument_arity(arguments)?
            }
            Constructor::Projection(value) => {
                children.len().checked_sub(1) == Some(self.argument_arity(value.arguments)?)
                    && value.trait_arguments as usize <= self.slots(value.arguments)?.len()
            }
            Constructor::FunctionItemArgs(_, id) => {
                children.len().checked_sub(1) == Some(self.argument_arity(id)?)
            }
            Constructor::Closure(_, id) => {
                children.len().checked_sub(2) == Some(self.argument_arity(id)?)
            }
            Constructor::ArrayInfer(id) => {
                children.len() == 1
                    && self
                        .arguments
                        .as_ref()
                        .ok_or(Error::InvalidIdentity)?
                        .constants
                        .ty(id)?
                        == TypeId::primitive(Primitive::Usize)
            }
            Constructor::Primitive(_)
            | Constructor::Parameter(_)
            | Constructor::Adt(..)
            | Constructor::Never => children.is_empty(),
            Constructor::Tuple => true,
            Constructor::Function { parameters, .. } => {
                children.len().checked_sub(1) == Some(parameters as usize)
            }
            _ => children.len() == 1,
        };
        if !valid {
            return Err(Error::InvalidIdentity);
        }
        self.push(Value::Construct(kind), children)
    }
    /// # Errors
    /// Rejects an out-of-range identity.
    pub fn root(&mut self, mut id: TermId) -> Result<TermId, Error> {
        let mut root = id;
        loop {
            let parent = self
                .terms
                .get(root.0 as usize)
                .ok_or(Error::InvalidIdentity)?
                .parent;
            if parent == root {
                break;
            }
            root = parent;
        }
        while id != root {
            let term = &mut self.terms[id.0 as usize];
            let parent = term.parent;
            term.parent = root;
            id = parent;
        }
        Ok(root)
    }
    /// Charge a typed substitution performed on behalf of this inference body.
    ///
    /// # Errors
    /// Rejects work beyond the same budget as inference graph operations.
    pub fn account_work(&mut self, amount: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), Error> {
        self.steps += 1;
        if self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn occurs(&mut self, variable: TermId, candidate: TermId) -> Result<bool, Error> {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.marks.fill(0);
            self.epoch = 1;
        }
        self.occurs_work.clear();
        self.occurs_work.push(candidate);
        while let Some(next) = self.occurs_work.pop() {
            self.step()?;
            let next = self.root(next)?;
            if next == variable {
                return Ok(true);
            }
            if self.marks[next.0 as usize] == self.epoch {
                continue;
            }
            self.marks[next.0 as usize] = self.epoch;
            let term = self.terms[next.0 as usize];
            self.occurs_work
                .extend_from_slice(&self.edges[term.start as usize..term.end as usize]);
        }
        Ok(false)
    }
    fn variable_pair(
        &mut self,
        left: TermId,
        right: TermId,
        a: Domain,
        b: Domain,
    ) -> Result<(), Error> {
        let domain = match (a, b) {
            (Domain::Any, value) | (value, Domain::Any) => value,
            (a, b) if a == b => a,
            _ => return Err(Error::Mismatch),
        };
        let (parent, child) =
            if self.terms[left.0 as usize].rank >= self.terms[right.0 as usize].rank {
                (left, right)
            } else {
                (right, left)
            };
        if self.terms[parent.0 as usize].rank == self.terms[child.0 as usize].rank {
            self.terms[parent.0 as usize].rank += 1;
        }
        self.link_root(child, parent)?;
        self.terms[parent.0 as usize].value = Value::Variable(domain);
        self.wake_resolution(dependencies::Dependency::Type(child))?;
        self.wake_resolution(dependencies::Dependency::Type(parent))?;
        Ok(())
    }
    fn bind(
        &mut self,
        store: &Store,
        variable: TermId,
        candidate: TermId,
        domain: Domain,
    ) -> Result<(), Error> {
        self.bind_checked::<true>(store, variable, candidate, domain)
    }
    fn bind_checked<const CHECK_OCCURS: bool>(
        &mut self,
        store: &Store,
        variable: TermId,
        candidate: TermId,
        domain: Domain,
    ) -> Result<(), Error> {
        let primitive = match self.terms[candidate.0 as usize].value {
            Value::Known(id) => match store.kind(id) {
                Some(Kind::Primitive(p)) => Some(p),
                _ => None,
            },
            Value::Construct(Constructor::Primitive(p)) => Some(p),
            _ => None,
        };
        let accepts = match domain {
            Domain::Any => true,
            Domain::Integer => primitive.is_some_and(|p| p.integer().is_some()),
            Domain::Float => primitive.is_some_and(Primitive::is_float),
        };
        if !accepts {
            return Err(Error::Mismatch);
        }
        if CHECK_OCCURS
            && matches!(self.terms[candidate.0 as usize].value, Value::Construct(_))
            && self.occurs(variable, candidate)?
        {
            return Err(Error::Occurs);
        }
        self.link_root(variable, candidate)?;
        self.wake_resolution(dependencies::Dependency::Type(variable))?;
        Ok(())
    }
    /// Equality, not coercion: never and numeric conversions require explicit callers.
    ///
    /// # Errors
    /// Reports incompatible shapes/domains, recursive types, invalid IDs and work limits.
    pub fn equate(&mut self, store: &Store, left: TermId, right: TermId) -> Result<(), Error> {
        self.unify(store, left, right, true, None)
    }
    /// Constrain type structure while leaving the lifetime relation to the caller.
    /// Coercions retain a directed relation separately; equality alone cannot
    /// describe covariance or mutable-reference invariance.
    ///
    /// # Errors
    /// Reports incompatible shapes/domains, recursive types, invalid IDs and work limits.
    pub fn equate_shape(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
    ) -> Result<(), Error> {
        self.unify(store, left, right, false, None)
    }
    /// Match a constant value's type without equating its storage lifetime to
    /// the scrutinee's lifetime. Reference mutability and referent shapes still
    /// agree; existing expression constraints remain intact.
    ///
    /// # Errors
    /// Reports incompatible shapes/domains, recursive types, invalid IDs and work limits.
    pub fn equate_pattern(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
    ) -> Result<(), Error> {
        self.equate_shape(store, left, right)
    }
    fn unify(
        &mut self,
        store: &Store,
        left: TermId,
        right: TermId,
        record_regions: bool,
        blocked: Option<&mut Vec<(TermId, TermId)>>,
    ) -> Result<(), Error> {
        self.pairs.clear();
        self.seen.clear();
        self.pairs.push((left, right));
        self.unify_pending::<true>(store, record_regions, blocked)
    }
    fn unify_pending<const CHECK_OCCURS: bool>(
        &mut self,
        store: &Store,
        record_regions: bool,
        mut blocked: Option<&mut Vec<(TermId, TermId)>>,
    ) -> Result<(), Error> {
        while let Some((a, b)) = self.pairs.pop() {
            self.step()?;
            self.join_slot_classes(a, b)?;
            let a = self.root(a)?;
            let b = self.root(b)?;
            if a == b {
                continue;
            }
            let left = self.defining_root(store, a, b)?;
            let b = self.defining_root(store, b, a)?;
            let a = left;
            if a == b {
                continue;
            }
            if let Some(blocked) = &mut blocked {
                let first = self.needs_normalization(store, a)?;
                let second = self.needs_normalization(store, b)?;
                if first || second {
                    let pair = if a.0 < b.0 { (a, b) } else { (b, a) };
                    if self.seen.insert(pair) {
                        blocked.push((a, b));
                    }
                    continue;
                }
            }
            match (
                self.terms[a.0 as usize].value,
                self.terms[b.0 as usize].value,
            ) {
                (Value::Variable(x), Value::Variable(y)) => self.variable_pair(a, b, x, y)?,
                (Value::Variable(domain), _) => {
                    self.bind_checked::<CHECK_OCCURS>(store, a, b, domain)?;
                }
                (_, Value::Variable(domain)) => {
                    self.bind_checked::<CHECK_OCCURS>(store, b, a, domain)?;
                }
                (Value::Known(x), Value::Known(y)) if x == y => {}
                _ => {
                    let pair = if a.0 < b.0 { (a, b) } else { (b, a) };
                    if !self.seen.insert(pair) {
                        continue;
                    }
                    self.structural(store, a, b, record_regions)?;
                }
            }
        }
        Ok(())
    }
    /// Inspect only the outer shape, materializing canonical children on demand.
    /// An unconstrained variable has no shape; this never applies fallback.
    ///
    /// # Errors
    /// Rejects invalid identities and compact arena overflow.
    pub fn head(&mut self, store: &Store, id: TermId) -> Result<Option<Constructor>, Error> {
        let id = self.root(id)?;
        if matches!(self.terms[id.0 as usize].value, Value::Variable(_)) {
            return Ok(None);
        }
        let kind = self.materialize(store, id)?;
        if kind.normalizable(store) && self.opaque_projection(id).is_none() {
            return Err(Error::Normalization);
        }
        if let Constructor::ArrayInfer(length) = kind
            && let Some(crate::types::arguments::ConstKind::Value(value)) =
                self.constants().value(length)?
        {
            return Ok(Some(Constructor::Array(
                u64::try_from(value.bits).map_err(|_| Error::InvalidIdentity)?,
            )));
        }
        Ok(Some(kind))
    }
    /// # Errors
    /// Rejects invalid identities, missing children and arena overflow.
    pub fn child(&mut self, store: &Store, id: TermId, index: usize) -> Result<TermId, Error> {
        let id = self.root(id)?;
        self.materialize(store, id)?;
        let term = self.terms[id.0 as usize];
        self.edges[term.start as usize..term.end as usize]
            .get(index)
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
    /// Follow built-in reference layers without resolving an unknown referent.
    /// # Errors
    /// Rejects invalid graph identities and excessive dereference depth.
    pub fn reference_target(
        &mut self,
        store: &Store,
        mut term: TermId,
    ) -> Result<(TermId, u32), Error> {
        let mut count = 0;
        while matches!(self.head(store, term)?, Some(Constructor::Reference { .. })) {
            count += 1;
            if count > 1_000_000 {
                return Err(Error::Budget);
            }
            term = self.child(store, term, 0)?;
        }
        Ok((term, count))
    }
    /// Number of children of a known constructor, without resolving inference variables.
    ///
    /// # Errors
    /// Rejects invalid identities or exhausted term storage.
    pub fn arity(&mut self, store: &Store, id: TermId) -> Result<Option<usize>, Error> {
        let id = self.root(id)?;
        if self.head(store, id)?.is_none() {
            return Ok(None);
        }
        let term = self.terms[id.0 as usize];
        Ok(Some((term.end - term.start) as usize))
    }
    /// Apply the edition-2024 never fallback to an otherwise unconstrained
    /// general variable at a never-coercion site, after all body constraints.
    /// Numeric variables retain their own defaults.
    ///
    /// # Errors
    /// Rejects an invalid inference identity.
    pub fn default_never(&mut self, id: TermId) -> Result<(), Error> {
        let id = self.root(id)?;
        if matches!(
            self.terms[id.0 as usize].value,
            Value::Variable(Domain::Any)
        ) {
            self.terms[id.0 as usize].value = Value::Known(TypeId::NEVER);
            self.wake_resolution(dependencies::Dependency::Type(id))?;
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one structural inference dispatch keeps constructor identity and argument equations together"
    )]
    fn structural(
        &mut self,
        store: &Store,
        a: TermId,
        b: TermId,
        record_regions: bool,
    ) -> Result<(), Error> {
        let left = self.materialize(store, a)?;
        let right = self.materialize(store, b)?;
        if left.normalizable(store) || right.normalizable(store) {
            return match (self.opaque_projection(a), self.opaque_projection(b)) {
                (Some(x), Some(y)) if x == y => Ok(()),
                (Some(_), Some(_)) => Err(Error::Mismatch),
                _ => Err(Error::Normalization),
            };
        }
        match (left, right) {
            (Constructor::Adt(x, known), Constructor::AdtArgs(y, inferred))
            | (Constructor::AdtArgs(y, inferred), Constructor::Adt(x, known))
            | (Constructor::FunctionItem(x, known), Constructor::FunctionItemArgs(y, inferred))
            | (Constructor::FunctionItemArgs(y, inferred), Constructor::FunctionItem(x, known))
                if x == y =>
            {
                let known = self.materialize_arguments(store, known, &mut Vec::new())?;
                self.equate_arguments(known, inferred, record_regions)?;
            }
            (Constructor::Closure(x, a), Constructor::Closure(y, b)) if x == y => {
                self.equate_arguments(a, b, record_regions)?;
            }
            (Constructor::Opaque(x, a), Constructor::Opaque(y, b)) if x == y => {
                self.equate_arguments(a, b, record_regions)?;
            }
            (Constructor::Dynamic { .. }, Constructor::Dynamic { .. }) => {
                self.object_equation(left, right, record_regions)?;
            }
            (Constructor::AdtArgs(x, a), Constructor::AdtArgs(y, b))
            | (Constructor::FunctionItemArgs(x, a), Constructor::FunctionItemArgs(y, b))
                if x == y =>
            {
                self.equate_arguments(a, b, record_regions)?;
            }
            (Constructor::ArrayInfer(a), Constructor::ArrayInfer(b)) => self.equate_consts(a, b)?,
            (Constructor::ArrayInfer(a), Constructor::Array(length))
            | (Constructor::Array(length), Constructor::ArrayInfer(a)) => {
                let b = self.const_length(length)?;
                self.equate_consts(a, b)?;
            }
            (Constructor::ArrayInfer(a), Constructor::ArrayTerm(id))
            | (Constructor::ArrayTerm(id), Constructor::ArrayInfer(a)) => {
                let b = self.known_const(store, id)?;
                self.equate_consts(a, b)?;
            }
            (
                Constructor::Reference {
                    mutable: x,
                    region: a,
                },
                Constructor::Reference {
                    mutable: y,
                    region: b,
                },
            ) if x == y => {
                if record_regions && a != b {
                    self.region_equalities.push((a, b));
                }
            }
            (Constructor::Adt(x, a), Constructor::Adt(y, b))
            | (Constructor::FunctionItem(x, a), Constructor::FunctionItem(y, b))
                if x == y =>
            {
                let a = store.region_args(a).ok_or(Error::InvalidIdentity)?;
                let b = store.region_args(b).ok_or(Error::InvalidIdentity)?;
                if a.len() != b.len() {
                    return Err(Error::Mismatch);
                }
                if record_regions {
                    self.region_equalities.extend(
                        a.iter()
                            .copied()
                            .zip(b.iter().copied())
                            .filter(|(a, b)| a != b),
                    );
                }
            }
            (
                Constructor::Function {
                    parameters: a,
                    unsafe_: x,
                    convention: av,
                    ..
                },
                Constructor::Function {
                    parameters: b,
                    unsafe_: y,
                    convention: bv,
                    ..
                },
            ) if !record_regions && a == b && x == y && av == bv => {
                // Quantifier subsumption belongs to the directed region proof.
                // Shape inference must permit specialization of a late binder.
            }
            (x, y) if x == y => {}
            _ => return Err(Error::Mismatch),
        }
        let a = self.terms[a.0 as usize];
        let b = self.terms[b.0 as usize];
        if a.end - a.start != b.end - b.start {
            return Err(Error::Mismatch);
        }
        self.pairs.extend(
            self.edges[a.start as usize..a.end as usize]
                .iter()
                .copied()
                .zip(self.edges[b.start as usize..b.end as usize].iter().copied()),
        );
        Ok(())
    }
    /// The constructor of a root term, expanding a known type on first use.
    #[inline]
    fn materialize(&mut self, store: &Store, id: TermId) -> Result<Constructor, Error> {
        match self.terms[id.0 as usize].value {
            Value::Construct(kind) => Ok(kind),
            Value::Known(ty) => self.materialize_known(store, id, ty),
            Value::Variable(_) => Err(Error::InvalidIdentity),
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive constructor dispatch keeps canonical children and inference arity together"
    )]
    fn materialize_known(
        &mut self,
        store: &Store,
        id: TermId,
        ty: TypeId,
    ) -> Result<Constructor, Error> {
        let mut children = std::mem::take(&mut self.child_scratch);
        children.clear();
        let constructor = match store.kind(ty).ok_or(Error::InvalidIdentity)? {
            Kind::Closure {
                identity,
                signature,
                environment,
                arguments,
            } => {
                children.push(self.known(store, signature)?);
                children.push(self.known(store, environment)?);
                Constructor::Closure(
                    identity,
                    self.materialize_arguments(store, arguments, &mut children)?,
                )
            }
            Kind::Never => Constructor::Never,
            Kind::Opaque(id, args) => {
                Constructor::Opaque(id, self.materialize_arguments(store, args, &mut children)?)
            }
            kind @ Kind::Dynamic { .. } => self.object_constructor(store, kind, &mut children)?,
            Kind::Primitive(p) => Constructor::Primitive(p),
            Kind::Parameter(id) => Constructor::Parameter(id),
            Kind::Projection(value) => self.materialize_projection(store, value, &mut children)?,
            Kind::Adt(item, args) => {
                if store.region_args(args).is_some() {
                    Constructor::Adt(item, args)
                } else {
                    Constructor::AdtArgs(
                        item,
                        self.materialize_arguments(store, args, &mut children)?,
                    )
                }
            }
            Kind::Tuple(list) => {
                for ty in store.list(list).ok_or(Error::InvalidIdentity)? {
                    children.push(self.known(store, *ty)?);
                }
                Constructor::Tuple
            }
            Kind::Array { element, length } => {
                children.push(self.known(store, element)?);
                Constructor::Array(length)
            }
            Kind::ArrayTerm { element, length } => {
                children.push(self.known(store, element)?);
                Constructor::ArrayTerm(length)
            }
            Kind::Slice(element) => {
                children.push(self.known(store, element)?);
                Constructor::Slice
            }
            Kind::Reference {
                mutable,
                region,
                referent,
            } => {
                children.push(self.known(store, referent)?);
                Constructor::Reference { mutable, region }
            }
            Kind::Pointer { mutable, pointee } => {
                children.push(self.known(store, pointee)?);
                Constructor::Pointer { mutable }
            }
            Kind::Function {
                bound_regions,
                parameters,
                result,
                unsafe_,
                convention,
            } => {
                let parameters = store.list(parameters).ok_or(Error::InvalidIdentity)?;
                for ty in parameters {
                    children.push(self.known(store, *ty)?);
                }
                children.push(self.known(store, result)?);
                Constructor::Function {
                    bound_regions,
                    parameters: u32::try_from(parameters.len()).map_err(|_| Error::Overflow)?,
                    unsafe_,
                    convention,
                }
            }
            Kind::FunctionItem {
                item,
                signature,
                arguments,
            } => {
                children.push(self.known(store, signature)?);
                if store.region_args(arguments).is_some() {
                    Constructor::FunctionItem(item, arguments)
                } else {
                    Constructor::FunctionItemArgs(
                        item,
                        self.materialize_arguments(store, arguments, &mut children)?,
                    )
                }
            }
        };
        let (start, end) = self.append_edges(&children)?;
        let term = &mut self.terms[id.0 as usize];
        term.value = Value::Construct(constructor);
        term.start = start;
        term.end = end;
        self.child_scratch = children;
        self.remember_settled(store, id, ty)?;
        Ok(constructor)
    }
}

pub struct Solved {
    /// One canonical type per original inference term, including unified aliases.
    pub types: Vec<TypeId>,
    /// Defining-use equations only; capture equality remains in the region proof.
    pub defining_opaques: HashMap<TypeId, TypeId>,
    pub region_equalities: Vec<(RegionId, RegionId)>,
    pub steps: usize,
}
impl Context {
    /// Default unconstrained numeric variables and publish canonical types once.
    /// Traversal and destruction remain iterative for deeply nested types.
    ///
    /// # Errors
    /// Rejects unconstrained general variables, invalid identities and work/arena limits.
    pub fn finish(mut self, store: &mut Store) -> Result<Solved, Error> {
        self.solve_types(store)
    }
    fn solve_types(&mut self, store: &mut Store) -> Result<Solved, Error> {
        if self.has_deferred_equations() {
            return Err(Error::Normalization);
        }
        let mut resolved = vec![None; self.terms.len()];
        let mut state = vec![0_u8; self.terms.len()];
        let mut work = Vec::new();
        let mut children = Vec::new();
        for ordinal in 0..self.terms.len() {
            let id = TermId(u32::try_from(ordinal).map_err(|_| Error::Overflow)?);
            work.push((id, false));
            while let Some((id, exiting)) = work.pop() {
                self.step()?;
                let id = self.root(id)?;
                if resolved[id.0 as usize].is_some() {
                    continue;
                }
                let term = self.terms[id.0 as usize];
                if exiting {
                    children.clear();
                    for index in term.start..term.end {
                        let child = self.root(self.edges[index as usize])?;
                        children.push(resolved[child.0 as usize].ok_or(Error::InvalidIdentity)?);
                    }
                    let Value::Construct(kind) = term.value else {
                        return Err(Error::InvalidIdentity);
                    };
                    let canonical = self.canonical(store, kind, &children)?;
                    resolved[id.0 as usize] = Some(canonical);
                    state[id.0 as usize] = 2;
                } else {
                    match term.value {
                        Value::Known(ty) => resolved[id.0 as usize] = Some(ty),
                        Value::Variable(domain) => {
                            resolved[id.0 as usize] = Some(match domain {
                                Domain::Integer => TypeId::primitive(Primitive::I32),
                                Domain::Float => TypeId::primitive(Primitive::F64),
                                Domain::Any => return Err(Error::Unresolved),
                            });
                        }
                        // A materialized canonical subtree keeps its identity;
                        // rebuilding it would only re-find the same type.
                        Value::Construct(_) if let Some(ty) = self.settled_type(id) => {
                            resolved[id.0 as usize] = Some(ty);
                        }
                        Value::Construct(_) => {
                            if state[id.0 as usize] == 1 {
                                return Err(Error::Occurs);
                            }
                            state[id.0 as usize] = 1;
                            work.push((id, true));
                            for child in self.edges[term.start as usize..term.end as usize]
                                .iter()
                                .rev()
                            {
                                work.push((*child, false));
                            }
                        }
                    }
                }
            }
            let root = self.root(id)?;
            resolved[ordinal] = resolved[root.0 as usize];
        }
        let types: Vec<_> = resolved
            .into_iter()
            .map(|ty| ty.ok_or(Error::InvalidIdentity))
            .collect::<Result<_, _>>()?;
        Ok(Solved {
            defining_opaques: self.defining_types(&types),
            types,
            region_equalities: std::mem::take(&mut self.region_equalities),
            steps: self.steps,
        })
    }
}
fn canonical(
    store: &mut Store,
    constructor: Constructor,
    children: &[TypeId],
) -> Result<TypeId, Error> {
    let first = || children.first().copied().ok_or(Error::InvalidIdentity);
    let kind = match constructor {
        Constructor::ArrayInfer(_)
        | Constructor::Projection(_)
        | Constructor::Opaque(..)
        | Constructor::Dynamic { .. }
        | Constructor::AdtArgs(..)
        | Constructor::FunctionItemArgs(..)
        | Constructor::Closure(..) => return Err(Error::InvalidIdentity),
        Constructor::Primitive(p) => Kind::Primitive(p),
        Constructor::Parameter(id) => Kind::Parameter(id),
        Constructor::Adt(item, args) => Kind::Adt(item, args),
        Constructor::Never => Kind::Never,
        Constructor::Tuple => {
            Kind::Tuple(store.intern_list(children).map_err(|_| Error::Overflow)?)
        }
        Constructor::Array(length) => Kind::Array {
            element: first()?,
            length,
        },
        Constructor::ArrayTerm(length) => Kind::ArrayTerm {
            element: first()?,
            length,
        },
        Constructor::Slice => Kind::Slice(first()?),
        Constructor::Reference { mutable, region } => Kind::Reference {
            mutable,
            region,
            referent: first()?,
        },
        Constructor::Pointer { mutable } => Kind::Pointer {
            mutable,
            pointee: first()?,
        },
        Constructor::Function {
            bound_regions,
            parameters,
            unsafe_,
            convention,
        } => {
            let parameters = parameters as usize;
            Kind::Function {
                bound_regions,
                parameters: store
                    .intern_list(children.get(..parameters).ok_or(Error::InvalidIdentity)?)
                    .map_err(|_| Error::Overflow)?,
                result: *children.get(parameters).ok_or(Error::InvalidIdentity)?,
                unsafe_,
                convention,
            }
        }
        Constructor::FunctionItem(item, arguments) => Kind::FunctionItem {
            item,
            arguments,
            signature: first()?,
        },
    };
    store.intern(kind).map_err(|_| Error::InvalidIdentity)
}
