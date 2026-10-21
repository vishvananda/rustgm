//! Suspended pattern dispatchers keep cache storage and incremental proof payload.
use super::{Declarations, Dispatcher, TypeId};
use crate::{support::retained::Allocation, types::lifetime_proof::Obligations};
pub(super) struct Retained {
    children: usize,
    visits: usize,
    caller: CallerRetention,
    owned: usize,
}
#[derive(Default)]
struct CallerRetention {
    environment: crate::types::parameters::environment::Retained,
    assumptions: crate::types::parameters::premise_index::Retained,
}
impl Dispatcher {
    pub(super) fn normalize_override(
        &mut self,
        facts: &mut Declarations,
        result: TypeId,
    ) -> Result<TypeId, String> {
        use crate::types::lifetime_proof::ScopeId;
        let (normalized, steps) = if self.retained.is_some() {
            // Concrete normalization reads only the root scope and allocator
            // frontier. Append its new debts through the accounting owner;
            // extend preserves free IDs and rebases the new local scope IDs.
            let mut added = Obligations::new(self.obligations.regions.end);
            let result = crate::sema::projections::concrete(&mut added, ScopeId::ROOT)?
                .normalize(facts, result)?;
            self.extend_evidence(added)?;
            result
        } else {
            crate::sema::projections::concrete(&mut self.obligations, ScopeId::ROOT)?
                .normalize(facts, result)?
        };
        self.context
            .account_work(steps)
            .map_err(|e| e.to_string())?;
        Ok(normalized)
    }
    pub(super) fn extend_evidence(&mut self, proof: Obligations) -> Result<(), &'static str> {
        if let Some(retained) = &mut self.retained {
            let full = proof.retained_allocation();
            let shallow = proof.shallow_allocation();
            retained.children = retained
                .children
                .saturating_add(full.bytes.saturating_sub(shallow.bytes));
            retained.visits = retained
                .visits
                .saturating_add(full.visits)
                .saturating_add(shallow.visits);
        }
        self.obligations.extend(proof)
    }
    pub(in crate::sema) fn retained_allocation(&mut self) -> (usize, usize) {
        if self.retained.is_none() {
            let full = self.obligations.retained_allocation();
            let shallow = self.obligations.shallow_allocation();
            self.retained = Some(Box::new(Retained {
                children: full.bytes.saturating_sub(shallow.bytes),
                caller: CallerRetention::default(),
                owned: 0,
                visits: full.visits.saturating_add(shallow.visits),
            }));
        }
        let mut total = Allocation::default();
        total.boxed(self);
        let (bytes, visits) = self.context.retained_allocation();
        total.include(Allocation { bytes, visits });
        let (bytes, visits) = self.normalize.retained_allocation();
        total.include(Allocation { bytes, visits });
        total.map(&self.selected);
        total.map(&self.constants);
        total.map(&self.expanded);
        if let Some(caller) = &self.caller {
            total.boxed(caller.as_ref());
        }
        total.include(self.obligations.shallow_allocation());
        if let Some(retained) = &mut self.retained {
            total.boxed(retained.as_ref());
            if let Some(caller) = &self.caller {
                let mut added = caller
                    .assumptions
                    .retain_allocation(&mut retained.caller.assumptions);
                if let (Some(environment), Some(shared)) =
                    (&caller.environment, &caller.shared_environment)
                {
                    added.include(
                        environment.retain_owned_from(shared, &mut retained.caller.environment),
                    );
                }
                retained.owned = retained.owned.saturating_add(added.bytes);
                total.visits = total.visits.saturating_add(added.visits);
            }
            total.bytes = total.bytes.saturating_add(retained.owned);
            total.bytes = total.bytes.saturating_add(retained.children);
            total.visits = total
                .visits
                .saturating_add(std::mem::take(&mut retained.visits));
        }
        (total.bytes, total.visits)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/suspended_dispatch.rs"]
mod tests;
