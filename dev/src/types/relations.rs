//! Directed type relations use actual region arguments and explicit binder scopes.
mod ancestry;
mod equivalence;
pub(crate) use equivalence::RegionEqualities;
mod clauses;
mod identity_guard;
mod membership;
mod projected_bounds;
mod queries;
mod scopes;
use super::structural;
mod type_bounds;
mod universal_bounds;
use super::region_graph::{Error, Graph, Kind as RegionKind, Region, Solution};
use crate::support::radix_map as persistent;
use crate::types::{
    Kind, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::{ParameterId, ParameterKind},
    variance::{Variance, Variances},
};
pub use scopes::ProofScope;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Scope(usize);
struct Binder {
    depth: u16,
    jump_start: usize,
    parent: Option<Scope>,
    start: usize,
    end: usize,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Relation {
    actual: TypeId,
    expected: TypeId,
    left: Scope,
    right: Scope,
    unsize: bool,
    reborrow: bool,
}
/// Body-level identities for the retained graph. Nested callable placeholders
/// remain graph-local and cannot be mistaken for named body lifetimes.
pub struct Identities {
    variables: HashMap<RegionId, Region>,
    universals: Vec<Region>,
    parameters: HashMap<ParameterId, Region>,
}
impl Identities {
    #[must_use]
    pub fn parameters(&self) -> impl ExactSizeIterator<Item = (ParameterId, Region)> + '_ {
        self.parameters.iter().map(|(&id, &region)| (id, region))
    }
    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<Region> {
        self.parameters.get(&id).copied()
    }
    #[must_use]
    pub fn universals(&self) -> impl ExactSizeIterator<Item = Region> + '_ {
        self.universals.iter().copied()
    }
    /// Unordered semantic inference identities; inspection sorts these numeric keys.
    #[must_use]
    pub fn variables(&self) -> impl ExactSizeIterator<Item = (RegionId, Region)> + '_ {
        self.variables.iter().map(|(&id, &region)| (id, region))
    }
    #[must_use]
    pub fn get(&self, id: RegionId) -> Option<Region> {
        if id == RegionId::STATIC {
            return Some(Region::STATIC);
        }
        match id.bound_parts() {
            Some((0, index)) => self.universals.get(usize::from(index)).copied(),
            Some(_) => None,
            None => self.variables.get(&id).copied(),
        }
    }
}
pub struct Proof {
    pub solution: Solution,
    pub identities: Identities,
}
/// One contract, one immutable type store and one completed nominal variance table.
pub struct Constraints<'a> {
    types: &'a Store,
    variances: &'a Variances,
    graph: Graph,
    projected: Option<Box<projected_bounds::State>>,
    variables: HashMap<RegionId, Region>,
    parameters: HashMap<ParameterId, Region>,
    binders: Vec<Binder>,
    ancestors: Vec<Scope>,
    arguments: Vec<Region>,
    universe: u32,
    seen: HashSet<Relation>,
    work: Vec<Relation>,
    steps: usize,
    limit: usize,
}
impl<'a> Constraints<'a> {
    /// Assume the well-formedness requirements of a declared function signature.
    /// # Errors
    /// Rejects missing type/parameter identities and exhausted query resources.
    pub fn assume_signature(
        &mut self,
        queries: &mut super::outlives::Queries<'_>,
        signature: TypeId,
    ) -> Result<(), String> {
        let Some(Kind::Function {
            parameters, result, ..
        }) = self.types.kind(signature)
        else {
            return Err("missing lifetime function signature".into());
        };
        let parameters = self
            .types
            .list(parameters)
            .ok_or("missing lifetime function parameters")?;
        for ty in parameters.iter().copied().chain(std::iter::once(result)) {
            self.assume_well_formed(queries, ty)?;
        }
        Ok(())
    }
    /// Assume only the lifetime requirements implied by a declared input type.
    /// # Errors
    /// Rejects missing type/parameter identities and exhausted query resources.
    pub fn assume_well_formed(
        &mut self,
        queries: &mut super::outlives::Queries<'_>,
        ty: TypeId,
    ) -> Result<(), String> {
        self.assume_requirements(queries.symbolic_requirements(ty)?)?;
        Ok(())
    }
    /// Add the components of an explicit type-outlives assumption to this environment.
    /// # Errors
    /// Rejects missing type/parameter identities and exhausted query resources.
    pub fn assume_type_outlives(
        &mut self,
        queries: &mut super::outlives::Queries<'_>,
        ty: TypeId,
        shorter: RegionId,
    ) -> Result<(), String> {
        let parts = queries.components(ty)?;
        for &longer in parts.scalars.regions {
            self.assume(longer, shorter).map_err(|e| e.to_string())?;
        }
        for &parameter in parts.scalars.parameters {
            self.assume_parameter(parameter, shorter)
                .map_err(|e| e.to_string())?;
        }
        for &projection in parts.projections {
            self.assume_projection(projection, shorter)?;
        }
        Ok(())
    }
    /// # Errors
    /// Rejects an oversized body parameter environment.
    pub fn new(
        types: &'a Store,
        variances: &'a Variances,
        universals: u32,
        limit: usize,
    ) -> Result<Self, Error> {
        let mut this = Self {
            types,
            variances,
            graph: Graph::new(limit),
            projected: None,
            variables: HashMap::new(),
            parameters: HashMap::new(),
            binders: Vec::new(),
            ancestors: Vec::new(),
            arguments: Vec::new(),
            universe: 0,
            seen: HashSet::new(),
            work: Vec::new(),
            steps: 0,
            limit,
        };
        this.binder(None, universals, true, 0)?;
        Ok(this)
    }
    /// A rigid type parameter's lifetime components are opaque to this body.
    /// Their common outlives extent can only be constrained by stated assumptions.
    /// # Errors
    /// Rejects duplicate, unknown or const parameters and exhausted storage.
    pub fn parameter(&mut self, id: ParameterId) -> Result<(), Error> {
        if self.parameters.contains_key(&id)
            || !matches!(
                self.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { .. })
            )
        {
            return Err(Error::Identity);
        }
        let region = self.graph.region(RegionKind::Universal { universe: 0 })?;
        self.parameters.insert(id, region);
        Ok(())
    }
    /// # Errors
    /// Rejects foreign parameters, nonuniversal bounds and exhausted storage.
    pub fn assume_parameter(&mut self, id: ParameterId, shorter: RegionId) -> Result<(), Error> {
        self.graph.assume(
            *self.parameters.get(&id).ok_or(Error::Identity)?,
            self.region(shorter, Scope(0))?,
        )
    }
    /// # Errors
    /// Rejects foreign parameters, missing regions and exhausted storage.
    pub fn parameter_outlives(
        &mut self,
        id: ParameterId,
        shorter: RegionId,
        source: u32,
    ) -> Result<(), Error> {
        self.graph.require(
            *self.parameters.get(&id).ok_or(Error::Identity)?,
            self.region(shorter, Scope(0))?,
            source,
        )
    }
    /// Register only the inference variables owned by this body.
    ///
    /// # Errors
    /// Rejects duplicate/non-inference identities and exhausted storage.
    pub fn variable(&mut self, id: RegionId) -> Result<(), Error> {
        if id == RegionId::STATIC || id.bound_parts().is_some() || self.variables.contains_key(&id)
        {
            return Err(Error::Identity);
        }
        let variable = self.graph.region(RegionKind::Variable { universe: 0 })?;
        self.variables.insert(id, variable);
        // A free inference argument denotes a lifetime available in this body.
        // A younger forall placeholder cannot supply even this local extent.
        self.graph.require(variable, Region::LOCAL, 0)
    }
    /// # Errors
    /// Rejects nonuniversal bounds, missing body identities and storage limits.
    pub fn assume(&mut self, longer: RegionId, shorter: RegionId) -> Result<(), Error> {
        self.graph.assume(
            self.region(longer, Scope(0))?,
            self.region(shorter, Scope(0))?,
        )
    }
    /// # Errors
    /// Rejects missing body identities and storage limits.
    pub fn outlives(
        &mut self,
        longer: RegionId,
        shorter: RegionId,
        source: u32,
    ) -> Result<(), Error> {
        self.graph.require(
            self.region(longer, Scope(0))?,
            self.region(shorter, Scope(0))?,
            source,
        )
    }
    /// A borrow of local storage cannot be returned under a caller's lifetime.
    /// This upper bound does not prove local liveness or alias permissions.
    ///
    /// # Errors
    /// Rejects missing body identities and storage limits.
    pub fn local(&mut self, borrow: RegionId, source: u32) -> Result<(), Error> {
        self.graph
            .require(Region::LOCAL, self.region(borrow, Scope(0))?, source)
    }
    fn binder(
        &mut self,
        parent: Option<Scope>,
        count: u32,
        universal: bool,
        universe: u32,
    ) -> Result<Scope, Error> {
        let (depth, jump_start) = self.prepare_ancestry(parent)?;
        let scope = Scope(self.binders.len());
        let start = self.arguments.len();
        for _ in 0..count {
            let kind = if universal {
                RegionKind::Universal { universe }
            } else {
                RegionKind::Variable { universe }
            };
            self.arguments.push(self.graph.region(kind)?);
        }
        self.binders.push(Binder {
            depth,
            jump_start,
            parent,
            start,
            end: self.arguments.len(),
        });
        Ok(scope)
    }
    fn region(&self, id: RegionId, mut scope: Scope) -> Result<Region, Error> {
        if id == RegionId::STATIC {
            return Ok(Region::STATIC);
        }
        let Some((depth, index)) = id.bound_parts() else {
            return self.variables.get(&id).copied().ok_or(Error::Identity);
        };
        scope = self.ancestor(scope, depth)?;
        let binder = self.binders.get(scope.0).ok_or(Error::Identity)?;
        self.arguments[binder.start..binder.end]
            .get(usize::from(index))
            .copied()
            .ok_or(Error::Identity)
    }
    fn push(&mut self, relation: Relation) -> Result<(), Error> {
        if self.steps.saturating_add(self.work.len()) >= self.limit {
            return Err(Error::Budget);
        }
        self.work.push(relation);
        Ok(())
    }
    fn child(
        &mut self,
        parent: Relation,
        actual: TypeId,
        expected: TypeId,
        variance: Variance,
    ) -> Result<(), Error> {
        if matches!(variance, Variance::Covariant | Variance::Invariant) {
            self.push(Relation {
                actual,
                expected,
                unsize: false,
                reborrow: false,
                ..parent
            })?;
        }
        if matches!(variance, Variance::Contravariant | Variance::Invariant) {
            self.push(Relation {
                actual: expected,
                expected: actual,
                left: parent.right,
                right: parent.left,
                unsize: false,
                reborrow: false,
            })?;
        }
        Ok(())
    }
    fn regions(
        &mut self,
        relation: Relation,
        actual: RegionId,
        expected: RegionId,
        variance: Variance,
        source: u32,
    ) -> Result<(), Error> {
        let actual = self.region(actual, relation.left)?;
        let expected = self.region(expected, relation.right)?;
        if matches!(variance, Variance::Covariant | Variance::Invariant) {
            self.graph.require(actual, expected, source)?;
        }
        if matches!(variance, Variance::Contravariant | Variance::Invariant) {
            self.graph.require(expected, actual, source)?;
        }
        Ok(())
    }
    /// Compare canonical types without rewriting or reparsing them. Each target
    /// callable binder introduces fresh universals; source binders instantiate
    /// with variables in that universe. Captured lifetimes resolve in the parent.
    ///
    /// # Errors
    /// Rejects incompatible shapes, missing identities and exhausted work.
    pub fn coerce(
        &mut self,
        actual: TypeId,
        expected: TypeId,
        unsize: bool,
        source: u32,
    ) -> Result<(), Error> {
        self.push(Relation {
            actual,
            expected,
            unsize,
            reborrow: false,
            left: Scope(0),
            right: Scope(0),
        })?;
        self.relations(source)
    }
    /// Compare a selected built-in reference conversion, retaining mutable ancestry.
    /// Nested type relations remain ordinary subtyping, not recursive coercions.
    /// # Errors
    /// Rejects invalid dereference selections, mutability, type shape or work limits.
    pub fn reborrow(
        &mut self,
        mut actual: TypeId,
        expected: TypeId,
        dereferences: u32,
        unsize: bool,
        source: u32,
    ) -> Result<(), Error> {
        let Some(Kind::Reference {
            region: target,
            mutable: target_mutable,
            ..
        }) = self.types.kind(expected)
        else {
            return Err(Error::Identity);
        };
        if dereferences == 0 || dereferences as usize > self.limit.saturating_sub(self.steps) {
            return Err(Error::Budget);
        }
        let mut ancestors = Vec::new();
        let mut selected = actual;
        for depth in 0..dereferences {
            self.steps += 1;
            let Some(Kind::Reference {
                region,
                mutable,
                referent,
            }) = self.types.kind(actual)
            else {
                return Err(Error::Identity);
            };
            if depth == 0 && target_mutable && !mutable {
                return Err(Error::Identity);
            }
            if !mutable {
                ancestors.clear();
            }
            ancestors.push(region);
            selected = actual;
            actual = referent;
        }
        for region in ancestors {
            self.outlives(region, target, source)?;
        }
        self.push(Relation {
            actual: selected,
            expected,
            unsize,
            reborrow: true,
            left: Scope(0),
            right: Scope(0),
        })?;
        self.relations(source)
    }
    fn relations(&mut self, source: u32) -> Result<(), Error> {
        while let Some(relation) = self.work.pop() {
            self.steps += 1;
            if relation.actual == relation.expected
                && (relation.left == relation.right
                    || self.types.escaping_binders(relation.actual) == Some(0))
            {
                continue;
            }
            if !self.seen.insert(relation) {
                continue;
            }
            self.relate(relation, source)?;
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive typed relation dispatch keeps variance and shape checks together"
    )]
    fn relate(&mut self, relation: Relation, source: u32) -> Result<(), Error> {
        let a = self.types.kind(relation.actual).ok_or(Error::Identity)?;
        let b = self.types.kind(relation.expected).ok_or(Error::Identity)?;
        let covariant = Variance::Covariant;
        match (a, b) {
            (Kind::Never, _) => {}
            (Kind::Primitive(x), Kind::Primitive(y)) if x == y => {}
            (
                Kind::Reference {
                    mutable: x,
                    region: a,
                    referent: left,
                },
                Kind::Reference {
                    mutable: y,
                    region: b,
                    referent: right,
                },
            ) if x == y || (relation.reborrow && x && !y) => {
                self.regions(relation, a, b, covariant, source)?;
                if relation.unsize {
                    let (Some(Kind::Array { element, .. }), Some(Kind::Slice(target))) =
                        (self.types.kind(left), self.types.kind(right))
                    else {
                        return Err(Error::Identity);
                    };
                    self.child(
                        relation,
                        element,
                        target,
                        if y { Variance::Invariant } else { covariant },
                    )?;
                } else {
                    self.child(
                        relation,
                        left,
                        right,
                        if y { Variance::Invariant } else { covariant },
                    )?;
                }
            }
            (
                Kind::Pointer {
                    mutable: x,
                    pointee: left,
                },
                Kind::Pointer {
                    mutable: y,
                    pointee: right,
                },
            ) if x == y => self.child(
                relation,
                left,
                right,
                if x { Variance::Invariant } else { covariant },
            )?,
            (
                Kind::Array {
                    element: left,
                    length: a,
                },
                Kind::Array {
                    element: right,
                    length: b,
                },
            ) if a == b => self.child(relation, left, right, covariant)?,
            (
                Kind::ArrayTerm {
                    element: left,
                    length: a,
                },
                Kind::ArrayTerm {
                    element: right,
                    length: b,
                },
            ) if a == b => self.child(relation, left, right, covariant)?,
            (Kind::Slice(left), Kind::Slice(right)) => {
                self.child(relation, left, right, covariant)?;
            }
            (Kind::Tuple(left), Kind::Tuple(right)) => {
                let a = self.types.list(left).ok_or(Error::Identity)?;
                let b = self.types.list(right).ok_or(Error::Identity)?;
                if a.len() != b.len() {
                    return Err(Error::Identity);
                }
                for (&a, &b) in a.iter().zip(b) {
                    self.child(relation, a, b, covariant)?;
                }
            }
            (Kind::Adt(x, left), Kind::Adt(y, right)) if x == y => {
                let variances = self.variances.get(x).unwrap_or(&[]);
                self.relate_arguments(relation, left, right, Some(variances), source)?;
            }
            (Kind::Opaque(x, left), Kind::Opaque(y, right)) if x == y => {
                self.relate_arguments(relation, left, right, None, source)?;
            }
            (
                Kind::Dynamic {
                    shape: a,
                    arguments: aa,
                    bound_regions: ac,
                    region: ar,
                },
                Kind::Dynamic {
                    shape: b,
                    arguments: ba,
                    bound_regions: bc,
                    region: br,
                },
            ) if a == b => {
                self.regions(relation, ar, br, covariant, source)?;
                let nested = self.nested_relation(relation, ac, bc)?;
                self.relate_arguments(nested, aa, ba, None, source)?;
            }
            (
                Kind::Closure {
                    identity: x,
                    arguments: aa,
                    signature: sa,
                    environment: ea,
                },
                Kind::Closure {
                    identity: y,
                    arguments: ba,
                    signature: sb,
                    environment: eb,
                },
            ) if x == y => {
                self.relate_arguments(relation, aa, ba, None, source)?;
                self.child(relation, sa, sb, Variance::Invariant)?;
                self.child(relation, ea, eb, Variance::Invariant)?;
            }
            (Kind::Function { .. }, Kind::Function { .. }) => self.function(relation, a, b)?,
            (Kind::FunctionItem { signature, .. }, Kind::Function { .. }) => {
                self.child(relation, signature, relation.expected, covariant)?;
            }
            (
                Kind::FunctionItem {
                    item: x,
                    signature: a,
                    ..
                },
                Kind::FunctionItem {
                    item: y,
                    signature: b,
                    ..
                },
            ) if x == y => self.child(relation, a, b, covariant)?,
            (Kind::Projection(_), Kind::Projection(_)) => {
                self.relate_projection(relation, source)?;
            }
            _ => return Err(Error::Identity),
        }
        Ok(())
    }
    fn relate_arguments(
        &mut self,
        relation: Relation,
        left: super::GenericArgs,
        right: super::GenericArgs,
        variances: Option<&[Variance]>,
        source: u32,
    ) -> Result<(), Error> {
        let a = self.types.arguments(left).ok_or(Error::Identity)?;
        let b = self.types.arguments(right).ok_or(Error::Identity)?;
        if a.len() != b.len() || variances.is_some_and(|v| v.len() != a.len()) {
            return Err(Error::Identity);
        }
        for (index, (a, b)) in a.zip(b).enumerate() {
            if self.steps >= self.limit {
                return Err(Error::Budget);
            }
            self.steps += 1;
            let variance = variances.map_or(Variance::Invariant, |v| v[index]);
            match (a, b) {
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    self.regions(relation, a, b, variance, source)?;
                }
                (Argument::Type(a), Argument::Type(b)) => self.child(relation, a, b, variance)?,
                (Argument::Const(a), Argument::Const(b)) if a == b => {}
                _ => return Err(Error::Identity),
            }
        }
        Ok(())
    }
    fn function(&mut self, relation: Relation, a: Kind, b: Kind) -> Result<(), Error> {
        let (
            Kind::Function {
                bound_regions: ac,
                parameters: ap,
                result: ar,
                unsafe_: au,
            },
            Kind::Function {
                bound_regions: bc,
                parameters: bp,
                result: br,
                unsafe_: bu,
            },
        ) = (a, b)
        else {
            return Err(Error::Identity);
        };
        if au != bu {
            return Err(Error::Identity);
        }
        let a = self.types.list(ap).ok_or(Error::Identity)?;
        let b = self.types.list(bp).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Err(Error::Identity);
        }
        let nested = self.nested_relation(relation, ac, bc)?;
        for (&a, &b) in a.iter().zip(b) {
            self.child(nested, a, b, Variance::Contravariant)?;
        }
        self.child(nested, ar, br, Variance::Covariant)
    }
    fn nested_relation(
        &mut self,
        relation: Relation,
        actual: u32,
        expected: u32,
    ) -> Result<Relation, Error> {
        self.universe = self.universe.checked_add(1).ok_or(Error::Budget)?;
        let right = self.binder(Some(relation.right), expected, true, self.universe)?;
        let left = self.binder(Some(relation.left), actual, false, self.universe)?;
        Ok(Relation {
            left,
            right,
            unsize: false,
            ..relation
        })
    }
    /// # Errors
    /// Rejects unentailed region requirements and exhausted propagation work.
    pub fn solve(mut self) -> Result<Solution, Error> {
        self.verify_projections()?;
        self.graph.solve()
    }

    /// Conditions implied by an independently justified declaration premise.
    /// This does not prove the input graph and cannot authorize a new goal.
    pub(crate) fn outer_conditions(self) -> Result<Vec<(RegionId, RegionId)>, Error> {
        if self.projected.is_some() || !self.parameters.is_empty() {
            return Err(Error::Identity);
        }
        let binder = self.binders.first().ok_or(Error::Identity)?;
        let mut names = HashMap::from([(Region::STATIC, RegionId::STATIC)]);
        for (slot, &region) in self.arguments[binder.start..binder.end].iter().enumerate() {
            let slot = u16::try_from(slot).map_err(|_| Error::Budget)?;
            names.insert(
                region,
                RegionId::bound(0, slot).map_err(|_| Error::Identity)?,
            );
        }
        self.graph
            .outer_conditions()?
            .into_iter()
            .map(|(longer, shorter)| {
                Ok((
                    *names.get(&longer).ok_or(Error::Identity)?,
                    *names.get(&shorter).ok_or(Error::Identity)?,
                ))
            })
            .collect()
    }
    /// Preserve the proved relations and body identity map for CFG lifetime use.
    /// No semantic relationship is reconstructed from rendered inspection text.
    ///
    /// # Errors
    /// Rejects unentailed requirements and exhausted propagation work.
    pub fn prove(mut self) -> Result<Proof, Error> {
        self.verify_projections()?;
        let end = self.binders.first().ok_or(Error::Identity)?.end;
        self.arguments.truncate(end);
        Ok(Proof {
            solution: self.graph.solve()?,
            identities: Identities {
                variables: self.variables,
                universals: self.arguments,
                parameters: self.parameters,
            },
        })
    }
}

impl From<persistent::Error> for Error {
    fn from(error: persistent::Error) -> Self {
        match error {
            persistent::Error::Identity => Self::Identity,
            persistent::Error::Budget => Self::Budget,
        }
    }
}
