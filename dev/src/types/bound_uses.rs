//! Collect occurrences of one enclosing binder from already elaborated types.
pub(crate) mod captures;
use super::{GenericArgs, Kind, RegionId, Store, TypeId, arguments::Argument};
use std::collections::HashSet;

/// Validate a signature before normalizing its projections. Projected input
/// occurrences do not constrain output lifetimes, even if their normal form would.
/// Captured outer lifetimes are distinct from this binder. Repeat after local
/// type inference to include references introduced by inferred annotations.
///
/// # Errors
/// Rejects malformed binders and output-only late lifetimes.
pub fn check_pointer(store: &Store, signature: TypeId) -> Result<(), &'static str> {
    let projected = store.contains_projections(signature) == Some(true);
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        ..
    }) = store.kind(signature)
    else {
        return Err("missing function pointer signature");
    };
    let parameters = store
        .list(parameters)
        .ok_or("missing function pointer inputs")?;
    let mut inputs = Uses::constraining();
    for ty in parameters {
        inputs.collect(store, *ty)?;
    }
    let mut output = Uses::default();
    output.collect(store, result)?;
    if !output.indices.is_subset(&inputs.indices) {
        return Err("function pointer output lifetime is unconstrained by its inputs");
    }
    if projected {
        // Even opaque input positions must refer to valid binder slots. Reuse
        // output's occurrence walk only after testing the output constraint.
        for ty in parameters {
            output.collect(store, *ty)?;
        }
    }
    if inputs
        .indices
        .iter()
        .chain(&output.indices)
        .any(|index| u32::from(*index) >= bound_regions)
    {
        return Err("function pointer references an unbound lifetime");
    }
    Ok(())
}

#[derive(Default)]
pub struct Uses {
    opaque_projections: bool,
    free: Option<HashSet<RegionId>>,
    pub indices: HashSet<u16>,
    seen: HashSet<(TypeId, u16)>,
    applications: HashSet<(super::traits::ApplicationId, u16)>,
    work: Vec<(TypeId, u16)>,
    pub steps: usize,
}
impl Uses {
    pub(crate) fn including_free() -> Self {
        Self {
            free: Some(HashSet::new()),
            ..Self::default()
        }
    }

