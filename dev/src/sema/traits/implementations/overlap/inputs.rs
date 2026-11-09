//! Existential type equations retain each header's separate lifetime premises.
use super::{Declarations, DefId, Perspective, Predicate};
use crate::sema::declaration_inputs;
use crate::support::bindings::LangItem;
use crate::types::{
    RegionId, TypeId,
    infer::{self, Context, Instantiation, TermId, arguments::Argument},
    lifetime_proof::Obligations,
    outlives::Queries,
    parameters::{ParameterKind, clauses::Clause, predicates::Bound},
    relations::Constraints,
    substitute,
};

#[derive(Default)]
pub(super) struct Inputs {
    /// Both headers' Sized parameters and Sized, Copy and Clone subjects.
    pub sized: Vec<TermId>,
    traits: Vec<infer::predicates::Predicate>,
    /// The header whose own clause each of `traits` is.
    trait_sides: Vec<Side>,
    /// Each header's own premises that imply Sized: its Sized parameters, its
    /// Sized, Copy and Clone subjects and its `Sized` clauses, written or
    /// elaborated (as `U: Copy` implies `U: Sized` through `Clone`).
    sized_premises: [Vec<TermId>; 2],
    /// Each header's own premises that imply `MetaSized`: its Sized premises
    /// and its `MetaSized` clauses, including the default of a `?Sized`
    /// parameter.
    meta_sized: [Vec<TermId>; 2],
    headers: u8,
    universals: u32,
    regions: Vec<(RegionId, RegionId)>,
    /// Each header's written type-outlives premises, with that header.
    outlives: Vec<(TermId, RegionId, Side)>,
    formed: Vec<TermId>,
}
pub(super) struct Header {
    substitution: Instantiation,
    lifetimes: substitute::Bound,
    bindings: Vec<(crate::types::parameters::ParameterId, Argument)>,
    side: Side,
}
/// One header of the compared pair.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Side {
    #[default]
    First,
    Second,
}
impl Side {
    const fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
        }
    }
    const fn other(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}
impl Inputs {
    pub fn instantiate(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        input: (Predicate, DefId),
    ) -> Result<(infer::predicates::Predicate, Header), String> {
        let (goal, mut header) = self.head(facts, context, input)?;
        self.conditions(facts, context, input.1, &mut header)?;
        Ok((goal, header))
    }

