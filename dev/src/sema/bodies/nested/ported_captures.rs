//! Captured locals name their ported parent regions by the body's universal slots.
use super::{Checker, Error};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, RegionId, TypeId, infer::Solved, parameters::predicates::Predicate,
    substitute::Bound,
};

/// Every free region except the replaced ports keeps its identity.
const FREE_REGIONS: std::ops::Range<u32> = 0..0x8000_0000;

impl Checker<'_> {
    /// A region that a captured local's type gained in this closure body is a
    /// port: it names a parent region, which the closure's captured places see
    /// as the port's universal slot, as they see every region imported from the
    /// parent. The private region equals that slot by the incoming interface
    /// equations, so name the slot, as an imported captured local does, in
    /// every solved type of the body before publication. A use of the local
    /// then has its captured place's type.
    pub(in crate::sema::bodies) fn close_captured_ports(
        &mut self,
        solved: &mut Solved,
        predicates: &mut [Predicate],
        arguments: &mut [GenericArgs],
    ) -> Result<(), Error> {
        let source = self.state.body.node;
        let Some(mut fold) = self.captured_port_fold(solved)? else {
            return Ok(());
        };
        let mut closed = HashMap::default();
        for ty in &mut solved.types {
            *ty = close(&mut fold, &mut self.facts.types, &mut closed, *ty)
                .map_err(|e| self.error(source, e))?;
        }
        let opaques = std::mem::take(&mut solved.defining_opaques);
        for (opaque, hidden) in opaques {
            let opaque = close(&mut fold, &mut self.facts.types, &mut closed, opaque)
                .map_err(|e| self.error(source, e))?;
            let hidden = close(&mut fold, &mut self.facts.types, &mut closed, hidden)
                .map_err(|e| self.error(source, e))?;
            solved.defining_opaques.insert(opaque, hidden);
        }
        for predicate in predicates {
            *predicate = fold
                .predicate(&mut self.facts.types, *predicate)
                .map_err(|e| self.error(source, e))?;
        }
        for argument in arguments {
            *argument = fold
                .apply_arguments(&mut self.facts.types, *argument)
                .map_err(|e| self.error(source, e))?;
        }
        // A child closure already closed its captured paths into this body's
        // context, where a ported region now has its slot.
        let children: Vec<_> = self
            .state
            .closures
            .as_ref()
            .map(|scheduler| {
                self.state
                    .closure_children
                    .iter()
                    .filter_map(|&index| {
                        Some(
                            scheduler
                                .entries
                                .get(index)?
                                .payload
                                .closure()
                                .ok()?
                                .identity(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        for identity in children {
            if let Some(environment) = self.facts.closures.environments.get_mut(&identity) {
                close_formation(&mut fold, &mut self.facts.types, &mut closed, environment)
                    .map_err(|e| self.error(source, e))?;
            }
        }
        self.facts.body_work.canonical_steps += fold.steps;
        Ok(())
    }

    /// The replacement of each ported region a captured local's solved type
    /// mentions, when there is one. Bound slots map to themselves.
    fn captured_port_fold(&mut self, solved: &Solved) -> Result<Option<Bound>, Error> {
        let source = self.state.body.node;
        let Some(closure) = self.state.closures.as_ref().and_then(|scheduler| {
            let entry = scheduler.entries.get(scheduler.active?)?;
            entry.payload.closure().ok()
        }) else {
            return Ok(None);
        };
        let interface = &closure.lifetimes;
        let Some(ports) = interface.outputs.as_ref() else {
            return Ok(None);
        };
        let mut observe = Bound::observe_regions();
        for term in closure.captured_types() {
            let ty = solved
                .types
                .get(term.0 as usize)
                .copied()
                .ok_or_else(|| self.error(source, "missing captured local type"))?;
            observe
                .apply(&mut self.facts.types, ty)
                .map_err(|e| self.error(source, e))?;
        }
        let slots: HashMap<_, _> = interface.opening.universal_pairs().collect();
        let count = interface.count().map_err(|e| self.error(source, e))?;
        self.facts.body_work.canonical_steps += observe.steps + slots.len();
        let ported: Vec<_> = observe
            .observed_regions()
            .filter_map(|region| Some((region, *slots.get(ports.pairs.get(&region)?)?)))
            .collect();
        if ported.is_empty() {
            return Ok(None);
        }
        let mut replacements: HashMap<_, _> = ported.into_iter().collect();
        for slot in 0..count {
            let slot = u16::try_from(slot)
                .map_err(|_| self.error(source, "too many closure lifetimes"))
                .and_then(|slot| RegionId::bound(0, slot).map_err(|e| self.error(source, e)))?;
            replacements.insert(slot, slot);
        }
        Ok(Some(Bound::restore_owned_regions(
            replacements,
            FREE_REGIONS,
        )))
    }
}

fn close(
    fold: &mut Bound,
    store: &mut crate::types::Store,
    closed: &mut HashMap<TypeId, TypeId>,
    ty: TypeId,
) -> Result<TypeId, &'static str> {
    if let Some(&known) = closed.get(&ty) {
        return Ok(known);
    }
    let mapped = fold.apply(store, ty)?;
    closed.insert(ty, mapped);
    Ok(mapped)
}

fn close_formation(
    fold: &mut Bound,
    store: &mut crate::types::Store,
    closed: &mut HashMap<TypeId, TypeId>,
    environment: &mut crate::sema::closures::Environment,
) -> Result<(), &'static str> {
    for ty in environment
        .formation_types
        .iter_mut()
        .chain(&mut environment.formation_fields)
    {
        *ty = close(fold, store, closed, *ty)?;
    }
    Ok(())
}
