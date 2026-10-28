//! Trait contracts retain direct predicates; applications share one capture-safe fold.
mod applied_bounds;
pub(super) mod assumptions;
pub(in crate::sema) mod callables;
pub(super) mod conformance;
pub(in crate::sema) mod effects;
pub(in crate::sema) mod equalities;
mod feature_bounds;
pub(super) mod formation;
pub(super) mod heads;
pub(super) mod implementations;
pub(in crate::sema) mod inherited_inputs;
pub(super) mod object_bounds;
pub(super) mod obligations;
pub(super) mod results;
pub(super) mod selection;
use super::{generics::Parameter, generics::Predicate as Located, generics::parameter_id};
use crate::support::hash::HashMap;
use crate::types::{
    ItemId, Store, TypeId,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
use std::sync::Arc;

pub(super) struct Header {
    pub self_type: TypeId,
    pub parameters: Vec<Parameter>,
    pub inputs: inherited_inputs::HeaderInputs,
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
        self.source_inputs()?;
        self.implies_input(store, 0, predicate.into())
    }
    pub(in crate::sema) fn source_inputs(&self) -> Result<&inherited_inputs::Source, &'static str> {
        self.inputs.source_ref()
    }
    pub(in crate::sema) fn source_inputs_mut(
        &mut self,
    ) -> Result<&mut inherited_inputs::Source, &'static str> {
        self.inputs.source_mut()
    }
    pub(in crate::sema) fn implies_input(
        &self,
        store: &Store,
        index: usize,
        predicate: crate::sema::declaration_inputs::Input,
    ) -> Result<bool, &'static str> {
        if !self.inputs.is_self(index, predicate)? {
            return Ok(false);
        }
        // Classification of the written Self subject does not open its clause.
        let subject = match predicate.value {
            Clause::UnstableFeature(_) => return Ok(false),
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
    fn implies(
        &self,
        store: &Store,
        index: usize,
        predicate: crate::sema::declaration_inputs::Input,
    ) -> Result<bool, &'static str> {
        if self.implies_input(store, index, predicate)? {
            return Ok(true);
        }
        let body = match predicate.value {
            Clause::UnstableFeature(_) => return Ok(false),
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
#[derive(Default)]
pub(super) struct Requirements {
    pub implied: Vec<Clause>,
    pub results: Vec<Clause>,
    pub required: Vec<Clause>,
    pub types: Vec<TypeId>,
    pub regions: Vec<crate::types::outlives::Pair>,
}

#[derive(Default)]
pub(super) struct Contracts {
    feature_policy: Option<Box<feature_bounds::Policy>>,
    headers: HashMap<ItemId, Header>,
    order: Vec<ItemId>,
    applications: HashMap<Predicate, Arc<Requirements>>,
    formation: Option<Box<formation::Cache>>,
    result_rules: results::Cache,
    object_bounds: object_bounds::Cache,
    result_bounds: HashMap<ItemId, Vec<Clause>>,
    steps: usize,
}

impl Contracts {
    /// Cost only: caller supplies an actual empty-result declaration header.
    pub(in crate::sema) fn declaration_growth(&self) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        let mut growth =
            Growth::map::<ItemId, Header>(self.headers.len(), self.headers.capacity(), 1);
        growth.include(Growth::vector::<ItemId>(
            self.order.len(),
            self.order.capacity(),
            1,
        ));
        growth
    }
    pub(in crate::sema) fn header_table_allocation(&self) -> crate::support::retained::Allocation {
        let mut result = crate::support::retained::Allocation::default();
        result.map(&self.headers);
        result.vector(&self.order);
        result
    }
    pub(super) fn header(&self, item: ItemId) -> Option<&Header> {
        self.headers.get(&item)
    }
    pub(super) fn headers(&self) -> impl Iterator<Item = (ItemId, &Header)> {
        self.order.iter().map(|&item| (item, &self.headers[&item]))
    }
    /// All result clauses and ordinary trait headers now belong to one immutable epoch.
    pub fn complete_results(&mut self) {
        self.result_rules.enable();
        self.object_bounds.enable();
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
                    header.source_inputs_mut()?.predicates.push(predicate);
                }
            }
        }
        self.applications.clear();
        self.formation = None;
        self.result_rules.invalidate();
        self.object_bounds.invalidate();
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
        self.object_bounds.invalidate();
        self.order.push(item);
        self.headers.insert(item, header);
        Ok(())
    }

    pub fn associated_bounds(&self, member: ItemId) -> &[Clause] {
        self.result_bounds.get(&member).map_or(&[], Vec::as_slice)
    }

    /// Check definition identities before expansion, including argument-growing cycles.
    pub fn check_cycles(&self, store: &Store) -> Result<(), (ItemId, &'static str)> {
        let mut states = HashMap::default();
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
                let inputs = header.inputs.view();
                for index in (0..inputs.len()).rev() {
                    let predicate = inputs
                        .get(index)
                        .ok_or((item, "trait input prefix is incomplete"))?;
                    let value = match predicate.value {
                        Clause::UnstableFeature(_) => continue,
                        Clause::Predicate(value) => value,
                        Clause::ForAll(id) => {
                            store
                                .quantified(id)
                                .ok_or((item, "invalid supertrait clause"))?
                                .predicate
                        }
                    };
                    if header
                        .implies_input(store, index, predicate)
                        .map_err(|error| (item, error))?
                        && let Some((application, _)) = value.bound.trait_parts()
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
}

#[cfg(test)]
#[path = "../../tests/invariants/clause_inheritance.rs"]
mod clause_inheritance_tests;
