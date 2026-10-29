//! Publish one completely prepared canonical inventory; executable proof stays separate.
use super::{Declarations, Error, Headers, Stage, error, mapped_source};
use crate::{
    resolve::model::DefId,
    sema::{generics::Parameter, lifetimes::FunctionRegions},
    support::retained,
    types::{TypeId, parameters::environment::Environment},
};
use std::{rc::Rc, sync::Arc};

impl Stage {
    pub(in crate::sema::crates::typed_headers) fn install(
        mut self,
        context: &mapped_source::Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        self.preflight(context, facts, budget)?;
        let owners = self.associated_rows(context, &facts.types, budget)?;
        budget.reserve(facts.associated.imported_allocation_bound(
            owners.len(),
            owners.iter().map(|row| row.members.len()).sum(),
        ))?;
        let (callables, functions) =
            super::template_ports::prepare(context, facts, &self.scopes, budget)?;
        let exact = super::verbatim_exports::prepare(context, facts, &functions, budget)?;
        let cells = super::definition_cells::prepare(
            context,
            &self.graph,
            facts,
            &self.scopes,
            &functions,
            budget,
        )?;
        let (contracts, self_bounds) = super::contract_ports::prepare(
            context,
            &self.graph,
            &facts.types,
            &mut self.scopes,
            budget,
        )?;
        let rows = context.descriptor.rows();
        budget.charge(
            rows.traits
                .len()
                .saturating_add(rows.clauses.len())
                .saturating_mul(2),
        )?;
        budget.reserve(
            retained::new_map::<crate::types::ItemId, bool>(rows.traits.len()).saturating_add(
                rows.traits
                    .len()
                    .saturating_add(rows.clauses.len())
                    .saturating_mul(size_of::<(crate::types::ItemId, bool)>()),
            ),
        )?;
        contracts
            .check_cycles(&facts.types)
            .map_err(|(_, message)| error(message))?;
        let before = table_allocation(facts, budget);
        budget.charge(before.visits.saturating_mul(2).saturating_add(1))?;
        // This shared work count is checked inside the owner installer BEFORE
        // its first write; failure cannot leave a partially installed owner.
        let mut work = budget.work.get();
        let exact = super::verbatim_exports::destination(exact, facts.foreign.as_deref_mut())?;
        let installed = facts.associated.install_imported(
            &facts.types,
            crate::sema::crates::ImportedOwners::Ordinary(Rc::clone(&self.graph)),
            owners,
            &mut work,
        );
        budget.work.set(work);
        installed.map_err(error)?;
        // Every remaining key, index and allocation bound was checked above.
        // No fallible remap, quota check or proof follows this first publication.
        for (definition, ty) in cells {
            facts.definitions[definition.0 as usize] = Some(ty);
        }
        publish_scopes(self.scopes, facts, budget);
        let linkage = facts.crates.get_or_insert_default();
        for function in functions {
            if let Some(symbol) = function.symbol {
                linkage.symbols.insert(function.definition, symbol);
            }
        }
        if let Some((plan, registry)) = exact {
            plan.install(registry);
        }
        facts.generics.traits = Some(Box::new(contracts));
        facts.generics.trait_self_bounds.extend(self_bounds);
        facts.lifetimes.callables = callables;
        budget.source = Some(crate::sema::crates::ImportedOwners::Ordinary(self.graph));
        let after = table_allocation(facts, budget);
        budget.observed.include(retained::Allocation {
            bytes: after.bytes.saturating_sub(before.bytes),
            visits: before.visits.saturating_add(after.visits),
        });
        Ok(())
    }