    pub fn head(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        (predicate, owner): (Predicate, DefId),
    ) -> Result<(infer::predicates::Predicate, Header), String> {
        let mut header = self.header(facts, context, owner)?;
        let predicate = header.lifetimes.predicate(&mut facts.types, predicate)?;
        context
            .account_work(header.lifetimes.steps)
            .map_err(|e| e.to_string())?;
        let goal = context
            .instantiate_predicate(&mut facts.types, predicate, &mut header.substitution)
            .map_err(|e| e.to_string())?;
        self.formed
            .extend(goal.arguments().filter_map(|arg| match arg {
                Argument::Type(ty) => Some(ty),
                _ => None,
            }));
        Ok((goal, header))
    }
    pub fn target(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        owner: DefId,
    ) -> Result<(TermId, Header), String> {
        let target = facts
            .associated
            .schema(owner)
            .ok_or("missing inherent coherence header")?
            .self_type;
        let mut header = self.header(facts, context, owner)?;
        let target = header.lifetimes.apply(&mut facts.types, target)?;
        context
            .account_work(header.lifetimes.steps)
            .map_err(|e| e.to_string())?;
        let target = context
            .instantiate_with(&mut facts.types, target, &mut header.substitution)
            .map_err(|e| e.to_string())?;
        self.formed.push(target);
        Ok((target, header))
    }
    fn header(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        owner: DefId,
    ) -> Result<Header, String> {
        let side = match self.headers {
            0 => Side::First,
            1 => Side::Second,
            _ => return Err("coherence inputs compare at most two headers".into()),
        };
        self.headers += 1;
        let ids = facts.generics.parameter_ids(&facts.types, owner)?;
        let mut bindings = Vec::with_capacity(ids.len());
        for id in ids {
            let argument = context
                .parameter_variable(&facts.types, id)
                .map_err(|e| e.to_string())?;
            if let Argument::Type(term) = argument
                && matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(ParameterKind::Type { sized: true })
                )
            {
                self.sized.push(term);
                self.sized_premises[side.index()].push(term);
                self.meta_sized[side.index()].push(term);
            }
            bindings.push((id, argument));
        }
        let substitution =
            Instantiation::new(context, &facts.types, &bindings).map_err(|e| e.to_string())?;
        let lifetimes = self.lifetimes(facts, owner)?;
        Ok(Header {
            substitution,
            lifetimes,
            bindings,
            side,
        })
    }
    /// Freeze the common intersection with actual source parameters as rigid
    /// representatives. A negative impl may not specialize an unknown input.
    pub fn freeze(
        facts: &mut Declarations,
        context: &mut Context,
        headers: &[&Header],
    ) -> Result<(), String> {
        for header in headers {
            for &(id, argument) in &header.bindings {
                context.account_work(1).map_err(|e| e.to_string())?;
                match argument {
                    Argument::Type(term)
                        if context
                            .head(&facts.types, term)
                            .map_err(|e| e.to_string())?
                            .is_none() =>
                    {
                        let ty = facts.types.intern(crate::types::Kind::Parameter(id))?;
                        let rigid = context.known(&facts.types, ty).map_err(|e| e.to_string())?;
                        context
                            .equate(&facts.types, term, rigid)
                            .map_err(|e| e.to_string())?;
                    }
                    Argument::Const(term) => match context.resolved_const(&mut facts.types, term) {
                        Err(infer::Error::Unresolved) => {
                            let value = facts
                                .types
                                .intern_const(crate::types::arguments::ConstKind::Parameter(id))?;
                            let rigid = context
                                .known_const(&facts.types, value)
                                .map_err(|e| e.to_string())?;
                            context
                                .equate_consts(term, rigid)
                                .map_err(|e| e.to_string())?;
                        }
                        Ok(_) => {}
                        Err(error) => return Err(error.to_string()),
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn excluded(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        headers: &[&Header],
        mode: crate::sema::coherence_policy::Mode,
        perspective: Perspective,
    ) -> Result<bool, String> {
        if mode.implicit() && self.has_absent_local_trait(facts, context, perspective)? {
            return Ok(true);
        }
        if mode.negative() {
            Self::freeze(facts, context, headers)?;
            if self.trait_sides.len() != self.traits.len() {
                return Err("coherence conditions lost their headers".into());
            }
            let premises = [
                self.premises(facts, context, Side::First)?,
                self.premises(facts, context, Side::Second)?,
            ];
            for (goal, side) in self.traits.iter().zip(&self.trait_sides) {
                // rustc 1.98 negates one impl's predicates under the other's
                // param_env, so only the other header's premises are assumed.
                let assumed = &premises[side.other().index()];
                if !crate::sema::traits::selection::may_overlap_with(
                    facts,
                    context,
                    goal,
                    crate::sema::coherence_policy::Mode::Strict,
                    perspective,
                    Some(assumed.negative()),
                    |facts, context, proof, sized, meta_sized| {
                        self.pay_negative(facts, context, proof, sized, meta_sized, Some(assumed))
                    },
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Materialize assumptions only after both actual heads can intersect.
    pub fn conditions(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        owner: DefId,
        header: &mut Header,
    ) -> Result<(), String> {
        self.reserve_traits(facts, context, owner)?;
        let before = header.lifetimes.steps;
        let mut lowering = None;
        for requirement in declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?
        {
            if let Clause::UnstableFeature(id) = requirement.value {
                context.account_work(1).map_err(|e| e.to_string())?;
                facts
                    .types
                    .feature_name(id)
                    .ok_or("invalid overlap feature condition")?;
                // An unavailable feature never proves two implementations disjoint.
                continue;
            }
            // rustc 1.98 evaluates a callable sugar clause, whose binder binds
            // nothing, like any other where clause. A clause whose binder binds
            // a region still proves no disjointness here.
            let Clause::Predicate(predicate) = crate::sema::traits::selection::vacuous(
                &mut facts.types,
                &mut lowering,
                requirement.value,
            )?
            else {
                continue;
            };
            if predicate.bound.trait_parts().is_some() {
                let premise = sizing_clause(facts, predicate.bound)?;
                let predicate = header.lifetimes.predicate(&mut facts.types, predicate)?;
                let goal = context
                    .instantiate_predicate(&mut facts.types, predicate, &mut header.substitution)
                    .map_err(|e| e.to_string())?;
                let side = header.side.index();
                if premise == Some(LangItem::Sized) {
                    self.sized_premises[side].push(goal.subject());
                }
                if matches!(premise, Some(LangItem::Sized | LangItem::MetaSized)) {
                    self.meta_sized[side].push(goal.subject());
                }
                self.traits.push(goal);
                self.trait_sides.push(header.side);
                continue;
            }
            if !matches!(
                predicate.bound,
                Bound::Sized | Bound::Copy | Bound::Clone | Bound::Outlives(_)
            ) {
                continue;
            }
            let predicate = header.lifetimes.predicate(&mut facts.types, predicate)?;
            let term = context
                .instantiate_with(&mut facts.types, predicate.ty, &mut header.substitution)
                .map_err(|e| e.to_string())?;
            if let Bound::Outlives(region) = predicate.bound {
                self.outlives.push((term, region, header.side));
            } else {
                self.sized.push(term);
                self.sized_premises[header.side.index()].push(term);
                self.meta_sized[header.side.index()].push(term);
            }
        }
        let lowered = lowering.map_or(0, |lowering: substitute::Bound| lowering.steps);
        context
            .account_work(
                header
                    .lifetimes
                    .steps
                    .saturating_sub(before)
                    .saturating_add(lowered),
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn reserve_traits(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        owner: DefId,
    ) -> Result<(), String> {
        let mut count = 0_usize;
        for requirement in declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?
        {
            context.account_work(1).map_err(|e| e.to_string())?;
            let value = match requirement.value {
                Clause::Predicate(value) => value,
                Clause::ForAll(id) => match facts.types.quantified(id) {
                    Some(scoped) if scoped.bound_regions == 0 => scoped.predicate,
                    Some(_) => continue,
                    None => return Err("invalid overlap quantified condition".into()),
                },
                Clause::UnstableFeature(_) => continue,
            };
            if value.bound.trait_parts().is_some() {
                count = count.saturating_add(1);
            }
        }
        if count != 0 {
            // Each reserved condition is one step of this pair's work. On regrowth
            // only the newly reserved conditions' bytes are paid, once, in the
            // source-header byte ledger; the conditions already reserved were paid.
            context.account_work(count).map_err(|e| e.to_string())?;
            // Ordinary PA20 crates have no source-header ledger. Do not create
            // one after declaration completion merely to account temporary work.
            if facts
                .must_use_contracts
                .as_deref()
                .and_then(|owner| owner.headers.as_ref())
                .is_some()
            {
                let mut bytes = if self.traits.capacity() - self.traits.len() < count {
                    count.saturating_mul(size_of::<infer::predicates::Predicate>())
                } else {
                    0
                };
                if self.trait_sides.capacity() - self.trait_sides.len() < count {
                    bytes = bytes.saturating_add(count.saturating_mul(size_of::<Side>()));
                }
                crate::sema::header_attributes::charge_attribute_retention(facts, count, bytes)
                    .map_err(|e| e.to_string())?;
            }
            self.traits
                .try_reserve(count)
                .map_err(|_| "coherence trait conditions allocation failed")?;
            self.trait_sides
                .try_reserve(count)
                .map_err(|_| "coherence trait condition headers allocation failed")?;
        }
        Ok(())
    }

    fn lifetimes(
        &mut self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<substitute::Bound, String> {
        let schema = facts
            .associated
            .schema(owner)
            .ok_or("missing coherence header schema")?;
        let mut names = Vec::with_capacity(schema.captures as usize);
        for _ in 0..schema.captures {
            let slot =
                u16::try_from(self.universals).map_err(|_| "too many coherence lifetimes")?;
            names.push(RegionId::bound(0, slot)?);
            self.universals += 1;
        }
        let mut mapping = substitute::Bound::new(names);
        for &(a, b) in &schema.bounds {
            self.regions
                .push((mapping.region(a, 0)?, mapping.region(b, 0)?));
        }
        Ok(mapping)
    }

    /// A goal whose absence `perspective` can know excludes the pair when no
    /// impl it could see holds. Open inputs still admit downstream
    /// implementations and remain possible.
    pub fn has_absent_local_trait(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        perspective: Perspective,
    ) -> Result<bool, String> {
        for goal in &self.traits {
            context.account_work(1).map_err(|e| e.to_string())?;
            if !super::absence_closed(facts, context, goal, perspective)? {
                continue;
            }
            if !crate::sema::traits::selection::may_overlap(facts, context, goal, perspective)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether `proof`'s lifetime obligations hold. Without `assumed`, as for
    /// `has_unsized` and the single-header comparisons, both headers' schema
    /// regions, written type-outlives premises and formed types are assumed,
    /// and a failed obligation is an error. A negative payment assumes only
    /// the `assumed` header's written type-outlives premises, as rustc 1.98's
    /// `resolve_regions` assumes that impl's `param_env` with no WF types and
    /// never relates two of its inference regions, and a failed obligation
    /// is not disjoint.
    pub fn prove(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        proof: &Obligations,
        assumed: Option<Side>,
    ) -> Result<bool, String> {
        if proof.is_empty() {
            return Ok(true);
        }
        let mut formed = Vec::new();
        if assumed.is_none() {
            for &term in &self.formed {
                if let Some(ty) = resolved(facts, context, term)? {
                    formed.push(ty);
                }
            }
        }
        let mut outlives = Vec::new();
        for &(term, region, side) in &self.outlives {
            if assumed.is_some_and(|assumed| assumed != side) {
                continue;
            }
            if let Some(ty) = resolved(facts, context, term)? {
                outlives.push((ty, region));
            }
        }
        let mut constraints = Constraints::new(
            &facts.types,
            &facts.lifetimes.variances,
            self.universals,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let mut parameters = crate::support::hash::HashSet::default();
        for &ty in formed
            .iter()
            .chain(outlives.iter().map(|(ty, _)| ty))
            .chain(proof.types.iter().map(|(ty, _)| ty))
        {
            for &parameter in queries.components(ty)?.scalars.parameters {
                context.account_work(1).map_err(|e| e.to_string())?;
                if parameters.insert(parameter) {
                    constraints
                        .parameter(parameter)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        if assumed.is_none() {
            for &(a, b) in &self.regions {
                constraints.assume(a, b).map_err(|e| e.to_string())?;
            }
        }
        for ty in formed {
            constraints.assume_well_formed(&mut queries, ty)?;
        }
        for (ty, region) in outlives {
            constraints.assume_type_outlives(&mut queries, ty, region)?;
        }
        proof.require(&mut constraints, &mut queries)?;
        match constraints.solve() {
            Ok(_) => Ok(true),
            Err(
                crate::types::region_graph::Error::Unsatisfied { .. }
                | crate::types::region_graph::Error::Verification { .. },
            ) if assumed.is_some() => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }

    /// Resolve one header's own Sized and `MetaSized` premises once, after the
    /// pair is frozen, sorted by identity for the negative queries' lookups.
    fn premises(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        side: Side,
    ) -> Result<Premises, String> {
        Ok(Premises {
            side,
            sized: sorted(facts, context, &self.sized_premises[side.index()])?,
            meta_sized: sorted(facts, context, &self.meta_sized[side.index()])?,
        })
    }

    /// A negative payment with `premises` assumes only that other header's
    /// Sized and `MetaSized` sets and written type-outlives premises. A rigid
    /// parameter is that header's placeholder, so no shape proves it Sized.
    /// The single-header comparisons have no premises: they pay Sized debts
    /// from their own header's Sized premises or by shape, and no `MetaSized`
    /// debt.
    pub fn pay_negative(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        proof: &Obligations,
        sized: &[TypeId],
        meta_sized: &[TypeId],
        premises: Option<&Premises>,
    ) -> Result<bool, String> {
        let Some(premises) = premises else {
            if !meta_sized.is_empty() {
                context.account_work(1).map_err(|e| e.to_string())?;
                return Ok(false);
            }
            return self.pay_own(facts, context, proof, sized);
        };
        for &ty in meta_sized {
            if !member(context, &premises.meta_sized, ty)? {
                return Ok(false);
            }
        }
        let mut shapes = None;
        for &ty in sized {
            if member(context, &premises.sized, ty)? {
                continue;
            }
            if matches!(facts.types.kind(ty), Some(crate::types::Kind::Parameter(_)))
                || !shapes
                    .get_or_insert_with(|| crate::sema::nominals::Proof::new(None))
                    .prove(
                        facts,
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        },
                    )?
            {
                return Ok(false);
            }
        }
        self.prove(facts, context, proof, Some(premises.side))
    }

    /// A single-header comparison's Sized debts are paid from its Sized
    /// premises or by shape.
    fn pay_own(
        &self,
        facts: &mut Declarations,
        context: &mut Context,
        proof: &Obligations,
        sized: &[TypeId],
    ) -> Result<bool, String> {
        if !sized.is_empty() {
            let mut assumed = crate::support::hash::HashSet::default();
            let growth =
                crate::support::retained::Growth::map::<TypeId, ()>(0, 0, self.sized.len());
            context
                .account_work(growth.work.saturating_add(growth.bytes))
                .map_err(|e| e.to_string())?;
            assumed
                .try_reserve(self.sized.len())
                .map_err(|_| "negative coherence Sized premises allocation failed")?;
            for &term in &self.sized {
                if let Some(ty) = resolved(facts, context, term)? {
                    assumed.insert(ty);
                }
            }
            let mut shapes = crate::sema::nominals::Proof::new(None);
            for &ty in sized {
                context.account_work(1).map_err(|e| e.to_string())?;
                if !assumed.contains(&ty)
                    && !shapes.prove(
                        facts,
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        },
                    )?
                {
                    return Ok(false);
                }
            }
        }
        self.prove(facts, context, proof, None)
    }
}

/// One header's own sizing premises, each sorted by identity and shared by
/// the negative queries of the other header's conditions.
pub(super) struct Premises {
    side: Side,
    sized: std::sync::Arc<[TypeId]>,
    meta_sized: std::sync::Arc<[TypeId]>,
}
impl Premises {
    fn negative(&self) -> crate::sema::traits::selection::NegativePremises<'_> {
        crate::sema::traits::selection::NegativePremises {
            sized: &self.sized,
            meta_sized: &self.meta_sized,
        }
    }
}

/// Resolve, sort and deduplicate one premise set, charged for its growth, its
/// shared copy and the sort.
fn sorted(
    facts: &mut Declarations,
    context: &mut Context,
    terms: &[TermId],
) -> Result<std::sync::Arc<[TypeId]>, String> {
    let count = terms.len();
    let growth = crate::support::retained::Growth::vector::<TypeId>(0, 0, count);
    let sorting = count.saturating_mul(count.max(1).ilog2() as usize + 1);
    // The resolved vector, then its shared copy.
    let shared = count.saturating_mul(size_of::<TypeId>());
    let work = growth.work.saturating_add(growth.bytes);
    context
        .account_work(work.saturating_add(shared).saturating_add(sorting))
        .map_err(|e| e.to_string())?;
    let mut premises = Vec::new();
    premises
        .try_reserve_exact(count)
        .map_err(|_| "negative coherence sizing premises allocation failed")?;
    for &term in terms {
        if let Some(ty) = resolved(facts, context, term)? {
            premises.push(ty);
        }
    }
    premises.sort_unstable_by_key(|ty| ty.0);
    premises.dedup();
    // Each side is shared by its conditions' queries, not copied per query.
    Ok(std::sync::Arc::from(premises))
}

/// Whether a sorted premise set has `ty`, charged for its binary search.
fn member(context: &mut Context, premises: &[TypeId], ty: TypeId) -> Result<bool, String> {
    context
        .account_work(premises.len().max(1).ilog2() as usize + 1)
        .map_err(|e| e.to_string())?;
    Ok(premises.binary_search_by_key(&ty.0, |ty| ty.0).is_ok())
}

/// The sizing kind of a clause naming an actual sizing declaration, written,
/// elaborated or the default of a `?Sized` parameter.
fn sizing_clause(facts: &Declarations, bound: Bound) -> Result<Option<LangItem>, String> {
    let Some((application, _)) = bound.trait_parts() else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let item = facts
        .types
        .trait_application(application)
        .ok_or("missing coherence premise application")?
        .item;
    Ok(catalog.sizing_kind(DefId(item.0))?)
}

fn resolved(
    facts: &mut Declarations,
    context: &mut Context,
    term: TermId,
) -> Result<Option<TypeId>, String> {
    context.account_work(1).map_err(|e| e.to_string())?;
    match context.resolved(&mut facts.types, term) {
        Ok(ty) => Ok(Some(ty)),
        Err(infer::Error::Unresolved | infer::Error::Normalization) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/negative_metasized_premises.rs"]
mod metasized_tests;
#[cfg(test)]
#[path = "../../../../../tests/invariants/coherence_premise_sides.rs"]
mod premise_sides_tests;
#[cfg(test)]
#[path = "../../../../../tests/invariants/vacuous_callable_coherence.rs"]
mod vacuous_tests;
