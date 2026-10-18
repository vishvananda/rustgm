//! Trait contracts retain direct predicates; applications share one capture-safe fold.
pub(super) mod assumptions;
pub(super) mod conformance;
pub(in crate::sema) mod equalities;
pub(super) mod heads;
pub(super) mod implementations;
pub(super) mod obligations;
pub(super) mod results;
pub(super) mod selection;
use super::{generics::Parameter, generics::Predicate as Located, generics::parameter_id};
use crate::types::{
    ItemId, Store, TypeId,
    arguments::Argument,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    substitute,
};
use std::{collections::HashMap, sync::Arc};

pub(super) struct Header {
    pub self_type: TypeId,
    pub self_bounds: std::collections::HashSet<crate::resolve::unit::NodeKey>,
    pub parameters: Vec<Parameter>,
    pub predicates: Vec<Located>,
    pub results: Vec<(ItemId, Located)>,
    pub regions: Vec<crate::types::outlives::Pair>,
}

impl Header {
    /// Only the written Self subject confers inheritance. An alias reducing to
    /// Self remains a separate requirement despite its identical canonical type.
    pub(in crate::sema) fn implies_self(
        &self,
        store: &Store,
        predicate: &Located,
    ) -> Result<bool, &'static str> {
        if !self.self_bounds.contains(&predicate.source) {
            return Ok(false);
        }
        // Classification of the written Self subject does not open its clause.
        let subject = match predicate.value {
            Clause::Predicate(value) => value.ty,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid Self clause")?
                    .predicate
                    .ty
            }
        };
        Ok(subject == self.self_type)
    }
    /// Classify only the subject identity; the caller retains the complete clause.
    fn implies(&self, store: &Store, predicate: &Located) -> Result<bool, &'static str> {
        if self.implies_self(store, predicate)? {
            return Ok(true);
        }
        let body = match predicate.value {
            Clause::Predicate(body) => body,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid trait requirement clause")?
                    .predicate
            }
        };
        Ok(matches!(body.bound, Bound::ProjectionEqual(_))
            && matches!(store.kind(body.ty),
                Some(crate::types::Kind::Projection(application))
                    if application.subject == self.self_type))
    }
}

/// Self bounds and equalities on Self's associated types are implied by a trait bound.
pub(super) struct Requirements {
    pub implied: Vec<Clause>,
    pub results: Vec<Clause>,
    pub required: Vec<Clause>,
    pub types: Vec<TypeId>,
    pub regions: Vec<crate::types::outlives::Pair>,
}

#[derive(Default)]
pub(super) struct Contracts {
    headers: HashMap<ItemId, Header>,
    order: Vec<ItemId>,
    applications: HashMap<Predicate, Arc<Requirements>>,
    result_rules: results::Cache,
    result_bounds: HashMap<ItemId, Vec<Clause>>,
    steps: usize,
}

impl Contracts {
    pub(super) fn headers(&self) -> impl Iterator<Item = (ItemId, &Header)> {
        self.order.iter().map(|&item| (item, &self.headers[&item]))
    }
    /// All result clauses and ordinary trait headers now belong to one immutable epoch.
    pub fn complete_results(&mut self) {
        self.result_rules.enable();
    }