    fn preflight(
        &self,
        context: &mapped_source::Context<'_>,
        facts: &Declarations,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        self.graph.require(
            context.imported,
            context.catalog,
            context.descriptor,
            budget,
        )?;
        self.graph.require_store(&facts.types).map_err(error)?;
        if facts
            .crates
            .as_ref()
            .and_then(|linkage| linkage.origin_catalog())
            .is_none_or(|catalog| !std::ptr::eq(catalog, context.catalog))
        {
            return Err(error(
                "source installation changed its actual semantic ownership catalog",
            ));
        }
        let rows = context.descriptor.rows();
        if self.scopes.len() != rows.scopes.len()
            || budget.source.is_some()
            || !budget.entries.is_empty()
            || facts.generics.traits.is_some()
            || !facts.lifetimes.callables.is_empty()
        {
            return Err(error(
                "source canonical installation cannot replace a partial or existing header inventory",
            ));
        }
        prepay_tables(facts, rows.scopes.len(), rows.functions.len(), budget)?;
        let mut constants = 0usize;
        for scope in &self.scopes {
            budget.charge(1)?;
            constants += usize::from(scope.function.as_ref().is_some_and(|c| c.header.constant));
            let definition = scope.definition;
            if facts.generics.parameters.contains_key(&definition)
                || facts.generics.environments.contains_key(&definition)
                || facts.generics.parents.contains_key(&definition)
                || facts.generics.predicates.contains_key(&definition)
                || facts.generics.trait_self_bounds.contains_key(&definition)
                || facts.lifetimes.functions.contains_key(&definition)
                || facts.methods.receivers.contains_key(&definition)
                || facts.function_indices.contains_key(&definition)
                || facts.constant_functions.contains(&definition)
                || facts
                    .crates
                    .as_ref()
                    .is_some_and(|linkage| linkage.symbols.contains_key(&definition))
            {
                return Err(error(
                    "source scope destination contains previously installed declaration facts",
                ));
            }
        }
        let growth = retained::Growth::map::<DefId, ()>(
            facts.constant_functions.len(),
            facts.constant_functions.capacity(),
            constants,
        );
        budget.charge(growth.work)?;
        budget.reserve(growth.bytes)?;
        Ok(())
    }
}

fn prepay_tables(
    facts: &Declarations,
    scopes: usize,
    functions: usize,
    budget: &mut Headers,
) -> Result<(), Error> {
    budget.reserve(retained::sum([
        retained::new_map::<DefId, Vec<Parameter>>(
            facts.generics.parameters.len().saturating_add(scopes),
        ),
        retained::new_map::<DefId, Arc<Environment>>(
            facts.generics.environments.len().saturating_add(scopes),
        ),
        retained::new_map::<DefId, DefId>(facts.generics.parents.len().saturating_add(scopes)),
        retained::new_map::<DefId, super::Header>(functions),
        retained::new_map::<DefId, FunctionRegions>(
            facts.lifetimes.functions.len().saturating_add(functions),
        ),
        retained::new_map::<DefId, String>(
            facts
                .crates
                .as_ref()
                .map_or(0, |l| l.symbols.len())
                .saturating_add(functions),
        ),
        retained::new_map::<DefId, (TypeId, u32)>(
            facts.methods.receivers.len().saturating_add(functions),
        ),
    ]))
}

fn publish_scopes(scopes: Vec<super::Scope>, facts: &mut Declarations, budget: &mut Headers) {
    for scope in scopes {
        if let Some(parent) = scope.parent {
            facts.generics.parents.insert(scope.definition, parent);
        }
        facts
            .generics
            .parameters
            .insert(scope.definition, scope.parameters);
        facts
            .generics
            .environments
            .insert(scope.definition, scope.environment);
        if let Some(callable) = scope.function {
            facts
                .lifetimes
                .functions
                .insert(scope.definition, callable.regions);
            if let Some(receiver) = callable.receiver {
                facts.methods.receivers.insert(scope.definition, receiver);
            }
            if callable.header.constant {
                facts.constant_functions.insert(scope.definition);
            }
            budget.entries.insert(scope.definition, callable.header);
        }
    }
}

fn table_allocation(facts: &Declarations, headers: &Headers) -> retained::Allocation {
    let mut result = facts.associated.imported_table_allocation();
    result.map(&facts.generics.parameters);
    result.map(&facts.generics.parents);
    result.map(&facts.generics.environments);
    result.map(&facts.generics.trait_self_bounds);
    result.map(&facts.lifetimes.functions);
    result.map(&facts.methods.receivers);
    result.set(&facts.constant_functions);
    result.map(&headers.entries);
    if let Some(linkage) = &facts.crates {
        result.map(&linkage.symbols);
    }
    result
}
