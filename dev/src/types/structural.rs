//! Complete structural equality is shared by ordinary and quantified input checks.
mod shapes;
use crate::support::hash::HashSet;
use crate::types::{
    GenericArgs, Kind, ListId, RegionId, Store, TypeId, arguments::Argument, region_graph::Error,
};
pub(crate) use shapes::Shapes;

/// Completed normal forms may differ in lifetimes while retaining rigid shape.
/// This view grants no input equation or lifetime permission.
pub(crate) struct Normalized<'t, const SUBSUMPTION: bool, C: Cost = ()> {
    types: &'t Store,
    work: Vec<(TypeId, TypeId, u32)>,
    seen: HashSet<(TypeId, TypeId)>,
    pub(crate) steps: usize,
    cost: C,
}
// All real depths are u16. A finish marker uses the existing tuple padding,
// preserving the12-byte work record and publishing only completed DAG pairs.
const FINISHED: u32 = u32::MAX;
const _: [(); 12] = [(); size_of::<(TypeId, TypeId, u32)>()];
impl<'t, const SUBSUMPTION: bool> Normalized<'t, SUBSUMPTION> {
    pub(crate) fn new(types: &'t Store) -> Self {
        Self {
            types,
            work: Vec::new(),
            seen: HashSet::default(),
            steps: 0,
            cost: (),
        }
    }
    /// Reuse the same completed-pair memo within one current-store query.
    pub(crate) fn new_charged<'p>(
        types: &'t Store,
        payment: crate::support::retained::Payment<'p>,
        memo: Memo,
    ) -> Normalized<'t, SUBSUMPTION, PaidCost<'p>> {
        Normalized {
            types,
            work: memo.work,
            seen: memo.seen,
            steps: memo.steps,
            cost: PaidCost {
                payment,
                failure: None,
            },
        }
    }
}
impl<const SUBSUMPTION: bool, C: Cost> Normalized<'_, SUBSUMPTION, C> {
    pub(crate) fn into_memo(self) -> Memo {
        Memo {
            work: self.work,
            seen: self.seen,
            steps: self.steps,
        }
    }
    fn push(&mut self, item: (TypeId, TypeId, u32)) -> Result<(), Error> {
        if C::PAID {
            let growth = crate::support::retained::Growth::vector::<(TypeId, TypeId, u32)>(
                self.work.len(),
                self.work.capacity(),
                1,
            );
            self.cost.charge(growth.work, growth.bytes)?;
            self.work.try_reserve(1).map_err(|_| Error::Budget)?;
        }
        self.work.push(item);
        Ok(())
    }
    pub(crate) fn require(&mut self, actual: TypeId, expected: TypeId) -> Result<(), Error> {
        if self.matches(actual, expected)? {
            Ok(())
        } else {
            Err(Error::Identity)
        }
    }
    /// A completed rigid mismatch is distinct from malformed type identities
    /// or exhausted comparison work. The caller supplies its semantic diagnostic.
    pub(crate) fn matches(&mut self, actual: TypeId, expected: TypeId) -> Result<bool, Error> {
        let result = self.compare_all(actual, expected);
        if !matches!(result, Ok(true)) {
            self.work.clear();
        }
        result
    }
    fn compare_all(&mut self, actual: TypeId, expected: TypeId) -> Result<bool, Error> {
        self.child(actual, expected, 0)?;
        while let Some((a, b, depth)) = self.work.pop() {
            self.charge(1)?;
            if depth == FINISHED {
                if C::PAID && !self.seen.contains(&(a, b)) {
                    let growth = crate::support::retained::Growth::map::<(TypeId, TypeId), ()>(
                        self.seen.len(),
                        self.seen.capacity(),
                        1,
                    );
                    self.cost.charge(growth.work, growth.bytes)?;
                    self.seen.try_reserve(1).map_err(|_| Error::Budget)?;
                }
                self.seen.insert((a, b));
                continue;
            }
            if a == b || self.seen.contains(&(a, b)) {
                continue;
            }
            self.charge(1)?;
            self.push((a, b, FINISHED))?;
            if !self.compare(a, b, u16::try_from(depth).map_err(|_| Error::Identity)?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/mismatch_answers.rs"]
mod mismatch_tests;
impl<'t, const SUBSUMPTION: bool, C: Cost> Compare<'t> for Normalized<'t, SUBSUMPTION, C> {
    fn types(&self) -> &'t Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err(Error::Budget);
        }
        self.cost.charge(count, 0)
    }
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.push((a, b, u32::from(depth)))?;
        Ok(())
    }
    fn regions(&mut self, _: RegionId, _: RegionId, _: u16) -> Result<bool, Error> {
        Ok(true)
    }
    fn callable_binders(&self, left: u32, right: u32) -> bool {
        SUBSUMPTION || left == right
    }
}