    pub(crate) fn free_regions(&self) -> impl Iterator<Item = RegionId> + '_ {
        self.free.iter().flat_map(|regions| regions.iter().copied())
    }

    fn contains_uses(&self, store: &Store, ty: TypeId, depth: u16) -> Result<bool, &'static str> {
        if self.free.is_some() {
            store
                .has_free_regions(ty, depth)
                .ok_or("invalid lifetime-use type")
        } else {
            store
                .escaping_binders(ty)
                .map(|escaping| escaping > depth)
                .ok_or("invalid bound-use type")
        }
    }

    /// Count only positions that determine a lifetime, excluding projection inputs.
    #[must_use]
    pub fn constraining() -> Self {
        Self {
            opaque_projections: true,
            ..Self::default()
        }
    }
    /// Accumulate several roots from the same binder and canonical store.
    /// Nested binders and alias-erased arguments do not constrain this binder.
    ///
    /// # Errors
    /// Rejects invalid canonical data and exhausted traversal resources.
    pub fn collect(&mut self, store: &Store, root: TypeId) -> Result<(), &'static str> {
        self.collect_inside(store, root, 0)
    }
    fn collect_inside(
        &mut self,
        store: &Store,
        root: TypeId,
        depth: u16,
    ) -> Result<(), &'static str> {
        if !self.contains_uses(store, root, depth)? {
            return Ok(());
        }
        self.push(root, depth)?;
        self.drain(store)
    }
    /// Include direct outlives regions and lifetimes appearing only in trait arguments.
    /// # Errors
    /// Rejects malformed predicates and exhausted traversal resources.
    pub fn predicate(
        &mut self,
        store: &Store,
        predicate: super::parameters::predicates::Predicate,
    ) -> Result<(), &'static str> {
        self.predicate_inside(store, predicate, 0)
    }
    /// Collect only the enclosing binder's uses, excluding quantified locals.
    /// # Errors
    /// Rejects malformed clauses and exhausted occurrence work.
    pub fn clause(
        &mut self,
        store: &Store,
        clause: super::parameters::clauses::Clause,
    ) -> Result<(), &'static str> {
        use super::parameters::clauses::Clause;
        match clause {
            Clause::Predicate(predicate) => self.predicate_inside(store, predicate, 0),
            Clause::ForAll(id) => {
                let value = store.quantified(id).ok_or("invalid bound-use clause")?;
                self.predicate_inside(store, value.predicate, 1)
            }
        }
    }
    fn predicate_inside(
        &mut self,
        store: &Store,
        predicate: super::parameters::predicates::Predicate,
        depth: u16,
    ) -> Result<(), &'static str> {
        use super::parameters::predicates::Bound;
        self.collect_inside(store, predicate.ty, depth)?;
        match predicate.bound {
            Bound::ProjectionEqual(result) => self.collect_inside(store, result, depth)?,
            Bound::Outlives(region) => self.region(region, depth),
            Bound::Trait(id) => {
                let application = store
                    .trait_application(id)
                    .ok_or("invalid bound-use trait application")?;
                if self.applications.insert((id, depth)) {
                    self.arguments(store, application.arguments, depth)?;
                    self.drain(store)?;
                }
            }
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) => {}
        }
        Ok(())
    }
    fn drain(&mut self, store: &Store) -> Result<(), &'static str> {
        while let Some((ty, depth)) = self.work.pop() {
            self.steps += 1;
            if !self.contains_uses(store, ty, depth)? || !self.seen.insert((ty, depth)) {
                continue;
            }
            match store.kind(ty).ok_or("invalid bound-use type")? {
                Kind::Reference {
                    region, referent, ..
                } => {
                    self.region(region, depth);
                    self.push(referent, depth)?;
                }
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    self.arguments(store, arguments, depth)?;
                    self.push(signature, depth)?;
                    self.push(environment, depth)?;
                }
                Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                    self.arguments(store, arguments, depth)?;
                }
                Kind::Dynamic {
                    arguments, region, ..
                } => {
                    self.region(region, depth);
                    let nested = depth
                        .checked_add(1)
                        .ok_or("bound-use binder depth overflow")?;
                    self.arguments(store, arguments, nested)?;
                }
                Kind::Projection(application) => {
                    if self.opaque_projections {
                        continue;
                    }
                    self.push(application.subject, depth)?;
                    let trait_ = store
                        .trait_application(application.trait_application)
                        .ok_or("invalid bound-use projection trait")?;
                    self.arguments(store, trait_.arguments, depth)?;
                    self.arguments(store, application.arguments, depth)?;
                }
                Kind::Tuple(list) => {
                    for ty in store.list(list).ok_or("invalid bound-use tuple")? {
                        self.push(*ty, depth)?;
                    }
                }
                Kind::Array { element: ty, .. }
                | Kind::ArrayTerm { element: ty, .. }
                | Kind::Slice(ty)
                | Kind::Pointer { pointee: ty, .. } => self.push(ty, depth)?,
                Kind::FunctionItem {
                    signature,
                    arguments,
                    ..
                } => {
                    self.arguments(store, arguments, depth)?;
                    self.push(signature, depth)?;
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or("bound-use binder depth overflow")?;
                    for ty in store
                        .list(parameters)
                        .ok_or("invalid bound-use signature")?
                    {
                        self.push(*ty, depth)?;
                    }
                    self.push(result, depth)?;
                }
                Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        store: &Store,
        args: GenericArgs,
        depth: u16,
    ) -> Result<(), &'static str> {
        let arguments = store.arguments(args).ok_or("invalid bound-use arguments")?;
        self.reserve(arguments.len())?;
        self.steps += arguments.len();
        for argument in arguments {
            match argument {
                Argument::Lifetime(region) => self.region(region, depth),
                Argument::Type(ty) => self.push(ty, depth)?,
                Argument::Const(_) => {}
            }
        }
        Ok(())
    }
    fn reserve(&self, amount: usize) -> Result<(), &'static str> {
        if self
            .steps
            .saturating_add(self.work.len())
            .saturating_add(amount)
            > 1_000_000
        {
            return Err("bound lifetime occurrence work budget exceeded");
        }
        Ok(())
    }
    fn push(&mut self, ty: TypeId, depth: u16) -> Result<(), &'static str> {
        self.reserve(1)?;
        self.work.push((ty, depth));
        Ok(())
    }
    fn region(&mut self, region: RegionId, depth: u16) {
        if region != RegionId::STATIC
            && region.bound_parts().is_none()
            && let Some(free) = &mut self.free
        {
            free.insert(region);
        }

        if let Some((binder, index)) = region.bound_parts()
            && binder == depth
        {
            self.indices.insert(index);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/clause_captures.rs"]
mod clause_tests;
