//! Complete structural equality is shared by ordinary and quantified input checks.
mod shapes;
use crate::support::hash::HashSet;
use crate::types::{
    GenericArgs, Kind, ListId, RegionId, Store, TypeId, arguments::Argument, region_graph::Error,
};
pub(crate) use shapes::Shapes;

/// Completed normal forms may differ in lifetimes while retaining rigid shape.
/// This view grants no input equation or lifetime permission.
pub(crate) struct Normalized<'t, const SUBSUMPTION: bool> {
    types: &'t Store,
    work: Vec<(TypeId, TypeId, u32)>,
    seen: HashSet<(TypeId, TypeId)>,
    pub(crate) steps: usize,
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
        }
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
                self.seen.insert((a, b));
                continue;
            }
            if a == b || self.seen.contains(&(a, b)) {
                continue;
            }
            self.charge(1)?;
            self.work.push((a, b, FINISHED));
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
impl<'t, const SUBSUMPTION: bool> Compare<'t> for Normalized<'t, SUBSUMPTION> {
    fn types(&self) -> &'t Store {
        self.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.work.push((a, b, u32::from(depth)));
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