pub(crate) trait Compare<'t> {
    fn types(&self) -> &'t Store;
    fn charge(&mut self, count: usize) -> Result<(), Error>;
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error>;
    fn regions(&mut self, a: RegionId, b: RegionId, depth: u16) -> Result<bool, Error>;
    /// Shape-only method checks defer callable quantifier subsumption to their
    /// directed lifetime proof. Ordinary structural identity keeps exact arity.
    fn callable_binders(&self, left: u32, right: u32) -> bool {
        left == right
    }
    fn arguments(&mut self, a: GenericArgs, b: GenericArgs, depth: u16) -> Result<bool, Error> {
        self.compare_arguments(a, b, depth)
    }
    fn compare_arguments(
        &mut self,
        a: GenericArgs,
        b: GenericArgs,
        depth: u16,
    ) -> Result<bool, Error> {
        let a = self.types().arguments(a).ok_or(Error::Identity)?;
        let b = self.types().arguments(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.zip(b) {
            self.charge(1)?;
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => self.child(a, b, depth)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if !self.regions(a, b, depth)? {
                        return Ok(false);
                    }
                }
                (Argument::Const(a), Argument::Const(b)) if a == b => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    fn list(&mut self, a: ListId, b: ListId, depth: u16) -> Result<bool, Error> {
        self.compare_list(a, b, depth)
    }
    fn compare_list(&mut self, a: ListId, b: ListId, depth: u16) -> Result<bool, Error> {
        let a = self.types().list(a).ok_or(Error::Identity)?;
        let b = self.types().list(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (&a, &b) in a.iter().zip(b) {
            self.child(a, b, depth)?;
        }
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive structural dispatch keeps conditional identity distinct from coercion"
    )]
    fn compare(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<bool, Error> {
        let a = self.types().kind(a).ok_or(Error::Identity)?;
        let b = self.types().kind(b).ok_or(Error::Identity)?;
        match (a, b) {
            (
                Kind::Closure {
                    identity: x,
                    arguments: a,
                    signature: sa,
                    environment: ea,
                },
                Kind::Closure {
                    identity: y,
                    arguments: b,
                    signature: sb,
                    environment: eb,
                },
            ) if x == y => {
                self.child(sa, sb, depth)?;
                self.child(ea, eb, depth)?;
                return self.arguments(a, b, depth);
            }
            (Kind::Never, Kind::Never) => {}
            (Kind::Foreign(a), Kind::Foreign(b)) if a == b => {}
            (Kind::Primitive(a), Kind::Primitive(b)) if a == b => {}
            (Kind::Parameter(a), Kind::Parameter(b)) if a == b => {}
            (
                Kind::Reference {
                    region: a,
                    mutable: x,
                    referent: left,
                },
                Kind::Reference {
                    region: b,
                    mutable: y,
                    referent: right,
                },
            ) if x == y => {
                if !self.regions(a, b, depth)? {
                    return Ok(false);
                }
                self.child(left, right, depth)?;
            }
            (
                Kind::Pointer {
                    mutable: x,
                    pointee: a,
                },
                Kind::Pointer {
                    mutable: y,
                    pointee: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::Array {
                    length: x,
                    element: a,
                },
                Kind::Array {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::ArrayTerm {
                    length: x,
                    element: a,
                },
                Kind::ArrayTerm {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::Pattern {
                    base: a,
                    constraint: x,
                },
                Kind::Pattern {
                    base: b,
                    constraint: y,
                },
            ) if x == y => self.child(a, b, depth)?,
            (Kind::Slice(a), Kind::Slice(b)) => self.child(a, b, depth)?,
            (Kind::Tuple(a), Kind::Tuple(b)) => return self.list(a, b, depth),
            (Kind::Adt(x, a), Kind::Adt(y, b)) if x == y => return self.arguments(a, b, depth),
            (Kind::Opaque(x, a), Kind::Opaque(y, b)) if x == y => {
                return self.arguments(a, b, depth);
            }
            (
                Kind::Future {
                    identity: x,
                    arguments: a,
                },
                Kind::Future {
                    identity: y,
                    arguments: b,
                },
            ) if x == y => return self.arguments(a, b, depth),
            (
                Kind::Dynamic {
                    shape: a,
                    arguments: aa,
                    bound_regions: ab,
                    region: ar,
                },
                Kind::Dynamic {
                    shape: b,
                    arguments: ba,
                    bound_regions: bb,
                    region: br,
                },
            ) if a == b && ab == bb => {
                if !self.regions(ar, br, depth)? {
                    return Ok(false);
                }
                return self.arguments(aa, ba, depth.checked_add(1).ok_or(Error::Budget)?);
            }
            (
                Kind::Function {
                    bound_regions: x,
                    parameters: a,
                    result: ar,
                    unsafe_: au,
                    convention: av,
                },
                Kind::Function {
                    bound_regions: y,
                    parameters: b,
                    result: br,
                    unsafe_: bu,
                    convention: bv,
                },
            ) if self.callable_binders(x, y) && au == bu && av == bv => {
                let depth = depth.checked_add(1).ok_or(Error::Budget)?;
                self.child(ar, br, depth)?;
                return self.list(a, b, depth);
            }
            (
                Kind::FunctionItem {
                    item: x,
                    arguments: a,
                    signature: ar,
                },
                Kind::FunctionItem {
                    item: y,
                    arguments: b,
                    signature: br,
                },
            ) if x == y => {
                self.child(ar, br, depth)?;
                return self.arguments(a, b, depth);
            }
            (Kind::Projection(a), Kind::Projection(b)) if a.item == b.item => {
                let at = self
                    .types()
                    .trait_application(a.trait_application)
                    .ok_or(Error::Identity)?;
                let bt = self
                    .types()
                    .trait_application(b.trait_application)
                    .ok_or(Error::Identity)?;
                if at.item != bt.item {
                    return Ok(false);
                }
                self.child(a.subject, b.subject, depth)?;
                return Ok(self.arguments(at.arguments, bt.arguments, depth)?
                    && self.arguments(a.arguments, b.arguments, depth)?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

/// Query-local candidate data only; the caller retains the actual current store.
/// It does not carry any input equation, region assumption or declaration proof.
#[derive(Default)]
pub(crate) struct Memo {
    work: Vec<(TypeId, TypeId, u32)>,
    seen: HashSet<(TypeId, TypeId)>,
    steps: usize,
}
pub(crate) trait Cost {
    const PAID: bool;
    fn charge(&mut self, work: usize, bytes: usize) -> Result<(), Error>;
}
impl Cost for () {
    const PAID: bool = false;
    fn charge(&mut self, _: usize, _: usize) -> Result<(), Error> {
        Ok(())
    }
}
pub(crate) struct PaidCost<'p> {
    payment: crate::support::retained::Payment<'p>,
    failure: Option<&'static str>,
}
impl Cost for PaidCost<'_> {
    const PAID: bool = true;
    fn charge(&mut self, work: usize, bytes: usize) -> Result<(), Error> {
        if self.failure.is_some() {
            return Err(Error::Budget);
        }
        if let Err(message) = (self.payment)(work, bytes) {
            self.failure = Some(message);
            return Err(Error::Budget);
        }
        Ok(())
    }
}
impl<const SUBSUMPTION: bool> Normalized<'_, SUBSUMPTION, PaidCost<'_>> {
    pub(crate) fn matches_charged(
        &mut self,
        actual: TypeId,
        expected: TypeId,
    ) -> Result<bool, &'static str> {
        self.matches(actual, expected).map_err(|error| {
            self.cost.failure.unwrap_or(match error {
                Error::Budget => "complete structural comparison work budget exceeded",
                _ => "invalid complete structural comparison identity",
            })
        })
    }
}
const _: [(); 72] = [(); size_of::<Normalized<'_, false>>()];
const _: [(); 104] = [(); size_of::<Normalized<'_, false, PaidCost<'_>>>()];
const _: [(); 64] = [(); size_of::<Memo>()];