    /// Associated member lookup uses only the complete ordinary supertrait graph.
    /// Finish those declaration-time bindings before publishing environments; cached
    /// lookup applications cannot then stand in for the final proof requirements.
    pub fn complete_bindings(
        &mut self,
        store: &Store,
        additions: Vec<(ItemId, Vec<Located>)>,
    ) -> Result<(), &'static str> {
        for (item, predicates) in additions {
            let header = self
                .headers
                .get_mut(&item)
                .ok_or("missing binding trait header")?;
            for predicate in predicates {
                if let Some(member) = header.owned_result(store, item, predicate.value)? {
                    header.results.push((member, predicate));
                    self.result_bounds
                        .entry(member)
                        .or_default()
                        .push(predicate.value);
                } else {
                    header.predicates.push(predicate);
                }
            }
        }
        self.applications.clear();
        self.result_rules.invalidate();
        Ok(())
    }
    pub fn declare(&mut self, item: ItemId, header: Header) -> Result<(), &'static str> {
        if self.headers.contains_key(&item) {
            return Err("trait contract already declared");
        }
        for &(member, predicate) in &header.results {
            self.result_bounds
                .entry(member)
                .or_default()
                .push(predicate.value);
        }
        self.result_rules.invalidate();
        self.order.push(item);
        self.headers.insert(item, header);
        Ok(())
    }

    pub fn associated_bounds(&self, member: ItemId) -> &[Clause] {
        self.result_bounds.get(&member).map_or(&[], Vec::as_slice)
    }

    /// Check definition identities before expansion, including argument-growing cycles.
    pub fn check_cycles(&self, store: &Store) -> Result<(), (ItemId, &'static str)> {
        let mut states = HashMap::new();
        let mut steps = 0_usize;
        for &root in &self.order {
            let mut work = vec![(root, false)];
            while let Some((item, finish)) = work.pop() {
                steps += 1;
                if steps + work.len() > 1_000_000 {
                    return Err((root, "supertrait graph work budget exceeded"));
                }
                if finish {
                    states.insert(item, true);
                    continue;
                }
                match states.get(&item) {
                    Some(true) => continue,
                    Some(false) => return Err((item, "cycle in supertrait requirements")),
                    None => {}
                }
                let header = self
                    .headers
                    .get(&item)
                    .ok_or((item, "missing trait contract"))?;
                states.insert(item, false);
                work.push((item, true));
                for predicate in header.predicates.iter().rev() {
                    let value = match predicate.value {
                        Clause::Predicate(value) => value,
                        Clause::ForAll(id) => {
                            store
                                .quantified(id)
                                .ok_or((item, "invalid supertrait clause"))?
                                .predicate
                        }
                    };
                    if header
                        .implies_self(store, predicate)
                        .map_err(|error| (item, error))?
                        && let Bound::Trait(application) = value.bound
                    {
                        let application = store
                            .trait_application(application)
                            .ok_or((item, "invalid supertrait application"))?;
                        work.push((application.item, false));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn requirements(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
    ) -> Result<Arc<Requirements>, &'static str> {
        if let Some(result) = self.applications.get(&predicate) {
            return Ok(Arc::clone(result));
        }
        let Bound::Trait(id) = predicate.bound else {
            return Err("trait contract requires a trait application");
        };
        let application = store
            .trait_application(id)
            .ok_or("invalid trait application")?;
        let header = self
            .headers
            .get(&application.item)
            .ok_or("missing trait contract")?;
        let arguments = store
            .arguments(application.arguments)
            .ok_or("invalid trait arguments")?;
        if arguments.len() != header.parameters.len() {
            return Err("trait contract argument arity mismatch");
        }
        self.steps = self
            .steps
            .saturating_add(1 + arguments.len() + header.predicates.len() + header.results.len());
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        let self_parameter = parameter_id(store, Argument::Type(header.self_type))?;
        let mut mapping = vec![(self_parameter, Argument::Type(predicate.ty))];
        let mut regions = Vec::new();
        let mut result = Requirements {
            implied: Vec::new(),
            results: Vec::new(),
            required: Vec::new(),
            types: Vec::new(),
            regions: Vec::new(),
        };
        for (parameter, argument) in header.parameters.iter().zip(arguments) {
            if let (Argument::Lifetime(_), Argument::Lifetime(region)) =
                (parameter.argument, argument)
            {
                regions.push(region);
                continue;
            }
            let id = parameter_id(store, parameter.argument)?;
            mapping.push((id, argument));
            if let Argument::Type(ty) = argument {
                result.types.push(ty);
                if matches!(
                    store.parameter(id).map(|p| p.kind),
                    Some(crate::types::parameters::ParameterKind::Type { sized: true })
                ) {
                    result.required.push(
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        }
                        .into(),
                    );
                }
            }
        }
        let mut replace = substitute::Bound::parameters_captured(store, &mapping, regions)?;
        for predicate in &header.predicates {
            let value = replace.clause(store, predicate.value)?;
            if header.implies(store, predicate)? {
                result.implied.push(value);
            } else {
                result.required.push(value);
            }
        }
        for (_, predicate) in &header.results {
            result.results.push(replace.clause(store, predicate.value)?);
        }
        for &(longer, shorter) in &header.regions {
            let (Argument::Lifetime(longer), Argument::Lifetime(shorter)) = (
                replace.apply_argument(store, Argument::Lifetime(longer))?,
                replace.apply_argument(store, Argument::Lifetime(shorter))?,
            ) else {
                return Err("invalid trait region substitution");
            };
            result.regions.push((longer, shorter));
        }
        self.steps = self.steps.saturating_add(replace.steps);
        if self.steps > 1_000_000 {
            return Err("trait contract work budget exceeded");
        }
        let result = Arc::new(result);
        self.applications.insert(predicate, Arc::clone(&result));
        Ok(result)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/clause_inheritance.rs"]
mod clause_inheritance_tests;
